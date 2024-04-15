use core::ffi::c_int;

use alloc::format;
use log::debug;

pub const SUIT_STORAGE_SLOT_SIZE: usize = 2048;

// Currently, the interactions with SUIT storage are handled by functions written
// in native C, ideally they could be reimplemented using unsafe rust bindings from
// riot_sys.
extern "C" {
    fn initiate_suit_fetch(
        adderss: *const u8,
        network_interface: c_int,
        signed_manifest_name: *const u8,
    );
    /// Responsible for loading the bytecode from the SUIT ram storage.
    /// The application bytes are written into the buffer.
    fn load_bytes_from_suit_storage(buffer: *mut u8, location_id: *const u8) -> u32;
}

/// Responsible for fetching data from a remote CoAP fileserver using a SUIT
/// compliant mechanism. It required the IPv6 address of the machine hosting
/// the fileserver and the name of the manifest file associated with the data
/// to fetch.
///
/// # Arguments
///
/// * `ip` - The IPv6 address of the machine hosting the fileserver
/// * `manifest` - The name of the manifest file associated with the data to fetch
pub fn suit_fetch(ip: &str, network_interface: &str, manifest: &str) {
    let ip_addr = format!("{}\0", ip);
    let suit_manifest = format!("{}\0", manifest);
    let netif = network_interface.parse::<c_int>().unwrap();
    unsafe {
        initiate_suit_fetch(ip_addr.as_ptr(), netif, suit_manifest.as_ptr());
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
