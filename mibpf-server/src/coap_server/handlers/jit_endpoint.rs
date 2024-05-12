//! This module contains and endpoint responsible for testing out the JIT.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use coap_message::{MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use log::debug;
use mibpf_common::{BinaryFileLayout, VMExecutionRequest};
use riot_wrappers::mutex::Mutex;

use crate::infra::jit_prog_storage::{self, JIT_SLOT_SIZE};
use crate::infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE};
pub struct JitTestHandler {
    jit_compilation_time: u32,
    execution_time: u32,
    result: i64,
}

impl JitTestHandler {
    pub fn new() -> Self {
        Self {
            execution_time: 0,
            result: 0,
            jit_compilation_time: 0,
        }
    }

    #[inline(always)]
    fn time_now(clock: *mut riot_sys::inline::ztimer_clock_t) -> u32 {
        unsafe { riot_sys::inline::ztimer_now(clock) }
    }
}

use crate::coap_server::handlers::util::preprocess_request_raw;
use crate::vm::middleware;
use crate::vm::middleware::helpers::HelperFunction;

#[repr(C, align(4))]
struct AlignedBuffer([u8; 6]);

static JIT_MEMORY: Mutex<[u8; JIT_SLOT_SIZE]> = Mutex::new([0; JIT_SLOT_SIZE]);

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
        debug!("eBPF program size: {} [B]", program.len());

        if request.configuration.binary_layout == BinaryFileLayout::RawObjectFile {
            let _ = mibpf_elf_utils::resolve_relocations(&mut program);
        }

        let helpers: Vec<HelperFunction> = Vec::from(middleware::ALL_HELPERS);

        let mut helpers_map = BTreeMap::new();
        for h in helpers {
            helpers_map.insert(h.id as u32, h.function);
        }

        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        // Additional scope so that the mutexguard gets unlocked and we can then
        // call jit storage again to read the program
        {
            let mut jit_memory_buffer =
                jit_prog_storage::acquire_storage_slot(request.configuration.suit_slot).unwrap();
            let jitting_start: u32 = Self::time_now(clock);
            let mut jit_memory = rbpf::JitMemory::new(
                program,
                jit_memory_buffer.as_mut(),
                &helpers_map,
                false,
                false,
                rbpf::InterpreterVariant::RawObjectFile,
            )
            .unwrap();
            self.jit_compilation_time = Self::time_now(clock) - jitting_start;

            debug!("JIT compilation successful");
            debug!("JIT Compilation step took: {} [us]", self.jit_compilation_time);
            debug!("jitted program size: {} [B]", jit_memory.offset);
        }

        let jitted_fn =
            jit_prog_storage::get_program_from_slot(request.configuration.suit_slot).unwrap();
        let mut ret = 0;

        let start: u32 = Self::time_now(clock);
        unsafe {
            ret = jitted_fn(1 as *mut u8, 2, 1234 as *mut u8, 4);
        }
        self.execution_time = Self::time_now(clock) - start;
        debug!("JIT execution successful: {}", ret);
        self.result = ret as i64;

        coap_numbers::code::CHANGED
    }

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response(&mut self, response: &mut impl MutableWritableMessage, request: u8) {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!(
            "{{\"jit_compilation_time\": {}, \"execution_time\": {}, \"result\": {}}}",
            self.jit_compilation_time, self.execution_time, self.result
        );
        response.set_payload(resp.as_bytes());
    }
}


