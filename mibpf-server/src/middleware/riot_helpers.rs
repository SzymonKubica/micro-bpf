// This module implements middleware layer to allow rBPF VM call into the RIOT
// host OS. It contains all of the helper functions required to make rBPF a
// drop-in replacement for the Femto-Container VM.
//
// The prototype for helpers follows the convention used by rBpF: five `u64` as arguments, and a
// `u64` as a return value. Hence some helpers have unused arguments, or return a 0 value in all
// cases, in order to respect this convention.
// Question: why do we need this convention?

use core::ffi::CStr;

use alloc::vec::Vec;
use riot_wrappers::gpio;
use riot_wrappers::stdio::println;
use rbpf::helpers;

#[derive(Copy, Clone)]
pub struct HelperFunction {
    pub index: u32,
    pub function: fn(u64, u64, u64, u64, u64) -> u64,
}

impl HelperFunction {
    pub const fn new(index: u32, function: fn(u64, u64, u64, u64, u64) -> u64) -> Self {
        HelperFunction { index, function }
    }
}

/// Indices of the helper functions are defined to be exactly the same as in the
/// case of Femto-Container eBPF VM to achieve compatibility.

/* Print/debug helper functions */
pub const BPF_PRINTF_IDX: u32 = 0x01;
pub const BPF_DEBUG_PRINT_IDX: u32 = 0x03;

/* Memory copy helper functions */
pub const BPF_MEMCPY_IDX: u32 = 0x02;

/* Key/value store functions */
pub const BPF_STORE_LOCAL_IDX: u32 = 0x10;
pub const BPF_STORE_GLOBAL_IDX: u32 = 0x11;
pub const BPF_FETCH_LOCAL_IDX: u32 = 0x12;
pub const BPF_FETCH_GLOBAL_IDX: u32 = 0x13;

/* Saul functions */
pub const BPF_SAUL_REG_FIND_NTH_IDX: u32 = 0x30;
pub const BPF_SAUL_REG_FIND_TYPE_IDX: u32 = 0x31;
pub const BPF_SAUL_REG_READ_IDX: u32 = 0x32;
pub const BPF_SAUL_REG_WRITE_IDX: u32 = 0x33;

/* (g)coap functions */
pub const BPF_GCOAP_RESP_INIT_IDX: u32 = 0x40;
pub const BPF_COAP_OPT_FINISH_IDX: u32 = 0x41;
pub const BPF_COAP_ADD_FORMAT_IDX: u32 = 0x42;
pub const BPF_COAP_GET_PDU_IDX: u32 = 0x43;

/* Format functions */
pub const BPF_FMT_S16_DFP_IDX: u32 = 0x50;
pub const BPF_FMT_U32_DEC_IDX: u32 = 0x51;

/* Time(r) functions */
pub const BPF_NOW_MS_IDX: u32 = 0x20;

/* ZTIMER */
pub const BPF_ZTIMER_NOW_IDX: u32 = 0x60;
pub const BPF_ZTIMER_PERIODIC_WAKEUP_IDX: u32 = 0x61;

pub const BPF_GPIO_READ_INPUT: u32 = 0x70;
pub const BPF_GPIO_READ_RAW: u32 = 0x71;
pub const BPF_GPIO_WRITE: u32 = 0x72;

