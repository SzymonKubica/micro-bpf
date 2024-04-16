use core::ffi::c_int;

use alloc::{
    format,
    string::{String, ToString},
};
use log::debug;
use riot_wrappers::{mutex::Mutex, thread};

use crate::infra::local_storage;

/// Size of each slot in the SUIT storage where the programs get loaded.
/// It is important that this value is consistent with what is specified in
/// the Makefile for this project using this line:
/// CFLAGS += -DCONFIG_SUIT_STORAGE_RAM_REGIONS=2 -DCONFIG_SUIT_STORAGE_RAM_SIZE=2048
pub const SUIT_STORAGE_SLOT_SIZE: usize = 2048;
pub const SUIT_STORAGE_SLOTS: usize = 2;

/// Stores status of all SUIT storage slots available for loading programs
pub static SUIT_STORAGE_STATE: Mutex<[SuitStorageSlotStatus; SUIT_STORAGE_SLOTS]> =
    Mutex::new([SuitStorageSlotStatus::Free; SUIT_STORAGE_SLOTS]);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SuitStorageSlotStatus {
    Free,
    /// A slot contains some bytecode but noone is currently executing it.
    Occupied,
    /// This is used when there is currently a long running that uses the local
    /// storage associated with the program loaded into that SUIT slot.
    Running,
}

// Currently, the interactions with SUIT storage are handled by functions written
// in native C, ideally they could be reimplemented using unsafe rust bindings from
// riot_sys.
extern "C" {
    fn initiate_suit_fetch(
        adderss: *const u8,
        network_interface: c_int,
        signed_manifest_name: *const u8,
        requestor_pid: riot_sys::kernel_pid_t,
    );
    /// Responsible for loading the bytecode from the SUIT ram storage.
    /// The application bytes are written into the buffer.
    fn load_bytes_from_suit_storage(buffer: *mut u8, location_id: *const u8) -> u32;
    /// Responsible for erasing a given SUIT storage slot
    fn handle_suit_storage_erase(location_id: *const u8);
}

/// Responsible for fetching data from a remote CoAP fileserver using a SUIT
/// compliant mechanism. It required the IPv6 address of the machine hosting
/// the fileserver and the name of the manifest file associated with the data
/// to fetch.
///
/// It uses message IPC with the SUIT worker thread to ensure that this function
/// doesn't return until the SUIT update finishes.
///
/// # Arguments
///
/// * `ip` - The IPv6 address of the machine hosting the fileserver
/// * `network_interface` - the network interface name used by the target microcontroller
///   most of the time it is either 5 or 6 and is needed to correctly format the
///   url that we'll send the request to to start pulling the data.
/// * `manifest` - The name of the manifest file associated with the data to fetch
/// * `slot` - The index of the SUIT storage slot into which the program is to
///   be loaded. It is important that this slot be consistent with the slot id
///   specified in the manifest file. Because of the SUIT update workflow, there
///   is no way of enforcing that the two slots match (one could sign a manifest)
///   specifying that slot is e.g. 0 and then say 1 in the request. In such case
///   the program would be loaded into slot 0 but we would mark slot 1 as occupied.
///   It is a responsiblility of the person using the system to ensure that those
///   two slots match.
pub fn suit_fetch(
    ip: &str,
    network_interface: &str,
    manifest: &str,
    slot: usize,
    erase: bool,
) -> Result<(), String> {
    let ip_addr = format!("{}\0", ip);
    let suit_manifest = format!("{}\0", manifest);
    let netif = network_interface.parse::<c_int>().unwrap();

    let mut slots = SUIT_STORAGE_STATE.lock();
    if slots[slot] != SuitStorageSlotStatus::Free && !erase {
        Err("Tried to load a program into an occupied slot".to_string())?;
    }

    if slots[slot] == SuitStorageSlotStatus::Running {
        Err("Tried to overwrite a slot that belongs to a currently running program".to_string())?;
    }

    let pid = thread::get_pid().into();
    debug!("Thread {} initiating SUIT fetch...", pid);

    debug!("Deregistering the local storage associated with the exising slot");
    local_storage::deregister_suit_slot(slot);

    unsafe {
        initiate_suit_fetch(ip_addr.as_ptr(), netif, suit_manifest.as_ptr(), pid);

        let mut msg: riot_sys::msg_t = Default::default();
        let _ = riot_sys::msg_receive(&mut msg);

        const SUIT_FETCH_SUCCESS: u32 = 0;
        if msg.content.value == SUIT_FETCH_SUCCESS {
            slots[slot] = SuitStorageSlotStatus::Occupied;
            debug!("SUIT fetch successful, marked slot {} as occupied.", slot);
            Ok(())
        } else {
            slots[slot] = SuitStorageSlotStatus::Free;
            Err("SUIT fetch failed".to_string())
        }
    }
}

pub fn suit_mark_slot_running(slot: usize) {
    let mut slots = SUIT_STORAGE_STATE.lock();
    slots[slot] = SuitStorageSlotStatus::Running;
}

pub fn suit_mark_slot_occupied(slot: usize) {
    let mut slots = SUIT_STORAGE_STATE.lock();
    slots[slot] = SuitStorageSlotStatus::Occupied;
}

/// Allows for erasing the SUIT storage containing a given program if e.g. it's
/// helper function verification has failed and it cannot be executed
pub fn suit_erase(slot: usize) -> Result<(), String> {
    let location = format!(".ram.{0}\0", slot);
    let mut slots = SUIT_STORAGE_STATE.lock();
    if slots[slot] != SuitStorageSlotStatus::Occupied {
        Err("Requested to erase an empty SUIT slot".to_string())?;
    }

    debug!("Erasing SUIT storage slot {}.", slot);
    unsafe {
        let location_ptr = location.as_ptr();
        handle_suit_storage_erase(location_ptr);
    };
    slots[slot] = SuitStorageSlotStatus::Free;
    Ok(())
}

/// Reads from the given suit storage into the provided program buffer
///
/// # Arguments
///
/// * `program_buffer` - A mutable slice of bytes to write the program into
/// * `slot` - The index of the SUIT storage slot from which to load the bytes.
pub fn load_program<'a>(program_buffer: &'a mut [u8], slot: usize) -> &'a mut [u8] {
    let location = format!(".ram.{0}\0", slot);
    let len;
    unsafe {
        let buffer_ptr = program_buffer.as_mut_ptr();
        let location_ptr = location.as_ptr();
        len = load_bytes_from_suit_storage(buffer_ptr, location_ptr);
    };

    debug!("{}[B] program loaded from SUIT storage slot {}.", len, slot);

    &mut program_buffer[..(len as usize)]
}
