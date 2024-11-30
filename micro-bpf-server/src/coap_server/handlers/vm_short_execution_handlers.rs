use alloc::format;
use core::convert::TryInto;

use log::{debug, error};

use riot_wrappers::gcoap::PacketBuffer;

use coap_message::{MinimalWritableMessage, MutableWritableMessage, ReadableMessage};

use crate::
    vm::construct_vm
;

use micro_bpf_common::VMExecutionRequest;

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
};

use super::{generic_request_error::GenericRequestError, util};

/// Executes a chosen eBPF VM while passing in a pointer to the incoming packet
/// to the executed program. The eBPF script can access the CoAP packet data.
pub struct VMExecutionOnCoapPktHandler;

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktHandler {
    fn handle(&mut self, pkt: PacketBuffer) -> isize {
        /// Given that the gcoap::Handler needs to return the length of the
        /// payload + PDU that was written into the packet buffer, in case
        /// of error we need to return 0. It is crucial that all eBPF programs
        /// that work directly on the packet data return the length of the payload that
        /// they have written so that the response can be formatted correctly
        /// and sent back to the client.
        const NO_BYTES_WRITTEN: isize = 0;

        let Ok(request_str) = preprocess_request_raw(&pkt) else {
            return NO_BYTES_WRITTEN;
        };

        let Ok(request) = VMExecutionRequest::decode(request_str) else {
            return NO_BYTES_WRITTEN;
        };

        debug!("Received VM Execution Request: {:?}", request.configuration);

        let init_result = construct_vm(request.configuration, request.allowed_helpers);

        let Ok(mut vm) = init_result else {
            error!(
                "Failed to initialize the VM: {}",
                init_result.err().unwrap()
            );
            return NO_BYTES_WRITTEN;
        };

        // It is very important that the program executing on the CoAP packet returns
        // the length of the payload + PDU so that the handler can send the
        // response accordingly. In case of error the response length should be set to 0.
        vm.full_run_on_coap_pkt(pkt).unwrap_or_else(|e| {
            debug!("Error: {:?}", e);
            0
        }) as isize
    }
}

// Allows for executing an instance of the eBPF VM directly in the CoAP server
// request handler callback. It stores the return value
// of the program so that it can format the CoAP response accordingly.
pub struct VMExecutionNoDataHandler {
    result: i64,
}

impl VMExecutionNoDataHandler {
    pub fn new() -> Self {
        Self { result: 0 }
    }

    fn handle_vm_execution(&mut self, request: VMExecutionRequest) -> Result<u8, u8> {
        let mut vm = construct_vm(request.configuration, request.allowed_helpers)
            .map_err(util::internal_server_error)?;

        self.result = vm.full_run().unwrap() as i64;
        Ok(coap_numbers::code::CHANGED)
    }
}

impl coap_handler::Handler for VMExecutionNoDataHandler {
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
            return Err(GenericRequestError(parsing_result.unwrap_err()));
        };
        match self.handle_vm_execution(request) {
            Ok(code) => Ok(code),
            Err(code) => Ok(code),
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
        let resp = format!("{{\"result\": {}}}", self.result);
        response.set_payload(resp.as_bytes())
    }
}
