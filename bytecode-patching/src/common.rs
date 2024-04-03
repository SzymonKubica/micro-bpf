use alloc::string::{String, ToString};
use alloc::vec::Vec;
use goblin::container::{Container, Endian};
use goblin::elf::{Elf, Reloc, SectionHeader};
use log::{debug, log_enabled, Level};

pub const INSTRUCTION_SIZE: usize = 8;
const SYMBOL_SIZE: usize = 6;

pub const LDDW_INSTRUCTION_SIZE: usize = 16;
pub const LDDW_OPCODE: u32 = 0x18;
pub const CALL_OPCODE: u32 = 0x85;

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
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, SYMBOL_SIZE) }
    }
}

/// Prints program bytes dividing them into rows of 8 bytes and printing the
/// row number in hex. This is done to resemble the output of utilities such as
/// `objdump`.
pub fn debug_print_program_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        if i % INSTRUCTION_SIZE == 0 {
            debug!("{:02x}: ", i);
        }
        debug!("{:02x} ", byte);
        if (i + 1) % INSTRUCTION_SIZE == 0 {
            debug!("");
        }
    }
}

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
        if let Some(name) = binary.strtab.get_at(section.sh_name) {
            if name == section_name {
                let section_start = section.sh_offset as usize;
                let section_end = (section.sh_offset + section.sh_size) as usize;
                return Ok(&program[section_start..section_end]);
            }
        }
    }

    return Err("Section not found".to_string());
}

/// Given the section offset and length information contained in the SectionHeader
/// it returns a mutable slice corresponding to the section data.
pub fn get_section_reference_mut<'a>(
    section: &SectionHeader,
    program: &'a mut [u8],
) -> &'a mut [u8] {
    let section_start = section.sh_offset as usize;
    let section_end = (section.sh_offset + section.sh_size) as usize;
    &mut program[section_start..section_end]
}

/// Extracts a mutable reference to a section with a given name from the ELF binary.
/// See [`extract_section`] for more details.
pub fn extract_section_mut<'a>(
    section_name: &'static str,
    program: &'a mut [u8],
) -> Result<&'a mut [u8], String> {
    let Ok(binary) = goblin::elf::Elf::parse(&program) else {
        return Err("Failed to parse the ELF binary".to_string());
    };

    for section in &binary.section_headers {
        if let Some(name) = binary.strtab.get_at(section.sh_name) {
            debug!("Section name: {}", name);
            if name == section_name {
                let section_start = section.sh_offset as usize;
                let section_end = (section.sh_offset + section.sh_size) as usize;
                return Ok(&mut program[section_start..section_end]);
            }
        }
    }

    return Err("Section not found".to_string());
}

/// Copies the bytes contained in a specific section in the ELF file.
pub fn get_section_bytes(section_name: &str, binary: &Elf<'_>, binary_buffer: &[u8]) -> Vec<u8> {
    debug!("Extracting section: {} ", section_name);
    let mut section_bytes: Vec<u8> = alloc::vec![];
    // Iterate over section headers to find the one with a matching name
    for section in &binary.section_headers {
        if Some(section_name) == binary.strtab.get_at(section.sh_name) {
            section_bytes.extend(
                &binary_buffer
                    [section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize],
            );

            if log_enabled!(Level::Debug) {
                debug!("Extracted bytes:");
                debug_print_program_bytes(&section_bytes);
            };
            return section_bytes;
        }
    }

    section_bytes
}

pub fn find_relocations(binary: &Elf<'_>, buffer: &[u8]) -> Vec<Reloc> {
    let mut relocations = alloc::vec![];

    let context = goblin::container::Ctx::new(Container::Big, Endian::Little);

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

pub fn round_section_length(section: &mut Vec<u8>) {
    if section.len() % INSTRUCTION_SIZE != 0 {
        let padding = INSTRUCTION_SIZE - section.len() % INSTRUCTION_SIZE;
        section.extend(alloc::vec![0; padding]);
    }
}
