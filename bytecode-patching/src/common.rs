use goblin::elf::{Elf, Reloc};
use log::{debug, log_enabled, Level};

pub const INSTRUCTION_SIZE: usize = 8;
const SYMBOL_SIZE: usize = 6;

pub const LDDW_INSTRUCTION_SIZE: usize = 16;
pub const LDDW_OPCODE: u32 = 0x18;

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

/// Copies the bytes contained in a specific section in the ELF file.
pub fn extract_section_bytes(
    section_name: &str,
    binary: &Elf<'_>,
    binary_buffer: &[u8],
) -> Vec<u8> {
    debug!("Extracting section: {} ", section_name);
    let mut section_bytes: Vec<u8> = vec![];
    // Iterate over section headers to find the one with a matching name
    for section in &binary.section_headers {
        if Some(section_name) == binary.strtab.get_at(section.sh_name) {
            section_bytes.extend(
                &binary_buffer
                    [section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize],
            );

            if log_enabled!(Level::Debug) {
                debug!("Extracted bytes:");
                print_bytes(&section_bytes);
            };
            return section_bytes;
        }
    }

    section_bytes
}

pub fn find_relocations(binary: &Elf<'_>, buffer: &[u8]) -> Vec<Reloc> {
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

pub fn print_bytes(bytes: &[u8]) {
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

pub fn round_section_length(section: &mut Vec<u8>) {
    if section.len() % INSTRUCTION_SIZE != 0 {
        let padding = INSTRUCTION_SIZE - section.len() % INSTRUCTION_SIZE;
        section.extend(vec![0; padding]);
    }
}
