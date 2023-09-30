use std::sync::atomic::{AtomicBool, Ordering};

pub struct Cancelable {}

impl Cancelable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn cancel(&self) {
        SIGNAL.store(true, Ordering::Relaxed);
    }

    pub fn is_canceled(&self) -> bool {
        SIGNAL.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        SIGNAL.store(false, Ordering::Relaxed);
    }
}

pub fn create_signal() -> Cancelable {
    ctrlc::set_handler(|| SIGNAL.store(true, Ordering::Relaxed))
        .expect("Error setting Ctrl-C handler");

    Cancelable::new()
}

static SIGNAL: AtomicBool = AtomicBool::new(false);
