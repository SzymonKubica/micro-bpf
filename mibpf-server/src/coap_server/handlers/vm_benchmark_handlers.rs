use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::convert::TryInto;
use mibpf_elf_utils::resolve_relocations;

use log::{debug, error, info};

use riot_wrappers::{gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE,
    model::requests::VMExecutionRequestIPC,
    vm::{construct_vm, timed_vm::BenchmarkResult, TimedVm},
};

use mibpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequest};

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

use super::util;

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

        let (program, mut vm) = construct_vm(
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
            "{{\"load\": {}, \"verif\": {}, \"exec\": {},\"prog\": {}, \"result\": {}}}",
            results.load_time,
            results.verification_time,
            results.execution_time,
            self.program_size,
            self.result
        );
        response.set_payload(resp.as_bytes());
    }
}

/// Responsible for benchmarking the VM execution by measuring program size,
/// verification time, (optionally relocation resolution time) and execution time.
pub struct VMExecutionOnCoapPktBenchmarkHandler {
    time_results: BenchmarkResult,
    program_size: u32,
    payload_written: isize,
}

impl VMExecutionOnCoapPktBenchmarkHandler {
    /// Given that the gcoap::Handler needs to return the length of the
    /// payload + PDU that was written into the packet buffer, in case
    /// of error we need to return 0. It is crucial that all eBPF programs
    /// that work directly on the packet data return the length of the payload that
    /// they have written so that the response can be formatted correctly
    /// and sent back to the client.
    const NO_BYTES_WRITTEN: isize = 0;
    pub fn new() -> Self {
        Self {
            time_results: Default::default(),
            program_size: 0,
            payload_written: 0,
        }
    }

    /// When the VM gets access to the COAP packet, the eBPF program is responsible
    /// for returning the length of the payload that was written into the packet
    /// buffer. This is needed so that the server infrastructure in RIOT knows
    /// how much data has been written and needs to be sent back to the client
    /// (as our handler needs to implement riot_wrappers::gcoap::Handler trait)
    fn handle_benchmark_execution(
        &mut self,
        request: VMExecutionRequest,
        pkt: &mut PacketBuffer,
    ) -> isize {
        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];

        let Ok((program, mut vm)) = construct_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        ) else {
            return Self::NO_BYTES_WRITTEN;
        };

        let mut vm = TimedVm::new(vm);

        self.program_size = program.len() as u32;

        self.payload_written = vm.full_run_on_coap_pkt(program, pkt).unwrap() as isize;
        self.time_results = vm.get_results();
        self.log_results();
        self.payload_written
    }

    fn log_results(&self) {
        debug!("VM Execution benchmark results:");
        debug!("Timings: \n{:?}", self.time_results);
        debug!("Program size: {} [B]", self.program_size);
        debug!("Payload written: {}", self.payload_written);
    }
}

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktBenchmarkHandler {
    fn handle(&mut self, pkt: &mut PacketBuffer) -> isize {
        let Ok(request_str) = preprocess_request_raw(pkt) else {
            return Self::NO_BYTES_WRITTEN;
        };

        let Ok(request) = VMExecutionRequest::decode(request_str) else {
            return Self::NO_BYTES_WRITTEN;
        };

        debug!("Received VM Execution Request: {:?}", request.configuration);

        self.handle_benchmark_execution(request, pkt)
    }
}
