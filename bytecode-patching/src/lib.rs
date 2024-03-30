use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    process::Command,
};

use goblin::{
    elf::{Elf, Reloc},
    elf64::sym::{STB_GLOBAL, STT_FUNC, STT_OBJECT, STT_SECTION},
};
use log::{debug, log_enabled, Level};

const INSTRUCTION_SIZE: usize = 8;
const LDDW_INSTRUCTION_SIZE: usize = 16;

const HEADER_SIZE: usize = 28;
const SYMBOL_SIZE: usize = 6;
const RELOCATED_CALL_SIZE: usize = 8;

const LDDWD_OPCODE: u32 = 0xB8;
const LDDWR_OPCODE: u32 = 0xD8;
const LDDW_OPCODE: u32 = 0x18;

/// The binary generated after the relocation script has the following format:
/// - Header: Contains the information about the lengths of the remaining sections
///   functions and read-only data. See [`Header`] for more details
/// - Data section
/// - Read-only data section
/// - Text section: Contains the code of the main entrypoint and the other functions
/// - Symbol structs: TODO: figure out why we need this
/// - Relocated function calls: custom metadata specifying how function calls should be relocated
struct Binary {
    header: Header,
    data: Vec<u8>,
    rodata: Vec<u8>,
    text: Vec<u8>,
    functions: Vec<Symbol>,
    relocated_calls: Vec<RelocatedCall>,
}

/// Relocate subcommand is responsible for performing the post-processing of the
/// compiled eBPF bytecode before it can be loaded onto the target device. It
/// handles function relocations and read only data relocations.
pub fn perform_relocations(
        source_object_file : &str,
        binary_file: Option<String>,
        strip_debug: bool,
) -> Result<(), String> {

    if strip_debug {
        let _ = strip_binary(source_object_file, binary_file.as_ref());
        println!("Relocating the original binary");
        let mut buffer = read_file_as_bytes(source_object_file);
        let _ = relocate_in_place(&mut buffer);
        println!("Now relocating the stripped binary");
        let mut buffer = read_file_as_bytes(binary_file.as_ref().unwrap());
        return relocate_in_place(&mut buffer);
    }

    let file_name = if let Some(binary_file) = binary_file {
        binary_file.clone()
    } else {
        "a.bin".to_string()
    };

    let binary_data = get_relocated_bytes(&source_object_file)?;

    let mut f = File::create(file_name).unwrap();
    if log_enabled!(Level::Debug) {
        debug!("Generated binary:");
        print_bytes(&binary_data);
    }
    f.write_all(&binary_data).unwrap();

    Ok(())
}

pub fn strip_binary(source_object_file: &str, binary_file: Option<&String>) -> Result<(), String> {
    // strip bpf/helper-tests/out/gcoap.o -d -R .BTF -R .BTF.ext -o test
    let file_name = if let Some(binary_file) = binary_file {
        binary_file.clone()
    } else {
        "a.bin".to_string()
    };

    let result = Command::new("strip")
        .arg(source_object_file)
        .arg("-d")
        .arg("-R")
        .arg(".BTF")
        .arg("-R")
        .arg(".BTF.ext")
        .arg("-o")
        .arg(file_name)
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to strip the binary: {}", e)),
    }
}
pub fn relocate_in_place(buffer: &mut [u8]) -> Result<(), String> {
    let program_address = buffer.as_ptr() as usize;
    let Ok(binary) = goblin::elf::Elf::parse(&buffer) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    let text_section = binary.section_headers.get(1).unwrap();

    let relocations = find_relocations(&binary, &buffer);
    let mut relocations_to_patch = vec![];
    for relocation in relocations {
        debug!(
            "Relocation found: offset: {:x}, r_addend: {:?}, r_sym: {}, r_type: {}",
            relocation.r_offset, relocation.r_addend, relocation.r_sym, relocation.r_type
        );
        if let Some(symbol) = binary.syms.get(relocation.r_sym) {
            // Here the value of the relocation tells us the offset in the binary
            // where the data that needs to be relocated is located.
            debug!(
                "Looking up the relocation symbol: name: {}, section: {}, value: {:x}, is_function? : {}",
                symbol.st_name,
                symbol.st_shndx,
                symbol.st_value,
                symbol.is_function()
            );
            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
            debug!(
                "The relocation symbol section is located at offset {:x}",
                section.sh_offset
            );

            relocations_to_patch.push((
                relocation.r_offset as usize,
                program_address as u32 + section.sh_offset as u32 + symbol.st_value as u32,
            ));
        }
    }

    let text = &mut buffer
        [text_section.sh_offset as usize..(text_section.sh_offset + text_section.sh_size) as usize];
    for (offset, value) in relocations_to_patch {
        if offset >= text.len() {
            continue;
        }

        debug!(
            "Patching text section at offset: {:x} with new immediate value: {:x}",
            offset, value
        );
        // we patch the text here
        // We only patch LDDW instructions
        if text[offset] != LDDW_OPCODE as u8 {
            debug!("No LDDW instruction at {} offset in .text section", offset);
            continue;
        }

        // We instantiate the instruction struct to modify it
        let instr_bytes = &text[offset..offset + 16];

        let mut instr: Lddw = Lddw::from(instr_bytes);
        // Also add the program base address here when relocating on the actual device
        instr.immediate_l += value;
        text[offset..offset + 16].copy_from_slice((&instr).into());

        println!("Patched text section: ");
        print_bytes(&text);
    }

    Ok(())
}

