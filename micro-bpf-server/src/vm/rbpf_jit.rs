use crate::vm::{middleware, VirtualMachine};
use alloc::{
    collections::BTreeMap,
    format,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::{cell::RefCell, ops::DerefMut, slice::from_raw_parts_mut};
use log::debug;
use micro_bpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, HelperFunctionID,
    VMConfiguration,
};
use micro_bpf_elf_utils::extract_allowed_helpers;

use rbpf::lib::Error;

use riot_sys;
use riot_wrappers::{gcoap::PacketBuffer, mutex::Mutex, stdio::println};

use super::{
    middleware::{
        helpers::{HelperAccessList, HelperFunction},
        CoapContext,
    },
    rbpf_vm::map_interpreter,
};
use crate::infra::jit_prog_storage::{self, JIT_SLOT_SIZE};
use crate::infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE};

pub struct RbpfJIT<'a> {
    pub program: Option<RefCell<&'a mut [u8]>>,
    pub layout: BinaryFileLayout,
    pub allowed_helpers: Vec<HelperFunctionID>,
    pub helper_access_verification: HelperAccessVerification,
    pub helper_access_list_source: HelperAccessListSource,
    pub recompile: bool,
    pub jit_prog_slot: usize,
    pub jit_program_length: usize,
    pub jitted_fn: Option<unsafe fn(*mut u8, usize, *mut u8, usize) -> u32>,
}

impl<'a> RbpfJIT<'a> {
    pub fn new(config: VMConfiguration, allowed_helpers: Vec<HelperFunctionID>) -> RbpfJIT<'a> {
        RbpfJIT {
            program: None,
            layout: config.binary_layout,
            allowed_helpers,
            helper_access_verification: config.helper_access_verification,
            helper_access_list_source: config.helper_access_list_source,
            recompile: config.jit_compile,
            jit_prog_slot: config.suit_slot,
            jit_program_length: 0,
            jitted_fn: None,
        }
    }
}

impl<'a> VirtualMachine for RbpfJIT<'a> {
    fn initialize_vm(&mut self) -> Result<(), String> {
        if !self.recompile {
            self.jitted_fn = Some(jit_prog_storage::get_program_from_slot(self.jit_prog_slot).unwrap());
            return Ok(());
        }
        let program = suit_storage::load_program_static(self.jit_prog_slot);

        if self.layout != BinaryFileLayout::RawObjectFile {
            Err("The JIT only supports raw object file binary layout")?;
        };

        let _ = jit_prog_storage::free_storage_slot(self.jit_prog_slot);
        // We take the list of helpers from the execute request as this is the
        // only one way supported by the raw elf file binary layout that we use for the JIT.
        let mut helpers_map = BTreeMap::new();
        let helper_access_list = HelperAccessList::from(self.allowed_helpers.clone());

        for h in helper_access_list.0 {
            helpers_map.insert(h.id as u32, h.function);
        }

        let jit_slot = self.jit_prog_slot;

        // Here we acquire a pointer to global storage where the jitted
        // program will be written. The additional scope is introduced so
        // that the acquired MutexGuard goes out of scope at the end of it
        // and so the lock is released. (RAII)
        {
            let mut slot_guard = jit_prog_storage::acquire_storage_slot(jit_slot).unwrap();
            let mut text_offset = 0;

            let program_cell = RefCell::new(program);
            {
                let mut program_mut = program_cell.borrow_mut();
                let mut jit_memory = rbpf::JitMemory::new(
                    &mut program_mut,
                    slot_guard.0.as_mut(),
                    &helpers_map,
                    true,
                    false,
                    rbpf::InterpreterVariant::RawObjectFile,
                )
                .unwrap();
                self.jit_program_length = jit_memory.offset;
                debug!("JIT compilation successful");
                debug!("jitted program size: {} [B]", jit_memory.offset);
                text_offset = jit_memory.text_offset;
            }


            self.program = Some(program_cell);
            slot_guard.1 = text_offset;
        }
        self.jitted_fn = Some(jit_prog_storage::get_program_from_slot(self.jit_prog_slot).unwrap());
        Ok(())
    }
    fn verify(&self) -> Result<(), String> {
        if !self.recompile {
            return Ok(());
        }
        let prog_ref_cell = self.program.as_ref().unwrap();
        let prog_ref = prog_ref_cell.borrow();
        let interpreter = map_interpreter(self.layout);
        rbpf::EbpfVmMbuff::verify_program(interpreter, prog_ref.as_ref());

        if self.helper_access_verification == HelperAccessVerification::PreFlight {
            let helpers_idxs = self
                .allowed_helpers
                .iter()
                .map(|id| *id as u32)
                .collect::<Vec<u32>>();
            rbpf::check_helpers(prog_ref.as_ref(), &helpers_idxs, interpreter)
                .map_err(|e| format!("Error when checking helper function access: {:?}", e))?;
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<u64, String> {
        let mut ret = 0;
        unsafe {
            // We don't pass any meaningful arguments here as the program doesn't
            // work on a COAP message packet buffer.
            ret = self.jitted_fn.unwrap()(0 as *mut u8, 0, 0 as *mut u8, 0);
        }
        debug!("JIT execution successful: {}", ret);
        Ok(ret as u64)
    }

    fn execute_on_coap_pkt(&mut self, pkt: PacketBuffer) -> Result<u64, String> {
        let mut pkt_box = alloc::boxed::Box::new(pkt);
        let coap_context: &mut [u8] = unsafe {
            const CONTEXT_SIZE: usize = core::mem::size_of::<CoapContext>();
            let ctx = pkt_box.as_mut() as *mut _ as *mut CoapContext;
            debug!("CoAP context: {:?}", *ctx);
            from_raw_parts_mut(ctx as *mut u8, CONTEXT_SIZE)
        };

        let mut ret = 0;
        unsafe {
            // We don't pass any meaningful arguments here as the program doesn't
            // work on a COAP message packet buffer.
            ret = self.jitted_fn.unwrap()(coap_context as *mut _ as *mut u8, 0, 0 as *mut u8, 0);
        }
        debug!("JIT execution successful: {}", ret);
        Ok(ret as u64)
    }

    fn get_program_length(&self) -> usize {
        self.jit_program_length
    }
}
