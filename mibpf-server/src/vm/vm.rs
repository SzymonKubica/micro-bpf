use core::str::FromStr;

use alloc::string::String;
use riot_wrappers::gcoap::PacketBuffer;

/// Structs implementing this interface should allow for executing eBPF programs
/// both raw and with access to the incoming CoAP packet.
pub trait VirtualMachine {
    /// Executes a given eBPF program and stores the return value of the
    /// program in `result`. It returns the VM execution time
    fn execute(&self, program: &[u8], result: &mut i64) -> u32;

    /// Executes a given eBPF program giving it access to the provided PacketBuffer
    /// and stores the return value of the program in `result`. The value returned
    /// by the program and written to `result` needs to represent the length of
    /// the packet PDU + payload. The reason for this is that the handler then
    /// needs to know this length when sending the response back
    fn execute_on_coap_pkt(&self, program: &[u8], pkt: &mut PacketBuffer, result: &mut i64) -> u32;
}

