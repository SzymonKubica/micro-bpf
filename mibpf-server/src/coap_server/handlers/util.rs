use alloc::boxed::Box;
use coap_message::ReadableMessage;
use core::convert::TryInto;
use riot_wrappers::gcoap::PacketBuffer;

use log::{debug, info};

use crate::model::requests::VMExecutionRequestMsg;

// This module contains common utility functions that are used by the handler
// implementations for all of the endpoints.

/// Allows for timing the request processing duration in CoAP servers.
/// It is initialised with a dynamic implementation of another endpoint handler
/// and then it handles request by timing the execution of the handle method
/// of the wrapped handler. It can be used to compose endpoints and quickly
/// toggle the timing functionality on and off. The request processing time is
/// then printed to the RIOT console.
pub struct TimedHandler<'a> {
    handler: &'a mut dyn riot_wrappers::gcoap::Handler,
}

impl<'a> TimedHandler<'a> {
    pub fn new(handler: &'a mut impl riot_wrappers::gcoap::Handler) -> Self {
        Self { handler }
    }
}

impl riot_wrappers::gcoap::Handler for TimedHandler<'_> {
    fn handle(&mut self, pkt: &mut PacketBuffer) -> isize {
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };

        let payload_len = self.handler.handle(pkt);

        let end: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };
        info!("Total request processing time: {} [us]", end - start);

        return payload_len;
    }
}

pub fn preprocess_request<'a, T>(request: &'a impl ReadableMessage) -> Result<T, u8>
where T: serde::de::Deserialize<'a> {
    if request.code().into() != coap_numbers::code::POST {
        return Err(coap_numbers::code::METHOD_NOT_ALLOWED);
    }

    // Request payload determines from which SUIT storage slot we are
    // reading the bytecode.
    let Ok(s) = core::str::from_utf8(request.payload()) else {
        return Err(coap_numbers::code::BAD_REQUEST);
    };

    debug!("Request payload received: {}", s);
    let Ok((request_data, _length)): Result<(T, usize), _> =
        serde_json_core::from_str(s)
    else {
        return Err(coap_numbers::code::BAD_REQUEST);
    };

    Ok(request_data)
}