pub fn get_relocated_bytes(source_object_file: &str) -> Result<Vec<u8>, String> {
    // Read in the object file into the buffer.
    let buffer = read_file_as_bytes(source_object_file);
    let Ok(binary) = goblin::elf::Elf::parse(&buffer) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    // First extract the bytes contained in all three main sections
    let mut text: Vec<u8> = extract_section_bytes(".text", &binary, &buffer);
    let mut data: Vec<u8> = extract_section_bytes(".data", &binary, &buffer);
    let mut rodata: Vec<u8> = extract_section_bytes(".rodata", &binary, &buffer);

    // Now handle all string literals that aren't placed in .rodata
    // section by default. We need to append them to the .rodata section
    // and maintain the information about the offsets at which they are
    // stored so that we can relocate loads from them later on.
    let str_section_offsets = append_string_literals(&mut rodata, &binary, &buffer);

    // Now we need to collect all global functions and append their names
    // to the rodata section. We also need to maintain the information
    // about the offsets at which the function names are stored.
    // This is maintained for compatibility with the rbpf bytecode patching
    // script. It isn't actually used by their VM.
    let symbol_structs: Vec<Symbol> = extract_function_symbols(&mut rodata, &binary);

    let relocated_calls: Vec<RelocatedCall> = find_relocated_calls(&binary, &buffer);

    resolve_rodata_relocations(&mut text, &binary, &buffer, &str_section_offsets);

    round_section_length(&mut data);
    round_section_length(&mut rodata);

    // Now we write the new binary file
    let header = Header {
        magic: 123,
        version: 0,
        flags: 0,
        data_len: data.len() as u32,
        rodata_len: rodata.len() as u32,
        text_len: text.len() as u32,
        functions_len: symbol_structs.len() as u32,
    };

    let output_binary: Binary = Binary {
        header,
        data,
        rodata,
        text,
        functions: symbol_structs,
        relocated_calls,
    };

    Ok(output_binary.into())
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
    magic: u32,
    version: u32,
    flags: u32,
    data_len: u32,
    rodata_len: u32,
    text_len: u32,
    functions_len: u32,
}

/// A symbol struct represents a function.
#[repr(C, packed)]
struct Symbol {
    // Offset to the name of the function in the .rodata section
    name_offset: u16,
    flags: u16,
    // Offset of the function in the .text section
    location_offset: u16,
}

impl<'a> Into<&'a [u8]> for &'a Symbol {
    fn into(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, SYMBOL_SIZE) }
    }
}

/// Load-double-word instruction, needed for bytecode patching for loads from
/// .data and .rodata sections.
#[repr(C, packed)]
struct Lddw {
    opcode: u8,
    registers: u8,
    offset: u16,
    immediate_l: u32,
    null1: u8,
    null2: u8,
    null3: u16,
    immediate_h: u32,
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
struct RelocatedCall {
    instruction_offset: u32,
    function_text_offset: u32,
}

impl<'a> Into<&'a [u8]> for &'a RelocatedCall {
    fn into(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, RELOCATED_CALL_SIZE) }
    }
}

