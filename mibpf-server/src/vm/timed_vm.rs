use core::cell::RefCell;

use alloc::{boxed::Box, string::String};
use riot_wrappers::gcoap::PacketBuffer;

use super::VirtualMachine;

pub struct TimedVm<'a> {
    vm: Box<dyn VirtualMachine<'a> + 'a>,
    clock: *mut riot_sys::inline::ztimer_clock_t,
    results: RefCell<BenchmarkResult>,
}

impl<'a> TimedVm<'a> {
    pub fn new(vm: Box<dyn VirtualMachine<'a> + 'a>) -> TimedVm<'a> {
        let clock = unsafe { riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t };
        Self {
            vm,
            clock,
            results: RefCell::new(Default::default()),
        }
    }

    #[inline(always)]
    fn time_now(&self) -> u32 {
        unsafe { riot_sys::inline::ztimer_now(self.clock) }
    }

    pub fn get_results(&self) -> BenchmarkResult {
        self.results.borrow().clone()
    }
}

impl<'a> VirtualMachine<'a> for TimedVm<'a> {
    fn resolve_relocations(&mut self, program: &'a mut [u8]) -> Result<&'a [u8], String> {
        let start = self.time_now();
        let result = self.vm.resolve_relocations(program);
        self.results.borrow_mut().relocation_resolution_time = self.time_now() - start;
        return result;
    }

    fn verify(&self) -> Result<(), String> {
        let start = self.time_now();
        let result = self.vm.verify();
        self.results.borrow_mut().verification_time = self.time_now() - start;
        return result;
    }

    fn initialise_vm(&mut self, program: &'a [u8]) -> Result<(), String> {
        let start = self.time_now();
        let result = self.vm.initialise_vm(program);
        self.results.borrow_mut().load_time = self.time_now() - start;
        return result;
    }

    fn execute(&mut self) -> Result<u64, String> {
        let start = self.time_now();
        let result = self.vm.execute();
        self.results.borrow_mut().execution_time = self.time_now() - start;
        return result;
    }

    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        let start = self.time_now();
        let result = self.vm.execute_on_coap_pkt(pkt);
        self.results.borrow_mut().execution_time = self.time_now() - start;
        return result;
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct BenchmarkResult {
    pub relocation_resolution_time: u32,
    pub load_time: u32,
    pub verification_time: u32,
    pub execution_time: u32,
}
