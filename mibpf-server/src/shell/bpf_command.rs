use alloc::sync::Arc;
use core::{fmt::Write, str::FromStr};
use riot_wrappers::{msg::v2::SendPort, mutex::Mutex};

use crate::vm::{rbpf_vm::BinaryFileLayout, VMExecutionRequest, VM_EXEC_REQUEST};

pub struct VMExecutionShellCommandHandler {
    execution_send: Arc<Mutex<SendPort<VMExecutionRequest, VM_EXEC_REQUEST>>>,
}

impl VMExecutionShellCommandHandler {
    pub fn new(
        execution_send: Arc<Mutex<SendPort<VMExecutionRequest, VM_EXEC_REQUEST>>>,
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
                "usage: {} [rBPF | FemtoContainer] <suit-storage-slot (int)> <bytecode-layout-option>",
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

        let binary_layout = BinaryFileLayout::from_str(&args[3]).unwrap_or_else(|err| {
            writeln!(stdio, "Invalid binary layout: {}", err).unwrap();
            BinaryFileLayout::FunctionRelocationMetadata
        });


        if let Ok(()) = self.execution_send.lock().try_send(VMExecutionRequest {
            suit_location: slot,
            vm_target,
            binary_layout: binary_layout.into(),
        }) {
            writeln!(stdio, "VM execution request sent successfully").unwrap();
        } else {
            writeln!(stdio, "Failed to send VM execution request").unwrap();
        }
    }
}
