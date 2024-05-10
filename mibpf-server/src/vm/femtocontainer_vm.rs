use core::ffi::c_void;

use alloc::{format, string::String};
use riot_wrappers::{gcoap::PacketBuffer, println};

use crate::vm::VirtualMachine;

pub struct FemtoContainerVm<'a> {
    pub program: &'a [u8],
}

impl<'a> FemtoContainerVm<'a> {
    pub fn new(program: &'a [u8]) -> Self {
        Self { program }
    }
}

impl<'a> VirtualMachine<'a> for FemtoContainerVm<'a> {
    fn resolve_relocations(&mut self) -> Result<(), String> {
        /// FemtoContainer VM doesn't support relocations so this is a no-op.
        Ok(())
    }

    fn verify_program(&self) -> Result<(), String> {
        let return_code = unsafe { verify_fc_program(self.program.as_ptr(), self.program.len()) };

        if return_code != 0 {
            return Err(format!(
                "FemtoContainer VM program verification failed with code {}",
                return_code
            ));
        } else {
            return Ok(());
        }
    }

    fn initialise_vm(&mut self, program: &'a [u8]) -> Result<(), String> {
        /// FemtoContainer VM doesn't require preflight initialisation
        Ok(())
    }

    fn execute(&mut self) -> Result<u64, String> {
        println!("Starting FemtoContainer VM execution.");
        unsafe {
            let mut result: i64 = 0;
            execute_fc_vm(
                self.program.as_ptr() as *const u8,
                self.program.len(),
                &mut result as *mut i64,
            );
            return Ok(result as u64);
        }
    }

    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        println!("Starting FemtoContainer VM execution.");
        unsafe {
            let mut result: i64 = 0;
            execute_fc_vm_on_coap_pkt(
                self.program.as_ptr() as *const u8,
                self.program.len(),
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
        program: *const u8,
        program_len: usize,
        pkt: *mut c_void, // PacketBuffer isn't ffi-safe so we need to pass *c_void
        return_value: *mut i64,
    ) -> u32;

    fn execute_fc_vm(program: *const u8, program_len: usize, return_value: *mut i64) -> u32;
    fn verify_fc_program(program: *const u8, program_len: usize) -> u32;
}
