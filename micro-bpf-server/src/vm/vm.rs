use alloc::{boxed::Box, string::String, vec::Vec};
use micro_bpf_common::{
    HelperFunctionID, TargetVM, VMConfiguration,
};
use riot_wrappers::gcoap::PacketBuffer;

use super::{
    rbpf_jit::RbpfJIT, FemtoContainerVm, RbpfVm,
};

/// Structs implementing this interface should allow for executing eBPF programs
/// both raw and with access to the incoming CoAP packet.
pub trait VirtualMachine {
    /// Loads, verifies, optionally resolves relocations and executes the program.
    fn full_run(&mut self) -> Result<u64, String> {
        self.initialize_vm()?;
        self.verify()?;
        self.execute()
    }
    fn full_run_on_coap_pkt(
        &mut self,
        pkt: PacketBuffer,
    ) -> Result<u64, String> {
        self.initialize_vm()?;
        self.verify()?;
        self.execute_on_coap_pkt(pkt)
    }
    /// Initializes the VM, in case of the JIT this step involves jit-compilation.
    /// In case of raw elf file binaries this is where the relocation resolution
    /// should take place. In all other case we simply attach all helper functions
    /// to the VM here.
    fn initialize_vm(&mut self) -> Result<(), String>;
    /// Verifies the program bytecode after it has been loaded into the VM.
    fn verify(&self) -> Result<(), String>;
    /// Executes a given program and returns its return value.
    fn execute(&mut self) -> Result<u64, String>;
    /// Executes a given eBPF program giving it access to the provided PacketBuffer
    /// and returns the return value of the program. The value returned
    /// by the program needs to represent the length of
    /// the packet PDU + payload. The reason for this is that the handler then
    /// needs to know this length when sending the response back.
    fn execute_on_coap_pkt(&mut self, pkt: PacketBuffer) -> Result<u64, String>;
    /// Returns the length of the program that is currently loaded into the VM.
    /// This is used for benchmarking, because when we are using the jit, we
    /// don't know the final program size until we execute it.
    fn get_program_length(&self) -> usize;
}

/// Responsible for constructing the VM. It loads the program bytecode from the
/// SUIT storage, and initialises the correct version of the VM struct.
/// The reason we do both of those things at the same time is that the lifetime
/// of the VM is tied to the lifetime of the program buffer (as every VM operates
/// on only one program).
pub fn construct_vm<'a>(
    config: VMConfiguration,
    allowed_helpers: Vec<HelperFunctionID>,
) -> Result<Box<dyn VirtualMachine>, String> {

    if config.jit {
        return Ok(Box::new(RbpfJIT::new(config, allowed_helpers)));
    }

    match config.vm_target {
        TargetVM::Rbpf => {
            return Ok(Box::new(RbpfVm::new(config, allowed_helpers)?));
        }
        TargetVM::FemtoContainer => {
            return Ok(Box::new(FemtoContainerVm::new(config.suit_slot)));
        }
    }
}
