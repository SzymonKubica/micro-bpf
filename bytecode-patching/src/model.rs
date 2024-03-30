// This module contains constants and abstractions used for modelling
// the binary file.
use log::debug;

pub const INSTRUCTION_SIZE: usize = 8;
pub const LDDW_INSTRUCTION_SIZE: usize = 16;

pub const HEADER_SIZE: usize = 28;
pub const SYMBOL_SIZE: usize = 6;
pub const RELOCATED_CALL_SIZE: usize = 8;

pub const LDDWD_OPCODE: u32 = 0xB8;
pub const LDDWR_OPCODE: u32 = 0xD8;
pub const LDDW_OPCODE: u32 = 0x18;

/// The binary generated after the relocation script has the following format:
/// - Header: Contains the information about the lengths of the remaining sections
///   functions and read-only data. See [`Header`] for more details.
/// - Data section
/// - Read-only data section
/// - Text section: Contains the code of the main entrypoint and the other functions
/// - Symbol structs: TODO: figure out why we need this
/// - Relocated function calls: custom metadata specifying how function calls should be relocated
pub struct Binary {
    pub header: Header,
    pub data: Vec<u8>,
    pub rodata: Vec<u8>,
    pub text: Vec<u8>,
    pub functions: Vec<Symbol>,
    pub relocated_calls: Vec<RelocatedCall>,
}

impl Into<Vec<u8>> for Binary {
    fn into(self) -> Vec<u8> {
        let header_bytes = unsafe {
            std::slice::from_raw_parts(&self.header as *const _ as *const u8, HEADER_SIZE)
        };
        let mut binary_data = Vec::from(header_bytes);
        binary_data.extend(self.data);
        binary_data.extend(self.rodata);
        binary_data.extend(self.text);

        for symbol in self.functions {
            let symbol: &[u8] = (&symbol).into();
            binary_data.extend(symbol);
        }

        for call in self.relocated_calls {
            debug!("Adding a relocated call: {:?}", call);
            let call_bytes: &[u8] = (&call).into();
            debug!("Call bytes: {:?}", call_bytes);
            binary_data.extend(call_bytes);
        }
        binary_data
    }
}

/// A header that is appended at the start of the generated binary. Contains
/// information about the length of the correspoinding sections in the binary
/// so that the VM executing the code can access the .rodata and .data sections
/// properly.
#[repr(C, packed)]
pub struct Header {
    pub magic: u32,
    pub version: u32,
    pub flags: u32,
    pub data_len: u32,
    pub rodata_len: u32,
    pub text_len: u32,
    pub functions_len: u32,
}

/// A symbol struct represents a function.
#[repr(C, packed)]
pub struct Symbol {
    // Offset to the name of the function in the .rodata section
    pub name_offset: u16,
    pub flags: u16,
    // Offset of the function in the .text section
    pub location_offset: u16,
}

impl<'a> Into<&'a [u8]> for &'a Symbol {
    fn into(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, SYMBOL_SIZE) }
    }
}

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
        unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
    }
}

impl<'a> Into<&'a [u8]> for &'a Lddw {
    fn into(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, LDDW_INSTRUCTION_SIZE) }
    }
}

/// A custom struct indicating that at a given instruction offset a call
/// `call -1` should be replaced with a call the function at a given offset
/// in the .text section.
#[derive(Debug)]
#[repr(C, packed)]
pub struct RelocatedCall {
    pub instruction_offset: u32,
    pub function_text_offset: u32,
}

impl<'a> Into<&'a [u8]> for &'a RelocatedCall {
    fn into(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, RELOCATED_CALL_SIZE) }
    }
}
