use riot_wrappers::gcoap::PacketBuffer;

pub trait VirtualMachine {
    /// Executes a given eBPF program and stores the return value of the
    /// program in `result`. It returns the VM execution time
    fn execute(&self, program: &[u8], result: &mut i64) -> u32;

    /// Executes a given eBPF program giving it access to the provided PacketBuffer
    /// and stores the return value of the program in `result`. It returns the VM execution time
    fn execute_on_coap_pkt(&self, program: &[u8], pkt: &mut PacketBuffer, result: &mut i64) -> u32;
}