/// String literals used in e.g. calls to printf are loaded into the
/// .rodata.str.1 section, we need to move them over to the rodata section.
/// In order to perform relocations properly later on, we need to maintain
/// the map from the name of the additional rodata section to the offset
/// to it relative to the original rodata section. This map is returned from this
/// functio.
fn append_string_literals<'a>(
    rodata: &mut Vec<u8>,
    binary: &Elf<'a>,
    buffer: &[u8],
) -> HashMap<&'a str, usize> {
    let mut str_section_offsets = std::collections::HashMap::new();

    for section in &binary.section_headers {
        if let Some(section_name) = binary.strtab.get_at(section.sh_name) {
            // The string literals are stored in the .rodata.str.1 section
            if section_name.contains(".rodata.") {
                str_section_offsets.insert(section_name, rodata.len());
                rodata.extend(
                    &buffer[section.sh_offset as usize
                        ..(section.sh_offset + section.sh_size) as usize],
                );
            }
        }
    }

    debug!(
        "Additional read-only string sections: {:?}",
        str_section_offsets
    );

    str_section_offsets
}

fn extract_function_symbols(rodata: &mut Vec<u8>, binary: &Elf<'_>) -> Vec<Symbol> {
    let mut symbol_structs: Vec<Symbol> = vec![];
    for symbol in binary.syms.iter() {
        if symbol.st_type() == STT_FUNC && symbol.st_bind() == STB_GLOBAL {
            let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();

            debug!("Found global function: {}", symbol_name);
            let offset_within_text = symbol.st_value as usize;
            let offset = rodata.len();
            let name_cstr = std::ffi::CString::new(symbol_name).unwrap();
            rodata.extend(name_cstr.to_bytes().iter());
            // Added flags for compatiblity with rbpf
            let flags = 0;
            symbol_structs.push(Symbol {
                name_offset: offset as u16,
                flags: flags as u16,
                location_offset: offset_within_text as u16,
            });
        }
    }
    symbol_structs
}

fn find_relocated_calls(binary: &Elf<'_>, buffer: &[u8]) -> Vec<RelocatedCall> {
    let mut relocated_calls: Vec<RelocatedCall> = vec![];
    let relocations = find_relocations(binary, buffer);
    for reloc in relocations {
        debug!("Relocation found : {:?}", reloc);
        if let Some(symbol) = binary.syms.get(reloc.r_sym) {
            if symbol.st_type() == STT_FUNC {
                let name = binary.strtab.get_at(symbol.st_name).unwrap();
                debug!(
                    "Relocation at instruction {} for function {} at {}",
                    reloc.r_offset, name, symbol.st_value
                );
                relocated_calls.push(RelocatedCall {
                    instruction_offset: reloc.r_offset as u32,
                    function_text_offset: symbol.st_value as u32,
                });
            }
        }
    }
    relocated_calls
}

fn resolve_rodata_relocations(
    text: &mut Vec<u8>,
    binary: &Elf<'_>,
    buffer: &[u8],
    str_section_offsets: &HashMap<&str, usize>,
) {
    let relocations = find_relocations(binary, buffer);
    for relocation in relocations {
        if let Some(symbol) = binary.syms.get(relocation.r_sym) {
            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
            let section_name = binary.strtab.get_at(section.sh_name).unwrap();
            match symbol.st_type() {
                STT_SECTION => {
                    debug!(
                        "Relocation at instruction {} for section {} at {}",
                        relocation.r_offset, section_name, symbol.st_value
                    )
                }
                STT_FUNC => continue, // We don't patch for functions
                _ => {
                    let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();
                    debug!(
                        "Relocation at instruction {} for symbol {} in {} at {}",
                        relocation.r_offset, symbol_name, section_name, symbol.st_value
                    )
                }
            }
        }

        patch_text(text, binary, relocation, &str_section_offsets);
    }
}

