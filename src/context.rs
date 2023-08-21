use std::sync::atomic::{AtomicU64, Ordering};

// This should be initialized from Daily.init().
// TODO(aleix): Is this even the way to do this? Do we need a global context?
pub static mut GLOBAL_CONTEXT: Option<DailyContext> = None;

pub struct DailyContext {
    request_id: AtomicU64,
}

impl DailyContext {
    pub fn new() -> Self {
        Self {
            request_id: AtomicU64::new(0),
        }
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
}
