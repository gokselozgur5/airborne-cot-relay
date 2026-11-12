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

impl Metrics {
    pub fn add_reconnect(&self) {
        bump(&self.reconnects, 1);
    }

    /// Counts one sent packet and records when it happened.
    pub fn record_sent(&self, now_ms: u64) {
        bump(&self.sent, 1);
        // Stored offset by one so that zero means "never sent".
        self.last_send
            .store(now_ms.saturating_add(1), Ordering::Relaxed);
    }

    pub fn set_queue_depth(&self, depth: u64) {
        self.queue_depth.store(depth, Ordering::Relaxed);
    }

    /// Milliseconds since the last send, or `None` if nothing was sent.
    pub fn last_send_age_ms(&self, now_ms: u64) -> Option<u64> {
        match self.last_send.load(Ordering::Relaxed) {
            0 => None,
            stored => Some(now_ms.saturating_sub(stored - 1)),
        }
    }
}

