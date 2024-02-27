// Copyright (C) 2020 Christian Amsüss
//
// This file is subject to the terms and conditions of the GNU Lesser
// General Public License v2.1. See the file LICENSE in the top level
// directory for more details.
#![no_std]

use riot_wrappers::cstr::cstr;
use riot_wrappers::{mutex::Mutex, stdio::println, thread, ztimer};
use riot_wrappers::{riot_main, riot_main_with_tokens};

mod allocator;
mod coap_server;
mod handlers;
mod logger;
mod middleware;
mod shell;
mod suit_storage;
mod vm;

// The second thread is running the CoAP network stack, therefore its
// stack memory size needs to be appropriately larger.
// The threading setup was adapted from here: https://gitlab.com/etonomy/riot-examples/-/tree/master/shell_threads?ref_type=heads
static COAP_THREAD_STACK: Mutex<[u8; 16384]> = Mutex::new([0; 16384]);
static SHELL_THREAD_STACK: Mutex<[u8; 5120]> = Mutex::new([0; 5120]);

extern crate alloc;
extern crate rbpf;
extern crate riot_sys;
extern crate rust_riotmodules;

use riot_wrappers::msg::v2 as msg;
use riot_wrappers::msg::v2::MessageSemantics;

use vm::VmTarget;

riot_main!(main);

#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    suit_location: u8,
    vm_target: u8,
}

impl Drop for ExecutionRequest {
    fn drop(&mut self) {
        println!("Dropping {:?} now.", self);
    }
}

pub type ExecutionPort = msg::ReceivePort<ExecutionRequest, 23>;

fn main(tok: thread::StartToken) -> ((), thread::TerminationToken) {
    extern "C" {
        fn do_gnrc_msg_queue_init();
    }

    // Initialise the logger
    logger::RiotLogger::init(log::LevelFilter::Info);

    // Need to initialise the gnrc message queue to allow for using
    // shell utilities such as ifconfig and ping
    // Not sure how it works, adapted from examples/suit_femtocontainer/gcoap_handler.c
    unsafe { do_gnrc_msg_queue_init() };

    // Allows for inter-thread synchronization, not used at the moment.
    let countdown = Mutex::new(3);

    tok.with_message_queue::<4, _>(|initial| {
        // Lock the stacks of the threads.
        let mut gcoapthread_stacklock = COAP_THREAD_STACK.lock();
        let mut shellthread_stacklock = SHELL_THREAD_STACK.lock();

        // We need message semantics for the vm thread
        let (_, semantics) = initial.take_msg_semantics();
        let (message_semantics, execution_port, execution_send): (_, ExecutionPort, _) =
            semantics.split_off();

        let mut gcoapthread_mainclosure =
            || coap_server::gcoap_server_main(&countdown, &execution_send).unwrap();
        let mut shellthread_mainclosure = || shell::shell_main(&countdown).unwrap();

        // Spawn the threads and then wait forever.
        thread::scope(|threadscope| {
            let gcoapthread = threadscope
                .spawn(
                    gcoapthread_stacklock.as_mut(),
                    &mut gcoapthread_mainclosure,
                    cstr!("secondthread"),
                    (riot_sys::THREAD_PRIORITY_MAIN - 3) as _,
                    (riot_sys::THREAD_CREATE_STACKTEST) as _,
                )
                .expect("Failed to spawn gcoap server thread");

            println!(
                "COAP server thread spawned as {:?} ({:?}), status {:?}",
                gcoapthread.pid(),
                gcoapthread.pid().get_name(),
                gcoapthread.status()
            );

            let shellthread = threadscope
                .spawn(
                    shellthread_stacklock.as_mut(),
                    &mut shellthread_mainclosure,
                    cstr!("shellthread"),
                    (riot_sys::THREAD_PRIORITY_MAIN - 2) as _,
                    (riot_sys::THREAD_CREATE_STACKTEST) as _,
                )
                .expect("Failed to spawn shell thread");

            println!(
                "Shell thread spawned as {:?} ({:?}), status {:?}",
                shellthread.pid(),
                shellthread.pid().get_name(),
                shellthread.status()
            );

            // We invoke the VM after everything else is running
            vm::vm_manager_main(&countdown, message_semantics, execution_port);

            loop {
                thread::sleep();
            }
        });
        unreachable!();
    });
}
