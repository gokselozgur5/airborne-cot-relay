//! Process runtime primitives shared by long-running relay loops.

pub mod signal;
pub mod sleep;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared flag that asks long-running loops to stop.
///
/// Clones share one flag, so a trigger on any clone is seen by all of them.
#[derive(Debug, Clone, Default)]
pub struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}

impl ShutdownSignal {
    /// Creates a signal that has not been triggered.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests shutdown; every clone observes the request.
    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Returns true once any clone has called `trigger`.
    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Returns the underlying flag for integrations that set it directly.
    pub fn shared_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}
