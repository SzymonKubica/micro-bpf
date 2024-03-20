use alloc::{
    sync::Arc,
    vec::{self, Vec},
};
use core::{fmt::Write, str::FromStr};
use rbpf::helpers;
use riot_wrappers::{msg::v2::SendPort, mutex::Mutex};

use crate::{
    model::{
        enumerations::{BinaryFileLayout, TargetVM, VMConfiguration},
        requests::VMExecutionRequestMsg,
    },
    vm::{
        middleware::{helpers::HelperFunctionEncoding, ALL_HELPERS},
        VM_EXEC_REQUEST,
    },
};

pub struct VMExecutionShellCommandHandler {
    execution_send: Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
}

impl VMExecutionShellCommandHandler {
    pub fn new(
        execution_send: Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
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

        let available_helpers = HelperFunctionEncoding::from(Vec::from(ALL_HELPERS)).0;

        if let Ok(()) = self.execution_send.lock().try_send(VMExecutionRequestMsg {
            configuration: vm_configuration.encode(),
            available_helpers,
        }) {
            writeln!(stdio, "VM execution request sent successfully").unwrap();
        } else {
            writeln!(stdio, "Failed to send VM execution request").unwrap();
        }
    }
}