/* Print/debug helper functions - implementation */
/// The goal is to allow for printing arbitrary text, it isn't possible at the moment.
pub fn bpf_printf(fmt: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> u64 {
    // We need to take in the format string dynamically, so format! or println!
    // won't work here. We need to call into C.
    extern "C" {
        fn printf(fmt: *const char, ...) -> i32;
    }
    //println!("bpf_printf: {fmt}, {a1}, {a2}, {a3}, {a4}");
    unsafe {
        printf(
            CStr::from_ptr(fmt as *const i8).as_ptr() as *const char,
            a1 as u32,
            a2 as u32,
            a3 as u32,
            a4 as u32,
        );
    }
    return 0;
}

/// Responsible for printing debug information. Prints a single value.
pub fn bpf_print_debug(a1: u64, unused2: u64, unused3: u64, unused4: u64, unused5: u64) -> u64 {
    println!("[DEBUG]: {a1}");
    return 0;
}

/* Standard library functions */

pub fn bpf_memcpy(dest_p: u64, src_p: u64, size: u64, unused4: u64, unused5: u64) -> u64 {
    let dest: *mut riot_sys::libc::c_void = dest_p as *mut riot_sys::libc::c_void;
    let src: *const riot_sys::libc::c_void = src_p as *const riot_sys::libc::c_void;
    let size = size as u32;
    unsafe {
        return riot_sys::memcpy(dest, src, size) as u64;
    }
}

/* Saul functions - implementation */

/// Find a SAUL device by its position in the registry. It returns a pointer to
/// the device which can then be used with reg_read / reg_write helpers to
/// manipulate the device.
pub fn bpf_saul_reg_find_nth(
    saul_dev_index: u64,
    unused2: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    unsafe { return riot_sys::saul_reg_find_nth(saul_dev_index as i32) as u64 }
}

/// Find the first device of the given type. The saul_dev_type needs to match
/// the list of all device classes is available here:
/// https://api.riot-os.org/group__drivers__saul.html#:~:text=category%20IDs.%20More...-,enum,-%7B%0A%C2%A0%C2%A0SAUL_ACT_ANY
pub fn bpf_saul_reg_find_type(
    saul_dev_type: u64,
    unused2: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    unsafe { return riot_sys::saul_reg_find_type(saul_dev_type as u8) as u64 }
}

/// Given a pointer to the SAUL device struct, it reads from the device into the
/// provided phydat_t struct.
pub fn bpf_saul_reg_read(
    dev_ptr: u64,
    data_ptr: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let data: *mut riot_sys::phydat_t = data_ptr as *mut riot_sys::phydat_t;
    let mut res = 0;
    unsafe {
        res = riot_sys::saul_reg_read(dev, data) as u64;
    }
    res
}

/// Given a pointer to the SAUL device struct, it writes the provided phydat_t
/// struct (pointed to by data_ptr) into the device.
pub fn bpf_saul_reg_write(
    dev_ptr: u64,
    data_ptr: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let data: *const riot_sys::phydat_t = data_ptr as *const riot_sys::phydat_t;
    let mut res = 0;
    unsafe {
        res = riot_sys::saul_reg_write(dev, data) as u64;
    }
    res
}

#[repr(align(8))]
#[derive(Debug)]
/// Wrapper for struct fields that need to be aligned at the 8-bytest binary.
/// It is introduced to mimice the __attribute__((aligned(8))) that is used
/// on bpf_coap_ctx_t on the eBPF C code side.
struct Align8<T>(pub T);

#[derive(Debug)]
struct CoapContext {
    /// Opaque pointer to the coap_pkt_t struct
    pkt: Align8<*mut riot_sys::coap_pkt_t>,
    /// Packet buffer
    buf: Align8<*mut u8>,
    /// Packet buffer length
    buf_len: usize,
}

/* (g)coap functions */
/// Initializes a CoAP response packet on a buffer.
/// Initializes payload location within the buffer based on packet setup.
pub fn bpf_gcoap_resp_init(
    coap_ctx_p: u64,
    resp_code: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;

    let resp_code = resp_code as u32;

    unsafe {
        // TODO: find out how to do debug logging
        //println!("coap_ctx: {:?}", *coap_ctx);
        //println!("buf_len: {:?}", (*coap_ctx).buf_len);
        //println!("packet payload len: {:?}", (*(*coap_ctx).pkt.0).payload_len);
        //println!("resp code: {:?}", resp_code);
        let res = riot_sys::gcoap_resp_init(
            (*coap_ctx).pkt.0,
            (*coap_ctx).buf.0,
            (*coap_ctx).buf_len as u32,
            resp_code,
        ) as u64;
        return res;
    }
}

pub fn bpf_coap_opt_finish(
    coap_ctx_p: u64,
    flags_u: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;
    unsafe {
        //println!("coap_ctx: {:?}", *coap_ctx);
        //println!("buf_len: {:?}", (*coap_ctx).buf_len);
        //println!("packet payload len: {:?}", (*(*coap_ctx).pkt.0).payload_len);
        return riot_sys::coap_opt_finish((*coap_ctx).pkt.0, flags_u as u16) as u64;
    }
}

/// Append a Content-Format option to the pkt buffer.
pub fn bpf_coap_add_format(
    coap_ctx_p: u64,
    format: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;
    unsafe {
        //println!("coap_ctx: {:?}", *coap_ctx);
        //println!("buf_len: {:?}", (*coap_ctx).buf_len);
        //println!("packet payload len: {:?}", (*(*coap_ctx).pkt.0).payload_len);
        // Again the type cast hacking is needed because we are using the function
        // from the inline module.
        return riot_sys::inline::coap_opt_add_format(
            (*coap_ctx).pkt.0 as *mut riot_sys::inline::coap_pkt_t,
            format as u16,
        ) as u64;
    }
}
pub fn bpf_coap_get_pdu(
    unused1: u64,
    unused2: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    return 0;
}

/// Returns the current time in milliseconds as measured by RIOT's ZTIMER.
pub fn bpf_now_ms(unused1: u64, unused2: u64, unused3: u64, unused4: u64, unused5: u64) -> u64 {
    let clock = unsafe { riot_sys::ZTIMER_MSEC as *mut riot_sys::inline::ztimer_clock_t };
    let now: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };
    now as u64
}

