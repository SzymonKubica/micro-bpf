use core::str::FromStr;

use alloc::{boxed::Box, format, string::String, vec::Vec};
use log::debug;
use mibpf_common::{
    BinaryFileLayout, HelperAccessVerification, HelperFunctionID, TargetVM, VMConfiguration,
};
use mibpf_elf_utils::resolve_relocations;
use riot_wrappers::gcoap::PacketBuffer;

use crate::infra::suit_storage;

use super::{rbpf_vm, FemtoContainerVm, RbpfVm};

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
    config: VMConfiguration,
    allowed_helpers: Vec<HelperFunctionID>,
    program_buffer: &'a mut [u8],
) -> Result<Box<dyn VirtualMachine + 'a>, String> {
    let mut program = suit_storage::load_program(program_buffer, config.suit_slot);

    // We exit early if the Femto-Container VM is to be used as it isn't
    // as configurable and most configuration options don't apply to it
    if config.vm_target == TargetVM::FemtoContainer {
        return Ok(Box::new(FemtoContainerVm { program }));
    }

    if config.helper_access_verification == HelperAccessVerification::PreFlight {
        // We first need to map our state to the structures that rbpf understands
        let helper_idxs = allowed_helpers
            .iter()
            .map(|id| *id as u32)
            .collect::<Vec<u32>>();
        let interpreter = rbpf_vm::map_interpreter(config.binary_layout);
        rbpf::check_helpers(program, &helper_idxs, interpreter)
            .map_err(|e| format!("Error when checking helper function access: {:?}", e))?;
    }

    if config.binary_layout == BinaryFileLayout::RawObjectFile {
        resolve_relocations(&mut program)?;
    }

    Ok(Box::new(RbpfVm::new(
        program,
        allowed_helpers,
        config.binary_layout,
    )))
}
