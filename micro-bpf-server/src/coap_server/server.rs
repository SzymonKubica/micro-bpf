use alloc::sync::Arc;
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

use crate::{model::requests::VMExecutionRequestIPC, vm::VM_EXEC_REQUEST};

use super::handlers::{
    miscellaneous::{ConsoleWriteHandler, RiotBoardHandler},
    suit_pull_endpoint::SuitPullHandler,
    Fletcher16NativeTestHandler, JitTestHandler, TimedHandler, VMExecutionBenchmarkHandler,
    VMExecutionNoDataHandler, VMExecutionOnCoapPktBenchmarkHandler, VMExecutionOnCoapPktHandler,
    VMLongExecutionHandler,
};

pub fn gcoap_server_main(
    execution_send: &Arc<Mutex<SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
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
    let mut jit_handler = GcoapHandler(JitTestHandler::new());
    let mut fletcher16_handler = GcoapHandler(Fletcher16NativeTestHandler::new());
    let mut long_execution_handler =
        GcoapHandler(VMLongExecutionHandler::new(execution_send.clone()));
    let mut benchmark_on_coap_pkt_handler = VMExecutionOnCoapPktBenchmarkHandler::new();

    let mut console_write_listener = SingleHandlerListener::new(
        cstr!("/console/write"),
        riot_sys::COAP_POST,
        &mut console_write_handler,
    );

    let mut jit_listener =
        SingleHandlerListener::new(cstr!("/jit/exec"), riot_sys::COAP_POST, &mut jit_handler);

    // Mock endpoint for benchmarking native execution of Fletcher16 algorithm.
    // TODO: move this to a separate project to not clutter the main one
    let mut fletcher16_listener = SingleHandlerListener::new(
        cstr!("/native/exec"),
        riot_sys::COAP_POST,
        &mut fletcher16_handler,
    );

    let mut riot_board_listener = SingleHandlerListener::new(
        cstr!("/riot/board"),
        riot_sys::COAP_GET,
        &mut riot_board_handler,
    );

    let mut coap_pkt_vm_listener = SingleHandlerListener::new(
        cstr!("/with_coap_pkt"),
        riot_sys::COAP_POST,
        &mut coap_pkt_timed_execution_handler,
    );

    let mut vm_listener = SingleHandlerListener::new(
        cstr!("/short-execution"),
        riot_sys::COAP_POST,
        &mut no_data_execution_handler,
    );

    let mut benchmark_listener = SingleHandlerListener::new(
        cstr!("/benchmark/short-execution"),
        riot_sys::COAP_POST,
        &mut benchmark_handler,
    );

    let mut benchmark_on_coap_listener = SingleHandlerListener::new(
        cstr!("/benchmark/with_coap_pkt"),
        riot_sys::COAP_POST,
        &mut benchmark_on_coap_pkt_handler,
    );

    let mut vm_spawn_listener = SingleHandlerListener::new(
        cstr!("/long-running"),
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
        greg.register(&mut jit_listener);
        greg.register(&mut fletcher16_listener);
        greg.register(&mut vm_listener);
        greg.register(&mut benchmark_listener);
        greg.register(&mut benchmark_on_coap_listener);
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
