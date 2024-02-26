use alloc::format;

extern "C" {
    fn initiate_suit_fetch(adderss: *const u8, signed_manifest_name: *const u8);
}

/// Responsible for fetching data from a remote CoAP fileserver using a SUIT
/// compliant mechanism. It required the IPv6 address of the machine hosting
/// the fileserver and the name of the manifest file associated with the data
/// to fetch.
pub fn suit_fetch(ip: &str, manifest: &str) {
    let ip_addr = format!("{}\0", ip);
    let suit_manifest = format!("{}\0", manifest);

    unsafe {
        initiate_suit_fetch(ip_addr.as_ptr(), suit_manifest.as_ptr());
    };
}
