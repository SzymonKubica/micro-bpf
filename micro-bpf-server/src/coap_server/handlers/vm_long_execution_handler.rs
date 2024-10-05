use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::convert::TryInto;
use micro_bpf_elf_utils::resolve_relocations;

use log::{debug, error, info};

use riot_wrappers::{gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys};

use coap_message::{MinimalWritableMessage, MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE,
    model::requests::VMExecutionRequestIPC,
    vm::{construct_vm, timed_vm::BenchmarkResult, TimedVm},
};

use micro_bpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequest};

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

use super::{generic_request_error::GenericRequestError, util};

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
    type ExtractRequestError = GenericRequestError;
    type BuildResponseError<M: MinimalWritableMessage> =
        <M as coap_message::MinimalWritableMessage>::SetPayloadError;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        let parsing_result = util::parse_request(request);
        let Ok(request) = parsing_result else {
            Err(parsing_result.unwrap_err())?
        };

        let message = VMExecutionRequestIPC {
            request: Box::new(request),
        };

        if let Ok(()) = self.execution_send.lock().try_send(message) {
            info!("VM execution request sent successfully");
            self.last_request_successful = true;
            Ok(coap_numbers::code::CHANGED)
        } else {
            error!("Failed to send execution request message.");
            self.last_request_successful = false;
            Err(GenericRequestError(coap_numbers::code::INTERNAL_SERVER_ERROR))
        }
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response<M: MutableWritableMessage>(
        &mut self,
        response: &mut M,
        request: Self::RequestData,
    ) -> Result<(), Self::BuildResponseError<M>> {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        if self.last_request_successful {
            response.set_payload(b"VM Execution request sent successfully!")
        } else {
            response.set_payload(b"Failed to send VM Execution request")
        }
    }
}
