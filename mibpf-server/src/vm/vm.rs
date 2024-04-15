use core::str::FromStr;

use alloc::{boxed::Box, string::String, vec::Vec};
use log::debug;
use mibpf_common::{HelperFunctionID, BinaryFileLayout, TargetVM, VMConfiguration};
use mibpf_elf_utils::resolve_relocations;
use riot_wrappers::gcoap::PacketBuffer;

use crate::infra::suit_storage;

use super::{FemtoContainerVm, RbpfVm};

/// Structs implementing this interface should allow for executing eBPF programs
/// both raw and with access to the incoming CoAP packet.
pub trait VirtualMachine {
    /// Executes a given eBPF program and stores the return value of the
    /// program in `result`. It returns the VM execution time
    fn execute(&mut self, result: &mut i64) -> u32;

    /// Executes a given eBPF program giving it access to the provided PacketBuffer
    /// and stores the return value of the program in `result`. The value returned
    /// by the program and written to `result` needs to represent the length of
    /// the packet PDU + payload. The reason for this is that the handler then
    /// needs to know this length when sending the response back
    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer, result: &mut i64) -> u32;
}

/// Responsible for initialising the VM. It loads the program bytecode from the
/// SUIT storage, (optionally) resolves relocations, and initialises the correct
/// version of the VM struct.
pub fn initialize_vm<'a>(
    configuration: VMConfiguration,
    allowed_helpers: Vec<HelperFunctionID>,
    program_buffer: &'a mut [u8],
) -> Result<Box<dyn VirtualMachine + 'a>, String> {
    let mut program = suit_storage::load_program(program_buffer, configuration.suit_slot);

    if configuration.binary_layout == BinaryFileLayout::RawObjectFile {
        resolve_relocations(&mut program)?;
    }

    let mut vm: Box<dyn VirtualMachine> = match configuration.vm_target {
        TargetVM::Rbpf => Box::new(RbpfVm::new(
            program,
            allowed_helpers,
            configuration.binary_layout,
        )),
        TargetVM::FemtoContainer => Box::new(FemtoContainerVm { program }),
    };

    Ok(vm)
}
