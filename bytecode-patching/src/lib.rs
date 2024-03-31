mod common;
mod femtocontainer_relocations;
mod model;
mod relocation_resolution;
extern crate alloc;

use femtocontainer_relocations::assemble_femtocontainer_binary;
use log::{debug, log_enabled, Level};

use common::*;
use model::*;

/// Relocate subcommand is responsible for performing the post-processing of the
/// compiled eBPF bytecode before it can be loaded onto the target device. It
/// handles function relocations and read only data relocations.
pub fn perform_relocations(
    source_object_file: &str,
    binary_file: Option<String>,
    strip_debug: bool,
) -> Result<(), String> {
    let binary_data = assemble_femtocontainer_binary(&source_object_file)?;

    let mut f = File::create(file_name).unwrap();
    if log_enabled!(Level::Debug) {
        debug!("Generated binary:");
        print_bytes(&binary_data);
    }
    f.write_all(&binary_data).unwrap();

    Ok(())
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
