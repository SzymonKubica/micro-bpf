use core::fmt::Write;
use embedded_hal::digital::v2::ToggleableOutputPin;
use riot_wrappers::gpio;

pub fn handle_command(stdio: &mut riot_wrappers::stdio::Stdio, args: riot_wrappers::shell::Args) {
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
                gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port, pin_num) }).unwrap();

            match &args[1] {
                "read-input" => {
                    let result = pin.configure_as_input(gpio::InputMode::In);
                    if let Ok(in_pin) = result {
                        writeln!(stdio, "Reading from GPIO port: {} pin: {}", port, pin_num)
                            .unwrap();
                        let pin_state = unsafe { riot_sys::gpio_read(in_pin.to_c()) };
                        writeln!(stdio, "Raw Pin state: {}", pin_state).unwrap();
                        let is_high_res = in_pin.is_high();
                        if is_high_res {
                            writeln!(stdio, "Pin state: 1").unwrap();
                        }
                    }
                }
                // Reads raw state of the pin, can be used to inspect
                // outputs to see their state without changing it to 0
                // which happens when we try to initialise them as inputs.
                "read-raw" => {
                    writeln!(stdio, "Reading from GPIO port: {} pin: {}", port, pin_num).unwrap();
                    let pin_state =
                        unsafe { riot_sys::gpio_read(riot_sys::macro_GPIO_PIN(port, pin_num)) };
                    writeln!(stdio, "Pin state: {}", pin_state).unwrap();
                }
                "write" => {
                    if args.len() < 5 {
                        return usage();
                    }
                    let result = pin.configure_as_output(gpio::OutputMode::Out);
                    if let Ok(mut out_pin) = result {
                        writeln!(stdio, "Writing to GPIO port: {} pin: {} ", port, pin_num)
                            .unwrap();
                        let _res = match args[4].parse::<u32>() {
                            Ok(0) => out_pin.set_low(),
                            Ok(_) => out_pin.set_high(),
                            _ => (),
                        };
                        let pin_state = unsafe { riot_sys::gpio_read(out_pin.to_c()) };
                        writeln!(stdio, "Pin state: {}", pin_state).unwrap();
                    }
                }
                "toggle" => {
                    let result = pin.configure_as_output(gpio::OutputMode::Out);
                    if let Ok(mut out_pin) = result {
                        writeln!(stdio, "Toggling GPIO port: {} pin: {}", port, pin_num).unwrap();
                        if let Ok(_) = out_pin.toggle() {
                            let pin_state = unsafe { riot_sys::gpio_read(out_pin.to_c()) };
                            writeln!(stdio, "Pin state: {}", pin_state).unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}
