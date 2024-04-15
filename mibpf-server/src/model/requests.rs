use core::ffi::c_void;

use crate::vm::middleware::helpers::{HelperAccessList, HelperFunction};
use alloc::{boxed::Box, vec::Vec};
use log::debug;
use mibpf_common::{BinaryFileLayout, TargetVM, VMConfiguration, VMExecutionRequest};
use riot_sys::msg_t;

use serde::{Deserialize, Serialize};

/// Wrapper around the [`mibpf_common::VMExecutionRequest`] to allow for sending
/// it over the RIOT IPC.
pub struct VMExecutionRequestIPC {
    pub request: Box<VMExecutionRequest>,
}

impl Into<msg_t> for &mut VMExecutionRequestIPC {
    fn into(self) -> msg_t {
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
        msg.content = riot_sys::msg_t__bindgen_ty_1 {
            ptr: self.request.as_mut() as *mut VMExecutionRequest as *mut c_void,
        };
        msg
    }
}

impl From<msg_t> for VMExecutionRequestIPC {
    fn from(msg: msg_t) -> Self {
        let ptr: *mut c_void = unsafe { msg.content.ptr };

        let req_ptr = ptr as *mut VMExecutionRequest;

        unsafe {
            return VMExecutionRequestIPC {
                request: Box::new(VMExecutionRequest {
                    configuration: (*req_ptr).configuration,
                    allowed_helpers: (*req_ptr).allowed_helpers.clone(),
                }),
            };
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
