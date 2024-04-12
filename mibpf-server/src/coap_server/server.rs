use core::ffi::c_int;

use alloc::{boxed::Box, sync::Arc, };
use riot_wrappers::{
    coap_handler::GcoapHandler,
    cstr::cstr,
    gcoap::{self, SingleHandlerListener},
    gnrc,
    msg::v2::SendPort,
    mutex::Mutex,
    riot_sys,
    stdio::println,
    thread, ztimer,
};
use mibpf_common::VMExecutionRequestMsg;
use crate::vm::VM_EXEC_REQUEST;

use super::handlers::{
    bpf_vm_endpoints::{
        VMExecutionNoDataHandler, VMExecutionOnCoapPktHandler, VMLongExecutionHandler, VMExecutionBenchmarkHandler,
    },
    miscellaneous::{ConsoleWriteHandler, RiotBoardHandler},
    suit_pull_endpoint::SuitPullHandler,
    TimedHandler,
};

pub fn gcoap_server_main(
    execution_send: &Arc<Mutex<SendPort<VMExecutionRequestMsg, {VM_EXEC_REQUEST}>>>,
) -> Result<(), ()> {
    // Each endpoint needs a request handler defined as its own struct implementing
    // the Handler trait. Then we need to initialise a listener for that endpoint
    // and add it as a resource in the gcoap scope.

    // Example handlers
    let mut console_write_handler = GcoapHandler(ConsoleWriteHandler);
    let mut riot_board_handler = GcoapHandler(RiotBoardHandler);
    let mut suit_pull_handler = GcoapHandler(SuitPullHandler::new());

    let mut coap_pkt_execution_handler = VMExecutionOnCoapPktHandler;
    let mut coap_pkt_timed_execution_handler = TimedHandler::new(&mut coap_pkt_execution_handler);
    let mut no_data_execution_handler = GcoapHandler(VMExecutionNoDataHandler::new());
    let mut benchmark_handler = GcoapHandler(VMExecutionBenchmarkHandler::new());
    let mut long_execution_handler =
        GcoapHandler(VMLongExecutionHandler::new(execution_send.clone()));

    let mut console_write_listener = SingleHandlerListener::new(
        cstr!("/console/write"),
        riot_sys::COAP_POST,
        &mut console_write_handler,
    );

    let mut riot_board_listener = SingleHandlerListener::new(
        cstr!("/riot/board"),
        riot_sys::COAP_GET,
        &mut riot_board_handler,
    );

    let mut coap_pkt_vm_listener = SingleHandlerListener::new(
        cstr!("/vm/exec/coap-pkt"),
        riot_sys::COAP_POST,
        &mut coap_pkt_timed_execution_handler,
    );

    let mut vm_listener = SingleHandlerListener::new(
        cstr!("/vm/exec"),
        riot_sys::COAP_POST,
        &mut no_data_execution_handler,
    );

    let mut benchmark_listener = SingleHandlerListener::new(
        cstr!("/vm/bench"),
        riot_sys::COAP_POST,
        &mut benchmark_handler,
    );

    let mut vm_spawn_listener = SingleHandlerListener::new(
        cstr!("/vm/spawn"),
        riot_sys::COAP_POST,
        &mut long_execution_handler,
    );

    let mut suit_pull_listener = SingleHandlerListener::new(
        cstr!("/suit/pull"),
        riot_sys::COAP_POST,
        &mut suit_pull_handler,
    );

    gcoap::scope(|greg| {
        // Endpoint handlers are registered here.
        greg.register(&mut console_write_listener);
        greg.register(&mut riot_board_listener);
        greg.register(&mut coap_pkt_vm_listener);
        greg.register(&mut vm_listener);
        greg.register(&mut benchmark_listener);
        greg.register(&mut vm_spawn_listener);
        greg.register(&mut suit_pull_listener);

        println!(
            "CoAP server ready; waiting for interfaces to settle before reporting addresses..."
        );

        let sectimer = ztimer::Clock::sec();
        sectimer.sleep_ticks(2);
        print_network_interfaces();

        // Sending main thread to sleep; can't return or the Gcoap handler would need to be
        // deregistered (which it can't).
        loop {
            thread::sleep();
        }
    });

    Ok(())
}

fn print_network_interfaces() {
    for netif in gnrc::Netif::all() {
        println!(
            "Active interface from PID {:?} ({:?})",
            netif.pid(),
            netif.pid().get_name().unwrap_or("unnamed")
        );
        match netif.ipv6_addrs() {
            Ok(addrs) => {
                for a in &addrs {
                    println!("    Address {:?}", a);
                }
            }
            _ => {
                println!("    Does not support IPv6.");
            }
        }
    }
}
