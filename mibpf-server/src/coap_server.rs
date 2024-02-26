use coap_handler_implementations::SimpleRendered;
use coap_message::{MessageOption, MutableWritableMessage, ReadableMessage};
use core::convert::TryInto;
use riot_wrappers::cstr::cstr;
use riot_wrappers::{
    coap_handler::GcoapHandler, gcoap, gcoap::SingleHandlerListener, gnrc, gpio, mutex::Mutex,
    riot_sys, stdio::println, thread, ztimer,
};
use riot_wrappers::msg::v2 as msg;

use crate::handlers::{
    execute_fc_on_coap_pkt, execute_vm_no_data, execute_vm_on_coap_pkt, handle_benchmark, handle_console_write_request, handle_riot_board_query, handle_suit_pull_request, spawn_vm_execution
};

pub fn gcoap_server_main(_countdown: &Mutex<u32>, execution_send: &msg::SendPort<crate::ExecutionRequest, 23>) -> Result<(), ()> {
    // Each endpoint needs a request handler defined as its own struct implementing
    // the Handler trait. Then we need to initialise a listener for that endpoint
    // and add it as a resource in the gcoap scope.

    // Example handlers
    let mut console_write_handler = GcoapHandler(handle_console_write_request());
    let mut riot_board_handler = GcoapHandler(handle_riot_board_query());

    // Handlers for executing benchmarks and initiating SUIT firmware fetch.
    let mut benchmark_handler = GcoapHandler(handle_benchmark());
    let mut suit_pull_handler = GcoapHandler(handle_suit_pull_request());

    let mut coap_pkt_execution_handler = execute_vm_on_coap_pkt();
    let mut no_data_execution_handler = GcoapHandler(execute_vm_no_data());
    let mut long_execution_handler = GcoapHandler(spawn_vm_execution(execution_send));

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
        &mut coap_pkt_execution_handler,
    );

    let mut vm_listener = SingleHandlerListener::new(
        cstr!("/vm/exec"),
        riot_sys::COAP_POST,
        &mut no_data_execution_handler,
    );

    let mut vm_spawn_listener = SingleHandlerListener::new(
        cstr!("/vm/spawn"),
        riot_sys::COAP_POST,
        &mut long_execution_handler,
    );


    let mut benchmark_listener = SingleHandlerListener::new(
        cstr!("/benchmark"),
        riot_sys::COAP_POST,
        &mut benchmark_handler,
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
