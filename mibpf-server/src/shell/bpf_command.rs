use core::fmt::Write;
use alloc::sync::Arc;
use riot_wrappers::{msg::v2::SendPort, mutex::Mutex};

use crate::vm::{VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE};

pub struct VMExecutionShellCommandHandler {
    execution_send: Arc<Mutex<SendPort<crate::vm::VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>>>,
}

impl VMExecutionShellCommandHandler {
    pub fn new(
        execution_send: Arc<
            Mutex<SendPort<crate::vm::VMExecutionRequest, VM_EXECUTION_REQUEST_TYPE>>,
        >,
    ) -> Self {
        Self { execution_send }
    }

    pub fn handle_command(
        &self,
        stdio: &mut riot_wrappers::stdio::Stdio,
        args: riot_wrappers::shell::Args,
    ) {
        let mut usage = || {
            writeln!(
                stdio,
                "usage: {} [rBPF | FemtoContainer] <suit-storage-slot (int)>",
                &args[0]
            )
            .unwrap();
        };

        if args.len() < 3 {
            return usage();
        }

        let Ok(slot) = args[2].parse::<u8>() else {
            return usage();
        };

        let vm_target: u8 = match &args[1] {
            "rBPF" => 0,
            "FemtoContainer" => 1,
            _ => return usage(),
        };

        self.execution_send.lock().try_send(VMExecutionRequest {
            suit_location: slot,
            vm_target,
        });
    }
}
