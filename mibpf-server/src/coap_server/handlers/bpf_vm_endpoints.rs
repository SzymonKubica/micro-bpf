use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
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
            let Ok(()) = relocate_in_place(program) else {
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

fn print_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        if i % INSTRUCTION_SIZE == 0 {
            println!("{:02x}: ", i);
        }
        println!("{:02x} ", byte);
        if (i + 1) % INSTRUCTION_SIZE == 0 {
            println!("");
        }
    }
}

const INSTRUCTION_SIZE: usize = 8;
const LDDW_INSTRUCTION_SIZE: usize = 16;
const LDDW_OPCODE: u32 = 0x18;

/// Load-double-word instruction, needed for bytecode patching for loads from
/// .data and .rodata sections.
#[repr(C, packed)]
struct Lddw {
    opcode: u8,
    registers: u8,
    offset: u16,
    immediate_l: u32,
    null1: u8,
    null2: u8,
    null3: u16,
    immediate_h: u32,
}

impl From<&[u8]> for Lddw {
    fn from(bytes: &[u8]) -> Self {
        unsafe { core::ptr::read(bytes.as_ptr() as *const _) }
    }
}

impl<'a> Into<&'a [u8]> for &'a Lddw {
    fn into(self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, LDDW_INSTRUCTION_SIZE) }
    }
}

pub fn relocate_in_place(buffer: &mut [u8]) -> Result<(), String> {
    let program_address = buffer.as_ptr() as usize;

    let Ok(binary) = goblin::elf::Elf::parse(&buffer) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    let text_section = binary.section_headers.get(1).unwrap();

    let relocations = find_relocations(&binary, &buffer);
    let mut relocations_to_patch = alloc::vec![];
    for relocation in relocations {
        debug!(
            "Relocation found: offset: {:x}, r_addend: {:?}, r_sym: {}, r_type: {}",
            relocation.r_offset, relocation.r_addend, relocation.r_sym, relocation.r_type,
        );
        if let Some(symbol) = binary.syms.get(relocation.r_sym) {
            // Here the value of the relocation tells us the offset in the binary
            // where the data that needs to be relocated is located.
            debug!(
                "Looking up the relocation symbol: name: {}, section: {}, value: {:x}, is_function? : {}",
                symbol.st_name,
                symbol.st_shndx,
                symbol.st_value,
                symbol.is_function()
            );
            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
            debug!(
                "The relocation symbol section is located at offset {:x}",
                section.sh_offset
            );

            debug!(
                "The program address is {:x}, the section offset is {:x}, the symbol value is {:x}, adding a relocation to process",
                program_address, section.sh_offset, symbol.st_value
            );
            relocations_to_patch.push((
                relocation.r_offset as usize,
                program_address as u32 + section.sh_offset as u32 + symbol.st_value as u32,
            ));
        }
    }

    let text = &mut buffer
        [text_section.sh_offset as usize..(text_section.sh_offset + text_section.sh_size) as usize];
    for (offset, value) in relocations_to_patch {
        if offset > text.len() {
            continue;
        }

        debug!(
            "Patching text section at offset: {:x} with new immediate value: {:x}",
            offset, value
        );
        // we patch the text here
        // We only patch LDDW instructions
        if text[offset] != LDDW_OPCODE as u8 {
            debug!("No LDDW instruction at {} offset in .text section", offset);
            continue;
        }

        // We instantiate the instruction struct to modify it
        let instr_bytes = &text[offset..offset + 16];

        let mut instr: Lddw = Lddw::from(instr_bytes);
        // Also add the program base address here when relocating on the actual device
        instr.immediate_l += value;
        text[offset..offset + 16].copy_from_slice((&instr).into());

        //info!("Patched text section: ");
        //print_bytes(&text);
    }

    Ok(())
}

fn find_relocations(binary: &Elf<'_>, buffer: &[u8]) -> Vec<Reloc> {
    let mut relocations = alloc::vec![];

    println!("Binary is little endian? : {}", binary.little_endian);
    let context = goblin::container::Ctx::new(Container::Big, Endian::Little);
    println!("Context: {:?}", context);

    for section in &binary.section_headers {
        if section.sh_type == goblin::elf::section_header::SHT_REL {
            let offset = section.sh_offset as usize;
            let size = section.sh_size as usize;
            let relocs =
                goblin::elf::reloc::RelocSection::parse(&buffer, offset, size, false, context)
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
