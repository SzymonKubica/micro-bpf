//! This module contains and endpoint responsible for testing out the JIT.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use coap_message::{MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use log::debug;
use micro_bpf_common::{BinaryFileLayout, VMExecutionRequest};
use riot_wrappers::mutex::Mutex;

use crate::infra::jit_prog_storage::{self, JIT_SLOT_SIZE};
use crate::infra::suit_storage::{self, SUIT_STORAGE_SLOT_SIZE};
pub struct JitTestHandler {
    jit_compilation_time: u32,
    execution_time: u32,
    result: i64,
    jit_prog_size: u32,
    prog_size: u32,
}

impl JitTestHandler {
    pub fn new() -> Self {
        Self {
            execution_time: 0,
            result: 0,
            jit_compilation_time: 0,
            jit_prog_size: 0,
            prog_size: 0,
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

static JIT_MEMORY: Mutex<[u8; JIT_SLOT_SIZE]> = Mutex::new([0; JIT_SLOT_SIZE]);
/// Before we can jit-compile the program we need to adjust all .data and .rodata
/// relocations so that they point to the sections that were copied over into the
/// jit memory buffer. Because of this we need calculate the addresses of the new
/// sections and then run the relocation resolution process so that the eBPF
/// program references the data in those new section in the jitted program buffer.
/// After that is done, we can jit compile it and so all relocated memory accesses
/// will correctly point to the data/rodata located inside of the jitted program.
///
/// The reason for doing this is that we want to be able to discard the source
/// eBPF program after we jit-compile it and thus save memory as jitted programs
/// are substantially smaller.
static PROGRAM_COPY_BUFFER: Mutex<[u8; JIT_SLOT_SIZE]> = Mutex::new([0; JIT_SLOT_SIZE]);

impl coap_handler::Handler for JitTestHandler {
    type RequestData = u8;

    fn estimate_length(&mut self, _request: &Self::RequestData) -> usize {
        1
    }

    fn build_response<M: MutableWritableMessage>(
        &mut self,
        response: &mut M,
        request: Self::RequestData,
    ) -> Result<(), Self::BuildResponseError<M>> {
        response.set_code(request.try_into().map_err(|_| ()).unwrap());
        let resp = format!(
            "{{\"prog_size\": {}, \"jit_prog_size\": {}, \"jit_comp_time\": {}, \"run_time\": {}, \"result\": {}}}",
            self.prog_size, self.jit_prog_size, self.jit_compilation_time, self.execution_time, self.result
        );
        response.set_payload(resp.as_bytes())
    }

    type ExtractRequestError;

    type BuildResponseError<M: MinimalWritableMessage>;

    fn extract_request_data<M: ReadableMessage>(
        &mut self,
        request: &M,
    ) -> Result<Self::RequestData, Self::ExtractRequestError> {
        /*
        TODO: clean this up.
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
        self.prog_size = program.len() as u32;

        let helpers: Vec<HelperFunction> = Vec::from(middleware::ALL_HELPERS);

        let mut helpers_map = BTreeMap::new();
        for h in helpers {
            helpers_map.insert(h.id as u32, h.function);
        }

        let jit_slot = request.configuration.suit_slot;

        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let mut text_offset = 0;
        {
            // Here we acquire a pointer to global storage where the jitted
            // program will be written. The additional scope is introduced so
            // that the acquired MutexGuard goes out of scope at the end of it
            // and so the lock is released. (RAII)
            let mut jit_memory_buffer = jit_prog_storage::acquire_storage_slot(jit_slot).unwrap();
            let jitting_start: u32 = Self::time_now(clock);
            let mut jit_memory = rbpf::JitMemory::new(
                program,
                PROGRAM_COPY_BUFFER.lock().as_mut(),
                jit_memory_buffer.as_mut(),
                &helpers_map,
                false,
                false,
                rbpf::InterpreterVariant::RawObjectFile,
            )
            .unwrap();
            self.jit_compilation_time = Self::time_now(clock) - jitting_start;

            debug!("JIT compilation successful");
            debug!(
                "JIT Compilation step took: {} [us]",
                self.jit_compilation_time
            );
            debug!("jitted program size: {} [B]", jit_memory.offset);
            self.jit_prog_size = jit_memory.offset as u32;
            text_offset = jit_memory.text_offset;
        }

        let jitted_fn = jit_prog_storage::get_program_from_slot(jit_slot, text_offset).unwrap();

        let mut ret = 0;
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        let start: u32 = Self::time_now(clock);
        unsafe {
            // We don't pass any meaningful arguments here as the program doesn't
            // work on a COAP message packet buffer.
            ret = jitted_fn(0 as *mut u8, 0, 0 as *mut u8, 0);
        }
        self.execution_time = Self::time_now(clock) - start;
        self.result = ret as i64;

        jit_prog_storage::free_storage_slot(jit_slot);
        debug!("JIT execution successful: {}", ret);
        */
        Ok(coap_numbers::code::CHANGED)
    }

}
