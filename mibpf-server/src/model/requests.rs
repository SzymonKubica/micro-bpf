use core::ffi::c_void;

use crate::vm::middleware::helpers::HelperFunction;
use alloc::{vec::Vec, boxed::Box};
use log::debug;
use mibpf_common::{BinaryFileLayout, TargetVM, VMConfiguration, VMExecutionRequestMsg};
use riot_sys::msg_t;

use serde::{Deserialize, Serialize};

/// Models a request to start an execution of a given instance of a eBPF VM,
/// it specifies the configuration of the VM instance and the list of helper
/// functions that should be made available to the program running in the VM.
pub struct VMExecutionRequest {
    pub configuration: VMConfiguration,
    pub available_helpers: Vec<HelperFunction>,
}

pub struct IPCExecutionMessage {
    pub request: Box<VMExecutionRequest>,
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
            available_helpers: request
                .available_helpers
                .iter()
                .map(|v| HelperFunction::from(*v))
                .collect::<Vec<HelperFunction>>(),
        }
    }
}

impl Into<VMExecutionRequestMsg> for VMExecutionRequest {
    fn into(self) -> VMExecutionRequestMsg {
        let helper_functions = self.available_helpers.iter().map(|v| (*v).into()).collect::<Vec<u8>>();
        VMExecutionRequestMsg {
            configuration: self.configuration.encode(),
            available_helpers: helper_functions,
        }
    }
}

// We turn the DTO struct into a raw u32 value because passing pointers in messages
// doesn't quite work.
impl Into<msg_t> for &mut VMExecutionRequest {
    fn into(self) -> msg_t {
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
        msg.content = riot_sys::msg_t__bindgen_ty_1 {
            ptr: self as *mut VMExecutionRequest as *mut c_void,
        };
        msg
    }
}

impl From<msg_t> for VMExecutionRequest {
    fn from(msg: msg_t) -> Self {
        let ptr: *mut c_void = unsafe { msg.content.ptr };

        let req_ptr = ptr as *mut VMExecutionRequest;

        unsafe {
            VMExecutionRequest {
                configuration: (*req_ptr).configuration,
                available_helpers: (*req_ptr).available_helpers.clone(),
            }
        }
    }
}

/// Responsible for notifying the VM manager that the execution of a given
/// VM is finished and the worker can be allocated a new job.
#[derive(Debug, Clone)]
pub struct VMExecutionCompleteMsg {
    pub worker_pid: i16,
}

impl VMExecutionCompleteMsg {
    pub fn new(worker_pid: i16) -> Self {
        VMExecutionCompleteMsg { worker_pid }
    }
}
