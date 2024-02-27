use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::{ffi::c_void, fmt};
use log::{debug, error};

use riot_wrappers::{
    cstr::cstr,
    msg::v2::{
        self as msg, MessageSemantics, NoConfiguredMessages, Processing, ReceivePort, SendPort,
    },
    mutex::Mutex,
    stdio::println,
    thread::{self, spawn},
};

use riot_sys;
use riot_sys::msg_t;

use crate::{
    infra::{log_thread_spawned, suit_storage},
    rbpf::{self, helpers},
    vm::VmTarget,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine},
};

static VM_SLOT_0_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_SLOT_1_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

/// Represents a request to execute an eBPF program on a particular VM. The
/// suit_location is the index of the SUIT storage slot from which the program
/// should be loaded. For instance, 0 corresponds to '.ram.0'. The vm_target
/// specifies which implementation of the VM should be used (FemtoContainers or
/// rBPF). 0 corresponds to rBPF and 1 corresponds to FemtoContainers. The
/// reason an enum isn't used here is that this struct is send in messages via
/// IPC api and adding an enum there resulted in the struct being too large to
/// send.
#[derive(Debug, Clone)]
pub struct VMExecutionRequest {
    pub suit_location: u8,
    pub vm_target: u8,
}

// We need to implement Drop for the execution request so that it can be
// dealocated after it is decoded an processed in the message channel.
impl Drop for VMExecutionRequest {
    fn drop(&mut self) {
        debug!("Dropping {:?} now.", self);
    }
}

pub const VM_EXECUTION_REQUEST_TYPE: u16 = 23;
pub type VMExecutionRequestPort = ReceivePort<VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>;

/// Responsible for managing execution of long-running eBPF programs. It receives
/// messages from other parts of the system that are requesting that a particular
/// instance of the VM should be started and execute a specified program.
pub struct VMExecutionManager {
    receive_port: VMExecutionRequestPort,
    send_port: Arc<Mutex<SendPort<VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>>>,
    message_semantics:
        Processing<NoConfiguredMessages, VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>,
}

impl VMExecutionManager {
    pub fn new(message_semantics: NoConfiguredMessages) -> Self {
        let (message_semantics, receive_port, send_port): (_, VMExecutionRequestPort, _) =
            message_semantics.split_off();

        VMExecutionManager {
            receive_port,
            message_semantics,
            send_port: Arc::new(Mutex::new(send_port)),
        }
    }

    /// Returns an atomically-counted reference to the send end of the message
    /// channel for sending requests to execute eBPF programs.
    pub fn get_send_port(
        &self,
    ) -> Arc<Mutex<SendPort<VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>>> {
        self.send_port.clone()
    }

    /// This is the main function of the thread that allow for executing long-running
    /// eBPF programs. It spawns worker threads and then sends messages to them to
    /// start executing long running eBPF programs.
    pub fn start(&self) {
        let mut slot_0_stacklock = VM_SLOT_0_STACK.lock();
        let mut slot_1_stacklock = VM_SLOT_1_STACK.lock();

        let mut slot_0_mainclosure = || vm_main_thread(VmTarget::Rbpf);
        let mut slot_1_mainclosure = || vm_main_thread(VmTarget::Rbpf);

        thread::scope(|threadscope| {
            let Ok(worker_0) = threadscope.spawn(
                slot_0_stacklock.as_mut(),
                &mut slot_0_mainclosure,
                cstr!("VM worker 0"),
                (riot_sys::THREAD_PRIORITY_MAIN - 4) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            ) else {
                error!("Failed to spawn VM worker 0");
                panic!();
            };

            log_thread_spawned(&worker_0, "VM worker 0");

            let Ok(worker_1) = threadscope.spawn(
                slot_1_stacklock.as_mut(),
                &mut slot_1_mainclosure,
                cstr!("VM worker 1"),
                (riot_sys::THREAD_PRIORITY_MAIN - 5) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            ) else {
                error!("Failed to spawn VM worker 1");
                panic!();
            };

            log_thread_spawned(&worker_1, "VM worker 1");

            loop {
                let code = self
                    .message_semantics
                    .receive()
                    .decode(&self.receive_port, |s, execution_request| unsafe {
                        let mut msg: msg_t = Default::default();
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
