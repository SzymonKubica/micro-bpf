use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
    sync::Arc,
};
use mibpf_elf_utils::resolve_relocations;
use core::convert::TryInto;
use goblin::{
    container::{Container, Endian},
    elf::{Elf, Reloc},
};

use log::{debug, error, info};
use serde::Deserialize;

use riot_wrappers::{
    coap_message::ResponseMessage, gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys,
    stdio::println,
};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::model::requests::VMExecutionRequest;

use mibpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequestMsg};

use crate::{
    coap_server::handlers::util::preprocess_request,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

/// Executes a chosen eBPF VM while passing in a pointer to the incoming packet
/// to the executed program. The eBPF script can access the CoAP packet data.
pub struct VMExecutionOnCoapPktHandler;

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktHandler {
    fn handle(&mut self, pkt: &mut PacketBuffer) -> isize {
        let Ok(request_data) = preprocess_request(pkt) else {
            return 0;
        };

        let request_data = VMExecutionRequest::from(&request_data);

        debug!(
            "Received request to execute VM with config: {:?}",
            request_data.configuration
        );

        const SUIT_STORAGE_SLOT_SIZE: usize = 2048;
        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
        let mut program =
            suit_storage::load_program(&mut program_buffer, request_data.configuration.suit_slot);

        if request_data.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
            // We need to perform relocations on the raw object file.
            match resolve_relocations(&mut program) {
                Ok(()) => {}
                Err(e) => {
                    debug!("Error resolving relocations in the program: {}", e);
                    return 0;
                }
            };
        }

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.configuration.suit_slot,
            program.len()
        );

        let mut vm: Box<dyn VirtualMachine> = match request_data.configuration.vm_target {
            TargetVM::Rbpf => {
                // When executing on a CoAP packet, the VM needs to have access
                // to the CoAP helpers plus any additional helpers specified by
                // the user.
                let mut helpers = Vec::from(middleware::COAP_HELPERS);
                helpers.append(&mut request_data.available_helpers.clone());
                Box::new(RbpfVm::new(
                    program,
                    helpers,
                    request_data.configuration.binary_layout,
                ))
            }
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {program }),
        };

        // It is very important that the program executing on the CoAP packet returns
        // the length of the payload + PDU so that the handler can send the
        // response accordingly.
        let mut payload_length = 0;
        let execution_time = vm.execute_on_coap_pkt(pkt, &mut payload_length);

        // The eBPF program needs to return the length of the Payload + PDU
        payload_length as isize
    }
}

// Allows for executing an instance of the eBPF VM directly in the CoAP server
// request handler callback. It stores the execution time and return value
// of the program so that it can format the CoAP response with those values accordingly.
pub struct VMExecutionNoDataHandler {
    execution_time: u32,
    result: i64,
}

impl VMExecutionNoDataHandler {
    pub fn new() -> Self {
        Self {
            execution_time: 0,
            result: 0,
        }
    }
}

impl coap_handler::Handler for VMExecutionNoDataHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let request_data = VMExecutionRequest::from(&request_data);

        // The SUIT ram storage for the program is 2048 bytes large so we won't
        // be able to load larger images. Hence 2048 byte buffer is sufficient
        let mut program_buffer: [u8; 2048] = [0; 2048];
        let mut program =
            suit_storage::load_program(&mut program_buffer, request_data.configuration.suit_slot);

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.configuration.suit_slot,
            program.len()
        );

        if request_data.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
            // We need to perform relocations on the raw object file.
            match resolve_relocations(&mut program) {
                Ok(()) => {}
                Err(e) => {
                    debug!("Error resolving relocations in the program: {}", e);
                    return 0;
                }
            };
        }

        // Dynamically dispatch between the two different VM implementations
        // depending on the requested target VM.
        let mut vm: Box<dyn VirtualMachine> = match request_data.configuration.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                    program,
                Vec::from(middleware::ALL_HELPERS),
                request_data.configuration.binary_layout,
            )),
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {program}),
        };

        self.execution_time = vm.execute(&mut self.result);

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!(
            "{{\"execution_time\": {}, \"result\": {}}}",
            self.execution_time, self.result
        );
        response.set_payload(resp.as_bytes());
    }
}

pub struct VMLongExecutionHandler {
    execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestMsg, {VM_EXEC_REQUEST}>>>,
}

impl VMLongExecutionHandler {
    pub fn new(
        execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestMsg, {VM_EXEC_REQUEST}>>>,
    ) -> Self {
        Self { execution_send }
    }
}

impl coap_handler::Handler for VMLongExecutionHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        if let Ok(()) = self.execution_send.lock().try_send(request_data) {
            info!("VM execution request sent successfully");
        } else {
            error!("Failed to send execution request message.");
            return coap_numbers::code::INTERNAL_SERVER_ERROR;
        }

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        response.set_payload(b"VM Execution request sent successfully!")
    }
}

pub struct VMExecutionBenchmarkHandler {
    load_time: u32,
    execution_time: u32,
    program_size: u32,
    result: i64,
}

impl VMExecutionBenchmarkHandler {
    pub fn new() -> Self {
        Self {
            execution_time: 0,
            result: 0,
            load_time: 0,
            program_size: 0,
        }
    }

    #[inline(always)]
    fn time_now(clock: *mut riot_sys::inline::ztimer_clock_t) -> u32 {
        unsafe { riot_sys::inline::ztimer_now(clock) }
    }
}

impl coap_handler::Handler for VMExecutionBenchmarkHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let preprocessing_result = preprocess_request(request);
        let request_data = match preprocessing_result {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let request_data = VMExecutionRequest::from(&request_data);

        // The SUIT ram storage for the program is 2048 bytes large so we won't
        // be able to load larger images. Hence 2048 byte buffer is sufficient
        let mut program_buffer: [u8; 2048] = [0; 2048];
        let mut program =
            suit_storage::load_program(&mut program_buffer, request_data.configuration.suit_slot);

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.configuration.suit_slot,
            program.len()
        );
        self.program_size = program.len() as u32;


        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = Self::time_now(clock);
        if request_data.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
            // We need to perform relocations on the raw object file.
            match resolve_relocations(&mut program) {
                Ok(()) => {}
                Err(e) => {
                    debug!("Error resolving relocations in the program: {}", e);
                    return 0;
                }
            };
        }
        let end: u32 = Self::time_now(clock);
        self.load_time = end - start;

        // Dynamically dispatch between the two different VM implementations
        // depending on the requested target VM.
        let mut vm: Box<dyn VirtualMachine> = match request_data.configuration.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                program,
                Vec::from(middleware::ALL_HELPERS),
                request_data.configuration.binary_layout,
            )),
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm { program }),
        };

        self.execution_time = vm.execute(&mut self.result);

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!(
            "{{\"load_time\": {}, \"execution_time\": {},\"program_size\": {}, \"result\": {}}}",
            self.load_time, self.execution_time, self.program_size, self.result
        );
        response.set_payload(resp.as_bytes());
    }
}
