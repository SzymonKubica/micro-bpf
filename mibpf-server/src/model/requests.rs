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

/// Encoded transfer object representing a request to start a given execution
/// of the eBPF VM. It contains the encoded configuration of the vm as well as
/// a bitstring (in a form of 3 u8s) specifying which helper functions can be
/// called by the program running in the VM.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VMExecutionRequestMsg {
    pub configuration: u8,
    pub available_helpers: [u8; 3],
}

impl Into<msg_t> for VMExecutionRequestMsg {
    fn into(mut self) -> msg_t {
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
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
    fn drop(&mut self) {}
}

/// Responsible for notifying the VM manager that the execution of a given
/// VM is finished and the worker can be allocated a new job.
#[derive(Debug, Clone)]
pub struct VMExecutionCompleteMsg {
    pub worker_pid: i16,
}
