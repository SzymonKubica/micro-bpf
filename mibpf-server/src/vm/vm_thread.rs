use core::ffi::c_void;

use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use log::{debug, error, info};

use riot_wrappers::{
    cstr::cstr,
    msg::v2::{MessageSemantics, NoConfiguredMessages, Processing, ReceivePort, SendPort},
    mutex::{Mutex, MutexGuard},
    stdio::println,
    thread::{self, CountedThread, CountingThreadScope},
};

use riot_sys;
use riot_sys::msg_t;

use crate::{
    infra::suit_storage,
    spawn_thread,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine}, model::{requests::{VMExecutionRequestMsg, VMExecutionRequest}, enumerations::TargetVM},
};

static VM_WORKER_0_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_1_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_2_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_3_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);


/// The unique identifier of the request type used to start the execution of the VM.
pub const VM_EXEC_REQUEST: u16 = 23;
pub type VMExecutionRequestPort = ReceivePort<VMExecutionRequestMsg, VM_EXEC_REQUEST>;

/// Responsible for managing execution of long-running eBPF programs. It receives
/// messages from other parts of the system that are requesting that a particular
/// instance of the VM should be started and execute a specified program.
pub struct VMExecutionManager {
    receive_port: VMExecutionRequestPort,
    send_port: Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
    message_semantics: Processing<NoConfiguredMessages, VMExecutionRequestMsg, VM_EXEC_REQUEST>,
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
    pub fn get_send_port(&self) -> Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>> {
        self.send_port.clone()
    }

    /// This is the main function of the thread that allow for executing long-running
    /// eBPF programs. It spawns worker threads and then sends messages to them to
    /// start executing long running eBPF programs.
    pub fn start(&self) {
        extern "C" {
            fn bpf_store_init();
        }

        // We need to initialise the global storage for the VM helpers.
        // Currently we repurpose the Femto-Container implementation
        unsafe {
            bpf_store_init();
        }

        let mut worker_0_stack = VM_WORKER_0_STACK.lock();
        let mut worker_1_stack = VM_WORKER_1_STACK.lock();
        let mut worker_2_stack = VM_WORKER_2_STACK.lock();
        let mut worker_3_stack = VM_WORKER_3_STACK.lock();

        let mut worker_0_main = || vm_main_thread();
        let mut worker_1_main = || vm_main_thread();
        let mut worker_2_main = || vm_main_thread();
        let mut worker_3_main = || vm_main_thread();

        thread::scope(|ts| {
            let pri = riot_sys::THREAD_PRIORITY_MAIN;
            // All worker threads need to be spawned at the start because the
            // thread scope doesn't allow for spawning new threads on the fly,
            // we always need to know the number of threads at the start.
            let worker_0 = spawn_thread!(ts, "Worker 0", worker_0_stack, worker_0_main, pri - 4);
            let worker_1 = spawn_thread!(ts, "Worker 1", worker_1_stack, worker_1_main, pri - 3);
            let worker_2 = spawn_thread!(ts, "Worker 2", worker_2_stack, worker_2_main, pri - 2);
            let worker_3 = spawn_thread!(ts, "Worker 3", worker_3_stack, worker_3_main, pri - 1);

            loop {
                let code = self
                    .message_semantics
                    .receive()
                    .decode(&self.receive_port, |_s, mut execution_request| unsafe {
                        // for now we route slot 0 to worker 0 and slot 1 to worker 1
                        let target = TargetVM::from(execution_request.vm_target);
                        let worker = match (execution_request.suit_slot, target) {
                            (0, TargetVM::Rbpf) => &worker_0,
                            (1, TargetVM::Rbpf) => &worker_1,
                            (0, TargetVM::FemtoContainer) => &worker_2,
                            (1, TargetVM::FemtoContainer) => &worker_3,
                            _ => panic!("Invalid VM configuration "),
                        };
                        let pid: riot_sys::kernel_pid_t = worker.pid().into();
                        info!("Sending execution request to the worker with PID: {}", pid);
                        let mut msg: msg_t = execution_request.into();
                        riot_sys::msg_send(&mut msg, pid);
                    })
                    .unwrap_or_else(|_m| {
                        println!(
                    "A message was received that was not previously decoded; we're dropping it."
                );
                    });
                println!("Result code {:?}", code);
            }
        });
    }
}

fn vm_main_thread() {
    loop {
        let mut msg: msg_t = Default::default();
        unsafe {
            let _ = riot_sys::msg_receive(&mut msg);
        }
        let execution_request_msg: &VMExecutionRequestMsg = msg.into();
        let execution_request = VMExecutionRequest::from(execution_request_msg);

        let mut program_buffer: [u8; 1024] = [0; 1024];
        let program = suit_storage::load_program(&mut program_buffer, execution_request.suit_slot);

        info!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            execution_request.suit_slot,
            program.len()
        );

        let vm: Box<dyn VirtualMachine> = match execution_request.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                Vec::from(middleware::ALL_HELPERS),
                execution_request.binary_layout,
            )),
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        let mut result: i64 = 0;
        let execution_time = vm.execute(&program, &mut result);

        let resp = format!("Execution_time: {}, result: {}", execution_time, result);
        println!("{}", &resp);
    }
}
