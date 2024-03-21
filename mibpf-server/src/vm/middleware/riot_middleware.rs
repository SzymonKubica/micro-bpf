// This module implements middleware layer to allow rBPF VM call into the RIOT
// host OS. It contains all of the helper functions required to make rBPF a
// drop-in replacement for the Femto-Container VM.
//
// The prototype for helpers follows the convention used by rBpF: five `u64` as arguments, and a
// `u64` as a return value. Hence some helpers have unused arguments, or return a 0 value in all
// cases, in order to respect this convention.

use core::cmp::max;
use core::ffi::{c_char, CStr};

use alloc::vec::Vec;
use log::{debug, error};
use rbpf::helpers;
use riot_wrappers::gpio;
use riot_wrappers::stdio::println;

use super::helpers::HelperFunction;

/// Indices of the helper functions are defined to be exactly the same as in the
/// case of Femto-Container eBPF VM to ensure compatibility.

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
pub const BPF_ZTIMER_PERIOD_WAKEUP_ID: u32 = 0x61;

pub const BPF_GPIO_READ_INPUT: u32 = 0x70;
pub const BPF_GPIO_READ_RAW: u32 = 0x71;
pub const BPF_GPIO_WRITE: u32 = 0x72;

/// List of all helpers together with their corresponding numbers (used
/// directly as function pointers in the compiled eBPF bytecode).
pub const ALL_HELPERS: [HelperFunction; 24] = [
    HelperFunction::new(helpers::BPF_TRACE_PRINTK_IDX, 0, helpers::bpf_trace_printf),
    HelperFunction::new(BPF_DEBUG_PRINT_IDX, 1, bpf_print_debug),
    HelperFunction::new(BPF_PRINTF_IDX, 2, bpf_printf),
    HelperFunction::new(BPF_STORE_LOCAL_IDX, 3, bpf_store_local),
    HelperFunction::new(BPF_STORE_GLOBAL_IDX, 4, bpf_store_global),
    HelperFunction::new(BPF_FETCH_LOCAL_IDX, 5, bpf_fetch_local),
    HelperFunction::new(BPF_FETCH_GLOBAL_IDX, 6, bpf_fetch_global),
    HelperFunction::new(BPF_MEMCPY_IDX, 7, bpf_memcpy),
    HelperFunction::new(BPF_NOW_MS_IDX, 8, bpf_now_ms),
    HelperFunction::new(BPF_ZTIMER_NOW_IDX, 9, bpf_ztimer_now),
    HelperFunction::new(BPF_ZTIMER_PERIOD_WAKEUP_ID, 10, bpf_ztimer_periodic_wakeup),
    HelperFunction::new(BPF_SAUL_REG_FIND_NTH_IDX, 11, bpf_saul_reg_find_nth),
    HelperFunction::new(BPF_SAUL_REG_FIND_TYPE_IDX, 12, bpf_saul_reg_find_type),
    HelperFunction::new(BPF_SAUL_REG_WRITE_IDX, 13, bpf_saul_reg_write),
    HelperFunction::new(BPF_SAUL_REG_READ_IDX, 14, bpf_saul_reg_read),
    HelperFunction::new(BPF_GCOAP_RESP_INIT_IDX, 15, bpf_gcoap_resp_init),
    HelperFunction::new(BPF_COAP_OPT_FINISH_IDX, 16, bpf_coap_opt_finish),
    HelperFunction::new(BPF_COAP_ADD_FORMAT_IDX, 17, bpf_coap_add_format),
    HelperFunction::new(BPF_COAP_GET_PDU_IDX, 18, bpf_coap_get_pdu),
    HelperFunction::new(BPF_FMT_S16_DFP_IDX, 19, bpf_fmt_s16_dfp),
    HelperFunction::new(BPF_FMT_U32_DEC_IDX, 20, bpf_fmt_u32_dec),
    HelperFunction::new(BPF_GPIO_READ_INPUT, 21, bpf_gpio_read_input),
    HelperFunction::new(BPF_GPIO_READ_RAW, 22, bpf_gpio_read_raw),
    HelperFunction::new(BPF_GPIO_WRITE, 23, bpf_gpio_write),
];

