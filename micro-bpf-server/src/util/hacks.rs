
// This dummy implementaion is required because of a compliation bug which
// complains about an undefined reference to rust_eh_personality. This shouldn't
// be happening as the release profile of this application specifies panic="abort"
// which means that we shouldn't need an eh_personality function.
#[no_mangle]
extern "C" fn rust_eh_personality() {}
