use crate::vm::{FemtoContainerVm, RbpfVm, VirtualMachine};
use crate::{suit_storage, ExecutionRequest};
use alloc::format;
use riot_wrappers::mutex::Mutex;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::fmt;
use riot_wrappers::{cstr::cstr, stdio::println};

use crate::middleware;
use crate::rbpf;
use crate::rbpf::helpers;
use riot_wrappers::thread::{self, spawn};

use riot_wrappers::msg::v2 as msg;
use riot_wrappers::msg::v2::MessageSemantics;

use riot_sys;
use riot_sys::msg_t;

static VM_SLOT_0_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_SLOT_1_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

#[derive(Debug, Copy, Clone)]
pub enum VmTarget {
    Rbpf,
    FemtoContainer,
}

/// This is the main function of the thread that allow for executing long-running
/// eBPF programs. It spawns worker threads and then sends messages to them to
/// start executing long running eBPF programs.
pub fn vm_manager_main(
    countdown: &Mutex<u32>,
    message_semantics: msg::Processing<msg::NoConfiguredMessages, crate::ExecutionRequest, 23>,
    execution_port: crate::ExecutionPort,
) {
    let mut slot_0_stacklock = VM_SLOT_0_STACK.lock();
    let mut slot_1_stacklock = VM_SLOT_1_STACK.lock();

    let mut slot_0_mainclosure = || vm_main_thread(VmTarget::Rbpf);
    let mut slot_1_mainclosure = || vm_main_thread(VmTarget::Rbpf);

    thread::scope(|threadscope| {
        let worker_0 = threadscope
            .spawn(
                slot_0_stacklock.as_mut(),
                &mut slot_0_mainclosure,
                cstr!("VM worker 0"),
                (riot_sys::THREAD_PRIORITY_MAIN - 4) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            )
            .expect("Failed to spawn VM worker 0");

        println!(
            "VM worker thread 0 spawned as {:?} ({:?}), status {:?}",
            worker_0.pid(),
            worker_0.pid().get_name(),
            worker_0.status()
        );

        let worker_1 = threadscope
            .spawn(
                slot_1_stacklock.as_mut(),
                &mut slot_1_mainclosure,
                cstr!("VM worker 1"),
                (riot_sys::THREAD_PRIORITY_MAIN - 5) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            )
            .expect("Failed to spawn VM worker 1");

        println!(
            "VM worker thread 1 spawned as {:?} ({:?}), status {:?}",
            worker_1.pid(),
            worker_1.pid().get_name(),
            worker_1.status()
        );

        loop {
            let code = message_semantics
                .receive()
                .decode(&execution_port, |s, execution_request| unsafe {
                    let mut msg: msg_t = Default::default();
                    println!("{}", "Sending message to worker 0");
                    msg.type_ = 0;
                    // The content of the message specifies which SUIT slot to load from
                    msg.content = riot_sys::msg_t__bindgen_ty_1 {
                        value: execution_request.suit_location as u32,
                    };
                    // for now we route slot 0 to worker 0 and slot 1 to worker 1
                    let worker = match execution_request.suit_location {
                        0 => &worker_0,
                        1 => &worker_1,
                        _ => panic!("Invalid slot number"),
                    };
                    let pid: riot_sys::kernel_pid_t = worker.pid().into();
                    println!("Pid of the worker {}", pid);
                    riot_sys::msg_send(&mut msg, pid);
                })
                .unwrap_or_else(|m| {
                    println!(
                    "A message was received that was not previously decoded; we're dropping it."
                );
                });
            println!("Result code {:?}", code);
        }
        unreachable!()
    });
}

fn vm_main_thread(target: VmTarget) {
    loop {
        let mut msg: msg_t = Default::default();
        unsafe {
            println!("initialised the empty message: {:?}", msg);
            let response = riot_sys::msg_receive(&mut msg);
        }

        // We are unpacking the union msg_t__bindgen_ty_1 => unsafe
        let suit_location = unsafe { msg.content.value };

        let mut program_buffer: [u8; 512] = [0; 512];

        let program = suit_storage::load_program(&mut program_buffer, suit_location as usize);

        println!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            suit_location,
            program.len()
        );

        // Dynamically dispatch between the two different VM implementations
        // depending on the request data.
        let vm: Box<dyn VirtualMachine> = match target {
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
}
