use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use coap_handler_implementations::SimpleRendered;
use coap_message::{MessageOption, MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use core::fmt;
use riot_wrappers::{cstr::cstr, stdio::println, ztimer::Clock};

use crate::rbpf;
use crate::rbpf::helpers;
// The riot_sys reimported through the wrappers doesn't seem to work.
use riot_sys;

pub struct SuitPullHandler {}

impl coap_handler::Handler for SuitPullHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        extern "C" {
            /// Responsible for loading the bytecode from the SUIT ram storage.
            /// The application bytes are written into the buffer.
            fn initiate_suit_fetch(adderss: *const u8, signed_manifest_name: *const u8);
        }

        if request.code().into() != coap_numbers::code::POST {
            return coap_numbers::code::METHOD_NOT_ALLOWED;
        }

        // Request payload determines from which SUIT manifest is used to fetch
        // the program image. It also contains the host ip address.
        let Ok(s) = core::str::from_utf8(request.payload()) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        println!("Request payload received: {}", s);

        // For now the payload is in the format ip-address;suit-manifest-name
        // TODO: use proper deserialization using serde.

        let mut parts = s.split(";");
        let ip_addr = format!("{}\0", parts.next().unwrap());
        let suit_manifest = format!("{}\0", parts.next().unwrap());

        unsafe {
            initiate_suit_fetch(ip_addr.as_ptr(), suit_manifest.as_ptr());
        };

        coap_numbers::code::CHANGED
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
        response.set_payload(b"Success");
    }
}

pub fn handle_suit_pull_request() -> impl coap_handler::Handler {
    SuitPullHandler {}
}
