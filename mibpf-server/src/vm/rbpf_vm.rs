use crate::vm::{middleware, VirtualMachine};
use alloc::{format, string::String, vec::Vec};
use serde::Deserialize;
use core::{ffi::c_void, ops::DerefMut, str::FromStr};

use rbpf::without_std::Error;

use riot_sys;
use riot_wrappers::{gcoap::PacketBuffer, mutex::Mutex, stdio::println};

/// Specifies the different binary file layouts that are supported by the VMs
/// Note that only the FemtoContainersHeader layout is compatible with the
/// FemtoContainer VM.
#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum BinaryFileLayout {
    /// The most basic layout of the produced binary. Used by the original version
    /// of the rBPF VM. It only includes the .text section from the ELF file.
    /// The limitation is that none of the .rodata relocations work in this case.
    OnlyTextSection,
    /// A custom layout used by the VM version implemented for Femto-Containers.
    /// It starts with a header section which specifies lengths of remaining sections
    /// (.data, .rodata, .text). See [`crate::relocate::Header`] for more detailed
    /// description of the header format.
    FemtoContainersHeader,
    /// An extension of the [`BytecodeLayout::FemtoContainersHeader`] bytecode
    /// layout. It appends additional metadata used for resolving function
    /// relocations and is supported by the modified version of rBPF VM.
    FunctionRelocationMetadata,
    /// Raw object files are sent to the device and the relocations are performed
    /// there. This allows for maximum compatibility (e.g. .data relocations)
    /// however it comes with a burden of an increased memory requirements.
    /// TODO: figure out if it is even feasible to perform that on the embedded
    /// device.
    RawObjectFile,
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

pub struct RbpfVm {
    pub registered_helpers: Vec<middleware::HelperFunction>,
    pub layout: BinaryFileLayout,
}

extern "C" {
    /// Copies all contents of the packet under *ctx into the provided memory region.
    /// It also recalculates pointers inside of that packet struct so that they point
    /// to correct offsets in the target memory buffer. This function is needed for
    /// executing the rBPF VM on raw packet data.
    fn copy_packet(buffer: *mut c_void, mem: *mut u8);
}

impl Default for RbpfVm {
    fn default() -> Self {
        Self::new(Vec::new(), BinaryFileLayout::FunctionRelocationMetadata)
    }
}

impl RbpfVm {
    pub fn new(helpers: Vec<middleware::HelperFunction>, layout: BinaryFileLayout) -> Self {
        RbpfVm {
            registered_helpers: helpers,
            layout,
        }
    }

    #[allow(dead_code)]
    pub fn add_helper(&mut self, helper: middleware::HelperFunction) {
        self.registered_helpers.push(helper);
    }

    fn timed_execution(&self, execution_fn: impl Fn() -> Result<u64, Error>) -> (i64, u32) {
        println!("Starting rBPf VM execution.");
        // This unsafe hacking is needed as the ztimer_now call expects to get an
        // argument of type riot_sys::inline::ztimer_clock_t but the ztimer_clock_t
        // ZTIMER_USEC that we get from riot_sys has type riot_sys::ztimer_clock_t.
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = Self::time_now(clock);
        let result = execution_fn();
        let end: u32 = Self::time_now(clock);
        let ret = if let Ok(val) = result {
            println!("Program returned: {:?} ({:#x})", val, val);
            val as i64
        } else {
            println!("Program returned: {:?}", result);
            -1
        };
        let execution_time = end - start;
        println!("Execution time: {} [us]", execution_time);
        (ret as i64, execution_time)
    }

    #[inline(always)]
    fn time_now(clock: *mut riot_sys::inline::ztimer_clock_t) -> u32 {
        unsafe { riot_sys::inline::ztimer_now(clock) }
    }
}

impl VirtualMachine for RbpfVm {
    fn execute(&self, program: &[u8], result: &mut i64) -> u32 {
        let mut vm = rbpf::EbpfVmNoData::new(Some(program)).unwrap();
        match self.layout {
            BinaryFileLayout::FemtoContainersHeader
            | BinaryFileLayout::FunctionRelocationMetadata => {
                vm.override_interpreter(rbpf::InterpreterVariant::Extended);
            }
            _ => {}
        }

        middleware::register_helpers(&mut vm, self.registered_helpers.clone());

        let (ret, execution_time) = self.timed_execution(|| vm.execute_program());
        *result = ret;
        execution_time
    }
    fn execute_on_coap_pkt(&self, program: &[u8], pkt: &mut PacketBuffer, result: &mut i64) -> u32 {
        // Memory for the packet.
        let mut mem: [u8; 512] = [0; 512];
        unsafe { copy_packet(pkt as *mut _ as *mut c_void, mem.as_mut_ptr() as *mut u8) };

        println!("Packet copy size: {}", mem.len());

        // Initialise the VM operating on a fixed memory buffer.
        let mut vm = rbpf::EbpfVmRaw::new(Some(program)).unwrap();
        vm.override_interpreter(rbpf::InterpreterVariant::Extended);

        middleware::register_helpers(&mut vm, self.registered_helpers.clone());

        let mutex = Mutex::new(mem);

        // Here we need to do some hacking with locks as closures don't like
        // capturing &mut references from environment. It does make sense.
        let (ret, execution_time) =
            self.timed_execution(|| vm.execute_program(mutex.lock().deref_mut()));
        *result = ret;
        execution_time
    }
}
