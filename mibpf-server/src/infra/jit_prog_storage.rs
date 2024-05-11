//! This module is responsible for providing access to global storage for
//! jitted programs. Its interface is designed to be similar to the [`super::suit_storage`].
//! The idea is that we have a number of jit storage slots of fixed size which
//! are statically allocated. Then the clients who want to store jitted programs
//! will obtain a mutable reference to the contents of one of the slots,
//! write the program there and then execute it by casting into a function pointer.

use super::suit_storage::{SUIT_STORAGE_SLOTS, SUIT_STORAGE_SLOT_SIZE};

pub const JIT_STORAGE_SLOTS: usize = SUIT_STORAGE_SLOTS;
pub const JIT_STORAGE_SLOT_SIZE: usize = SUIT_STORAGE_SLOT_SIZE;

static JIT_PROGRAM_SLOTS: Mutex<[[u8; SUIT_STORAGE_SLOT_SIZE]; SUIT_STORAGE_SLOTS]> =
    Mutex::new([[0; SUIT_STORAGE_SLOT_SIZE]; SUIT_STORAGE_SLOTS]);


