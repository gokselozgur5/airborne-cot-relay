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

fn bump(counter: &AtomicU64, n: u64) {
    let mut cur = counter.load(Ordering::Relaxed);
    while let Err(seen) = counter.compare_exchange_weak(
        cur,
        cur.saturating_add(n),
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        cur = seen;
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_packets_in(&self, n: u64) {
        bump(&self.packets_in, n);
    }

    pub fn add_dropped(&self, n: u64) {
        bump(&self.dropped, n);
    }
}

