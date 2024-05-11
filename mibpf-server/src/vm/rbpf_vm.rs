use crate::vm::{middleware, VirtualMachine};
use alloc::{
    format,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::{ops::DerefMut, slice::from_raw_parts_mut};
use mibpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, HelperFunctionID,
    VMConfiguration,
};
use mibpf_elf_utils::extract_allowed_helpers;

use rbpf::without_std::Error;

use riot_sys;
use riot_wrappers::{gcoap::PacketBuffer, mutex::Mutex, stdio::println};

use super::middleware::{
    helpers::{HelperAccessList, HelperFunction},
    CoapContext,
};

/// An adapter struct which wraps around the rbpf VM so that it is compatible
/// with the interface defined in the `VirtualMachine` trait.
pub struct RbpfVm<'a> {
    pub vm: Option<rbpf::EbpfVmMbuff<'a>>,
    pub layout: BinaryFileLayout,
    pub allowed_helpers: Vec<HelperFunctionID>,
    pub helper_access_verification: HelperAccessVerification,
    pub helper_access_list_source: HelperAccessListSource,
}

impl<'a> RbpfVm<'a> {
    pub fn new(
        config: VMConfiguration,
        allowed_helpers: Vec<HelperFunctionID>,
    ) -> Result<RbpfVm<'a>, String> {
        Ok(RbpfVm {
            vm: None,
            layout: config.binary_layout,
            allowed_helpers,
            helper_access_verification: config.helper_access_verification,
            helper_access_list_source: config.helper_access_list_source,
        })
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

impl<'a> VirtualMachine<'a> for RbpfVm<'a> {
    fn resolve_relocations(&mut self, program: &'a mut [u8]) -> Result<&'a [u8], String> {
        if self.layout == BinaryFileLayout::RawObjectFile {
            mibpf_elf_utils::resolve_relocations(program)?;
        };
        Ok(program)
    }
    fn verify(&self) -> Result<(), String> {
        // The VM runs the verification when the new program is loaded into it.
        if let Some(vm) = self.vm.as_ref() {
            vm.verify_loaded_program()
                .map_err(|e| format!("Error: {:?}", e))?;

            if self.helper_access_verification == HelperAccessVerification::PreFlight {
                let interpreter = map_interpreter(self.layout);
                let helpers_idxs = self
                    .allowed_helpers
                    .iter()
                    .map(|id| *id as u32)
                    .collect::<Vec<u32>>();
                vm.verify_helper_calls(&helpers_idxs, interpreter)
                    .map_err(|e| format!("Error when checking helper function access: {:?}", e))?;
            }
        } else {
            Err("VM not initialised".to_string())?;
        }

        Ok(())
    }

    fn initialise_vm(&mut self, program: &'a [u8]) -> Result<(), String> {
        // We need to make a decision whether we use the helper list that was
        // sent in the request or read the allowed helpers from the metadata appended
        // to the program binary.
        let helper_access_list = match self.helper_access_list_source {
            mibpf_common::HelperAccessListSource::ExecuteRequest => {
                HelperAccessList::from(self.allowed_helpers.clone())
            }
            mibpf_common::HelperAccessListSource::BinaryMetadata => {
                if self.layout == BinaryFileLayout::ExtendedHeader {
                    HelperAccessList::from(extract_allowed_helpers(program))
                } else {
                    Err("Tried to extract allowed helper function indices from an incompatible binary file")?
                }
            }
        };
        self.vm = Some(
            rbpf::EbpfVmMbuff::new(Some(program), map_interpreter(self.layout))
                .map_err(|e| format!("Error: {:?}", e))?,
        );
        middleware::helpers::register_helpers(
            self.vm.as_mut().unwrap(),
            helper_access_list.0.clone(),
        );
        Ok(())
    }

    fn execute(&mut self) -> Result<u64, String> {
        if let Some(vm) = self.vm.as_mut() {
            vm.execute_program(&alloc::vec![], &alloc::vec![], alloc::vec![])
                .map_err(|e| format!("Error: {:?}", e))
        } else {
            Err("VM not initialised".to_string())
        }
    }
    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        /// Coap context struct containing information about the buffer,
        /// packet and its length. It is passed into the VM as the main buffer
        /// on which the program operates.
        let coap_context: &mut [u8] = unsafe {
            const CONTEXT_SIZE: usize = core::mem::size_of::<CoapContext>();
            let ctx = pkt as *mut _ as *mut CoapContext;
            println!("Context: {:?}", *ctx);
            from_raw_parts_mut(ctx as *mut u8, CONTEXT_SIZE)
        };

        // Actual packet struct
        let mem = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            println!("Coap context: {:?}", *ctx);
            from_raw_parts_mut((*ctx).pkt as *mut u8, (*ctx).len)
        };

        let pkt_buffer_region: (u64, u64) = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            ((*ctx).buf as *const u8 as u64, (*ctx).len as u64)
        };

        if let Some(vm) = self.vm.as_mut() {
            vm.execute_program(mem, coap_context, alloc::vec![pkt_buffer_region])
                .map_err(|e| format!("Error: {:?}", e))
        } else {
            Err("VM not initialised".to_string())
        }
    }
}
