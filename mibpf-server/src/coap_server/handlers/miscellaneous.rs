use coap_message::{MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use riot_wrappers::{riot_sys, stdio::println};

pub struct RiotBoardHandler;
impl coap_handler::Handler for RiotBoardHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        if request.code().into() != coap_numbers::code::GET {
            return coap_numbers::code::METHOD_NOT_ALLOWED;
        }
        return coap_numbers::code::VALID;
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(
        &mut self,
        response: &mut impl MutableWritableMessage,
        request: Self::RequestData,
    ) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        println!("Request for the riot board name received");
        let board_name = core::str::from_utf8(riot_sys::RIOT_BOARD)
            .expect("Oddly named board crashed CoAP stack");
        response.set_payload(board_name.as_bytes());
    }
}

pub struct ConsoleWriteHandler;
impl coap_handler::Handler for ConsoleWriteHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        if request.code().into() != coap_numbers::code::POST {
            return coap_numbers::code::METHOD_NOT_ALLOWED;
        }
        match core::str::from_utf8(request.payload()) {
            Ok(s) => {
                println!("{}", s);
                coap_numbers::code::CHANGED
            }
            _ => coap_numbers::code::BAD_REQUEST,
        }
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(
        &mut self,
        response: &mut impl MutableWritableMessage,
        request: Self::RequestData,
    ) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let result = "Success";
        response.set_payload(result.as_bytes());
    }
}
