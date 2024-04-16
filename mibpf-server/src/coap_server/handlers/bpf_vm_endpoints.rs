use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::convert::TryInto;
use mibpf_elf_utils::resolve_relocations;

use log::{debug, error, info};

use riot_wrappers::{
    gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys,
};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE, model::requests::VMExecutionRequestIPC,
    vm::initialize_vm,
};

use mibpf_common::{BinaryFileLayout, TargetVM, VMExecutionRequest};

use crate::{
    coap_server::handlers::util::preprocess_request_raw,
    infra::suit_storage,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine, VM_EXEC_REQUEST},
};

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

        let Ok(mut vm) = init_result else {
            error!(
                "Failed to initialize the VM: {}",
                init_result.err().unwrap()
            );
            return NO_BYTES_WRITTEN;
        };

        // It is very important that the program executing on the CoAP packet returns
        // the length of the payload + PDU so that the handler can send the
        // response accordingly.
        let mut payload_length = 0;
        let _execution_time = vm.execute_on_coap_pkt(pkt, &mut payload_length);

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
        let request_data = match preprocess_request_raw(request) {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let Ok(request) = VMExecutionRequest::decode(request_data) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];

        let init_result = initialize_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        );

        let Ok(mut vm) = init_result else {
            error!(
                "Failed to initialize the VM: {}",
                init_result.err().unwrap()
            );
            return coap_numbers::code::INTERNAL_SERVER_ERROR;
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
    execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
}

impl VMLongExecutionHandler {
    pub fn new(
        execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
    ) -> Self {
        Self { execution_send }
    }
}

impl coap_handler::Handler for VMLongExecutionHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let request_data: String = match preprocess_request_raw(request) {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let Ok(request) = VMExecutionRequest::decode(request_data) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        let message = VMExecutionRequestIPC {
            request: Box::new(request),
        };

        if let Ok(()) = self.execution_send.lock().try_send(message) {
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

/// Responsible for benchmarking the VM execution by measuring program size,
/// load time (including optional relocation resolution) and execution time.
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
        let request_data = match preprocess_request_raw(request) {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let Ok(request) = VMExecutionRequest::decode(request_data) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
        let mut program =
            suit_storage::load_program(&mut program_buffer, request.configuration.suit_slot);

        self.program_size = program.len() as u32;

        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = Self::time_now(clock);
        if request.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
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
        let mut vm: Box<dyn VirtualMachine> = match request.configuration.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                program,
                Vec::from(middleware::ALL_HELPERS)
                    .into_iter()
                    .map(|f| f.id)
                    .collect(),
                request.configuration.binary_layout,
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
