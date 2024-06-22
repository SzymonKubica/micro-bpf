// This module implements middleware layer to allow rBPF VM call into the RIOT
// host OS. It contains all of the helper functions required to make rBPF a
// drop-in replacement for the Femto-Container VM.
//
// The prototype for helpers follows the convention used by rBpF: five `u64` as arguments, and a
// `u64` as a return value. Hence some helpers have unused arguments, or return a 0 value in all
// cases, in order to respect this convention.

use core::ffi::{c_char, CStr};

use log::debug;
use riot_wrappers::gpio;
use riot_wrappers::stdio::println;

use crate::{
    infra::local_storage::{self, local_storage_store},
    peripherals::{hd44780_lcd::{hd44780_t, HD44780LCD}, keypad_shield_buttons::KeypadShieldButtons},
};

use super::helpers::HelperFunction;
use micro_bpf_common::HelperFunctionID as ID;

// Alias the type to make the table below more concise
type HF = HelperFunction;

/// List of all helpers together with their corresponding numbers (used
/// directly as function pointers in the compiled eBPF bytecode).
pub const ALL_HELPERS: [HelperFunction; 30] = [
    HF::new(ID::BPF_DEBUG_PRINT_IDX, bpf_print_debug),
    HF::new(ID::BPF_PRINTF_IDX, bpf_printf),
    HF::new(ID::BPF_STORE_LOCAL_IDX, bpf_store_local),
    HF::new(ID::BPF_STORE_GLOBAL_IDX, bpf_store_global),
    HF::new(ID::BPF_FETCH_LOCAL_IDX, bpf_fetch_local),
    HF::new(ID::BPF_FETCH_GLOBAL_IDX, bpf_fetch_global),
    HF::new(ID::BPF_MEMCPY_IDX, bpf_memcpy),
    HF::new(ID::BPF_NOW_MS_IDX, bpf_now_ms),
    HF::new(ID::BPF_ZTIMER_NOW_IDX, bpf_ztimer_now),
    HF::new(ID::BPF_PERIODIC_WAKEUP_IDX, bpf_periodic_wakeup),
    HF::new(ID::BPF_SAUL_REG_FIND_NTH_IDX, bpf_saul_reg_find_nth),
    HF::new(ID::BPF_SAUL_REG_FIND_TYPE_IDX, bpf_saul_reg_find_type),
    HF::new(ID::BPF_SAUL_REG_WRITE_IDX, bpf_saul_reg_write),
    HF::new(ID::BPF_SAUL_REG_READ_IDX, bpf_saul_reg_read),
    HF::new(ID::BPF_SAUL_REG_READ_TEMP, bpf_saul_read_temp),
    HF::new(ID::BPF_GCOAP_RESP_INIT_IDX, bpf_gcoap_resp_init),
    HF::new(ID::BPF_COAP_OPT_FINISH_IDX, bpf_coap_opt_finish),
    HF::new(ID::BPF_COAP_ADD_FORMAT_IDX, bpf_coap_add_format),
    HF::new(ID::BPF_COAP_GET_PDU_IDX, bpf_coap_get_pdu),
    HF::new(ID::BPF_STRLEN_IDX, bpf_strlen),
    HF::new(ID::BPF_FMT_S16_DFP_IDX, bpf_fmt_s16_dfp),
    HF::new(ID::BPF_FMT_U32_DEC_IDX, bpf_fmt_u32_dec),
    HF::new(ID::BPF_GPIO_READ_INPUT, bpf_gpio_read_input),
    HF::new(ID::BPF_GPIO_READ_RAW, bpf_gpio_read_raw),
    HF::new(ID::BPF_GPIO_WRITE, bpf_gpio_write),
    HF::new(ID::BPF_HD44780_INIT, bpf_hd44780_init),
    HF::new(ID::BPF_HD44780_CLEAR, bpf_hd44780_clear),
    HF::new(ID::BPF_HD44780_PRINT, bpf_hd44780_print),
    HF::new(ID::BPF_HD44780_SET_CURSOR, bpf_hd44780_set_cursor),
    HF::new(ID::BPF_KEYPAD_GET_INPUT, bpf_keypad_get_input),
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

/// Local storage for the eBPF programs is managed on a per SUIT slot basis.
/// It means that once bytecode of a particular program is loaded into a given
/// SUIT storage slot, a BTreeMap storing the key-value pairs for that program
/// is initialised. Note that it is flushed when the SUIT storage slot is overwritten
/// with some other program. In case of long running VM instances it is possible
/// that the program gets removed from the SUIT storage while the VM is still
/// executing the program. This would cause the long running VM to access local
/// storage of some other program which we don't want. Because of this, the
/// SUIT storage module ensures that a program containing bytecode of a long
/// running VM cannot be overwritten.
pub fn bpf_store_local(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    // Local store/fetch requires changing the VM interpreter to maintain the
    // state of the key-value store btree and will require a bit more work.
    local_storage::local_storage_store(key as usize, value as i32) as u64
}
pub fn bpf_fetch_local(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let value = value as *mut i32;
    unsafe {
        *value = local_storage::local_storage_fetch(key as usize).unwrap_or(0);
    }
    return 0;
}

pub fn bpf_store_global(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    debug!("Storing key: {:#x}, value: {:#x}", key, value);
    //debug!("Arguments to the helper: {:#x}, {:#x}, {:#x}, {:#x}, {:#x}", key, value, _a3, _a4, _a5);
    // We need to truncate the values as for some reason the higher bits of the
    // registers that are passed in are still set.
    unsafe { bpf_store_update_global(key as u32, value as u32) as u64 }
}

pub fn bpf_fetch_global(key: u64, value: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    debug!("Fetching key: {:#x}, value: {:#x}", key, value);
    unsafe {
        debug!(
            "Actual value in memory: {} ({:#x})",
            *(value as *mut u32),
            *(value as *mut u32)
        );
    }
    //debug!("Arguments to the helper: {:#x}, {:#x}, {:#x}, {:#x}, {:#x}", key, value, _a3, _a4, _a5);
    unsafe { bpf_store_fetch_global(key as u32, value as *mut u32) as u64 }
}

/* Standard library functions */

pub fn bpf_memcpy(dest_p: u64, src_p: u64, size: u64, _a4: u64, _a5: u64) -> u64 {
    let dest: *mut riot_sys::libc::c_void = dest_p as *mut riot_sys::libc::c_void;
    let src: *const riot_sys::libc::c_void = src_p as *const riot_sys::libc::c_void;
    let size = size as u32;
    debug!("Copying {} bytes from {:x} to {:x}", size, src_p, dest_p);
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
/// https://api.riot-os.org/group__drivers__saul.html#:~:text=category%20ID.%20More...-,enum,-%7B%0A%C2%A0%C2%A0SAUL_ACT_ANY
pub fn bpf_saul_reg_find_type(saul_dev_type: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    unsafe { return riot_sys::saul_reg_find_type(saul_dev_type as u8) as u64 }
}

/// Given a pointer to the SAUL device struct, it reads from the device into the
/// provided phydat_t struct.
pub fn bpf_saul_reg_read(dev_ptr: u64, data_ptr: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let data: *mut riot_sys::phydat_t = data_ptr as *mut riot_sys::phydat_t;
    //debug!("Reading from saul device at: {:x}", dev_ptr);
    //debug!("Reading into phydat at: {:x}", data_ptr);
    unsafe { riot_sys::saul_reg_read(dev, data) as u64 }
}

/// Given a pointer to the SAUL device struct, it reads the value 0 from the device into the
/// provided integer.
pub fn bpf_saul_read_temp(dev_ptr: u64, value_ptr: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev: *mut riot_sys::saul_reg_t = dev_ptr as *mut riot_sys::saul_reg_t;
    let mut reading: riot_sys::phydat_t = Default::default();
    unsafe {
        let result = riot_sys::saul_reg_read(dev, &mut reading as *mut riot_sys::phydat_t);
        *(value_ptr as *mut u32) = reading.val[0] as u32;
        result as u64
    }
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
        debug!("coap pkt: {:?}", (*coap_ctx).pkt);
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
pub fn bpf_periodic_wakeup(last_wakeup: u64, period: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let last_wakeup: *mut u32 = last_wakeup as *mut u32;
    let period: u32 = period as u32;
    unsafe { riot_sys::ztimer_periodic_wakeup(riot_sys::ZTIMER_USEC, last_wakeup, period) }

    return 0;
}

/* Format and string functions - implementation */

pub fn bpf_strlen(str_ptr: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let str_ptr = str_ptr as *const i8;
    unsafe {
        let c_str = CStr::from_ptr(str_ptr);
        return c_str.to_bytes().len() as u64;
    }
}

/// Convert 16-bit fixed point number to a decimal string.
/// Returns the length of the resulting string.
pub fn bpf_fmt_s16_dfp(out_p: u64, val: u64, fp_digits: u64, _a4: u64, _a5: u64) -> u64 {
    debug!("Formatting s16 dfp args: {:?}, {:?}, {:?}, {:?}, {:?}", out_p, val, fp_digits, _a4, _a5);

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

pub fn bpf_hd44780_init(_a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev = HD44780LCD::new();
    let dev_ptr: *mut hd44780_t = dev.into();
    return dev_ptr as u64;
}

pub fn bpf_hd44780_clear(dev: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev = HD44780LCD::from(dev as *mut hd44780_t);
    dev.clear();
    return 0;
}
pub fn bpf_hd44780_print(dev: u64, data: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev = HD44780LCD::from(dev as *mut hd44780_t);
    let string = unsafe { CStr::from_ptr(data as *const i8) };
    if let Ok(str) = string.to_str() {
        dev.print(str);
        return 0;
    }
    return 0;
}
pub fn bpf_hd44780_set_cursor(dev: u64, row: u64, column: u64, _a4: u64, _a5: u64) -> u64 {
    let dev = HD44780LCD::from(dev as *mut hd44780_t);
    dev.set_cursor(row as u8, column as u8);
    return 0;
}

pub fn bpf_keypad_get_input(adc_index: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> u64 {
    let dev = KeypadShieldButtons::new(adc_index as u8).unwrap();
    let direction = dev.read_direction();
    return direction as u64;
}
