//! Unified millisecond clock shared by every relay component.
//!
//! Time is expressed as milliseconds since the Unix epoch in a `u64`.

use std::sync::Arc;

/// Source of time in milliseconds.
pub trait Clock {
    /// Returns the current time in milliseconds.
    fn now_ms(&self) -> u64;
}

impl<C: Clock + ?Sized> Clock for &C {
    fn now_ms(&self) -> u64 {
        (**self).now_ms()
    }
}

impl<C: Clock + ?Sized> Clock for Arc<C> {
    fn now_ms(&self) -> u64 {
        (**self).now_ms()
    }
}
