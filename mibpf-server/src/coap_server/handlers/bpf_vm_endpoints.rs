use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use core::convert::TryInto;

use log::{debug, error, info};
use serde::Deserialize;
// The riot_sys reimported through the wrappers doesn't seem to work.

use riot_sys;
use riot_wrappers::{
    coap_message::ResponseMessage, gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex,
    stdio::println,
};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage,
    vm::{
        middleware, rbpf_vm::BinaryFileLayout, FemtoContainerVm, RbpfVm, VMExecutionRequest,
        VirtualMachine, VM_EXECUTION_REQUEST_TYPE,
    },
};

/// The handler expects to receive a request that contains a vm_target
/// and the SUIT storage location from where to load the program.
#[derive(Deserialize)]
struct RequestData {
    pub vm_target: VmTarget,
    pub binary_layout: BinaryFileLayout,
    pub suit_location: usize,
}

#[derive(Deserialize)]
enum VmTarget {
    Rbpf,
    FemtoContainer,
}

impl Into<u8> for VmTarget {
    fn into(self) -> u8 {
        match self {
            VmTarget::Rbpf => 0,
            VmTarget::FemtoContainer => 1,
        }
    }
}

impl From<u8> for VmTarget {
    fn from(val: u8) -> Self {
        match val {
            0 => VmTarget::Rbpf,
            1 => VmTarget::FemtoContainer,
            _ => panic!("Unknown VM target: {}", val),
        }
    }
}

/// Executes a chosen eBPF VM while passing in a pointer to the incoming packet
/// to the executed program. The eBPF script can access the CoAP packet data.
struct VMExecutionOnCoapPktHandler {
    execution_time: u32,
    result: i64,
}

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktHandler {
    fn handle(&mut self, pkt: &mut PacketBuffer) -> isize {
        let request_data = self.handle_request(pkt);
        //let mut lengthwrapped = ResponseMessage::new(pkt);
        //self.build_response(&mut lengthwrapped, request_data);
        //let length = lengthwrapped.finish();
        //println!("Response length: {}", length);
        131
    }
}

impl VMExecutionOnCoapPktHandler {
    fn handle_request(&mut self, request: &mut PacketBuffer) -> u8 {
        // Measure the total request processing time.
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        // The SUIT ram storage for the program is 2048 bytes large so we won't
        // be able to load larger images. Hence 2048 byte buffer is sufficient
        let mut program_buffer: [u8; 2048] = [0; 2048];
        let program = suit_storage::load_program(&mut program_buffer, request_data.suit_location);

        println!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.suit_location,
            program.len()
        );

        // Dynamically dispatch between the two different VM implementations
        // depending on the request data.
        let vm: Box<dyn VirtualMachine> = match request_data.vm_target {
            VmTarget::Rbpf => Box::new(RbpfVm::new(
                Vec::from(middleware::ALL_HELPERS),
                request_data.binary_layout,
            )),
            VmTarget::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        self.execution_time = vm.execute_on_coap_pkt(&program, request, &mut self.result);

        let end: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };
        println!("Total request processing time: {} [us]", end - start);
        coap_numbers::code::CHANGED
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
       format_execution_response(self.execution_time, self.result, response, request);
    }
}

pub fn execute_vm_on_coap_pkt() -> impl riot_wrappers::gcoap::Handler {
    VMExecutionOnCoapPktHandler {
        execution_time: 0,
        result: 0,
    }
}

/// Executes a chosen eBPF VM while passing in a pointer to the incoming packet
/// to the executed program. The eBPF script can access the CoAP packet data.
/// Its fields are used to store the results of the most recent VM execution
/// which are then used to construct the CoAP response.
struct VMExecutionNoDataHandler {
    execution_time: u32,
    result: i64,
}

impl coap_handler::Handler for VMExecutionNoDataHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        // The SUIT ram storage for the program is 2048 bytes large so we won't
        // be able to load larger images. Hence 2048 byte buffer is sufficient
        let mut program_buffer: [u8; 2048] = [0; 2048];
        let program = suit_storage::load_program(&mut program_buffer, request_data.suit_location);

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.suit_location,
            program.len()
        );

        // Dynamically dispatch between the two different VM implementations
        // depending on the requested target VM.
        let vm: Box<dyn VirtualMachine> = match request_data.vm_target {
            VmTarget::Rbpf => Box::new(RbpfVm::new(
                Vec::from(middleware::ALL_HELPERS),
                request_data.binary_layout,
            )),
            VmTarget::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        self.execution_time = vm.execute(&program, &mut self.result);

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        format_execution_response(self.execution_time, self.result, response, request);
    }
}

pub fn execute_vm_no_data() -> impl coap_handler::Handler {
    VMExecutionNoDataHandler {
        execution_time: 0,
        result: 0,
    }
}

struct VMLongExecutionHandler {
    execution_send:
        Arc<Mutex<msg::SendPort<crate::vm::VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>>>,
}

impl coap_handler::Handler for VMLongExecutionHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        if let Ok(()) = self.execution_send.lock().try_send(VMExecutionRequest {
            suit_location: request_data.suit_location as u8,
            vm_target: request_data.vm_target.into(),
            binary_layout: request_data.binary_layout.into(),
        }) {
            info!("VM execution request sent successfully");
        } else {
            error!("Failed to send execution request message.");
        }

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        // TODO: add meaningful response
        response.set_payload("VM spawned successfully".as_bytes());
    }
}

pub fn spawn_vm_execution(
    execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequest, 23>>>,
) -> impl coap_handler::Handler {
    VMLongExecutionHandler { execution_send }
}

/* Common utility functions for the handlers */

fn format_execution_response(
    execution_time: u32,
    result: i64,
    response: &mut impl MutableWritableMessage,
    request: u8,
) {
    response.set_code(request.try_into().map_err(|_| ()).unwrap());
    let resp = format!(
        "{{\"execution_time\": {}, \"result\": {}}}",
        execution_time, result
    );
    response.set_payload(resp.as_bytes());
}

fn preprocess_request(request: &impl ReadableMessage) -> Result<RequestData, u8> {
    if request.code().into() != coap_numbers::code::POST {
        return Err(coap_numbers::code::METHOD_NOT_ALLOWED);
    }

    // Request payload determines from which SUIT storage slot we are
    // reading the bytecode.
    let Ok(s) = core::str::from_utf8(request.payload()) else {
        return Err(coap_numbers::code::BAD_REQUEST);
    };

    println!("Request payload received: {}", s);
    let Ok((request_data, _length)): Result<(RequestData, usize), _> = serde_json_core::from_str(s)
    else {
        return Err(coap_numbers::code::BAD_REQUEST);
    };

    Ok(request_data)
}
