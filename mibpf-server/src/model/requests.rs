use core::ffi::c_void;

use crate::{
    model::enumerations::BinaryFileLayout,
    model::enumerations::TargetVM,
    vm::middleware::helpers::{HelperFunction, HelperFunctionEncoding},
};
use alloc::vec::Vec;
use log::debug;
use riot_sys::msg_t;
use serde::{Deserialize, Serialize};

use super::enumerations::VMConfiguration;

/// Models a request to start an execution of a given instance of a eBPF VM,
/// it specifies the configuration of the VM instance and the list of helper
/// functions that should be made available to the program running in the VM.
pub struct VMExecutionRequest {
    pub configuration: VMConfiguration,
    pub available_helpers: Vec<HelperFunction>,
}

impl VMExecutionRequest {
    pub fn new(suit_location: usize, vm_target: TargetVM, binary_layout: BinaryFileLayout) -> Self {
        VMExecutionRequest {
            configuration: VMConfiguration::new(vm_target, binary_layout, suit_location),
            available_helpers: Vec::new(),
        }
    }
}

impl From<&VMExecutionRequestMsg> for VMExecutionRequest {
    fn from(request: &VMExecutionRequestMsg) -> Self {
        VMExecutionRequest {
            configuration: VMConfiguration::decode(request.configuration),
            available_helpers: HelperFunctionEncoding(request.available_helpers).into(),
        }
    }
}

/// TODO: update documentation comment
/// Represents a request to execute an eBPF program on a particular VM. The
/// suit_location is the index of the SUIT storage slot from which the program
/// should be loaded. For instance, 0 corresponds to '.ram.0'. The vm_target
/// specifies which implementation of the VM should be used (FemtoContainers or
/// rBPF). 0 corresponds to rBPF and 1 corresponds to FemtoContainers. The
/// reason an enum isn't used here is that this struct is sent in messages via
/// IPC api and adding an enum there resulted in the struct being too large to
/// send. It also specifies the binary layout format that the VM should expect
/// in the loaded program
///
/// It also specifies the helpers that the VM should be allowed to call, given
/// that there are currently 24 available helper functions, we use an u32 to
/// specify which ones are allowed.
#[derive(Clone, Serialize, Deserialize)]
#[repr(C, packed)]
pub struct VMExecutionRequestMsg {
    pub configuration: u8,
    pub available_helpers: [u8; 3],
}

impl Into<msg_t> for VMExecutionRequestMsg {
    fn into(mut self) -> msg_t {
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
        // The content of the message specifies which SUIT slot to load from
        msg.content = riot_sys::msg_t__bindgen_ty_1 {
            ptr: &mut self as *mut VMExecutionRequestMsg as *mut c_void,
        };
        msg
    }
}

impl From<msg_t> for &VMExecutionRequestMsg {
    fn from(msg: msg_t) -> Self {
        let execution_request_ptr: *mut VMExecutionRequestMsg =
            unsafe { msg.content.ptr as *mut VMExecutionRequestMsg };
        unsafe { &*execution_request_ptr }
    }
}

// We need to implement Drop for the execution request so that it can be
// dealocated after it is decoded an processed in the message channel.
impl Drop for VMExecutionRequestMsg {
    fn drop(&mut self) {
        debug!("Dropping execution request message now.");
    }
}

/// Responsible for notifying the VM manager that the execution of a given
/// VM is finished and the worker can be allocated a new job.
#[derive(Debug, Clone)]
pub struct VMExecutionCompleteMsg {
    pub worker_pid: i16,
}
