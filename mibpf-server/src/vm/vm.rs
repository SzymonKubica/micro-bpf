use alloc::{boxed::Box, format, string::String, vec::Vec};
use mibpf_common::{
    BinaryFileLayout, HelperAccessVerification, HelperFunctionID, TargetVM, VMConfiguration,
};
use mibpf_elf_utils::{extract_allowed_helpers, resolve_relocations};
use riot_wrappers::gcoap::PacketBuffer;

use crate::infra::{local_storage, suit_storage};

use super::{middleware::helpers::HelperAccessList, rbpf_vm, FemtoContainerVm, RbpfVm};

/// Structs implementing this interface should allow for executing eBPF programs
/// both raw and with access to the incoming CoAP packet.
pub trait VirtualMachine {
    /// Loads, verifies, optionally resolves relocations and executes the program.
    fn full_run(&mut self) -> Result<i64, String> {
        self.resolve_relocations()?;
        self.verify_program()?;
        self.execute()
    }
    fn verify_program(&self) -> Result<(), String>;
    /// Patches the program bytecode using the relocation metadata to fix accesses
    /// to .data and .rodata sections.
    fn resolve_relocations(&mut self) -> Result<(), String>;
    /// Executes a given program and returns its return value.
    fn execute(&mut self) -> Result<u64, String>;
    /// Executes a given eBPF program giving it access to the provided PacketBuffer
    /// and returns the return value of the program. The value returned
    /// by the program needs to represent the length of
    /// the packet PDU + payload. The reason for this is that the handler then
    /// needs to know this length when sending the response back.
    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String>;
}

/// Responsible for initialising the VM. It loads the program bytecode from the
/// SUIT storage, (optionally) resolves relocations, and initialises the correct
/// version of the VM struct.
pub fn initialize_vm<'a>(
    config: VMConfiguration,
    allowed_helpers: Vec<HelperFunctionID>,
    program_buffer: &'a mut [u8],
) -> Result<Box<dyn VirtualMachine + 'a>, String> {
    let mut program = suit_storage::load_program(program_buffer, config.suit_slot);

    match config.vm_target {
        TargetVM::Rbpf => {
            let vm = RbpfVm::new(&mut program, config, allowed_helpers)?;
            return Ok(Box::new(vm));
        }
        TargetVM::FemtoContainer => {
            let vm = FemtoContainerVm::new(&program);
            return Ok(Box::new(vm));
        }
    }
}
