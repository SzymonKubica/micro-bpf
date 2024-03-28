use alloc::sync::Arc;
use riot_wrappers::msg::v2::SendPort;
use riot_wrappers::mutex::Mutex;
use riot_wrappers::shell::CommandList;

use riot_wrappers::cstr::cstr;

use internal_representation::VMExecutionRequestMsg;
use crate::shell::{bpf_command, gpio_command};
use crate::vm::VM_EXEC_REQUEST;

pub fn shell_main(
    execution_send: &Arc<Mutex<SendPort<VMExecutionRequestMsg, VM_EXEC_REQUEST>>>,
) -> Result<(), ()> {
    let mut line_buf = [0u8; 128];

    extern "C" {
        fn init_message_queue();
        fn bpf_store_init();
    }

    // Initialise the gnrc message queue to allow for using
    // shell utilities such as ifconfig and ping
    unsafe { init_message_queue() };

    // TODO: add the command to execute loaded bpf programs
    let commands = riot_shell_commands::all();

    let bpf_handler = bpf_command::VMExecutionShellCommandHandler::new(execution_send.clone());

    let commands = trait_identity(commands).and(
        cstr!("gpio"),
        cstr!("Access GPIO pins"),
        gpio_command::handle_command,
    );

    let commands = trait_identity(commands).and(
        cstr!("bpf-execute"),
        cstr!("Execute and manage eBPF programs"),
        |stdio: &mut _, args: riot_wrappers::shell::Args<'_>| {
            bpf_handler.handle_command(stdio, args);
        },
    );

    trait_identity(commands).run_forever_with_buf(&mut line_buf);
}

// Workaround for a bug described here: https://github.com/RIOT-OS/rust-riot-wrappers/issues/76
fn trait_identity(c: impl CommandList) -> impl CommandList {
    c
}
