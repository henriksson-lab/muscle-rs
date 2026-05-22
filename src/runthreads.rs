// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Run `body` in parallel on each worker thread, using the globally requested thread count.
#[track_caller]
pub fn run_threads<F>(body: F)
where
    F: Fn(uint) + Sync,
{
    let thread_count = get_requested_thread_count();
    std::thread::scope(|scope| {
        for thread_index in 0..thread_count {
            let body = &body;
            scope.spawn(move || body(thread_index));
        }
    });
}
