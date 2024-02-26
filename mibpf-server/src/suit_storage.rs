use alloc::format;

extern "C" {
    /// Responsible for loading the bytecode from the SUIT ram storage.
    /// The application bytes are written into the buffer.
    fn initiate_suit_fetch(adderss: *const u8, signed_manifest_name: *const u8);
}

pub fn suit_fetch(ip: &str, manifest: &str) {
    let ip_addr = format!("{}\0", ip);
    let suit_manifest = format!("{}\0", manifest);

    unsafe {
        initiate_suit_fetch(ip_addr.as_ptr(), suit_manifest.as_ptr());
    };
}
