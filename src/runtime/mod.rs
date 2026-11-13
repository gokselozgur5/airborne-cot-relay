//! Process runtime primitives shared by long-running relay loops.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared flag that asks long-running loops to stop.
///
/// Clones share one flag, so a trigger on any clone is seen by all of them.
#[derive(Debug, Clone, Default)]
pub struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}
