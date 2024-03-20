use alloc::vec::Vec;
use log::error;

use super::ALL_HELPERS;

#[derive(Copy, Clone)]
pub struct HelperFunction {
    /// The ID of the helper function that is used by the VM to call the helper.
    /// It should be consistent with the one defined in the C header file with
    /// all the helpers that is used to compile the eBPF programs
    pub id: u32,
    /// The ordinal number of the helper function in the list of all helpers, it
    /// is used for configuring the helpers that are accessible by a given instance
    /// of the VM.
    pub index: u32,
    /// The actual implementation of the helper function, it always accepts 5
    /// arguments and the eBPF calling convention works by putting all arguments
    /// to the function into registers r1 - r5. One thing is that the helper functions
    /// can access all of those 5 registers even if the function doesn't actually
    /// take in all 5 arguments.
    pub function: fn(u64, u64, u64, u64, u64) -> u64,
}

impl HelperFunction {
    pub const fn new(id: u32, index: u32, function: fn(u64, u64, u64, u64, u64) -> u64) -> Self {
        HelperFunction {
            id,
            index,
            function,
        }
    }
}

/// Different versions of the rBPF VM have different implementations of the function
/// for registering helpers, however there is no common trait which encapsulates
/// that functionality. Because of this, when registering helpers for those VMs
/// the register_helper function depends on the type of the VM that we have,
/// this is unfortunate as it doesn't allow for easy swapping of the helpers.
/// Because of this problem, the trait AcceptingHelpers was introduced.
pub trait AcceptingHelpers {
    fn register_helper(&mut self, helper: HelperFunction);
}

/* Implementations of the custom trait for all rBPF VMs */
impl AcceptingHelpers for rbpf::EbpfVmFixedMbuff<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmRaw<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmNoData<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmMbuff<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id, helper.function);
    }
}

/// Registers all helpers provided by Femto-Container VM. Those are library-like
/// functions and are currently unused.
#[allow(dead_code)]
pub fn register_all(vm: &mut impl AcceptingHelpers) {
    for helper in ALL_HELPERS {
        vm.register_helper(helper);
    }
}

#[allow(dead_code)]
pub fn register_helpers(vm: &mut impl AcceptingHelpers, helpers: Vec<HelperFunction>) {
    for helper in helpers {
        vm.register_helper(helper);
    }
}

pub struct HelperFunctionEncoding(pub [u8; 3]);

impl From<Vec<HelperFunction>> for HelperFunctionEncoding {
    fn from(helpers: Vec<HelperFunction>) -> Self {
        let mut encoding = [0; 3];
        for helper in helpers {
            if helper.index > 31 {
                error!("Helper index too large: {}", helper.index);
                return HelperFunctionEncoding(encoding);
            }
            // The first 8 helpers are configured by the first u8, the next
            // by the second one and so on.
            let bucket = (helper.index / 8) as usize;
            encoding[bucket] |= 1 << (helper.index % 8);
        }
        HelperFunctionEncoding(encoding)
    }
}

impl Into<Vec<HelperFunction>> for HelperFunctionEncoding {
    fn into(self) -> Vec<HelperFunction> {
        let mut available_helpers = alloc::vec![];
        for i in 0..ALL_HELPERS.len() {
            let bucket = (i / 8) as usize;
            if self.0[bucket] & (1 << (i % 8)) > 0 {
                available_helpers.push(ALL_HELPERS[i]);
            }
        }
        return available_helpers;
    }
}
