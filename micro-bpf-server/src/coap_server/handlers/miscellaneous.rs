use alloc::format;
use coap_message::{MinimalWritableMessage, MutableWritableMessage, ReadableMessage};
use core::{convert::TryInto, ops::DerefMut};
use riot_wrappers::{riot_sys, stdio::println};

use crate::vm::RUNNING_WORKERS;

use super::jit_deploy_handler::GenericRequestError;

pub struct RiotBoardHandler;
impl coap_handler::Handler for RiotBoardHandler {
    type RequestData = u8;
    type ExtractRequestError = GenericRequestError;
    type BuildResponseError<M: MinimalWritableMessage> =
        <M as coap_message::MinimalWritableMessage>::SetPayloadError;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        if request.code().into() != coap_numbers::code::GET {
            return Ok(coap_numbers::code::METHOD_NOT_ALLOWED);
        }
        return Ok(coap_numbers::code::VALID);
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
        println!("Request for the riot board name received");
        let board_name = core::str::from_utf8(riot_sys::RIOT_BOARD)
            .expect("Oddly named board crashed CoAP stack");
        response.set_payload(board_name.as_bytes())
    }
}

pub struct RunningVMHandler;
impl coap_handler::Handler for RunningVMHandler {
    type RequestData = u8;
    type ExtractRequestError = GenericRequestError;
    type BuildResponseError<M: MinimalWritableMessage> =
        <M as coap_message::MinimalWritableMessage>::SetPayloadError;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        if request.code().into() != coap_numbers::code::GET {
            return Ok(coap_numbers::code::METHOD_NOT_ALLOWED);
        }
        return Ok(coap_numbers::code::VALID);
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

        let mut guard = RUNNING_WORKERS.lock();
        let running_workers = guard.deref_mut().clone();
        response.set_payload(format!("{:?}", running_workers).as_bytes())
    }
}

pub struct ConsoleWriteHandler;
impl coap_handler::Handler for ConsoleWriteHandler {
    type RequestData = u8;
    type ExtractRequestError = GenericRequestError;
    type BuildResponseError<M: MinimalWritableMessage> =
        <M as coap_message::MinimalWritableMessage>::SetPayloadError;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        if request.code().into() != coap_numbers::code::POST {
            return Ok(coap_numbers::code::METHOD_NOT_ALLOWED);
        }
        match core::str::from_utf8(request.payload()) {
            Ok(s) => {
                println!("{}", s);
                Ok(coap_numbers::code::CHANGED)
            }
            _ => Ok(coap_numbers::code::BAD_REQUEST),
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
        let result = "Success";
        response.set_payload(result.as_bytes())
    }
}
