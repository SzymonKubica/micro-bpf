use core::ffi::c_void;

use crate::{model::enumerations::BinaryFileLayout, model::enumerations::TargetVM};
use log::debug;
use riot_sys::msg_t;
use serde::{Deserialize, Serialize};

/// Models a request to start an execution of a given instance of a eBPF VM,
/// specifies the target implementation of the VM, the layout of the binary that
/// the VM should expect and the SUIT storage location from where the binary
/// should be loaded. It also specifies the list of helper functions that
/// can be used by the VM.
#[derive(Serialize, Deserialize)]
pub struct VMExecutionRequest {
    pub vm_target: TargetVM,
    pub binary_layout: BinaryFileLayout,
    pub suit_slot: usize,
}

impl VMExecutionRequest {
    pub fn new(suit_location: usize, vm_target: TargetVM, binary_layout: BinaryFileLayout) -> Self {
        VMExecutionRequest {
            suit_slot: suit_location,
            vm_target,
            binary_layout,
        }
    }
}

impl From<&VMExecutionRequestMsg> for VMExecutionRequest {
    fn from(request: &VMExecutionRequestMsg) -> Self {
        VMExecutionRequest {
            suit_slot: request.suit_slot as usize,
            vm_target: TargetVM::from(request.vm_target),
            binary_layout: BinaryFileLayout::from(request.binary_layout),
        }
    }
}

/// Represents a request to execute an eBPF program on a particular VM. The
/// suit_location is the index of the SUIT storage slot from which the program
/// should be loaded. For instance, 0 corresponds to '.ram.0'. The vm_target
/// specifies which implementation of the VM should be used (FemtoContainers or
/// rBPF). 0 corresponds to rBPF and 1 corresponds to FemtoContainers. The
/// reason an enum isn't used here is that this struct is send in messages via
/// IPC api and adding an enum there resulted in the struct being too large to
/// send. It also specifies the binary layout format that the VM should expect
/// in the loaded program
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct VMExecutionRequestMsg {
    pub vm_target: u8,
    pub binary_layout: u8,
    pub suit_slot: u8,
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
        debug!("Dropping {:?} now.", self);
    }
}
