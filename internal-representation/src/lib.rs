/*
 * This module contains the structs for the internal representation of objects
 * used by the different subcommands of the tool.
 */

use core::fmt;

use serde_derive::{Deserialize, Serialize};

/// Specifies which version of the eBPF VM is to be used when the program is
/// executed by the microcontroller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum TargetVM {
    /// The eBPF program will be executed by the rBPF VM.
    Rbpf = 0,
    /// The eBPF program will be executed by the FemtoContainer VM.
    FemtoContainer = 1,
}

impl From<u8> for TargetVM {
    fn from(v: u8) -> Self {
        match v {
            0 => TargetVM::Rbpf,
            1 => TargetVM::FemtoContainer,
            _ => panic!("Invalid VmTarget value"),
        }
    }
}

impl From<&str> for TargetVM {
    fn from(s: &str) -> Self {
        match s {
            "FemtoContainer" => TargetVM::FemtoContainer,
            "rBPF" => TargetVM::Rbpf,
            _ => panic!("Invalid vm target: {}", s),
        }
    }
}

impl fmt::Display for TargetVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Specifies the different binary file layouts that are supported by the VMs
/// Note that only the FemtoContainersHeader layout is compatible with the
/// FemtoContainer VM.
#[derive(Serialize, Eq, Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum BinaryFileLayout {
    /// The most basic layout of the produced binary. Used by the original version
    /// of the rBPF VM. It only includes the .text section from the ELF file.
    /// The limitation is that none of the .rodata relocations work in this case.
    OnlyTextSection = 0,
    /// A custom layout used by the VM version implemented for Femto-Containers.
    /// It starts with a header section which specifies lengths of remaining sections
    /// (.data, .rodata, .text). See [`crate::relocate::Header`] for more detailed
    /// description of the header format.
    FemtoContainersHeader = 1,
    /// An extension of the [`BytecodeLayout::FemtoContainersHeader`] bytecode
    /// layout. It appends additional metadata used for resolving function
    /// relocations and is supported by the modified version of rBPF VM.
    FunctionRelocationMetadata = 2,
    /// Raw object files are sent to the device and the relocations are performed
    /// there. This allows for maximum compatibility (e.g. .data relocations)
    /// however it comes with a burden of an increased memory footprint.
    RawObjectFile = 3,
}

impl From<&str> for BinaryFileLayout {
    fn from(s: &str) -> Self {
        match s {
            "OnlyTextSection" => BinaryFileLayout::OnlyTextSection,
            "FemtoContainersHeader" => BinaryFileLayout::FemtoContainersHeader,
            "FunctionRelocationMetadata" => BinaryFileLayout::FunctionRelocationMetadata,
            "RawObjectFile" => BinaryFileLayout::RawObjectFile,
            _ => panic!("Invalid binary layout: {}", s),
        }
    }
}

impl From<u8> for BinaryFileLayout {
    fn from(val: u8) -> Self {
        match val {
            0 => BinaryFileLayout::OnlyTextSection,
            1 => BinaryFileLayout::FemtoContainersHeader,
            2 => BinaryFileLayout::FunctionRelocationMetadata,
            3 => BinaryFileLayout::RawObjectFile,
            _ => panic!("Unknown binary file layout: {}", val),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VMExecutionRequestMsg {
    pub configuration: u8,
    pub available_helpers: [u8; 3],
}

/// Models the request that is sent to the target device to pull a specified
/// binary file from the CoAP fileserver.
/// The handler expects to get a request which consists of the IPv6 address of
/// the machine running the CoAP fileserver and the name of the manifest file
/// specifying which binary needs to be pulled.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuitPullRequest {
    pub ip_addr: String,
    pub manifest: String,
    pub riot_network_interface: String,
}

/// Encapsulates the configuration of a given VM execution, it defines the target
/// implementation of the VM, the file layout of the binary that the VM expects
/// and the SUIT storage slot from where the file needs to be loaded.
#[derive(PartialEq, Eq, Debug)]
pub struct VMConfiguration {
    pub vm_target: TargetVM,
    pub binary_layout: BinaryFileLayout,
    pub suit_slot: u8,
}

impl VMConfiguration {
    pub fn new(vm_target: TargetVM, binary_layout: BinaryFileLayout, suit_slot: u8) -> Self {
        VMConfiguration {
            vm_target,
            binary_layout,
            suit_slot,
        }
    }

    /// Encodes the VM configuration into a u8. The reason we need this is that
    /// RIOT message passing IPC infrastructure limits the size of the transported
    /// messages to 64 bits. In order to fully specify a given VM execution,
    /// we need all fields of the VMConfiguration struct and the metadata specifying
    /// which helper functions the VM is allowed to call. Encoding the configuration
    /// as a single u8 allows us to use the remaining bits to specify the helper
    /// metadata.
    ///
    /// The encoding is as follows:
    /// - The least significant bit specifies whether we should use the rbpf
    /// or the FemtoContainers VM. 0 corresponds to rbpf and 1 to FemtoContainers.
    /// - The next bit specifies the SUIT storage slot storing the eBPF program
    /// bytecode. There are only two available slots provided by RIOT so a single
    /// bit is sufficient.
    /// - The remaining bits are used to encode the binary layout that the VM
    /// should expect in the loaded program bytecode. Currently there are only 4
    /// options so 2 bits are sufficient. This can be adapted in the future.
    pub fn encode(&self) -> u8 {
        let mut encoding: u8 = 0;
        encoding |= self.vm_target as u8;
        encoding |= (self.suit_slot as u8) << 1;
        encoding |= (self.binary_layout as u8) << 2;
        encoding
    }

    /// Decodes the VM configuration according to the encoding specified above.
    pub fn decode(encoding: u8) -> Self {
        VMConfiguration {
            vm_target: TargetVM::from(encoding & 0b1),
            suit_slot: (encoding >> 1) & 0b1,
            binary_layout: BinaryFileLayout::from((encoding >> 2) & 0b11),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_after_encode_is_identity() {
        let configuration = VMConfiguration::new(
            TargetVM::FemtoContainer,
            BinaryFileLayout::FemtoContainersHeader,
            1,
        );

        let encoded = configuration.encode();
        let decoded = VMConfiguration::decode(encoded);

        assert_eq!(configuration, decoded);
    }
}
