use crate::vm::{middleware, VirtualMachine};
use alloc::{format, vec::Vec};
use core::{ops::DerefMut, slice::from_raw_parts_mut};
use mibpf_common::{BinaryFileLayout, HelperAccessVerification, HelperFunctionID, VMConfiguration};
use mibpf_elf_utils::extract_allowed_helpers;

use rbpf::without_std::Error;

use riot_sys;
use riot_wrappers::{gcoap::PacketBuffer, mutex::Mutex, stdio::println};

use super::middleware::{
    helpers::{HelperAccessList, HelperFunction},
    CoapContext,
};

pub struct RbpfVm<'a> {
    /// The program buffer needs to be mutable in case we need to perform relocation
    /// resolution.
    pub program: &'a mut [u8],
    pub vm: rbpf::EbpfVmMbuff<'a>,
    pub layout: BinaryFileLayout,
}

impl<'a> RbpfVm<'a> {
    pub fn new(
        program: &'a mut [u8],
        config: VMConfiguration,
        allowed_helpers: Vec<HelperFunctionID>,
    ) -> RbpfVm<'a> {
        // We instantiate the VM and preserve a reference to the program buffer
        // in case we need to perform relocation resolution in the next step.
        let mut vm =
            rbpf::EbpfVmMbuff::new(Some(program), map_interpreter(config.binary_layout)).unwrap();

        // We need to make a decision whether we use the helper list that was
        // sent in the request or read the allowed helpers from the metadata appended
        // to the program binary.
        let allowed_helpers = match config.helper_access_list_source {
            mibpf_common::HelperAccessListSource::ExecuteRequest => {
                HelperAccessList::from(allowed_helpers)
            }
            mibpf_common::HelperAccessListSource::BinaryMetadata => {
                if config.binary_layout == BinaryFileLayout::ExtendedHeader {
                    HelperAccessList::from(extract_allowed_helpers(&program))
                } else {
                    Err("Tried to extract allowed helper function indices from an incompatible binary file")?
                }
            }
        };
        middleware::helpers::register_helpers(&mut vm, allowed_helpers.0.clone());
        RbpfVm {
            program,
            vm,
            layout,
        }
    }

    // TODO: deprecate and move all of timing to the special wrapper.
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
    fn verify_program(&self) -> Result<(), alloc::string::String> {
        // The VM runs the verification when the new program is loaded into it.
        self.vm.set_program(self.program)?;
        if self.config.helper_access_verification == HelperAccessVerification::PreFlight {
            let interpreter = map_interpreter(self.config.binary_layout);
            let helpers_idxs = allowed_helpers
                .iter()
                .map(|id| *id as u32)
                .collect::<Vec<u32>>();
            rbpf::check_helpers(program, &helpers_idxs, interpreter)
                .map_err(|e| format!("Error when checking helper function access: {:?}", e))?;
        }
        Ok(())
    }

    fn resolve_relocations(&mut self) -> Result<(), alloc::string::String> {
        mibpf_elf_utils::resolve_relocations(&mut self.program)?;
        self.vm.set_program(self.program)
    }
    fn execute(&mut self) -> u64 {
        self.vm
            .execute_program(&alloc::vec![], &alloc::vec![], alloc::vec![])
    }
    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer, result: &mut i64) -> u32 {
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

        self.vm
            .execute_program(
                mem_mutex.lock().deref_mut(),
                buffer_mutex.lock().deref_mut(),
                alloc::vec![pkt_buffer_region],
            )
            .map_err(|e| format!("Error: {}", e.to_string()))
    }
}