fn find_relocations(binary: &Elf<'_>, buffer: &[u8]) -> Vec<Reloc> {
    let mut relocations = vec![];
    let context = goblin::container::Ctx::default();
    print!("Relocation parsing context: {:?}", context);
    for section in &binary.section_headers {
        if section.sh_type == goblin::elf::section_header::SHT_REL {
            let offset = section.sh_offset as usize;
            let size = section.sh_size as usize;
            let relocs =
                goblin::elf::reloc::RelocSection::parse(&buffer, offset, size, false, context)
                    .unwrap();
            relocs.iter().for_each(|reloc| relocations.push(reloc));
        }
    }

    relocations
}

fn patch_text(
    text: &mut [u8],
    binary: &Elf<'_>,
    reloc: Reloc,
    str_section_offsets: &HashMap<&str, usize>,
) {
    debug!("Patching text for relocation symbol: {:?}", reloc);
    let symbol = binary.syms.get(reloc.r_sym).unwrap();
    let section = binary.section_headers.get(symbol.st_shndx).unwrap();
    let section_name = binary.strtab.get_at(section.sh_name).unwrap();
    let mut offset = 0;

    // We don't do eny relocations in case of functions as they are handled
    // in a custom way by the VM (we append their relocation structs at the end of the binary
    // file)
    if symbol.st_type() == STT_FUNC {
        debug!("No patching is performed for function calls.");
        return;
    }

    if symbol.st_type() == STT_SECTION {
        if let Some(off) = str_section_offsets.get(section_name) {
            offset = *off;
        } else {
            debug!("No offset found for section: {}", section_name);
            return;
        }
    } else if symbol.st_type() == STT_OBJECT {
        offset = symbol.st_value as usize;
    }

    // We only patch LDDW instructions
    if text[reloc.r_offset as usize] != LDDW_OPCODE as u8 {
        debug!("No LDDW instruction at {}", reloc.r_offset);
        return;
    }

    let opcode = if section_name.contains(".rodata.str") {
        LDDWR_OPCODE
    } else {
        LDDWD_OPCODE
    };

    // We instantiate the instruction struct to modify it
    let instr_bytes = &text[reloc.r_offset as usize..reloc.r_offset as usize + 16];
    debug!(
        "Replacing {:?} at {} with {} at {}",
        instr_bytes, reloc.r_offset, opcode, reloc.r_offset
    );

    let mut instr: Lddw = Lddw::from(instr_bytes);
    instr.opcode = opcode as u8;
    instr.immediate_l += offset as u32;

    text[reloc.r_offset as usize..reloc.r_offset as usize + 16].copy_from_slice((&instr).into());
}

fn read_file_as_bytes(source_object_file: &str) -> Vec<u8> {
    let mut f = File::open(&source_object_file).expect("File not found.");
    let metadata = fs::metadata(&source_object_file).expect("Unable to read file metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}

/// Copies the bytes contained in a specific section in the ELF file.
fn extract_section_bytes(section_name: &str, binary: &Elf<'_>, binary_buffer: &[u8]) -> Vec<u8> {
    debug!("Extracting section: {} ", section_name);
    let mut section_bytes: Vec<u8> = vec![];
    // Iterate over section headers to find the one with a matching name
    for section in &binary.section_headers {
        let name = binary.strtab.get_at(section.sh_name);

        if let Some(other_section_name) = name {
            if other_section_name == section_name {
                section_bytes.extend(
                    &binary_buffer[section.sh_offset as usize
                        ..(section.sh_offset + section.sh_size) as usize],
                );
            }
        }
    }

    if log_enabled!(Level::Debug) {
        debug!("Extracted bytes:");
        print_bytes(&section_bytes);
    };
    section_bytes
}

fn print_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        if i % INSTRUCTION_SIZE == 0 {
            print!("{:02x}: ", i);
        }
        print!("{:02x} ", byte);
        if (i + 1) % INSTRUCTION_SIZE == 0 {
            println!();
        }
    }
}

fn round_section_length(section: &mut Vec<u8>) {
    if section.len() % INSTRUCTION_SIZE != 0 {
        let padding = INSTRUCTION_SIZE - section.len() % INSTRUCTION_SIZE;
        section.extend(vec![0; padding]);
    }
}
