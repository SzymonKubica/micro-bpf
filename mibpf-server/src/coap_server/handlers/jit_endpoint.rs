//! This module contains and endpoint responsible for testing out the JIT.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use coap_message::{MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use log::debug;
use mibpf_common::VMExecutionRequest;
use riot_wrappers::mutex::Mutex;

use crate::infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE};
pub struct JitTestHandler {}

use crate::coap_server::handlers::util::preprocess_request_raw;

static JIT_MEMORY: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

impl coap_handler::Handler for JitTestHandler {
    type RequestData = u8;

    fn extract_request_data(&mut self, request: &impl ReadableMessage) -> Self::RequestData {
        let request_data = match preprocess_request_raw(request) {
            Ok(request_data) => request_data,
            Err(code) => return code,
        };

        let Ok(request) = VMExecutionRequest::decode(request_data) else {
            return coap_numbers::code::BAD_REQUEST;
        };

        debug!("JIT execution request received");
        let mut program_buffer = [0; SUIT_STORAGE_SLOT_SIZE];
        let mut program =
            suit_storage::load_program(&mut program_buffer, request.configuration.suit_slot);

        let compiler = rbpf::JitCompiler::new();
        let mut jit_memory_buffer = JIT_MEMORY.lock();
        let helpers = BTreeMap::new();
        let mut jit_memory = rbpf::JitMemory::new(
            program,
            jit_memory_buffer.as_mut_slice(),
            &helpers,
            false,
            false,
        )
        .unwrap();

        let mut compiler = rbpf::JitCompiler::new();
        compiler.jit_compile(&mut jit_memory, program, false, false, &helpers);
        debug!("JIT compilation successful");

        let mut bytecode_str = String::new();
        jit_memory.contents[0..jit_memory.offset]
            .iter()
            .enumerate()
            .for_each(|(i, b)| {
                bytecode_str = format!(
                    "{}{:02x}{}",
                    bytecode_str,
                    b,
                    if i % 4 == 3 { "\n" } else { "" }
                )
            });
        debug!("Compiled bytecode: \n{}", bytecode_str);

        debug!(
            "Address of the bytecode: {:#x}",
            jit_memory.contents.as_ptr() as u32
        );


        let jitted_fn = jit_memory.get_prog();
        let mut ret = 0;
        let mut ret2 = 0;

        unsafe {
            extern "C" {
                fn test();
            }
            test();

        }

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!("Jit execution successful");
        response.set_payload(resp.as_bytes());
    }
}
