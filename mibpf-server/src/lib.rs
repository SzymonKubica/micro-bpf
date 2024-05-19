// Copyright (C) 2024 Szymon Kubica
//
// TODO: figure out licensing
// This file is subject to the terms and conditions of the GNU Lesser
// General Public License v2.1. See the file LICENSE in the top level
// directory for more details.
#![no_std]

extern crate alloc;
extern crate macros;
extern crate mibpf_common;
extern crate mibpf_elf_utils;
extern crate rbpf;
extern crate riot_sys;
extern crate rust_riotmodules;

use log::error;
use riot_wrappers::{mutex::Mutex, riot_main, thread};

mod coap_server;
mod infra;
mod model;
mod peripherals;
mod shell;
mod util;
mod vm;

// The coap thread is running the CoAP network stack, therefore its
// stack memory size needs to be appropriately larger.
// The threading setup was adapted from here: https://gitlab.com/etonomy/riot-examples/-/tree/master/shell_threads?ref_type=heads
static COAP_THREAD_STACK: Mutex<[u8; 8192]> = Mutex::new([0; 8192]);
static SHELL_THREAD_STACK: Mutex<[u8; 4096]> = Mutex::new([0; 4096]);

riot_main!(main);

fn main(token: thread::StartToken) -> ((), thread::EndToken) {
    util::logger::initialise_logger();

    extern "C" {
        fn sound_sensor_saul_register();
        fn photoresistor_saul_register();
        fn initialise_adc(adc_index: u8) -> u32;
    }

    unsafe {
        initialise_adc(0);
        initialise_adc(1);
        sound_sensor_saul_register();
        photoresistor_saul_register();
    }

    // We need to initialise the message queue so that the CoAP server can send
    // requests to the VM executor responsible for spawning instances of the VM.
    token.with_message_queue::<4, _>(|token| {
        // The execution manager needs to take the message semantics to
        // open up the message channel for receiving message requests.
        let (_, semantics) = token.take_msg_semantics();
        let vm_manager = vm::VMExecutionManager::new(semantics);

        // We need to initialize a send port so that other threads can send messages to
        // the main VM executor to request executing eBPF programs.
        let send_port = vm_manager.get_send_port();

        let mut shell_stack = SHELL_THREAD_STACK.lock();
        let mut gcoap_stack = COAP_THREAD_STACK.lock();

        // Because of the implementation details of the thread scope below, we
        // need to declare the main closures of the threads here instead of
        // inlining them.
        let mut gcoap_main = || coap_server::gcoap_server_main(&send_port).unwrap();
        let mut shell_main = || shell::shell_main(&send_port).unwrap();

        let pri = riot_sys::THREAD_PRIORITY_MAIN;

        thread::scope(|scope| {
            let _gcoapthread =
                spawn_thread!(scope, "CoAP server", gcoap_stack, gcoap_main, pri - 1);
            let _shellthread = spawn_thread!(scope, "Shell", shell_stack, shell_main, pri + 2);
            vm_manager.start();
        });
        unreachable!();
    });
}
