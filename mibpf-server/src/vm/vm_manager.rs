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
    model::{
        enumerations::TargetVM,
        requests::{VMExecutionCompleteMsg, VMExecutionRequest, VMExecutionRequestMsg},
    },
    spawn_thread,
    vm::{middleware, FemtoContainerVm, RbpfVm, VirtualMachine},
};

// Because of the lifetime rules we need to preallocate the stacks of all of the
// VM worker threads beforehand as static constants.
static VM_WORKER_0_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_1_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_2_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_WORKER_3_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

/// The unique identifier of the request type used to start the execution of the VM.
pub const VM_EXEC_REQUEST: u16 = 23;
pub const VM_COMPLETE_NOTIFY: u16 = 24;

pub type VMExecutionRequestPort = ReceivePort<VMExecutionRequestMsg, VM_EXEC_REQUEST>;
pub type VMExecutionCompletePort = ReceivePort<VMExecutionCompleteMsg, VM_COMPLETE_NOTIFY>;
pub type ExecutionSendPort = Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>;
pub type CompletionSendPort = Arc<Mutex<SendPort<VMExecutionCompleteMsg, VM_COMPLETE_NOTIFY>>>;

/// Responsible for managing execution of long-running eBPF programs. It receives
/// messages from other parts of the system that are requesting that a particular
/// instance of the VM should be started and execute a specified program.
pub struct VMExecutionManager {
    request_receive_port: VMExecutionRequestPort,
    request_send_port: ExecutionSendPort,
    notification_receive_port: VMExecutionCompletePort,
    notification_send_port: CompletionSendPort,
    message_semantics: Processing<
        Processing<NoConfiguredMessages, VMExecutionRequestMsg, VM_EXEC_REQUEST>,
        VMExecutionCompleteMsg,
        VM_COMPLETE_NOTIFY,
    >,
}

impl VMExecutionManager {
    pub fn new(message_semantics: NoConfiguredMessages) -> Self {
        let (message_semantics, receive_port, send_port): (_, VMExecutionRequestPort, _) =
            message_semantics.split_off();

        let (message_semantics, receive_port_2, send_port_2): (_, VMExecutionCompletePort, _) =
            message_semantics.split_off();

        VMExecutionManager {
            request_receive_port: receive_port,
            request_send_port: Arc::new(Mutex::new(send_port)),
            notification_receive_port: receive_port_2,
            notification_send_port: Arc::new(Mutex::new(send_port_2)),
            message_semantics,
        }
    }

    /// Returns an atomically-counted reference to the send end of the message
    /// channel for sending requests to execute eBPF programs.
    pub fn get_send_port(&self) -> Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>> {
        self.request_send_port.clone()
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

        let notification_port = self.notification_send_port.clone();

        let mut worker_0_main = || vm_main_thread(&notification_port);
        let mut worker_1_main = || vm_main_thread(&notification_port);
        let mut worker_2_main = || vm_main_thread(&notification_port);
        let mut worker_3_main = || vm_main_thread(&notification_port);

        thread::scope(|ts| {
            let pri = riot_sys::THREAD_PRIORITY_MAIN;
            // All worker threads need to be spawned at the start because the
            // thread scope doesn't allow for spawning new threads on the fly,
            // we always need to know the number of threads at the start.
            // We need to set different priorities for different workers because
            // otherwise they will keep blocking each other.
            let worker_0 = spawn_thread!(ts, "Worker 0", worker_0_stack, worker_0_main, pri - 4);
            let worker_1 = spawn_thread!(ts, "Worker 1", worker_1_stack, worker_1_main, pri - 3);
            let worker_2 = spawn_thread!(ts, "Worker 2", worker_2_stack, worker_2_main, pri - 2);
            let worker_3 = spawn_thread!(ts, "Worker 3", worker_3_stack, worker_3_main, pri - 1);

            let mut free_workers: Vec<i16> = alloc::vec![
                worker_0.pid().into(),
                worker_1.pid().into(),
                worker_2.pid().into(),
                worker_3.pid().into(),
            ];

            loop {
                let message = self.message_semantics.receive();

                // First process any completion notifications
                let result =
                    message.decode(&self.notification_receive_port, |_s, mut notification| {
                        Self::handle_job_complete_notification(&mut free_workers, &notification)
                    });

                // Now handle any execution requests
                let code = if let Err(message) = result {
                    message
                        .decode(
                            &self.request_receive_port,
                            |_s, mut execution_request| unsafe {
                                Self::handle_execution_request(&mut free_workers, execution_request)
                            },
                        )
                        .unwrap_or_else(|_m| {
                            error!("Failed to decode message.");
                        });
                } else {
                    result.unwrap();
                };

                println!("Result code {:?}", code);
            }
        });
    }

    pub fn handle_execution_request(workers: &mut Vec<i16>, request: VMExecutionRequestMsg) {
        if workers.is_empty() {
            error!("No free workers to execute the request.");
            return;
        }
        let pid: riot_sys::kernel_pid_t = workers.pop().unwrap();
        info!("Sending execution request to the worker with PID: {}", pid);
        let mut msg: msg_t = request.into();
        unsafe {
            riot_sys::msg_send(&mut msg, pid);
        };
    }

    pub fn handle_job_complete_notification(
        workers: &mut Vec<i16>,
        notification: &VMExecutionCompleteMsg,
    ) {
        info!(
            "Received notification from worker with PID: {}",
            notification.worker_pid
        );

        info!("Adding worker back to the pool of free workers.");
        workers.push(notification.worker_pid)
    }
}

fn vm_main_thread(send_port: &CompletionSendPort) {
    loop {
        let mut msg: msg_t = Default::default();
        unsafe {
            let _ = riot_sys::msg_receive(&mut msg);
        }
        let execution_request_msg: &VMExecutionRequestMsg = msg.into();
        println!("Received a message: {:?}", execution_request_msg);
        let execution_request = VMExecutionRequest::from(execution_request_msg);

        let vm_config = execution_request.configuration;

        info!("Received an execution request to spawn a VM with configuration: {:?}", vm_config);

        let mut program_buffer: [u8; 1024] = [0; 1024];
        let program = suit_storage::load_program(&mut program_buffer, vm_config.suit_slot);

        info!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            vm_config.suit_slot,
            program.len()
        );

        let vm: Box<dyn VirtualMachine> = match vm_config.vm_target {
            TargetVM::Rbpf => Box::new(RbpfVm::new(
                execution_request.available_helpers,
                vm_config.binary_layout,
            )),
            TargetVM::FemtoContainer => Box::new(FemtoContainerVm {}),
        };

        let mut result: i64 = 0;
        let execution_time = vm.execute(&program, &mut result);

        let resp = format!("Execution_time: {}, result: {}", execution_time, result);
        println!("{}", &resp);

        if let Ok(()) = send_port.lock().try_send(VMExecutionCompleteMsg {
            worker_pid: riot_wrappers::thread::get_pid().into(),
        }) {
            info!("VM execution completion notification sent successfully");
        } else {
            error!("Failed to send notification message.");
        }
    }
}
