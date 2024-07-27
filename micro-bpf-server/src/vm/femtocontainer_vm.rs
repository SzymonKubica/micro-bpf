use core::ffi::c_void;

use alloc::{format, string::String};
use log::debug;
use riot_wrappers::{gcoap::PacketBuffer, println};

use crate::{infra::suit_storage, vm::VirtualMachine};

pub struct FemtoContainerVm<'a> {
    program: Option<&'a [u8]>,
    suit_slot: usize,
}

impl<'a> FemtoContainerVm<'a> {
    pub fn new(suit_slot: usize) -> Self {
        Self {
            program: None,
            suit_slot,
        }
    }
}

impl<'a> VirtualMachine for FemtoContainerVm<'a> {
    fn verify(&self) -> Result<(), String> {
        let Some(program) = self.program else {
            Err("VM not initialised")?
        };
        let return_code = unsafe { verify_fc_program(program.as_ptr(), program.len()) };

        if return_code != 0 {
            return Err(format!(
                "FemtoContainer VM program verification failed with code {}",
                return_code as i32,
            ));
        } else {
            return Ok(());
        }
    }

    fn initialize_vm(&mut self) -> Result<(), String> {
        let program = suit_storage::load_program_static(self.suit_slot);
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

    fn get_program_length(&self) -> usize {
        self.program.map_or(0, |p| p.len())
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
    fn sensor_processing_from_storage() -> u32;
    fn temperature_read() -> u32;
    fn test_printf() -> u32;
}
