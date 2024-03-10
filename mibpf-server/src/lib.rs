// Copyright (C) 2024 Szymon Kubica
//
// TODO: figure out licensing
// This file is subject to the terms and conditions of the GNU Lesser
// General Public License v2.1. See the file LICENSE in the top level
// directory for more details.
#![no_std]

extern crate alloc;
extern crate rbpf;
extern crate riot_sys;
extern crate rust_riotmodules;

use core::ffi::c_int;

use log::{error, info};
use riot_wrappers::{cstr::cstr, mutex::Mutex, println, riot_main, thread};

mod coap_server;
mod infra;
mod shell;
mod vm;

use crate::infra::log_thread_spawned;

// The coap thread is running the CoAP network stack, therefore its
// stack memory size needs to be appropriately larger.
// The threading setup was adapted from here: https://gitlab.com/etonomy/riot-examples/-/tree/master/shell_threads?ref_type=heads
static COAP_THREAD_STACK: Mutex<[u8; 16384]> = Mutex::new([0; 16384]);
static SHELL_THREAD_STACK: Mutex<[u8; 5120]> = Mutex::new([0; 5120]);

riot_main!(main);

// This dummy implementaion is required because of a compliation bug which
// complains about an undefined reference to rust_eh_personality. This shouldn't
// be happening as the release profile of this application specifies panic="abort"
// which means that we shouldn't need an eh_personality function.
#[no_mangle]
extern "C" fn rust_eh_personality() {}

fn main(token: thread::StartToken) -> ((), thread::EndToken) {
    extern "C" {
        fn init_message_queue();
        fn bpf_store_init();
    }

    unsafe {
        bpf_store_init();
    }

    // Initialise the logger
    if let Ok(()) = infra::logger::RiotLogger::init(log::LevelFilter::Debug) {
        info!("Logger initialised");
    } else {
        println!("Failed to initialise logger");
    }

    // Initialise the gnrc message queue to allow for using
    // shell utilities such as ifconfig and ping
    unsafe { init_message_queue() };

    token.with_message_queue::<4, _>(|token| {
        // We need message semantics for the vm thread
        let (_, semantics) = token.take_msg_semantics();

        // The execution manager needs to take the message semantics to
        // open up the message channel for receiving message requests.
        let vm_manager = vm::VMExecutionManager::new(semantics);

        // We need to get a send port so that other threads can send messages to
        // the main VM executor to request executing eBPF programs.
        let send_port = vm_manager.get_send_port();

        // We need to lock the stacks for all of the spawned threads.
        let mut shellthread_stacklock = SHELL_THREAD_STACK.lock();
        let mut gcoapthread_stacklock = COAP_THREAD_STACK.lock();

        // Here we define the main functions that will be executed by the threads
        let mut gcoapthread_mainclosure = || coap_server::gcoap_server_main(&send_port).unwrap();
        let mut shellthread_mainclosure = || shell::shell_main(&send_port).unwrap();

        // Spawn the threads and then wait forever.
        thread::scope(|threadscope| {
            if let Ok(gcoapthread) = threadscope.spawn(
                gcoapthread_stacklock.as_mut(),
                &mut gcoapthread_mainclosure,
                cstr!("secondthread"),
                (riot_sys::THREAD_PRIORITY_MAIN - 3) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            ) {
                log_thread_spawned(&gcoapthread, "CoAP server");
            } else {
                error!("Failed to spawn CoAP server thread");
            }

            if let Ok(shellthread) = threadscope.spawn(
                shellthread_stacklock.as_mut(),
                &mut shellthread_mainclosure,
                cstr!("shellthread"),
                (riot_sys::THREAD_PRIORITY_MAIN - 2) as _,
                (riot_sys::THREAD_CREATE_STACKTEST) as _,
            ) {
                log_thread_spawned(&shellthread, "Shell");
            } else {
                error!("Failed to spawn shell thread");
            }

            vm_manager.start();
        });
        unreachable!();
    });
}
