use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::convert::TryInto;
use mibpf_elf_utils::resolve_relocations;

use log::{debug, error, info};

use riot_wrappers::{gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE,
    model::requests::VMExecutionRequestIPC,
    vm::{initialize_vm, timed_vm::BenchmarkResult, TimedVm},
};

use mibpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequest};

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

use super::util;

/// Executes a chosen eBPF VM while passing in a pointer to the incoming packet
/// to the executed program. The eBPF script can access the CoAP packet data.
pub struct VMExecutionOnCoapPktHandler;

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktHandler {
    fn handle(&mut self, pkt: &mut PacketBuffer) -> isize {
        /// Given that the gcoap::Handler needs to return the length of the
        /// payload + PDU that was written into the packet buffer, in case
        /// of error we need to return 0. It is crucial that all eBPF programs
        /// that work directly on the packet data return the length of the payload that
        /// they have written so that the response can be formatted correctly
        /// and sent back to the client.
        const NO_BYTES_WRITTEN: isize = 0;

        let Ok(request_str) = preprocess_request_raw(pkt) else {
            return NO_BYTES_WRITTEN;
        };

        let Ok(request) = VMExecutionRequest::decode(request_str) else {
            return NO_BYTES_WRITTEN;
        };

        debug!("Received VM Execution Request: {:?}", request.configuration);

        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
        let init_result = initialize_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        );

        let Ok((program, mut vm)) = init_result else {
            error!(
                "Failed to initialize the VM: {}",
                init_result.err().unwrap()
            );
            return NO_BYTES_WRITTEN;
        };

        // It is very important that the program executing on the CoAP packet returns
        // the length of the payload + PDU so that the handler can send the
        // response accordingly. In case of error the response length should be set to 0.
        vm.full_run_on_coap_pkt(program, pkt).unwrap_or(0) as isize
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
        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];

        let (program, mut vm) = initialize_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        )
        .map_err(util::internal_server_error)?;

        self.result = vm.full_run(program).unwrap() as i64;
        Ok(coap_numbers::code::CHANGED)
    }
}

impl coap_handler::Handler for VMExecutionNoDataHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let parsing_result = util::parse_request(request);
        let Ok(request) = parsing_result else {
            return parsing_result.unwrap_err();
        };
        match self.handle_vm_execution(request) {
            Ok(code) => code,
            Err(code) => code,
        }
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!("{{\"return\": {}}}", self.result);
        response.set_payload(resp.as_bytes());
    }
}

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

/// Responsible for benchmarking the VM execution by measuring program size,
/// verification time, (optionally relocation resolution time) and execution time.
pub struct VMExecutionBenchmarkHandler {
    time_results: BenchmarkResult,
    program_size: u32,
    result: i64,
}

impl VMExecutionBenchmarkHandler {
    pub fn new() -> Self {
        Self {
            time_results: Default::default(),
            program_size: 0,
            result: 0,
        }
    }

    fn handle_benchmark_execution(&mut self, request: VMExecutionRequest) -> Result<u8, u8> {
        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];

        let (program, mut vm) = initialize_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        )
        .map_err(util::internal_server_error)?;

        let mut vm = TimedVm::new(vm);

        self.program_size = program.len() as u32;

        self.result = vm.full_run(program).unwrap() as i64;
        self.time_results = vm.get_results();

        Ok(coap_numbers::code::CHANGED)
    }
}

impl coap_handler::Handler for VMExecutionBenchmarkHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let parsing_result = util::parse_request(request);
        let Ok(request) = parsing_result else {
            return parsing_result.unwrap_err();
        };

        match self.handle_benchmark_execution(request) {
            Ok(code) => code,
            Err(code) => code,
        }
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let results = self.time_results;
        let resp = format!(
            "{{\"reloc\": {}, \"load\": {}, \"verif\": {}, \"exec\": {},\"prog\": {}, \"res\": {}}}",
            results.relocation_resolution_time, results.load_time, results.verification_time, results.execution_time, self.program_size, self.result
        );
        response.set_payload(resp.as_bytes());
    }
}
