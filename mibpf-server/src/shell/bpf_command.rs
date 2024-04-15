use crate::{
    model::requests::VMExecutionRequestIPC,
    vm::{middleware::ALL_HELPERS, VM_EXEC_REQUEST},
};
use alloc::{
    boxed::Box,
    sync::Arc,
    vec::{self, Vec},
};
use core::{fmt::Write, str::FromStr};
use mibpf_common::{
    BinaryFileLayout, HelperAccessListSource, HelperAccessVerification, TargetVM, VMConfiguration,
    VMExecutionRequest,
};
use rbpf::helpers;
use riot_wrappers::{msg::v2::SendPort, mutex::Mutex};

pub struct VMExecutionShellCommandHandler {
    execution_send: Arc<Mutex<SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
}

impl VMExecutionShellCommandHandler {
    pub fn new(
        execution_send: Arc<Mutex<SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
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
            BinaryFileLayout::ExtendedHeader
        });

        let vm_configuration = VMConfiguration::new(
            vm_target,
            slot,
            binary_layout,
            HelperAccessVerification::PreFlight,
            HelperAccessListSource::ExecuteRequest,
        );

        let allowed_helpers = Vec::from(ALL_HELPERS).into_iter().map(|f| f.id).collect();

        let request = VMExecutionRequest {
            configuration: vm_configuration,
            allowed_helpers,
        };

        let message = VMExecutionRequestIPC {
            request: Box::new(request),
        };

        match self.execution_send.lock().try_send(message) {
            Ok(_) => writeln!(stdio, "VM execution request sent successfully").unwrap(),
            Err(_) => writeln!(stdio, "Failed to send VM execution request").unwrap(),
        }
    }
}
