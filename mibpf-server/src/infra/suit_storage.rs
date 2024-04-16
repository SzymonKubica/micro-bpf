use core::ffi::c_int;

use alloc::format;
use log::debug;
use riot_wrappers::thread;

pub const SUIT_STORAGE_SLOT_SIZE: usize = 2048;

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
/// * `manifest` - The name of the manifest file associated with the data to fetch
pub fn suit_fetch(ip: &str, network_interface: &str, manifest: &str) -> Result<(), ()> {
    let ip_addr = format!("{}\0", ip);
    let suit_manifest = format!("{}\0", manifest);
    let netif = network_interface.parse::<c_int>().unwrap();

    let pid = thread::get_pid().into();
    debug!("Thread {} initiating SUIT fetch...", pid);

    unsafe {
        initiate_suit_fetch(ip_addr.as_ptr(), netif, suit_manifest.as_ptr(), pid);

        let mut msg: riot_sys::msg_t = Default::default();
        let _ = riot_sys::msg_receive(&mut msg);

        match msg.content.value {
            0 => Ok(()),
            _ => Err(()),
        }
    }
}

/// Allows for erasing the SUIT storage containing a given program if e.g. it's
/// helper function verification has failed and it cannot be executed
pub fn suit_erase(slot: usize) {
    debug!("Erasing SUIT storage slot {}.", slot);
    let location = format!(".ram.{0}\0", slot);
    unsafe {
        let location_ptr = location.as_ptr();
        handle_suit_storage_erase(location_ptr);
    };
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
