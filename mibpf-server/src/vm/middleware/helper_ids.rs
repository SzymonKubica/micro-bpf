//! This module defines all available helper IDs. The requirement is that every
//! single helper function ID is unique, hence we store them in an enum.
//! Files containing helper definitions should depend on enumeration variants
//! defined in this file.
//!
//! In case of the helper functions that were implemented for the FemtoContainer
//! VM, we use the same set of IDs for compatibility.

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum HelperTableID {
    /* Print/debug helper functions */
    BPF_PRINTF_IDX = 0x01,
    BPF_DEBUG_PRINT_IDX = 0x03,

    /* Memory copy helper functions */
    BPF_MEMCPY_IDX = 0x02,

    /* Key/value store functions */
    BPF_STORE_LOCAL_IDX = 0x10,
    BPF_STORE_GLOBAL_IDX = 0x11,
    BPF_FETCH_LOCAL_IDX = 0x12,
    BPF_FETCH_GLOBAL_IDX = 0x13,

    /* Saul functions */
    BPF_SAUL_REG_FIND_NTH_IDX = 0x30,
    BPF_SAUL_REG_FIND_TYPE_IDX = 0x31,
    BPF_SAUL_REG_READ_IDX = 0x32,
    BPF_SAUL_REG_WRITE_IDX = 0x33,

    /* (g)coap functions */
    BPF_GCOAP_RESP_INIT_IDX = 0x40,
    BPF_COAP_OPT_FINISH_IDX = 0x41,
    BPF_COAP_ADD_FORMAT_IDX = 0x42,
    BPF_COAP_GET_PDU_IDX = 0x43,

    /* Format and string functions */
    BPF_STRLEN_IDX = 0x52,
    BPF_FMT_S16_DFP_IDX = 0x50,
    BPF_FMT_U32_DEC_IDX = 0x51,

    /* Time(r) functions */
    BPF_NOW_MS_IDX = 0x20,

    /* ZTIMER */
    BPF_ZTIMER_NOW_IDX = 0x60,
    BPF_PERIODIC_WAKEUP_IDX = 0x61,

    BPF_GPIO_READ_INPUT = 0x70,
    BPF_GPIO_READ_RAW = 0x71,
    BPF_GPIO_WRITE = 0x72,

    /* HD44780 LCD */
    BPF_HD44780_INIT = 0x80,
    BPF_HD44780_CLEAR = 0x81,
    BPF_HD44780_PRINT = 0x82,
    BPF_HD44780_SET_CURSOR = 0x83,
}

impl Into<u32> for HelperTableID {
    fn into(self) -> u32 {
        self as u32
    }
}
