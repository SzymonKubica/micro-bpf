// This module contains constants and abstractions used for modelling
// the binary file.

use crate::common::LDDW_INSTRUCTION_SIZE;

/// Load-double-word instruction, needed for bytecode patching for loads from
/// .data and .rodata sections.
#[repr(C, packed)]
pub struct Lddw {
    pub opcode: u8,
    pub registers: u8,
    pub offset: u16,
    pub immediate_l: u32,
    pub null1: u8,
    pub null2: u8,
    pub null3: u16,
    pub immediate_h: u32,
}

impl From<&[u8]> for Lddw {
    fn from(bytes: &[u8]) -> Self {
        unsafe { core::ptr::read(bytes.as_ptr() as *const _) }
    }
}

impl<'a> Into<&'a [u8]> for &'a Lddw {
    fn into(self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, LDDW_INSTRUCTION_SIZE) }
    }
}

pub const RELOCATED_CALL_SIZE: usize = 8;
/// A custom struct indicating that at a given instruction offset a call
/// `call -1` should be replaced with a call the function at a given offset
/// in the .text section. It is used by the extended relocation scripts to allow
/// for using calls to functions inside of the program which aren't PC relative.
#[derive(Debug)]
#[repr(C, packed)]
pub struct RelocatedCall {
    pub instruction_offset: u32,
    pub function_text_offset: u32,
}

impl<'a> Into<&'a [u8]> for &'a RelocatedCall {
    fn into(self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, RELOCATED_CALL_SIZE) }
    }
}
