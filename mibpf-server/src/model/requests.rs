use core::ffi::c_void;

use crate::vm::middleware::helpers::{HelperFunction, HelperFunctionEncoding};
use alloc::vec::Vec;
use mibpf_common::{BinaryFileLayout, TargetVM, VMConfiguration, VMExecutionRequestMsg};
use log::debug;
use riot_sys::msg_t;

use serde::{Deserialize, Serialize};

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

impl Into<VMExecutionRequestMsg> for VMExecutionRequest {
    fn into(self) -> VMExecutionRequestMsg {
        VMExecutionRequestMsg {
            configuration: self.configuration.encode(),
            available_helpers: HelperFunctionEncoding::from(self.available_helpers).0,
        }
    }
}

// We turn the DTO struct into a raw u32 value because passing pointers in messages
// doesn't quite work.
impl Into<msg_t> for VMExecutionRequest {
    fn into(mut self) -> msg_t {
        let mut value: u32 = 0;
        let helpers = HelperFunctionEncoding::from(self.available_helpers).0;
        value |= (self.configuration.encode() as u32) << 24;
        for i in 0..3 {
            value |= (helpers[i] as u32) << (8 * (2 - i));
        }
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
        msg.content = riot_sys::msg_t__bindgen_ty_1 { value };
        msg
    }
}

impl From<msg_t> for VMExecutionRequest {
    fn from(msg: msg_t) -> Self {
        let value: u32 = unsafe { msg.content.value };

        let configuration = ((value >> 24) & 0xFF) as u8;
        let mut available_helpers = [0; 3];
        for i in 0..3 {
            available_helpers[i] = ((value >> (8 * (2 - i))) & 0xFF) as u8;
        }

        VMExecutionRequest {
            configuration: VMConfiguration::decode(configuration),
            available_helpers: HelperFunctionEncoding(available_helpers).into(),
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
