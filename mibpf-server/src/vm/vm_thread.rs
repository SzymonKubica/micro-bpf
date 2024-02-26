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

static VM_SLOT_0_STACK: Mutex<[u8; 600]> = Mutex::new([0; 600]);
static VM_SLOT_1_STACK: Mutex<[u8; 600]> = Mutex::new([0; 600]);

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
) {

    type Slot0ExecutionPort = msg::ReceivePort<ExecutionRequest, 24>;
    type Slot1ExecutionPort = msg::ReceivePort<ExecutionRequest, 25>;

    slot_0_stacklock = VM_SLOT_0_STACK.lock();
    slot_1_stacklock = VM_SLOT_1_STACK.lock();

    thread::scope(|threadscope| {});
    unreachable!()
}

/// Responsible for executing requests to spawn VMs for a specific SUIT storage
/// slot
fn vm_execution_handler_per_slot(
    message_semantics: msg::Processing<msg::NoConfiguredMessages, crate::ExecutionRequest, 23>,
    execution_port: crate::ExecutionPort,
    target_slot: u8,
) {
    loop {
        let code = message_semantics
            .receive()
            .decode(&execution_port, |s, execution_request| {
                println!("Execution request received from {:?}", s);
                if execution_request.suit_location != target_slot {
                    return;
                }
                handle_execution_request(execution_request.clone());
            })
            .unwrap_or_else(|m| {
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
