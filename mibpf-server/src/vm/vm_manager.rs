use core::ffi::c_void;

use alloc::{sync::Arc, vec::Vec};
use log::{debug, error, info};

use riot_wrappers::{
    msg::v2::{MessageSemantics, NoConfiguredMessages, Processing, ReceivePort, SendPort},
    mutex::Mutex,
    stdio::println,
    thread::{self},
};

use riot_sys;
use riot_sys::msg_t;

use mibpf_common::VMExecutionRequest;

use crate::{
    infra::suit_storage::SUIT_STORAGE_SLOT_SIZE,
    model::requests::{VMExecutionCompleteMsg, VMExecutionRequestIPC},
    spawn_thread,
    vm::initialize_vm,
};

// Because of the lifetime rules we need to preallocate the stacks of all of the
// VM worker threads beforehand as static constants.
static VM_WORKER_0_STACK: Mutex<[u8; 6144]> = Mutex::new([0; 6144]);
static VM_WORKER_1_STACK: Mutex<[u8; 6144]> = Mutex::new([0; 6144]);
static VM_WORKER_2_STACK: Mutex<[u8; 6144]> = Mutex::new([0; 6144]);
static VM_WORKER_3_STACK: Mutex<[u8; 6144]> = Mutex::new([0; 6144]);

/// The unique identifier of the request type used to start the execution of the VM.
pub const VM_EXEC_REQUEST: u16 = 23;
pub const VM_COMPLETE_NOTIFY: u16 = 24;

pub type VMExecutionRequestPort = ReceivePort<VMExecutionRequestIPC, VM_EXEC_REQUEST>;
pub type VMExecutionCompletePort = ReceivePort<VMExecutionCompleteMsg, VM_COMPLETE_NOTIFY>;
pub type ExecutionSendPort = Arc<Mutex<SendPort<VMExecutionRequestIPC, VM_EXEC_REQUEST>>>;
pub type CompletionSendPort = Arc<Mutex<SendPort<VMExecutionCompleteMsg, VM_COMPLETE_NOTIFY>>>;

/// Responsible for managing execution of long-running eBPF programs. It receives
/// messages from other parts of the system that are requesting that a particular
/// instance of the VM should be started and execute a specified program.
pub struct VMExecutionManager {
    /// The port for receiving the VM execution requests from the CoAP server
    /// and the shell.
    request_receive_port: VMExecutionRequestPort,
    /// Send port for messages that request initiating an execution, it should
    /// be cloned and passed into any threads that want to send messages to the
    /// VM executor.
    request_send_port: ExecutionSendPort,
    /// The port used by the manager to learn that a particular VM has finished
    /// executing its eBPF program and is now free to be allocated a new workload.
    notification_receive_port: VMExecutionCompletePort,
    /// Send port that is passed to the worker threads to allow them to send
    /// execution completion notifications.
    notification_send_port: CompletionSendPort,
    /// Message semantics specifying the two available types of IPC messages
    /// that can be sent to the manager.
    message_semantics: Processing<
        Processing<NoConfiguredMessages, VMExecutionRequestIPC, VM_EXEC_REQUEST>,
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
    pub fn get_send_port(&self) -> Arc<Mutex<SendPort<VMExecutionRequestIPC, VM_EXEC_REQUEST>>> {
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
        let mut worker_1_main = worker_0_main.clone();
        let mut worker_2_main = worker_0_main.clone();
        let mut worker_3_main = worker_0_main.clone();

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
                let result = message.decode(&self.notification_receive_port, |_s, notification| {
                    Self::handle_job_complete_notification(&mut free_workers, &notification)
                });

                // Now handle any execution requests
                let code = if let Err(message) = result {
                    message
                        .decode(&self.request_receive_port, |_s, execution_request| {
                            Self::handle_execution_request(&mut free_workers, execution_request)
                        })
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

    pub fn handle_execution_request(workers: &mut Vec<i16>, request: VMExecutionRequestIPC) {
        if workers.is_empty() {
            error!("No free workers to execute the request.");
            return;
        }
        let pid: riot_sys::kernel_pid_t = workers.pop().unwrap();
        info!("Sending execution request to the worker with PID: {}", pid);
        let mut execution_request = *request.request;
        let mut msg: msg_t = Default::default();
        msg.type_ = 0;
        msg.content.ptr = &mut execution_request as *mut VMExecutionRequest as *mut c_void;
        unsafe {
            riot_sys::msg_send(&mut msg as *mut msg_t, pid);
        };
    }

    pub fn handle_job_complete_notification(
        workers: &mut Vec<i16>,
        notification: &VMExecutionCompleteMsg,
    ) {
        info!(
            "Received notification from worker with PID: {}
            Adding worker back to the pool of free workers.",
            notification.worker_pid
        );
        workers.push(notification.worker_pid)
    }
}

/// Each VM worker thread waits for incoming messages from the `VMExecutionManager`
/// that represent requests to start executing an instance of the eBPF VM. Once
/// a message is received, the worker starts executing the program until it
/// terminates. Current limitation is that the worker has no way of preempting
/// the executing program unless it crashes or voluntarily terminates.
fn vm_main_thread(send_port: &CompletionSendPort) {
    loop {
        // Here we use the msg v1 RIOT API as each VM worker cannot pass the
        // send port back to the VM manager (who created it).
        let mut msg: msg_t = Default::default();
        unsafe {
            let _ = riot_sys::msg_receive(&mut msg);
        }

        let wrapper: VMExecutionRequestIPC = msg.into();
        let request = *wrapper.request;

        info!(
            "Received an execution request to spawn a VM with configuration: {:?}",
            request.configuration
        );

        let mut program_buffer: [u8; SUIT_STORAGE_SLOT_SIZE] = [0; SUIT_STORAGE_SLOT_SIZE];
        if let Ok(mut vm) = initialize_vm(
            request.configuration,
            request.allowed_helpers,
            &mut program_buffer,
        ) {
            let mut result: i64 = 0;
            let execution_time = vm.execute(&mut result);
            debug!("Execution_time: {}, result: {}", execution_time, result);
        } else {
            error!("Failed to initialize the VM.");
        };

        // Now we notify the VM execution manager that the eBPF program has
        // terminated and so the manager add us to the pool of free workers
        // and send new execution requests
        let completion_notification = VMExecutionCompleteMsg::new(thread::get_pid().into());
        match send_port.lock().try_send(completion_notification) {
            Ok(()) => info!("VM execution completion notification sent successfully"),
            Err(_) => error!("Failed to send notification message."),
        }
    }
}
