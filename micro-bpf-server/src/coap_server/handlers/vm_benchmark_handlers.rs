use alloc::format;
use core::convert::TryInto;

use log::{debug, info};

use riot_wrappers::gcoap::PacketBuffer;

use coap_message::{MinimalWritableMessage, MutableWritableMessage, ReadableMessage};

use crate::{
    coap_server::handlers::util::preprocess_request_concrete_impl,
    vm::{construct_vm, timed_vm::BenchmarkResult, TimedVm},
};

use micro_bpf_common::VMExecutionRequest;

use crate::vm::VirtualMachine;

use super::{generic_request_error::GenericRequestError, util};

/// Responsible for benchmarking the VM execution by measuring program size,
/// verification time, (optionally relocation resolution time) and execution time.
pub struct VMExecutionBenchmarkHandler {
    time_results: BenchmarkResult,
    program_size: u32,
    result: i64,
}

impl VMExecutionBenchmarkHandler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            time_results: Default::default(),
            program_size: 0,
            result: 0,
        }
    }

    fn handle_benchmark_execution(&mut self, request: VMExecutionRequest) -> Result<u8, u8> {
        let vm = construct_vm(request.configuration, request.allowed_helpers)
            .map_err(util::internal_server_error)?;

        let mut vm = TimedVm::new(vm);

        self.result = vm.full_run().unwrap() as i64;
        self.time_results = vm.get_results();
        self.program_size = vm.get_program_length() as u32;

        Ok(coap_numbers::code::CHANGED)
    }
}

impl coap_handler::Handler for VMExecutionBenchmarkHandler {
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
            Err(GenericRequestError(parsing_result.unwrap_err()))?
        };

        self.handle_benchmark_execution(request)
            .map_err(|err| GenericRequestError(err))
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
        let results = self.time_results;
        let resp = format!(
            "{{\"total\": {}, \"load\": {}, \"verif\": {}, \"exec\": {},\"prog\": {}, \"result\": {}}}",
            results.total_time,
            results.load_time,
            results.verification_time,
            results.execution_time,
            self.program_size,
            self.result
        );
        response.set_payload(resp.as_bytes())
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
    #[allow(dead_code)]
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
        pkt: PacketBuffer,
    ) -> isize {
        let Ok(vm) = construct_vm(request.configuration, request.allowed_helpers) else {
            return Self::NO_BYTES_WRITTEN;
        };

        let mut vm = TimedVm::new(vm);

        self.program_size = vm.get_program_length() as u32;
        self.payload_written = vm.full_run_on_coap_pkt(pkt).unwrap() as isize;
        self.time_results = vm.get_results();
        self.log_results();
        self.payload_written
    }

    fn log_results(&self) {
        info!("VM Execution benchmark results:");
        info!("Timings: \n{:?}", self.time_results);
        info!("Program size: {} [B]", self.program_size);
        info!("Payload written: {}", self.payload_written);
    }
}

impl riot_wrappers::gcoap::Handler for VMExecutionOnCoapPktBenchmarkHandler {
    fn handle(&mut self, pkt: PacketBuffer) -> isize {
        let Ok(request_str) = preprocess_request_concrete_impl(&pkt) else {
            return Self::NO_BYTES_WRITTEN;
        };

        let Ok(request) = VMExecutionRequest::decode(request_str) else {
            return Self::NO_BYTES_WRITTEN;
        };

        debug!("Received VM Execution Request: {:?}", request.configuration);

        self.handle_benchmark_execution(request, pkt)
    }
}
