use alloc::{
    string::{String, ToString},
    vec,
};
use log::{debug, error};

use crate::{
    common::{
        find_relocations, get_section_reference_mut, CALL_OPCODE, INSTRUCTION_SIZE,
        LDDW_INSTRUCTION_SIZE, LDDW_OPCODE,
    },
    model::{Call, Lddw},
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
    let program_addr = program.as_ptr() as usize;
    let Ok(binary) = goblin::elf::Elf::parse(&program) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    let relocations = find_relocations(&binary, &program);
    let mut relocations_to_patch = vec![];
    for relocation in relocations {
        debug!("Relocation found: {:?}", relocation);
        if let Some(symbol) = binary.syms.get(relocation.r_sym) {
            // Here the value of the relocation tells us the offset in the binary
            // where the data that needs to be relocated is located.
            debug!("Relocation symbol found: {:?}", symbol);
            let section = binary.section_headers.get(symbol.st_shndx).unwrap();
            debug!(
                "Symbol is located in section at offset {:x}",
                section.sh_offset
            );

            let relocated_addr = (program_addr as u64 + section.sh_offset + symbol.st_value) as u32;
            relocations_to_patch.push((relocation.r_offset as usize, relocated_addr));
        }
    }

    let Some(text_section) = binary.section_headers.get(1) else {
        return Err("Failed to find the .text section".to_string());
    };
    let text = get_section_reference_mut(text_section, program);

    for (offset, value) in relocations_to_patch {
        if offset >= text.len() {
            error!("Found a relocation outside of the .text section");
            continue;
        }

        debug!(
            "Patching text section at offset: {:x} with new immediate value: {:x}",
            offset, value
        );
        match text[offset] as u32 {
            LDDW_OPCODE => {
                let mut instr: Lddw = Lddw::from(&text[offset..offset + LDDW_INSTRUCTION_SIZE]);
                instr.immediate_l = value;
                text[offset..offset + LDDW_INSTRUCTION_SIZE].copy_from_slice((&instr).into());
            }
            CALL_OPCODE => {
                let mut instr: Call = Call::from(&text[offset..offset + INSTRUCTION_SIZE]);
                instr.registers = 0x3 << 4;
                instr.immediate = value;
                text[offset..offset + INSTRUCTION_SIZE].copy_from_slice((&instr).into());
            }
            _ => {
                error!("Unsupported relocation opcode at offset: {:x}", offset);
            }
        }

        //debug!("Patched text section: ");
        //debug_print_program_bytes(&text);
    }

    Ok(())
}
