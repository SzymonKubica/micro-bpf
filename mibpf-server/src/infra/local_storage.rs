//! This module contains an implementation of local storage for eBPF programs
//! running in the system. The local storage is represented as a key-value
//! storage that belongs to a given program.
//!
//! What is meant by 'belongs to' is that before being executed, each program
//! gets written into a SUIT storage slot using the infrastructure provided by
//! RIOT. At the same time, a static BTreeMap for that program is initialised.
//! This means that the local storage of a given program will persist as long as
//! its bytecode remains loaded in the same SUIT storage slot.
//!
//! It is important to note that before we execute an instance of the VM, we
//! actually load the program from the SUIT storage into a slice on the current
//! thread's stack. This means that there is nothing that enforces that the local
//! storage needs to be coupled with the SUIT slot. This design was chosen because
//! of its simplicity. We could for instance provide thread-local storage but that
//! could lead to issues as different worker threads can be chosen to execute
//! a program loaded into some SUIT storage slot.
//!
//! To make the explanation more explicit, after the program is loaded into a
//! given SUIT storage slot, it can be executed multiple times and possibly even
//! concurrently. This is because we always copy programs bytecode before executing it.
//!

use alloc::{collections::BTreeMap, vec::Vec};
use log::{debug, error};
use riot_wrappers::{mutex::Mutex, thread};

use super::suit_storage::{self, SUIT_STORAGE_SLOTS};

const EMPTY_MAP: BTreeMap<usize, i32> = BTreeMap::new();
/// Each SUIT storage slot has its associated BTreeMap storage.
static LOCAL_STORAGE: Mutex<[BTreeMap<usize, i32>; SUIT_STORAGE_SLOTS]> =
    Mutex::new([EMPTY_MAP; SUIT_STORAGE_SLOTS]);

/// In order to determine which thread is currently executing a program from which
/// SUIT storage slot, we need a global map which stores that information

static THREAD_TO_STORAGE_SLOT: Mutex<BTreeMap<riot_sys::kernel_pid_t, usize>> =
    Mutex::new(BTreeMap::new());

pub fn local_storage_store(key: usize, value: i32) -> i32 {
    // We need the pid of the thread to be able to determine which slot belongs
    // to the current thread.
    let slot_number = lookup_slot_number();


    if let Some(slot_number) = slot_number {
        let mut storage = LOCAL_STORAGE.lock();
        storage[slot_number].insert(key, value);
        return value;
    } else {
        error!("No slot number found corresponding to the current thread");
        return 0;
    }
}

pub fn local_storage_fetch(key: usize) -> Option<i32> {
    let slot_number = lookup_slot_number();

    if let None = slot_number {
        error!("No slot number found corresponding the current thread");
        return None;
    }

    let storage = LOCAL_STORAGE.lock();
    return storage[slot_number.unwrap()].get(&key).copied();
}

fn lookup_slot_number() -> Option<usize> {
    let pid = thread::get_pid().into();
    let map = THREAD_TO_STORAGE_SLOT.lock();
    return map.get(&pid).copied();
}

/// Registers a SUIT slot for the current thread. This is necessary because
/// when a given thread is executing a program loaded into slot `n`, then we
/// don't wanto to allow other threads for loading programs in there.
pub fn register_suit_slot(slot: usize) {
    let pid = thread::get_pid().into();
    debug!("Registering SUIT slot {} for thread {}", slot, pid);
    let mut map = THREAD_TO_STORAGE_SLOT.lock();
    map.insert(pid, slot);
}

pub fn deregister_suit_slot(slot: usize) {
    let mut map = THREAD_TO_STORAGE_SLOT.lock();
    let pids_to_remove: Vec<riot_sys::kernel_pid_t> = map
        .iter()
        .filter(|(_, s)| **s == slot)
        .map(|(p, _)| *p)
        .collect();
    debug!(
        "Deregistering SUIT slot {} from threads: {:?}",
        slot, pids_to_remove
    );
    for pid in pids_to_remove {
        map.remove(&pid);
    }

    let mut storage = LOCAL_STORAGE.lock();
    storage[slot] = BTreeMap::new();
}
