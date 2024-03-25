use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::convert::TryInto;
use goblin::elf::{Elf, Reloc};

use log::{debug, error, info};
use serde::Deserialize;

use riot_wrappers::{
    coap_message::ResponseMessage, gcoap::PacketBuffer, msg::v2 as msg, mutex::Mutex, riot_sys,
    stdio::println,
};

use coap_message::{MutableWritableMessage, ReadableMessage};

use crate::{
    coap_server::handlers::util::preprocess_request,
    infra::suit_storage,
    model::{
        enumerations::{BinaryFileLayout, TargetVM},
        requests::{VMExecutionRequest, VMExecutionRequestMsg},
    },
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

        // We need to perform relocations on the raw object file.
        if request_data.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
            let Ok(()) = resolve_relocations(program) else {
                return 0;
            };
        }

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.configuration.suit_slot,
            program.len()
        );

        let vm: Box<dyn VirtualMachine> = match request_data.configuration.vm_target {
            TargetVM::Rbpf => {
                // When executing on a CoAP packet, the VM needs to have access
                // to the CoAP helpers plus any additional helpers specified by
                // the user.
                let mut helpers = Vec::from(middleware::COAP_HELPERS);
                helpers.append(&mut request_data.available_helpers.clone());
                Box::new(RbpfVm::new(
                    helpers,
                    request_data.configuration.binary_layout,
                ))
            }
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        // It is very important that the program executing on the CoAP packet returns
        // the length of the payload + PDU so that the handler can send the
        // response accordingly.
        let mut payload_length = 0;
        let execution_time = vm.execute_on_coap_pkt(&program, pkt, &mut payload_length);

        // The eBPF program needs to return the length of the Payload + PDU
        payload_length as isize
    }
}

fn resolve_relocations(program: &mut [u8]) -> Result<(), String> {
    unsafe {
        let program_start = program.as_mut_ptr() as u32;
        info!(
            "Performing relocations for program staring at {}",
            program_start
        );
    }

    let Ok(binary) = goblin::elf::Elf::parse(program) else {
        return Err("Failed to parse the Elf binary".to_string());
    };

    let relocations = find_relocations(&binary, program);
    for relocation in relocations {
        if let Some(symbol) = binary.syms.get(relocation.r_sym) {
            debug!("Relocation symbol: {} ", relocation.r_sym);
            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
            match symbol.st_type() {
                STT_SECTION => {
                    debug!(
                        "Relocation at instruction {} for symbol {}",
                        relocation.r_offset, relocation.r_sym
                    )
                }
                STT_FUNC => continue, // We don't patch for functions
                _ => {
                    let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();
                    debug!(
                        "Relocation at instruction {} for symbol {} in at {}",
                        relocation.r_offset, symbol_name, symbol.st_value
                    )
                }
            }
        }
    }

    Ok(())
}

fn find_relocations(binary: &Elf<'_>, buffer: &[u8]) -> Vec<Reloc> {
    let mut relocations = alloc::vec![];
    for section in &binary.section_headers {
        if section.sh_type == goblin::elf::section_header::SHT_REL {
            let offset = section.sh_offset as usize;
            let size = section.sh_size as usize;
            let relocs = goblin::elf::reloc::RelocSection::parse(
                &buffer,
                offset,
                size,
                false,
                goblin::container::Ctx::default(),
            )
            .unwrap();
            relocs.iter().for_each(|reloc| relocations.push(reloc));
        }
    }

    relocations
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
        let program =
            suit_storage::load_program(&mut program_buffer, request_data.configuration.suit_slot);

        debug!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            request_data.configuration.suit_slot,
            program.len()
        );

        // Dynamically dispatch between the two different VM implementations
        // depending on the requested target VM.
        let vm: Box<dyn VirtualMachine> = match request_data.configuration.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                Vec::from(middleware::ALL_HELPERS),
                request_data.configuration.binary_layout,
            )),
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        self.execution_time = vm.execute(&program, &mut self.result);

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
    execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
}

impl VMLongExecutionHandler {
    pub fn new(
        execution_send: Arc<Mutex<msg::SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
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
