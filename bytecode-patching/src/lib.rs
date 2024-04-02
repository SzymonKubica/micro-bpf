#![no_std]
#![warn(missing_docs)]

//! Library for manipulating ELF files to allow for executing them on different
//! implementations of an eBPF VM on microcontrollers.
//!
//! The main purpose of this library it to act similar to a static linker for
//! eBPF programs to allow for executing them in constrained environments.
//! Depending on implementations details of the eBPF VM used for executing the
//! programs, different pre-processing steps of the object files are required.
//!
//! The program lifecycle is as follows:
//! - The program is compiled using `clang` and `llc` for the `bpf` target ISA.
//! - The resulting object file is processed to make it compatible with a particular
//!   version of the VM.
//!
//! Currently the following pre-processing steps are supported:
//! - Extracting the `.text` section from the ELF file and executing only that
//!   (this is used by the original version of the rbpf VM and is the simplest
//!   approach, albeit it lacks generality and the space of supported programs
//!   is limited)
//! - Applying ahead-of-time relocations used by the Femto-Container version
//!   of the VM. This allows for accessing the `.data` and `.rodata` sections
//!   of the program by using special instructions that weren't originally included
//!   in the eBPF ISA.
//! - Applying extended AOT relocations to allow for calling non-static functions
//!   inside of the eBPF programs (adds support for non-PC-relative function calls)
//!
//! The second workflow that supported by the library involves sending raw ELF
//! object files to the target microcontroller device and performing relocations
//! there once the program memory address is known. This allows for achieving
//! the best compatibility, however it comes with a performance overhead of
//! parsing the ELF file on the device each time the program is loaded.
//!
//! In order to support the second type of the relocation workflow, this library
//! supports `no_std`.
mod common;
mod femtocontainer_relocations;
mod model;
mod relocation_resolution;
extern crate alloc;

use alloc::{string::String, vec};
pub use common::print_program_bytes;
pub use femtocontainer_relocations::assemble_femtocontainer_binary;
use log::debug;

use common::*;
use model::*;

/// Extracts the section with a given name from the ELF binary.
///
/// It returns a slice containing the bytes corresponding to the required section
/// inside of the ELF file. This relies on the section headers and the string
/// symbol table being present in the ELF file whose bytes are given in the
/// `program` argument. If this information has been stripped off the ELF file,
/// this function will fail to find it and return an error.
///
/// # Examples
/// For instance, we can extract the bytes contained in the `.text` section
/// inside of the elf file as follows:
/// ```
/// let program_bytes = read_bytes_from_file(source_object_file);
/// let text_section_bytes = extract_section(".text", &program_bytes)?;
/// ```
/// The `program_bytes` variable above contains **all** bytes in the ELF file
/// (together with the header section). It is very important that the array of
/// bytes corresponds to an actual ELF file, otherwise the function will not
/// be able to parse it correctly and the required section will not be found.
pub fn extract_section<'a>(
    section_name: &'static str,
    program: &'a [u8],
) -> Result<&'a [u8], String> {
    let Ok(binary) = goblin::elf::Elf::parse(&program) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    for section in &binary.section_headers {
        if Some(section_name) == binary.strtab.get_at(section.sh_name) {
            let section_start = section.sh_offset as usize;
            let section_end = (section.sh_offset + section.sh_size) as usize;
            return Ok(&program[section_start..section_end]);
        }
    }

    return Err("Section not found".to_string());
}

pub fn relocate_in_place(program: &mut [u8]) -> Result<(), String> {
    let program_address = program.as_ptr() as usize;
    let Ok(binary) = goblin::elf::Elf::parse(&program) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    let text_section = binary.section_headers.get(1).unwrap();

    let relocations = find_relocations(&binary, &program);
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

    let text = &mut program
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
        print_program_bytes(&text);
    }

    Ok(())
}
