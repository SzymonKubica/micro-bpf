pub mod allocator {
    use alloc::alloc::*;
    use core::ffi::c_void;
    use riot_wrappers::riot_sys::free;
    use riot_wrappers::riot_sys::malloc;

    /// The global allocator type.
    #[derive(Default)]
    pub struct Allocator;

    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            malloc(layout.size() as u32) as *mut u8
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            free(ptr as *mut c_void);
        }
    }

    /// The static global allocator.
    /// It's purpose is to allow for using alloc rust crate allowing for
    /// using dynamically allocated data structures. The implementation of
    /// this allocator forwards the calls to the RIOT implementations of
    /// malloc and free.
    #[global_allocator]
    static GLOBAL_ALLOCATOR: Allocator = Allocator;
}
