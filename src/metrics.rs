//! Lightweight relay counters and gauges built on std atomics.

use std::sync::atomic::{AtomicU64, Ordering};

/// Shared relay metrics. All updates saturate instead of wrapping.
#[derive(Debug, Default)]
pub struct Metrics {
    packets_in: AtomicU64,
    dropped: AtomicU64,
    sent: AtomicU64,
    reconnects: AtomicU64,
    queue_depth: AtomicU64,
    last_send: AtomicU64,
}
