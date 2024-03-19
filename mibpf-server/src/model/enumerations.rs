use core::str::FromStr;

use alloc::{format, string::String};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VMConfiguration {
    pub vm_target: TargetVM,
    pub binary_layout: BinaryFileLayout,
    pub suit_slot: usize,
}

impl VMConfiguration {
    pub fn new(vm_target: TargetVM, binary_layout: BinaryFileLayout, suit_slot: usize) -> Self {
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
            suit_slot: ((encoding >> 1) & 0b1) as usize,
            binary_layout: BinaryFileLayout::from((encoding >> 2) & 0b11),
        }
    }
}

/// Specifies the different binary file layouts that are supported by the VMs
/// Note that only the FemtoContainersHeader layout is compatible with the
/// FemtoContainer VM.
#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
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
    /// however it comes with a burden of an increased memory requirements.
    /// TODO: figure out if it is even feasible to perform that on the embedded
    /// device.
    RawObjectFile = 3,
}

impl Into<u8> for BinaryFileLayout {
    fn into(self) -> u8 {
        match self {
            BinaryFileLayout::OnlyTextSection => 0,
            BinaryFileLayout::FemtoContainersHeader => 1,
            BinaryFileLayout::FunctionRelocationMetadata => 2,
            BinaryFileLayout::RawObjectFile => 3,
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

impl FromStr for BinaryFileLayout {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OnlyTextSection" => Ok(BinaryFileLayout::OnlyTextSection),
            "FemtoContainersHeader" => Ok(BinaryFileLayout::FemtoContainersHeader),
            "FunctionRelocationMetadata" => Ok(BinaryFileLayout::FunctionRelocationMetadata),
            "RawObjectFile" => Ok(BinaryFileLayout::RawObjectFile),
            _ => Err(format!("Unknown binary file layout: {}", s)),
        }
    }
}

/// The target VM for the execution request
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TargetVM {
    Rbpf,
    FemtoContainer,
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

impl Into<u8> for TargetVM {
    fn into(self) -> u8 {
        match self {
            TargetVM::Rbpf => 0,
            TargetVM::FemtoContainer => 1,
        }
    }
}

impl FromStr for TargetVM {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rBPF" => Ok(TargetVM::Rbpf),
            "FemtoContainer" => Ok(TargetVM::FemtoContainer),
            _ => Err(String::from(s)),
        }
    }
}
