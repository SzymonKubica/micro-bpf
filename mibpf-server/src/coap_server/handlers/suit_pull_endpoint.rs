use alloc::{format, string::String, vec::Vec};
use core::{convert::TryInto, fmt};
use serde::{Deserialize, Serialize};

use coap_handler_implementations::SimpleRendered;
use coap_message::{MessageOption, MutableWritableMessage, ReadableMessage};
use riot_wrappers::{cstr::cstr, stdio::println, ztimer::Clock};
// The riot_sys reimported through the wrappers doesn't seem to work.
use riot_sys;

use crate::{
    infra::suit_storage,
    rbpf,
    rbpf::helpers,
};
pub struct SuitPullHandler {
    last_fetched_manifest: Option<String>,
}

/// The handler expects to get a request which consists of the IPv6 address of
/// the machine running the CoAP fileserver and the name of the manifest file
/// specifying which binary needs to be pulled.
#[derive(Serialize, Deserialize, Debug)]
struct SuitPullRequest<'a> {
    pub ip_addr: &'a str,
    pub manifest: &'a str,
}

impl coap_handler::Handler for SuitPullHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        if request.code().into() != coap_numbers::code::POST {
            return coap_numbers::code::METHOD_NOT_ALLOWED;
        }

        // Request payload determines from which SUIT manifest is used to fetch
        // the program image. It also contains the host ip address.
        let Ok(s) = core::str::from_utf8(request.payload()) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        println!("Request payload received: {}", s);

        let Ok((request_data, length)): Result<(SuitPullRequest, usize), _> =
            serde_json_core::from_str(s)
        else {
            return coap_numbers::code::BAD_REQUEST;
        };

        suit_storage::suit_fetch(request_data.ip_addr, request_data.manifest);

        self.last_fetched_manifest = Some(String::from(request_data.manifest));

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
    SuitPullHandler {
        last_fetched_manifest: None,
    }
}
