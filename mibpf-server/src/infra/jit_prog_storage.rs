//! This module is responsible for providing access to global storage for
//! jitted programs. Its interface is designed to be similar to the [`super::suit_storage`].
//! The idea is that we have a number of jit storage slots of fixed size which
//! are statically allocated. Then the clients who want to store jitted programs
//! will obtain a mutable reference to the contents of one of the slots,
//! write the program there and then execute it by casting into a function pointer.

use alloc::{format, string::String};
use riot_wrappers::mutex::{Mutex, MutexGuard};

use super::suit_storage::{SUIT_STORAGE_SLOTS, SUIT_STORAGE_SLOT_SIZE};

pub const JIT_STORAGE_SLOTS_NUM: usize = SUIT_STORAGE_SLOTS;
pub const JIT_SLOT_SIZE: usize = SUIT_STORAGE_SLOT_SIZE;

static JIT_PROGRAM_SLOTS: [Mutex<[u8; JIT_SLOT_SIZE]>; JIT_STORAGE_SLOTS_NUM] =
    [EMPTY_SLOT; JIT_STORAGE_SLOTS_NUM];

const EMPTY_SLOT: Mutex<[u8; JIT_SLOT_SIZE]> = Mutex::new([0; JIT_SLOT_SIZE]);

// We globally maintain whether a slot is in use or not
static JIT_SLOT_STATE: Mutex<[bool; JIT_STORAGE_SLOTS_NUM]> =
    Mutex::new([false; JIT_STORAGE_SLOTS_NUM]);

pub fn acquire_storage_slot(
    slot_index: usize,
) -> Result<MutexGuard<'static, [u8; JIT_SLOT_SIZE]>, String> {
    validate_slot_index(slot_index);

    let mut slot_states = JIT_SLOT_STATE.lock();
    let slot_occupied = slot_states[slot_index];

    if slot_occupied {
        Err(format!("Slot index {} already occupied", slot_index))?;
    }

    slot_states[slot_index] = true;
    Ok(JIT_PROGRAM_SLOTS[slot_index].lock())
}

pub fn get_program_from_slot(
    slot_index: usize,
) -> Result<unsafe fn(*mut u8, usize, *mut u8, usize) -> u32, String> {
    validate_slot_index(slot_index);

    let mut slot_states = JIT_SLOT_STATE.lock();
    let slot_occupied = slot_states[slot_index];

    if !slot_occupied {
        Err(format!("Slot index {} doesn't contain a jitted program", slot_index))?;
    }

    Ok(rbpf::JitMemory::get_prog_from_slice(
        JIT_PROGRAM_SLOTS[slot_index].lock().as_mut(),
    ))
}

fn validate_slot_index(slot_index: usize) -> Result<(), String> {
    if slot_index > JIT_STORAGE_SLOTS_NUM {
        Err(format!("Slot index {} out of bounds", slot_index))?;
    }

    Ok(())
}
