use riot_wrappers::mutex::Mutex;
use riot_wrappers::shell::CommandList;

use riot_wrappers::{cstr::cstr, stdio::println};

use crate::shell::gpio_command;

pub fn shell_main(countdown: &Mutex<u32>) -> Result<(), ()> {
    let mut line_buf = [0u8; 128];

    // TODO: add the command to execute loaded bpf programs
    let mut commands = riot_shell_commands::all();

    let commands = trait_identity(commands).and(
        cstr!("gpio"),
        cstr!("Access GPIO pins"),
        gpio_command::handle_command,
    );

    trait_identity(commands).run_forever(&mut line_buf);
    unreachable!();
}

// Workaround for a bug described here: https://github.com/RIOT-OS/rust-riot-wrappers/issues/76
fn trait_identity(
    mut c: impl riot_wrappers::shell::CommandList,
) -> impl riot_wrappers::shell::CommandList {
    c
}
