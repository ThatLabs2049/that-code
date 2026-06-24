use std::sync::atomic::{AtomicBool, Ordering};

pub struct RagIndexState {
    cancel: AtomicBool,
}

impl RagIndexState {
    pub fn new() -> Self {
        Self {
            cancel: AtomicBool::new(false),
        }
    }

    pub fn begin(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn cancel_flag(&self) -> &AtomicBool {
        &self.cancel
    }
}

impl Default for RagIndexState {
    fn default() -> Self {
        Self::new()
    }
}
