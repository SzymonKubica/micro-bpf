use alloc::string::String;

use super::VirtualMachine;

pub struct TimedVm {
    vm: Box<dyn VirtualMachine>,
    clock: *mut riot_sys::inline::ztimer_clock_t,
    results: BenchmarkResult,
}

impl TimedVm {
    pub fn new(vm: Box<dyn VirtualMachine>) -> Self {
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        Self {
            vm,
            clock,
            results: Default::default(),
        }
    }

    #[inline(always)]
    fn time_now() -> u32 {
        unsafe { riot_sys::inline::ztimer_now(self.clock) }
    }

    pub fn get_results(&self) -> &BenchmarkResult {
        &self.results
    }
}

impl VirtualMachine for TimedVm {
    fn resolve_relocations(&mut self, program: &'a mut [u8]) -> Result<&'a [u8], String> {
        let start = Self::time_now();
        let result = self.vm.resolve_relocations(program);
        self.results.relocation_resolution_time = Self::time_now() - start;
        return result;
    }

    fn verify(&self) -> Result<(), String> {
        let start = Self::time_now();
        let result = self.vm.verify();
        self.results.verification_time = Self::time_now() - start;
        return result;
    }

    fn initialise_vm(&mut self, program: &'a [u8]) -> Result<(), String> {
        let start = Self::time_now();
        let result = self.vm.initialise_vm(program);
        self.results.load_time = Self::time_now() - start;
        return result;
    }

    fn execute(&mut self) -> Result<u64, String> {
        let start = Self::time_now();
        let result = self.vm.execute();
        self.results.execution_time = Self::time_now() - start;
        return result;
    }

    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        let start = Self::time_now();
        let result = self.vm.execute_on_coap_pkt(pkt);
        self.results.execution_time = Self::time_now() - start;
        return result;
    }
}

#[derive(Default, Debug)]
pub struct BenchmarkResult {
    pub relocation_resolution_time: u32,
    pub load_time: u32,
    pub verification_time: u32,
    pub execution_time: u32,
}
