//! This module is responsible for providing access to global storage for
//! jitted programs. Its interface is designed to be similar to the [`super::suit_storage`].
//!
//! The idea is that we have a number of jit storage slots of fixed size which
//! are statically allocated. Then the clients who want to store jitted programs
//! need to  obtain a mutable reference to the contents of one of the slots,
//! write the program there and then execute it by casting into a function pointer.
//!
//! Note that by default the number of JIT storage slots is half of the number
//! of actual SUIT storage slots to save memory.

use alloc::{format, string::String};
use log::debug;
use riot_wrappers::mutex::{Mutex, MutexGuard};

use super::suit_storage::{SUIT_STORAGE_SLOTS, SUIT_STORAGE_SLOT_SIZE};

pub const JIT_STORAGE_SLOTS_NUM: usize = SUIT_STORAGE_SLOTS / 2;
pub const JIT_SLOT_SIZE: usize = SUIT_STORAGE_SLOT_SIZE;

/// Each slot is a tuple of the program bytes and an offset to the start of the
/// .text section inside of the program. This offset is needed so that when we
/// execute the program we don't call into the start address of the compiled
/// program but call into the specific offset to ensure that we start executing
/// from the first instruction in the program and not from e.g. some contents
/// of the .data or .rodata sections.
static JIT_PROGRAM_SLOTS: [Mutex<([u8; JIT_SLOT_SIZE], usize)>; JIT_STORAGE_SLOTS_NUM] =
    [EMPTY_SLOT; JIT_STORAGE_SLOTS_NUM];

/// Default empty slot in the JIT program storage, it needs to be protected by
/// a mutex as it can be accessed by multiple threads but only one at the time
/// can be writing a program to it (multiple threads can execute a single program
/// as it is a read-only operation).
const EMPTY_SLOT: Mutex<([u8; JIT_SLOT_SIZE], usize)> = Mutex::new(([0; JIT_SLOT_SIZE], 0));

// We globally maintain whether a slot is in use or not
static JIT_SLOT_STATE: Mutex<[bool; JIT_STORAGE_SLOTS_NUM]> =
    Mutex::new([false; JIT_STORAGE_SLOTS_NUM]);

// Global dictionary of the offsets to the .text sections in the jitted programs
static JIT_SLOT_TEXT_OFFSETS: Mutex<[usize; JIT_STORAGE_SLOTS_NUM]> =
    Mutex::new([0; JIT_STORAGE_SLOTS_NUM]);

/// Should be used to get access to one jit storage slots to be able to write
/// the jit-compiled program into it.
pub fn acquire_storage_slot(
    slot_index: usize,
) -> Result<MutexGuard<'static, ([u8; JIT_SLOT_SIZE], usize)>, String> {
    validate_slot_index(slot_index)?;

    let mut slot_states = JIT_SLOT_STATE.lock();
    let slot_occupied = slot_states[slot_index];

    if slot_occupied {
        Err(format!("Slot index {} already occupied", slot_index))?;
    }
    slot_states[slot_index] = true;
    Ok(JIT_PROGRAM_SLOTS[slot_index].lock())
}

pub fn free_storage_slot(slot_index: usize) -> Result<(), String> {
    validate_slot_index(slot_index)?;

    let mut slot_states = JIT_SLOT_STATE.lock();
    let slot_occupied = slot_states[slot_index];

    if !slot_occupied {
        Err(format!(
            "Slot index {} doesn't contain a jitted program",
            slot_index
        ))?;
    }

    slot_states[slot_index] = false;
    let mut guard = JIT_PROGRAM_SLOTS[slot_index].lock();
    guard.0.fill(0);
    guard.1 = 0;
    Ok(())
}

pub fn get_program_from_slot(
    slot_index: usize,
) -> Result<unsafe fn(*mut u8, usize, *mut u8, usize) -> u32, String> {
    validate_slot_index(slot_index)?;

    let mut slot_states = JIT_SLOT_STATE.lock();
    let slot_occupied = slot_states[slot_index];

    if !slot_occupied {
        Err(format!(
            "Slot index {} doesn't contain a jitted program",
            slot_index
        ))?;
    }

    let mut guard = JIT_PROGRAM_SLOTS[slot_index].lock();
    debug!("Loading previously jitted program from slot {}", slot_index);

    let offset = guard.1.clone();
    Ok(rbpf::JitMemory::get_prog_from_slice(
        guard.0.as_mut(),
        offset,
    ))
}

fn log_program_contents(program: &[u8], length: usize) {
    let mut prog_str: String = String::new();
    for (i, b) in program.iter().take(length).enumerate() {
        prog_str.push_str(&format!("{:02x}", *b));
        if i % 4 == 3 {
            prog_str.push_str("\n");
        }
    }
    debug!("program bytes:\n{}", prog_str);
}

fn validate_slot_index(slot_index: usize) -> Result<(), String> {
    if slot_index > JIT_STORAGE_SLOTS_NUM {
        Err(format!("Slot index {} out of bounds", slot_index))?;
    }

    Ok(())
}