pub const COAP_HELPERS: [HelperFunction; 4] = [
    HelperFunction::new(BPF_GCOAP_RESP_INIT_IDX, 15, bpf_gcoap_resp_init),
    HelperFunction::new(BPF_COAP_OPT_FINISH_IDX, 16, bpf_coap_opt_finish),
    HelperFunction::new(BPF_COAP_ADD_FORMAT_IDX, 17, bpf_coap_add_format),
    HelperFunction::new(BPF_COAP_GET_PDU_IDX, 18, bpf_coap_get_pdu),
];

/* Print/debug helper functions - implementation */

/// Allows for printing arbitrary text to the RIOT shell console output.
pub fn bpf_printf(fmt: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> u64 {
    // We need to take in the format string dynamically, so format! or println!
    // won't work here. We need to call into C.
    extern "C" {
        fn printf(fmt: *const c_char, ...) -> i32;
    }
    unsafe {
        printf(
            CStr::from_ptr(fmt as *const i8).as_ptr() as *const c_char,
            a1 as u32,
            a2 as u32,
            a3 as u32,
            a4 as u32,
        );
    }
    return 0;
}

/// Responsible for printing debug information. Prints a single value.
/// DEPRECATED: Use bpf_printf instead. It was implemented before `bpf_printf`
/// as that one didn't work initially because of issues with accessing .rodata
/// sections of the program.
pub fn bpf_print_debug(a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    println!("[DEBUG]: {a1}");
    return 0;
}

/* Key/value store functions - implementation */

extern "C" {
    fn bpf_store_update_global(key: u32, value: u32) -> i64;
    fn bpf_store_fetch_global(key: u32, value: *mut u32) -> i64;
}
pub fn bpf_store_local(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    // Local store/fetch requires changing the VM interpreter to maintain the
    // state of the key-value store btree and will require a bit more work.
    unimplemented!()
}
pub fn bpf_store_global(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unsafe { bpf_store_update_global(key as u32, value as u32) as u64 }
}
pub fn bpf_fetch_local(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unimplemented!()
}
pub fn bpf_fetch_global(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unsafe { bpf_store_fetch_global(key as u32, value as *mut u32) as u64 }
}

/* Standard library functions */

pub fn bpf_memcpy(dest_p: u64, src_p: u64, size: u64, _a4: u64, _a5: u64) -> u64 {
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
pub fn bpf_saul_reg_find_nth(saul_dev_index: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unsafe { return riot_sys::saul_reg_find_nth(saul_dev_index as i32) as u64 }
}

/// Find the first device of the given type. The saul_dev_type needs to match
/// the list of all device classes is available here:
/// https://api.riot-os.org/group__drivers__saul.html#:~:text=category%20IDs.%20More...-,enum,-%7B%0A%C2%A0%C2%A0SAUL_ACT_ANY
pub fn bpf_saul_reg_find_type(saul_dev_type: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unsafe { return riot_sys::saul_reg_find_type(saul_dev_type as u8) as u64 }
}

/// Given a pointer to the SAUL device struct, it reads from the device into the
/// provided phydat_t struct.
pub fn bpf_saul_reg_read(dev_ptr: u64, data_ptr: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let data: *mut riot_sys::phydat_t = data_ptr as *mut riot_sys::phydat_t;
    unsafe { riot_sys::saul_reg_read(dev, data) as u64 }
}

/// Given a pointer to the SAUL device struct, it writes the provided phydat_t
/// struct (pointed to by data_ptr) into the device.
pub fn bpf_saul_reg_write(dev_ptr: u64, data_ptr: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let data: *const riot_sys::phydat_t = data_ptr as *const riot_sys::phydat_t;
    unsafe { riot_sys::saul_reg_write(dev, data) as u64 }
}

#[derive(Debug)]
pub struct CoapContext {
    pub pkt: *mut riot_sys::coap_pkt_t,
    pub buf: *mut u8,
    pub len: usize,
}

/* (g)coap functions */
/// Initializes a CoAP response packet on a buffer.
/// Initializes payload location within the buffer based on packet setup.
pub fn bpf_gcoap_resp_init(coap_ctx_p: u64, resp_code: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;

    let resp_code = resp_code as u32;

    unsafe {
        debug!("coap_ctx: {:?}", *coap_ctx);
        debug!("buf_len: {:?}", (*coap_ctx).len);
        debug!("packet payload len: {:?}", (*(*coap_ctx).pkt).payload_len);
        debug!("resp code: {:?}", resp_code);
        let res = riot_sys::gcoap_resp_init(
            (*coap_ctx).pkt,
            (*coap_ctx).buf,
            (*coap_ctx).len as u32,
            resp_code,
        ) as u64;
        return res;
    }
}

pub fn bpf_coap_opt_finish(coap_ctx_p: u64, flags_u: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;
    unsafe {
        debug!("coap_ctx: {:?}", *coap_ctx);
        debug!("buf_len: {:?}", (*coap_ctx).len);
        debug!("packet payload len: {:?}", (*(*coap_ctx).pkt).payload_len);
        return riot_sys::coap_opt_finish((*coap_ctx).pkt, flags_u as u16) as u64;
    }
}

/// Append a Content-Format option to the pkt buffer.
pub fn bpf_coap_add_format(coap_ctx_p: u64, format: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let coap_ctx: *const CoapContext = coap_ctx_p as *const CoapContext;
    unsafe {
        debug!("coap_ctx: {:?}", *coap_ctx);
        debug!("buf_len: {:?}", (*coap_ctx).len);
        debug!("packet payload len: {:?}", (*(*coap_ctx).pkt).payload_len);
        // Again the type cast hacking is needed because we are using the function
        // from the inline module.
        return riot_sys::inline::coap_opt_add_format(
            (*coap_ctx).pkt as *mut riot_sys::inline::coap_pkt_t,
            format as u16,
        ) as u64;
    }
}

/// TODO: implement this helper
pub fn bpf_coap_get_pdu(_a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    return 0;
}

/// Returns the current time in milliseconds as measured by RIOT's ZTIMER.
pub fn bpf_now_ms(_a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let clock = unsafe { riot_sys::ZTIMER_MSEC as *mut riot_sys::inline::ztimer_clock_t };
    let now: u32 = unsafe { riot_sys::inline::ztimer_now(clock) };
    now as u64
}

/// Returns the current time in microseconds as measured by RIOT's ZTIMER.
pub fn bpf_ztimer_now(_a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
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
    _a3: u64,
    _a4: u64,
    _a5: u64,
) -> u64 {
    let last_wakeup: *mut u32 = last_wakeup as *mut u32;
    let period: u32 = period as u32;
    unsafe { riot_sys::ztimer_periodic_wakeup(riot_sys::ZTIMER_USEC, last_wakeup, period) }

    return 0;
}

/* Format functions - implementation */

/// Convert 16-bit fixed point number to a decimal string.
/// Returns the length of the resulting string.
pub fn bpf_fmt_s16_dfp(out_p: u64, val: u64, fp_digits: u64, _a4: u64, _a5: u64) -> u64 {
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
pub fn bpf_fmt_u32_dec(out_p: u64, val: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    extern "C" {
        fn fmt_u32_dec(out: *mut u8, val: u32) -> usize;
    }

    let out = out_p as *mut u8;
    unsafe {
        return fmt_u32_dec(out, val as u32) as u64;
    }
}

/* GPIO functions - implementation */
pub fn bpf_gpio_read_input(port: u64, pin_num: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let pin = gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32) })
        .unwrap();
    let result = pin.configure_as_input(gpio::InputMode::In);
    if let Ok(in_pin) = result {
        let pin_state = unsafe { riot_sys::gpio_read(in_pin.to_c()) };
        return pin_state as u64;
    }
    return 0;
}

/// Reads raw state of the pin, can be used to inspect the state of outputs without
/// changing it. E.g. if we have a pin powering a led and then turn it to input
/// to read its state, it will return 0 as changing a pin to input changes its
/// state
pub fn bpf_gpio_read_raw(port: u64, pin_num: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let pin_state =
        unsafe { riot_sys::gpio_read(riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32)) };
    return pin_state as u64;
}

pub fn bpf_gpio_write(port: u64, pin_num: u64, val: u64, _a4: u64, _a5: u64) -> u64 {
    let pin = gpio::GPIO::from_c(unsafe { riot_sys::macro_GPIO_PIN(port as u32, pin_num as u32) })
        .unwrap();
    let result = pin.configure_as_output(gpio::OutputMode::Out);
    if let Ok(out_pin) = result {
        unsafe { riot_sys::gpio_write(out_pin.to_c(), val as i32) };
        return 1;
    }
    return 0;
}
