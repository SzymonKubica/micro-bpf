use core::ffi::c_void;

use alloc::{format, string::String};
use log::debug;
use riot_wrappers::{gcoap::PacketBuffer, println};

use crate::vm::VirtualMachine;

pub struct FemtoContainerVm<'a> {
    program: Option<&'a [u8]>,
}

impl<'a> FemtoContainerVm<'a> {
    pub fn new() -> Self {
        Self { program: None }
    }
}

impl<'a> VirtualMachine<'a> for FemtoContainerVm<'a> {
    fn resolve_relocations(&mut self, program: &'a mut [u8]) -> Result<&'a mut [u8], String> {
        /// FemtoContainer VM doesn't support relocations so this is an identity mapping.
        Ok(program)
    }

    fn verify(&self) -> Result<(), String> {
        let Some(program) = self.program else {
            Err("VM not initialised")?
        };
        let return_code = unsafe { verify_fc_program(program.as_ptr(), program.len()) };

        if return_code != 0 {
            return Err(format!(
                "FemtoContainer VM program verification failed with code {}",
                return_code
            ));
        } else {
            return Ok(());
        }
    }

    fn initialize_vm(&mut self, program: &'a mut [u8]) -> Result<(), String> {
        self.program = Some(program);
        unsafe {
            initialize_fc_vm(program.as_ptr() as *const u8, program.len());
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<u64, String> {
        debug!("Starting FemtoContainer VM execution.");
        let Some(program) = self.program else {
            Err("VM not initialised")?
        };
        let mut result: i64 = 0;
        // We need to define the stack here and pass it into the VM.
        // For some reason the static stack allocation in the c file doesn't work.
        let mut stack: [u8; 512] = [0; 512];
        unsafe {
            execute_fc_vm(&mut stack as *mut u8, &mut result as *mut i64);
        }
        Ok(result as u64)
    }

    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        debug!("Starting FemtoContainer VM execution.");
        let Some(program) = self.program else {
            Err("VM not initialised")?
        };
        unsafe {
            let mut result: i64 = 0;
            // We need to define the stack here and pass it into the VM.
            // For some reason the static stack allocation in the c file doesn't work.
            let mut stack: [u8; 512] = [0; 512];
            execute_fc_vm_on_coap_pkt(
                &mut stack as *mut u8,
                pkt as *mut PacketBuffer as *mut c_void,
                &mut result as *mut i64,
            );
            return Ok(result as u64);
        }
    }
}

extern "C" {
    /// Executes a femtocontainer VM where the eBPF program has access
    /// to the pointer to the CoAP packet.
    fn execute_fc_vm_on_coap_pkt(
        stack: *mut u8,
        pkt: *mut c_void, // PacketBuffer isn't ffi-safe so we need to pass *c_void
        result: *mut i64,
    ) -> u32;

    fn initialize_fc_vm(program: *const u8, program_len: usize) -> u32;
    fn execute_fc_vm(stack: *mut u8, result: *mut i64) -> u32;
    fn verify_fc_program(program: *const u8, program_len: usize) -> u32;
}
