use serde_derive::{Deserialize, Serialize};

/// Encoded transfer object representing a request to start a given execution
/// of the eBPF VM. It contains the encoded configuration of the vm as well as
/// a bitstring (in a form of 3 u8s) specifying which helper functions can be
/// called by the program running in the VM.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[repr(C, packed)]
pub struct VMExecutionRequestMsg {
    pub configuration: u8,
    pub available_helpers: [u8; 3],
}

// We need to implement Drop for the execution request so that it can be
// dealocated after it is decoded an processed in the message channel.
impl Drop for VMExecutionRequestMsg {
    fn drop(&mut self) {}
}

/// Models the request that is sent to the target device to pull a specified
/// binary file from the CoAP fileserver.
/// The handler expects to get a request which consists of the IPv6 address of
/// the machine running the CoAP fileserver and the name of the manifest file
/// specifying which binary needs to be pulled.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuitPullRequest {
    pub ip_addr: String,
    pub manifest: String,
    pub riot_network_interface: String,
}
