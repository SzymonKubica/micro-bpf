use crate::{vm::{
    middleware::{ALL_HELPERS},
    VM_EXEC_REQUEST,
}, model::requests::{IPCExecutionMessage, VMExecutionRequest}};
use alloc::{
    sync::Arc,
    vec::{self, Vec}, boxed::Box,
};
use core::{fmt::Write, str::FromStr};
use mibpf_common::{BinaryFileLayout, TargetVM, VMConfiguration, VMExecutionRequestMsg};
use rbpf::helpers;
use riot_wrappers::{msg::v2::SendPort, mutex::Mutex};

pub struct VMExecutionShellCommandHandler {
    execution_send: Arc<Mutex<SendPort<IPCExecutionMessage, {VM_EXEC_REQUEST}>>>,
}

impl VMExecutionShellCommandHandler {
    pub fn new(
        execution_send: Arc<Mutex<SendPort<IPCExecutionMessage, {VM_EXEC_REQUEST}>>>,
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
            writeln!(
                stdio,
                "Available bytecode layout options: OnlyTextSection, FemtoContainersHeader, FunctionRelocationMetadata, RawObjectFile",
            )
            .unwrap();
        };

        if args.len() < 3 {
            return usage();
        }

        let Ok(slot) = args[2].parse::<usize>() else {
            return usage();
        };

        let Ok(vm_target) = TargetVM::from_str(&args[1]) else {
            return usage();
        };

        let binary_layout = BinaryFileLayout::from_str(&args[3]).unwrap_or_else(|err| {
            writeln!(stdio, "Invalid binary layout: {}", err).unwrap();
            BinaryFileLayout::FunctionRelocationMetadata
        });

        let vm_configuration = VMConfiguration::new(vm_target, binary_layout, slot);

        let available_helpers = Vec::from(ALL_HELPERS);

        let request = VMExecutionRequest {
            configuration: vm_configuration,
            available_helpers,
        };

        let message = IPCExecutionMessage {
            request: Box::new(request),
        };

        match self.execution_send.lock().try_send(message) {
            Ok(_) => writeln!(stdio, "VM execution request sent successfully").unwrap(),
            Err(_) => writeln!(stdio, "Failed to send VM execution request").unwrap(),
        }
    }
}
