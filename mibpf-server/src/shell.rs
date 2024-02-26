use riot_wrappers::mutex::Mutex;
use riot_wrappers::shell::CommandList;

use core::fmt::Write;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;
use riot_wrappers::gpio;
use riot_wrappers::{cstr::cstr, stdio::println};

pub fn shell_main(countdown: &Mutex<u32>) -> Result<(), ()> {
    let mut line_buf = [0u8; 128];
    // Only include the default RIOT shell commands for now.
    // TODO: add the command to execute loaded bpf programs
    let mut commands = riot_shell_commands::all();
    let commands = trait_identity(commands).and(
        cstr!("gpio"),
        cstr!("Access GPIO pins"),
        |stdio: &mut _, args: riot_wrappers::shell::Args| {
            let mut usage = || {
                writeln!(
                    stdio,
                    "usage: {} [read-input|read-raw|write|toggle] <port> <pin> (<value-to-write>)",
                    &args[0]
                )
                .unwrap();
            };

            if args.len() < 4 {
                return usage();
            }

            match (args[2].parse::<u32>(), args[3].parse::<u32>()) {
                (Ok(port), Ok(pin_num)) => {
                    let pin =
                        gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port, pin_num) })
                            .unwrap();

                    match &args[1] {
                        "read-input" => {
                            let result = pin.configure_as_input(gpio::InputMode::In);
                            if let Ok(mut in_pin) = result {
                                writeln!(
                                    stdio,
                                    "Reading from GPIO port: {} pin: {}",
                                    port, pin_num
                                );
                                let pin_state = unsafe { riot_sys::gpio_read(in_pin.to_c()) };
                                writeln!(stdio, "Raw Pin state: {}", pin_state);
                                let is_high_res = in_pin.is_high();
                                if is_high_res {
                                    writeln!(stdio, "Pin state: 1");
                                }
                            }
                        }
                        // Reads raw state of the pin, can be used to inspect
                        // outputs to see their state without changing it to 0
                        // which happens when we try to initialise them as inputs.
                        "read-raw" => {
                            writeln!(stdio, "Reading from GPIO port: {} pin: {}", port, pin_num);
                            let pin_state = unsafe {
                                riot_sys::gpio_read(riot_sys::macro_GPIO_PIN(port, pin_num))
                            };
                            writeln!(stdio, "Pin state: {}", pin_state);
                        }
                        "write" => {
                            if args.len() < 5 {
                                return usage();
                            }
                            let result = pin.configure_as_output(gpio::OutputMode::Out);
                            if let Ok(mut out_pin) = result {
                                writeln!(stdio, "Writing to GPIO port: {} pin: {} ", port, pin_num);
                                let res = match args[4].parse::<u32>() {
                                    Ok(0) => out_pin.set_low(),
                                    Ok(_) => out_pin.set_high(),
                                    _ => (),
                                };
                                let pin_state = unsafe { riot_sys::gpio_read(out_pin.to_c()) };
                                writeln!(stdio, "Pin state: {}", pin_state);
                            }
                        }
                        "toggle" => {
                            let result = pin.configure_as_output(gpio::OutputMode::Out);
                            if let Ok(mut out_pin) = result {
                                writeln!(stdio, "Toggling GPIO port: {} pin: {}", port, pin_num);
                                if let Ok(_) = out_pin.toggle() {
                                    let pin_state = unsafe { riot_sys::gpio_read(out_pin.to_c()) };
                                    writeln!(stdio, "Pin state: {}", pin_state);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        },
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
