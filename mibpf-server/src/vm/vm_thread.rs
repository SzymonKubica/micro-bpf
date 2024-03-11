use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
use log::{debug, error};

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
    infra::{log_thread_spawned, suit_storage},
    vm::{middleware, rbpf_vm::BinaryFileLayout, FemtoContainerVm, RbpfVm, VirtualMachine},
};

use super::VmTarget;

static VM_SLOT_0_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);
static VM_SLOT_1_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

/// Represents a request to execute an eBPF program on a particular VM. The
/// suit_location is the index of the SUIT storage slot from which the program
/// should be loaded. For instance, 0 corresponds to '.ram.0'. The vm_target
/// specifies which implementation of the VM should be used (FemtoContainers or
/// rBPF). 0 corresponds to rBPF and 1 corresponds to FemtoContainers. The
/// reason an enum isn't used here is that this struct is send in messages via
/// IPC api and adding an enum there resulted in the struct being too large to
/// send. It also specifies the binary layout format that the VM should expect
/// in the loaded program
#[derive(Debug, Clone)]
pub struct VMExecutionRequest {
    pub suit_location: u8,
    pub vm_target: u8,
    pub binary_layout: u8,
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

/// Responsible for spawning a new thread using the given thread scope. The reason
/// it can't be a plain function is that the threadscope is a mutable reference
/// that is valid only inside of the scope closure, and so it can't be passed
/// into a function. This macro allows for spawning multiple threads inside of
/// a single scope without having to paste a lot of the boilerplate.
#[macro_export]
macro_rules! spawn_thread {
    ($threadscope:expr, $name: expr, $stacklock:expr, $mainclosure:expr, $priority:expr ) => {{
        let Ok(thread) = $threadscope.spawn(
            $stacklock.as_mut(),
            &mut $mainclosure,
            cstr!($name),
            (riot_sys::THREAD_PRIORITY_MAIN - $priority) as _,
            (riot_sys::THREAD_CREATE_STACKTEST) as _,
        ) else {
            let msg = format!("Failed to spawn {}", $name);
            error!("{}", msg);
            panic!();
        };
        log_thread_spawned(&thread, $name);
        thread
    }};
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

        let mut slot_0_mainclosure = || vm_main_thread(VmTarget::Rbpf, 0);
        let mut slot_1_mainclosure = || vm_main_thread(VmTarget::Rbpf, 1);

        thread::scope(|threadscope| {
            let worker_0 = spawn_thread!(
                threadscope,
                "VM worker 0",
                slot_0_stacklock,
                slot_0_mainclosure,
                4
            );

            let worker_1 = spawn_thread!(
                threadscope,
                "VM worker 1",
                slot_1_stacklock,
                slot_1_mainclosure,
                5
            );

            loop {
                let code = self
                    .message_semantics
                    .receive()
                    .decode(&self.receive_port, |_s, execution_request| unsafe {
                        let mut msg: msg_t = Default::default();
                        msg.type_ = 0;
                        // The content of the message specifies which SUIT slot to load from
                        msg.content = riot_sys::msg_t__bindgen_ty_1 {
                            value: execution_request.binary_layout as u32,
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

fn vm_main_thread(target: VmTarget, suit_slot: u8) {
    loop {
        let mut msg: msg_t = Default::default();
        unsafe {
            let _ = riot_sys::msg_receive(&mut msg);
        }

        // We are unpacking the union msg_t__bindgen_ty_1 => unsafe
        let bytecode_layout_index = unsafe { msg.content.value };

        let mut program_buffer: [u8; 1024] = [0; 1024];

        let program = suit_storage::load_program(&mut program_buffer, suit_slot as usize);

        println!(
            "Loaded program bytecode from SUIT storage slot {}, program length: {}",
            suit_slot,
            program.len()
        );

        let bytecode_layout = BinaryFileLayout::from(bytecode_layout_index as u8);
        // Dynamically dispatch between the two different VM implementations
        // depending on the request data.
        let vm: Box<dyn VirtualMachine> = match target {
            VmTarget::Rbpf => Box::new(RbpfVm::new(
                Vec::from(middleware::ALL_HELPERS),
                bytecode_layout,
            )),
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
