use core::num::ParseIntError;

use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use log::error;

use super::ALL_HELPERS;
use mibpf_common::HelperFunctionID;

#[derive(Copy, Clone)]
pub struct HelperFunction {
    /// The ID of the helper function that is used by the VM to call the helper.
    /// It should be consistent with the one defined in the C header file with
    /// all the helpers that is used to compile the eBPF programs
    pub id: HelperFunctionID,
    /// The actual implementation of the helper function, it always accepts 5
    /// arguments and the eBPF calling convention works by putting all arguments
    /// to the function into registers r1 - r5. One thing is that the helper functions
    /// can access all of those 5 registers even if the function doesn't actually
    /// take in all 5 arguments.
    pub function: fn(u64, u64, u64, u64, u64) -> u64,
}

impl HelperFunction {
    pub const fn new(id: HelperFunctionID, function: fn(u64, u64, u64, u64, u64) -> u64) -> Self {
        HelperFunction { id, function }
    }
}

pub struct HelperAccessList(pub Vec<HelperFunction>);

impl From<String> for HelperAccessList {
    fn from(value: String) -> Self {
        let allowed_helpers_ids = (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()
            .map_err(|e| format!("Unable to parse: {}", e))
            .unwrap();

        HelperAccessList::from(allowed_helpers_ids)
    }
}

impl From<Vec<u8>> for HelperAccessList {
    fn from(value: Vec<u8>) -> Self {
        let allowed_helpers: Vec<HelperFunctionID> = value
            .into_iter()
            .filter_map(|id| num::FromPrimitive::from_u8(id))
            .collect();
        HelperAccessList::from(allowed_helpers)
    }
}

/// We need to implement this so that it is possible to map from a list of
/// helper function IDs to the actual list of function pointers.
impl From<Vec<HelperFunctionID>> for HelperAccessList {
    fn from(value: Vec<HelperFunctionID>) -> Self {
        let helper_map = ALL_HELPERS
            .iter()
            .map(|h| (h.id, h.clone()))
            .collect::<BTreeMap<HelperFunctionID, HelperFunction>>();

        let helpers = value
            .iter()
            .map(|v| helper_map.get(v).unwrap().clone())
            .collect::<Vec<HelperFunction>>();
        HelperAccessList(helpers)
    }
}

impl Into<u8> for HelperFunction {
    fn into(self) -> u8 {
        return self.id as u8;
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
        let _ = self.register_helper(helper.id.into(), helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmRaw<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id.into(), helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmNoData<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id.into(), helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmMbuff<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        let _ = self.register_helper(helper.id.into(), helper.function);
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