/// Returns the current time in microseconds as measured by RIOT's ZTIMER.
pub fn bpf_ztimer_now(unused1: u64, unused2: u64, unused3: u64, unused4: u64, unused5: u64) -> u64 {
    let now: u32 = unsafe {
        // An explicit cast into *mut riot_sys::inline::ztimer_clock_t is needed here
        // because the type of riot_sys::ZTIMER_USEC is riot_sys::ztimer_clock_t
        // and the compiler complains about the mismatching type.
        riot_sys::inline::ztimer_now(riot_sys::ZTIMER_USEC as *mut riot_sys::inline::ztimer_clock_t)
    };
    now as u64
}

/// Suspend the calling thread until the time (last_wakeup + period)
pub fn bpf_ztimer_periodic_wakeup(
    last_wakeup: u64,
    period: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let last_wakeup: *mut u32 = last_wakeup as *mut u32;
    let period: u32 = period as u32;
    unsafe { riot_sys::ztimer_periodic_wakeup(riot_sys::ZTIMER_USEC, last_wakeup, period) }

    return 0;
}

/* Format functions - implementation */

/// Convert 16-bit fixed point number to a decimal string.
/// Returns the length of the resulting string.
pub fn bpf_fmt_s16_dfp(out_p: u64, val: u64, fp_digits: u64, unused4: u64, unused5: u64) -> u64 {
    extern "C" {
        fn fmt_s16_dfp(out: *mut u8, val: i16, fp_digits: i32) -> usize;
    }

    let out = out_p as *mut u8;
    unsafe {
        return fmt_s16_dfp(out, val as i16, fp_digits as i32) as u64;
    }
}

/// Convert a uint32 value to decimal string.
/// Returns the number of characters written to (or needed in) out
pub fn bpf_fmt_u32_dec(out_p: u64, val: u64, unused3: u64, unused4: u64, unused5: u64) -> u64 {
    extern "C" {
        fn fmt_u32_dec(out: *mut u8, val: u32) -> usize;
    }

    let out = out_p as *mut u8;
    unsafe {
        return fmt_u32_dec(out, val as u32) as u64;
    }
}

/* GPIO functions - implementation */
pub fn bpf_gpio_read_input(
    port: u64,
    pin_num: u64,
    unused3: u64,
    unused4: u64,
    unused5: u64,
) -> u64 {
    let pin = gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32) })
        .unwrap();
    let result = pin.configure_as_input(gpio::InputMode::In);
    if let Ok(mut in_pin) = result {
        let pin_state = unsafe { riot_sys::gpio_read(in_pin.to_c()) };
        return pin_state as u64;
    }
    return 0;
}
// Reads raw state of the pin, can be used to inspect the state of outputs without
// changing it. E.g. if we have a pin powering a led and then turn it to input
// to read its state, it will return 0 as changing a pin to input changes its
// state
pub fn bpf_gpio_read_raw(port: u64, pin_num: u64, unused3: u64, unused4: u64, unused5: u64) -> u64 {
    let pin_state =
        unsafe { riot_sys::gpio_read(riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32)) };
    return pin_state as u64;
}
pub fn bpf_gpio_write(port: u64, pin_num: u64, val: u64, unused4: u64, unused5: u64) -> u64 {
    let pin = gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32) })
        .unwrap();
    unsafe {
        let result = pin.configure_as_output(gpio::OutputMode::Out);
        if let Ok(mut out_pin) = result {
            unsafe { riot_sys::gpio_write(out_pin.to_c(), val as i32) };
            return 1;
        }
        return 0;
    }
}

