//! A crate with global state, shared by several extension modules.

use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Increments the counter and returns its new value.
pub fn increment() -> u32 {
    COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}
