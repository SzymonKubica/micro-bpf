use core::cell::RefCell;

use alloc::{boxed::Box, string::String};
use log::debug;
use riot_wrappers::gcoap::PacketBuffer;

use super::VirtualMachine;

pub struct TimedVm {
    vm: Box<dyn VirtualMachine>,
    clock: *mut riot_sys::inline::ztimer_clock_t,
    results: RefCell<BenchmarkResult>,
}

impl TimedVm {
    pub fn new(vm: Box<dyn VirtualMachine>) -> TimedVm {
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

impl VirtualMachine for TimedVm {
    fn verify(&self) -> Result<(), String> {
        let start = self.time_now();
        let result = self.vm.verify();
        let end = self.time_now();

        self.results.borrow_mut().verification_time = end - start;
        result
    }

    fn initialize_vm(&mut self) -> Result<(), String> {
        let start = self.time_now();
        let result = self.vm.initialize_vm();
        let end = self.time_now();

        self.results.borrow_mut().load_time = end - start;
        result
    }

    fn execute(&mut self) -> Result<u64, String> {
        let start = self.time_now();
        let result = self.vm.execute();
        let end = self.time_now();

        self.results.borrow_mut().execution_time = end - start;
        result
    }

    fn execute_on_coap_pkt(&mut self, pkt: &mut PacketBuffer) -> Result<u64, String> {
        let start = self.time_now();
        let result = self.vm.execute_on_coap_pkt(pkt);
        let end = self.time_now();

        self.results.borrow_mut().execution_time = end - start;
        result
    }

    fn full_run(&mut self) -> Result<u64, String> {
        let start = self.time_now();
        self.initialize_vm()?;
        self.verify()?;
        let result = self.execute();
        let end = self.time_now();
        self.results.borrow_mut().total_time = end - start;
        result
    }
    fn full_run_on_coap_pkt(
        &mut self,
        pkt: &mut PacketBuffer,
    ) -> Result<u64, String> {
        let start = self.time_now();
        self.initialize_vm()?;
        self.verify()?;
        let result = self.execute_on_coap_pkt(pkt);
        debug!("Timed VM execution returned: {}.", result.clone().unwrap() as i64);
        let end = self.time_now();
        self.results.borrow_mut().total_time = end - start;
        result
    }

    fn get_program_length(&self) -> usize {
        self.vm.get_program_length()
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct BenchmarkResult {
    pub load_time: u32,
    pub verification_time: u32,
    pub execution_time: u32,
    pub total_time: u32,
}