/// List of all helpers together with their corresponding numbers (used
/// directly as function pointers in the compiled eBPF bytecode).
pub const ALL_HELPERS: [HelperFunction; 20] = [
    // Print/debug helper functions
    HelperFunction::new(helpers::BPF_TRACE_PRINTK_IDX, helpers::bpf_trace_printf),
    HelperFunction::new(BPF_DEBUG_PRINT_IDX, bpf_print_debug),
    HelperFunction::new(BPF_PRINTF_IDX, bpf_printf),
    HelperFunction::new(BPF_MEMCPY_IDX, bpf_memcpy),
    // Time(r) functions
    HelperFunction::new(BPF_NOW_MS_IDX, bpf_now_ms),
    HelperFunction::new(BPF_ZTIMER_NOW_IDX, bpf_ztimer_now),
    HelperFunction::new(BPF_ZTIMER_PERIODIC_WAKEUP_IDX, bpf_ztimer_periodic_wakeup),
    // Saul functions
    HelperFunction::new(BPF_SAUL_REG_FIND_NTH_IDX, bpf_saul_reg_find_nth),
    HelperFunction::new(BPF_SAUL_REG_FIND_TYPE_IDX, bpf_saul_reg_find_type),
    HelperFunction::new(BPF_SAUL_REG_WRITE_IDX, bpf_saul_reg_write),
    HelperFunction::new(BPF_SAUL_REG_READ_IDX, bpf_saul_reg_read),
    HelperFunction::new(BPF_GCOAP_RESP_INIT_IDX, bpf_gcoap_resp_init),
    HelperFunction::new(BPF_COAP_OPT_FINISH_IDX, bpf_coap_opt_finish),
    HelperFunction::new(BPF_COAP_ADD_FORMAT_IDX, bpf_coap_add_format),
    HelperFunction::new(BPF_COAP_GET_PDU_IDX, bpf_gcoap_resp_init),
    HelperFunction::new(BPF_FMT_S16_DFP_IDX, bpf_fmt_s16_dfp),
    HelperFunction::new(BPF_FMT_U32_DEC_IDX, bpf_fmt_u32_dec),
    HelperFunction::new(BPF_GPIO_READ_INPUT, bpf_gpio_read_input),
    HelperFunction::new(BPF_GPIO_READ_RAW, bpf_gpio_read_raw),
    HelperFunction::new(BPF_GPIO_WRITE, bpf_gpio_write),
];

/// Different versions of the rBPF VM have different implementations of the function
/// for registering helpers, however there is no common trait which encapsulates
/// that functionality. Because of this, when registering helpers for those VMs
/// the register_helper function depends on the type of the VM that we have,
/// this is unfortunate as it doesn't allow for easy swapping of the helpers.
/// Because of this problem, the triait AcceptingHelpers was introduced.
pub trait AcceptingHelpers {
    fn register_helper(&mut self, helper: HelperFunction);
}

/* Implementations of the custom trait for all rBPF VMs */
impl AcceptingHelpers for rbpf::EbpfVmFixedMbuff<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        self.register_helper(helper.index, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmRaw<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        self.register_helper(helper.index, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmNoData<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        self.register_helper(helper.index, helper.function);
    }
}

impl AcceptingHelpers for rbpf::EbpfVmMbuff<'_> {
    fn register_helper(&mut self, helper: HelperFunction) {
        self.register_helper(helper.index, helper.function);
    }
}

/// Registers all helpers provided by Femto-Container VM.
pub fn register_all(vm: &mut impl AcceptingHelpers) {
    for helper in ALL_HELPERS {
        vm.register_helper(helper);
    }
}

pub fn register_helpers(vm: &mut impl AcceptingHelpers, helpers: Vec<HelperFunction>) {
    for helper in helpers {
        vm.register_helper(helper);
    }
}
