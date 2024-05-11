//! This module is responsible for providing access to global storage for
//! jitted programs. Its interface is designed to be similar to the [`super::suit_storage`].
//! The idea is that we have a number of jit storage slots of fixed size which
//! are statically allocated. Then the clients who want to store jitted programs
//! will obtain a mutable reference to the contents of one of the slots,
//! write the program there and then execute it by casting into a function pointer.

use riot_wrappers::mutex::{Mutex, MutexGuard};

use super::suit_storage::{SUIT_STORAGE_SLOTS, SUIT_STORAGE_SLOT_SIZE};

pub const JIT_STORAGE_SLOTS_NUM: usize = SUIT_STORAGE_SLOTS;
pub const JIT_SLOT_SIZE: usize = SUIT_STORAGE_SLOT_SIZE;

static JIT_PROGRAM_SLOTS: Mutex<[[u8; JIT_SLOT_SIZE]; JIT_STORAGE_SLOTS_NUM]> =
    Mutex::new([[0; JIT_SLOT_SIZE]; JIT_STORAGE_SLOTS_NUM]);

// We globally maintain whether a slot is in use or not
static JIT_SLOT_STATE: Mutex<[bool; JIT_STORAGE_SLOTS_NUM]> = Mutex::new([false; JIT_STORAGE_SLOTS_NUM]);

pub fn acquire_storage(
    slot_index: usize,
) -> Option<MutexGuard<'static, [[u8; JIT_SLOT_SIZE]; JIT_STORAGE_SLOTS_NUM]>> {

    let mut slot_states = JIT_SLOT_STATE.lock();
    if slot_index > JIT_STORAGE_SLOTS_NUM {
        return None;
    }

    let slot_occupied = slot_states[slot_index];

    if slot_occupied {
        return None;
    }

    slot_states[slot_index] = true;
    let guard = JIT_PROGRAM_SLOTS.lock();
    return Some(guard);
}
