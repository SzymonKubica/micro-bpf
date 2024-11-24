pub mod server;
mod handlers;

pub use server::gcoap_server_main;
#[cfg(feature = "dev_endpoints")]
pub use server::gcoap_server_testing;

