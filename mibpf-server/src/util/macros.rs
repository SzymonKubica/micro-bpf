/// Responsible for spawning a new thread using the given thread scope. The reason
/// it can't be a plain function is that the threadscope is a mutable reference
/// that is valid only inside of the scope closure, and so it can't be passed
/// into a function. This macro allows for spawning multiple threads inside of
/// a single scope without having to paste a lot of the boilerplate.
#[macro_export]
macro_rules! spawn_thread {
    ($threadscope:expr, $name: expr, $stacklock:expr, $mainclosure:expr, $priority:expr ) => {{
        use crate::util::logger::log_thread_spawned;
        use alloc::format;
        let Ok(thread) = $threadscope.spawn(
            $stacklock.as_mut(),
            &mut $mainclosure,
            riot_wrappers::cstr::cstr!($name),
            ($priority) as _,
            (riot_sys::THREAD_CREATE_STACKTEST) as _,
        ) else {
            let msg = format!("Failed to spawn {}", $name);
            error!("{}", msg);
            panic!();
        };
        log_thread_spawned(&thread, $name);
        thread
    }};
}
