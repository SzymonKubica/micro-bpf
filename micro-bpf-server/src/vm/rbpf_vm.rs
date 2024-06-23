use crate::{
    infra::suit_storage,
    vm::{middleware, VirtualMachine},
};
use alloc::{
    format,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use log::debug;
use core::{ops::DerefMut, slice::from_raw_parts_mut};
use micro_bpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, HelperFunctionID,
    VMConfiguration,
};
use micro_bpf_elf_utils::extract_allowed_helpers;

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
    pub program_length: usize,
    pub suit_slot: usize,
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
            program_length: 0,
            suit_slot: config.suit_slot,
        })
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

impl<'a> VirtualMachine for RbpfVm<'a> {
    fn initialize_vm(&mut self) -> Result<(), String> {
        let program = suit_storage::load_program_static(self.suit_slot);

        // We need to make a decision whether we use the helper list that was
        // sent in the request or read the allowed helpers from the metadata appended
        // to the program binary.
        let helper_access_list = match self.helper_access_list_source {
            micro_bpf_common::HelperAccessListSource::ExecuteRequest => {
                HelperAccessList::from(self.allowed_helpers.clone())
            }
            micro_bpf_common::HelperAccessListSource::BinaryMetadata => {
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
        self.program_length = program.len();
        middleware::helpers::register_helpers(
            self.vm.as_mut().unwrap(),
            helper_access_list.0.clone(),
        );
        Ok(())
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
            debug!("CoAP context: {:?}", *ctx);
            from_raw_parts_mut(ctx as *mut u8, CONTEXT_SIZE)
        };

        // Actual packet struct
        let mem = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            debug!("CoAP context: {:?}", *ctx);
            from_raw_parts_mut((*ctx).pkt as *mut u8, (*ctx).len)
        };

        let pkt_buffer_region: (u64, u64) = unsafe {
            let ctx = pkt as *mut _ as *mut CoapContext;
            ((*ctx).buf as *const u8 as u64, (*ctx).len as u64)
        };

        if let Some(vm) = self.vm.as_mut() {
            let result = vm.execute_program(mem, coap_context, alloc::vec![pkt_buffer_region])
                .map_err(|e| format!("Error: {:?}", e));
            debug!("CoAP execution result: {:?}", result.clone().unwrap_or(0));
            return result;

        } else {
            Err("VM not initialised".to_string())
        }
    }

    fn get_program_length(&self) -> usize {
        return self.program_length;
    }
}
