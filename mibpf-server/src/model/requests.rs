use core::ffi::c_void;

use crate::vm::middleware::helpers::{HelperFunction, HelperAccessList};
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
    pub allowed_helpers: Vec<HelperFunction>,
}

pub struct IPCExecutionMessage {
    pub request: Box<VMExecutionRequest>,
}

impl From<&VMExecutionRequestMsg> for VMExecutionRequest {
    fn from(request: &VMExecutionRequestMsg) -> Self {
        VMExecutionRequest {
            configuration: VMConfiguration::decode(request.configuration),
            allowed_helpers: HelperAccessList::from(request.allowed_helpers.clone()).0,
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
                allowed_helpers: (*req_ptr).allowed_helpers.clone(),
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
