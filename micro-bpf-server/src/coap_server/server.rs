use alloc::sync::Arc;
use riot_wrappers::{
    coap_handler::v0_2::GcoapHandler,
    cstr::cstr,
    gcoap::{self, SingleHandlerListener},
    gnrc,
    msg::v2::SendPort,
    mutex::Mutex,
    riot_sys,
    stdio::println,
    thread,
    ztimer::{self, Ticks},
};

use crate::{model::requests::VMExecutionRequestIPC, vm::VM_EXEC_REQUEST};

use super::handlers::{
    miscellaneous::{ConsoleWriteHandler, RiotBoardHandler, RunningVMHandler},
    suit_pull_endpoint::SuitPullHandler,
    Fletcher16NativeTestHandler, TimedHandler, VMExecutionBenchmarkHandler,
    VMExecutionNoDataHandler, VMExecutionOnCoapPktBenchmarkHandler, VMExecutionOnCoapPktHandler,
    VMLongExecutionHandler,
};

/// The main entrypoint of the gCoAP server. It is responsible for handling
/// requests from the deployment frame work to load / execute programs.
///
/// It accepts a thread-safe reference to an IPC send port that is used by the
/// long running VM execution handler to send IPC message requests to worker
/// threads responsible for executing long-running eBPF programs.
///
/// In order to add a new request handler to this server, one needs to define
/// a [`coap_handler::Handler`] and then wrap it in a `GcoapHandler()`. Note that
/// the documentation hints don't work in this case so one cannot look into what
/// that GcoapHandler actually is. This is because it comes from the riot_wrappers
/// crate which is included in the project as a part of the RIOT build system
/// and so sometimes rust analyzer has trouble finding definitions of its types.
pub fn gcoap_server_main(
    execution_send: &Arc<Mutex<SendPort<VMExecutionRequestIPC, { VM_EXEC_REQUEST }>>>,
) -> Result<(), ()> {
    // Each endpoint needs a request handler defined as its own struct implementing
    // the Handler trait. Then we need to initialise a listener for that endpoint
    // and add it as a resource in the gcoap scope.

    // Handlers for querying the state of the deployed system
    let mut running_vm_handler = GcoapHandler(RunningVMHandler);

    // Suit pull handler for deploying eBPF binaries
    let mut suit_pull_handler = GcoapHandler(SuitPullHandler::new());

    // Handlers for executing deployed programs
    let mut coap_pkt_execution_handler = VMExecutionOnCoapPktHandler;
    let mut coap_pkt_timed_execution_handler = TimedHandler::new(&mut coap_pkt_execution_handler);
    let mut no_data_execution_handler = GcoapHandler(VMExecutionNoDataHandler::new());
    let mut long_execution_handler =
        GcoapHandler(VMLongExecutionHandler::new(execution_send.clone()));

    /* Definitions of listeners for the handlers */
    let mut running_vm_listener = SingleHandlerListener::new(
        cstr!("/running_vm"),
        riot_sys::COAP_GET,
        &mut running_vm_handler,
    );
    let mut suit_pull_listener = SingleHandlerListener::new(
        cstr!("/suit/pull"),
        riot_sys::COAP_POST,
        &mut suit_pull_handler,
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
    let mut vm_spawn_listener = SingleHandlerListener::new(
        cstr!("/long-running"),
        riot_sys::COAP_POST,
        &mut long_execution_handler,
    );
    gcoap::scope(|greg| {
        // Endpoint handlers are registered here.
        greg.register(&mut coap_pkt_vm_listener);
        greg.register(&mut running_vm_listener);
        greg.register(&mut vm_listener);
        greg.register(&mut vm_spawn_listener);
        greg.register(&mut suit_pull_listener);

        println!(
            "CoAP server ready; waiting for interfaces to settle before reporting addresses..."
        );

        let sectimer = ztimer::Clock::sec();
        sectimer.sleep(Ticks(2));
        print_network_interfaces();

        // Sending main thread to sleep; can't return or the Gcoap handler would need to be
        // deregistered (which it can't).
        loop {
            thread::sleep();
        }
    });

    Ok(())
}

/// Main entrpoint of the server responsible for handling various testing-related
/// endpoints. This is defined separately to ensure that when running a production
/// deployment, one does not expose handlers that are only meant for debugging.
pub fn gcoap_server_testing() -> Result<(), ()> {
    // Each endpoint needs a request handler defined as its own struct implementing
    // the Handler trait. Then we need to initialise a listener for that endpoint
    // and add it as a resource in the gcoap scope.

    // Handlers for querying the state of the deployed system
    let mut console_write_handler = GcoapHandler(ConsoleWriteHandler);
    let mut riot_board_handler = GcoapHandler(RiotBoardHandler);

    // Handlers for executing benchmarks
    let mut benchmark_handler = GcoapHandler(VMExecutionBenchmarkHandler::new());
    let mut benchmark_on_coap_pkt_handler = VMExecutionOnCoapPktBenchmarkHandler::new();
    let mut fletcher16_handler = GcoapHandler(Fletcher16NativeTestHandler::new());

    /* Definitions of listeners for the handlers */
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
    // Mock endpoint for benchmarking native execution of Fletcher16 algorithm.
    let mut fletcher16_listener = SingleHandlerListener::new(
        cstr!("/native/exec"),
        riot_sys::COAP_POST,
        &mut fletcher16_handler,
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

    gcoap::scope(|greg| {
        // Endpoint handlers are registered here.
        greg.register(&mut console_write_listener);
        greg.register(&mut riot_board_listener);
        greg.register(&mut fletcher16_listener);
        greg.register(&mut benchmark_listener);
        greg.register(&mut benchmark_on_coap_listener);

        println!(
            "CoAP server testing server ready."
        );

        let sectimer = ztimer::Clock::sec();
        sectimer.sleep(Ticks(2));

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
