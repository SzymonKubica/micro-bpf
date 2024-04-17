use crate::vm::{middleware, VirtualMachine};
use alloc::vec::Vec;
use core::{ops::DerefMut, slice::from_raw_parts_mut};
use mibpf_common::{BinaryFileLayout, HelperFunctionID};

use rbpf::without_std::Error;

use riot_sys;
use riot_wrappers::{gcoap::PacketBuffer, mutex::Mutex, stdio::println};

use super::middleware::{
    helpers::{HelperAccessList, HelperFunction},
    CoapContext,
};

pub struct RbpfVm<'a> {
    pub registered_helpers: Vec<HelperFunction>,
    pub vm: rbpf::EbpfVmMbuff<'a>,
    pub layout: BinaryFileLayout,
}

impl<'a> RbpfVm<'a> {
    pub fn new(
        program: &'a [u8],
        helpers: Vec<HelperFunctionID>,
        layout: BinaryFileLayout,
    ) -> RbpfVm<'a> {
        RbpfVm {
            registered_helpers: HelperAccessList::from(helpers).0,
            vm: rbpf::EbpfVmMbuff::new(Some(program), map_interpreter(layout)).unwrap(),
            layout,
        }
    }

    #[allow(dead_code)]
    pub fn add_helper(&mut self, helper: HelperFunction) {
        self.registered_helpers.push(helper);
    }

    fn timed_execution(&self, execution_fn: impl Fn() -> Result<u64, Error>) -> (i64, u32) {
        println!("Starting rBPf VM execution.");
        // This unsafe hacking is needed as the ztimer_now call expects to get an
        // argument of type riot_sys::inline::ztimer_clock_t but the ztimer_clock_t
        // ZTIMER_USEC that we get from riot_sys has type riot_sys::ztimer_clock_t.
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = Self::time_now(clock);
        let result = execution_fn();
        let end: u32 = Self::time_now(clock);
        let ret = if let Ok(val) = result {
            println!("Program returned: {:?} ({:#x})", val, val);
            val as i64
        } else {
            println!("Program returned: {:?}", result);
            -1
        };
        let execution_time = end - start;
        println!("Execution time: {} [us]", execution_time);
        (ret as i64, execution_time)
    }

    #[inline(always)]
    fn time_now(clock: *mut riot_sys::inline::ztimer_clock_t) -> u32 {
        unsafe { riot_sys::inline::ztimer_now(clock) }
    }
}

pub fn map_interpreter(layout: BinaryFileLayout) -> rbpf::InterpreterVariant {
    match layout {
        BinaryFileLayout::FemtoContainersHeader => rbpf::InterpreterVariant::FemtoContainersHeader,
        BinaryFileLayout::ExtendedHeader => rbpf::InterpreterVariant::ExtendedHeader,
        BinaryFileLayout::RawObjectFile => rbpf::InterpreterVariant::RawObjectFile,
        BinaryFileLayout::OnlyTextSection => rbpf::InterpreterVariant::Default,
    }
}

impl VirtualMachine for RbpfVm<'_> {
    fn execute(&mut self, result: &mut i64) -> u32 {
        middleware::helpers::register_helpers(&mut self.vm, self.registered_helpers.clone());

        let (ret, execution_time) =
            self.timed_execution(|| self.vm.execute_program(&alloc::vec![], &alloc::vec![], alloc::vec![]));
        *result = ret;
        execution_time
    }
    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer, result: &mut i64) -> u32 {
        middleware::helpers::register_helpers(&mut self.vm, self.registered_helpers.clone());

        /// Coap context struct containing information about the buffer,
        /// packet and its length. It is passed into the VM as the main buffer
        /// on which the program operates.
        let coap_context: &mut [u8] = unsafe {
            const CONTEXT_SIZE: usize = core::mem::size_of::<CoapContext>();
            let ctx = pkt as *mut _ as *mut CoapContext;
            println!("Context: {:?}", *ctx);
            from_raw_parts_mut(ctx as *mut u8, CONTEXT_SIZE)
        };

        let buffer_mutex = Mutex::new(coap_context);

        // Actual packet struct
        let mem = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            println!("Coap context: {:?}", *ctx);
            from_raw_parts_mut((*ctx).pkt as *mut u8, (*ctx).len)
        };

        let mem_mutex = Mutex::new(mem);

        let pkt_buffer_region: (u64, u64) = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            ((*ctx).buf as *const u8 as u64, (*ctx).len as u64)
        };

        // Here we need to do some hacking with locks as closures don't like
        // capturing &mut references from environment. It does make sense.
        let (ret, execution_time) = self.timed_execution(|| {
            self.vm.execute_program(
                mem_mutex.lock().deref_mut(),
                buffer_mutex.lock().deref_mut(),
                alloc::vec![pkt_buffer_region]
            )
        });
        *result = ret;
        execution_time
    }
}
