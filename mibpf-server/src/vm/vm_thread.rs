use crate::vm::{FemtoContainerVm, RbpfVm, VirtualMachine};
use crate::ExecutionRequest;
use alloc::format;
use riot_wrappers::mutex::Mutex;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use riot_wrappers::{cstr::cstr, stdio::println};

use crate::middleware;
use crate::rbpf;
use crate::rbpf::helpers;
use riot_wrappers::thread;

use riot_wrappers::msg::v2 as msg;
use riot_wrappers::msg::v2::MessageSemantics;

use riot_sys;

#[derive(Debug, Copy, Clone)]
pub enum VmTarget {
    Rbpf,
    FemtoContainer,
}

/// This is the main function of the thread that allow for executing long-running
/// eBPF programs. It will be spawned as a separate thread and will execute
/// the programs by loading the bytecode from a specified SUIT storage location.
pub fn vm_thread_main(
    countdown: &Mutex<u32>,
    message_semantics: msg::Processing<msg::NoConfiguredMessages, crate::ExecutionRequest, 23>,
    execution_port: crate::ExecutionPort,
) -> Result<(), ()> {
    // Those values should be populated from the message received through IPC
    //

    loop {
        let code = message_semantics
            .receive()
            .decode(&execution_port, |s, execution_request| {
                println!("Execution request received from {:?}", s);
                handle_execution_request(execution_request)
            })
            .unwrap_or_else(|m| {
                // Given the above is exhaustive of the created ports, this won't happen, and we
                // could just as well .unwrap() -- but comment out the or_else part and this turns
                // up.
                //
                // This is *also* executed when the message type isn't known; if that's the case,
                // it will panic when dropping. (You can test that by setting an unknown type_ in
                // the above ztimer).
                //
                // If we don't want special handling, this branch can be removed: known messages
                // will be ignored, but still unknowns cause a panic when the result is dropped.
                //
                // (If we were very sure we don't receive bad messages, and OK with leaking ones we
                // didn't decode, we could also core::mem::forget(m) here and suffer no checking
                // code at all.)
                println!(
                    "A message was received that was not previously decoded; we're dropping it."
                );
            });
        // Returning something from the decoders is not something expected to be common, but it's
        // possible as long as all decoders return alike, and provides better static checks than
        // `let mut result = None;` and assigning to result in the decode closures. (If not used,
        // the closures can just all implicitly return () as any trailing semicolon does).
        println!("Result code {:?}", code);
    }
}

fn handle_execution_request(request: ExecutionRequest) {
    let suit_location = request.suit_location as i32;
    let vm_target = match request.vm_target {
        0 => VmTarget::Rbpf,
        _ => VmTarget::FemtoContainer,
    };

    let mut program_buffer: [u8; 2048] = [0; 2048];
    let location = format!(".ram.{0}\0", suit_location);

    let program = read_program_from_suit_storage(&mut program_buffer, &location);

    println!(
        "Loaded program bytecode from SUIT storage location {}, program length: {}",
        location,
        program.len()
    );

    // Dynamically dispatch between the two different VM implementations
    // depending on the request data.
    let vm: Box<dyn VirtualMachine> = match vm_target {
        VmTarget::Rbpf => Box::new(RbpfVm::new(Vec::from(middleware::ALL_HELPERS))),
        VmTarget::FemtoContainer => Box::new(FemtoContainerVm {}),
    };

    let mut result: i64 = 0;
    let execution_time = vm.execute(&program, &mut result);

    let resp = format!(
        "{{\"execution_time\": {}, \"result\": {}}}",
        execution_time, result
    );
    println!("{}", &resp);
}

// TODO: move the functions for interacting with the SUIT storage into their
// separate module.
fn read_program_from_suit_storage<'a>(program_buffer: &'a mut [u8], location: &str) -> &'a [u8] {
    let mut length = 0;
    unsafe {
        let buffer_ptr = program_buffer.as_mut_ptr();
        let location_ptr = location.as_ptr() as *const char;
        length = load_bytes_from_suit_storage(buffer_ptr, location_ptr);
    };
    &program_buffer[..(length as usize)]
}

extern "C" {
    /// Responsible for loading the bytecode from the SUIT ram storage.
    /// The application bytes are written into the buffer.
    fn load_bytes_from_suit_storage(buffer: *mut u8, location: *const char) -> u32;
}
