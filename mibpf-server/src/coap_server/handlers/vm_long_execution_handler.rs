use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::convert::TryInto;
use mibpf_elf_utils::resolve_relocations;

use log::{debug, error, info};

use riot_wrappers::{gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE,
    model::requests::VMExecutionRequestIPC,
    vm::{construct_vm, timed_vm::BenchmarkResult, TimedVm},
};

use mibpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequest};

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

use super::util;

pub struct VMLongExecutionHandler {
    execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
    last_request_successful: bool,
}

impl VMLongExecutionHandler {
    pub fn new(
        execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
    ) -> Self {
        Self {
            execution_send,
            last_request_successful: false,
        }
    }
}

impl coap_handler::Handler for VMLongExecutionHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let parsing_result = util::parse_request(request);
        let Ok(request) = parsing_result else {
            return parsing_result.unwrap_err();
        };

        let message = VMExecutionRequestIPC {
            request: Box::new(request),
        };

        if let Ok(()) = self.execution_send.lock().try_send(message) {
            info!("VM execution request sent successfully");
            self.last_request_successful = true;
            coap_numbers::code::CHANGED
        } else {
            error!("Failed to send execution request message.");
            self.last_request_successful = false;
            coap_numbers::code::INTERNAL_SERVER_ERROR
        }
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        if self.last_request_successful {
            response.set_payload(b"VM Execution request sent successfully!");
        } else {
            response.set_payload(b"Failed to send VM Execution request");
        }
    }
}