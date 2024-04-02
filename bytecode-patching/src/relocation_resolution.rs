use alloc::{
    string::{String, ToString},
    vec,
};
use log::debug;

use crate::{
    common::{find_relocations, LDDW_OPCODE},
    debug_print_program_bytes,
    model::Lddw,
};

/// Applies relocations to the given program binary.
///
/// The relocations are performed in-place by replacing placeholder instructions
/// such as `call -1` or `lddw 0` with the actual offsets according to the
/// relocation information specified in the ELF file.
///
/// The intended use of this function is to resolve relocations after the program
/// has been loaded into the memory of the microcontroller running the eBPF VM.
/// This way, we are able to support the `.data` relocations and achieve good
/// compatibility with respect to the types of programs that can be supported.
///
/// Limitations of this approach are:
/// - the relocations in the ELF file need to be resolved each time we want to
///   load the program and execute it in the VM. This can be slow if the program
///   has many relocation entries.
/// - the size of raw ELF files can be up to 10x larger than the size of binaries
///   produced using alternative approaches (e.g. extracting just the `.text` section)
///   because of this, it is recommended that the object file is pre-processed
///   with the `strip` command to remove the redundant debug information before
///   it is sent to the microcontroller where the actual relocations take place.
pub fn resolve_relocations(program: &mut [u8]) -> Result<(), String> {
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

        debug!("Patched text section: ");
        debug_print_program_bytes(&text);
    }

    Ok(())
}
