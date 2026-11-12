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

/// Point-in-time copy of all metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub packets_in: u64,
    pub dropped: u64,
    pub sent: u64,
    pub reconnects: u64,
    pub queue_depth: u64,
    pub last_send_age_ms: Option<u64>,
}

impl Metrics {
    pub fn snapshot(&self, now_ms: u64) -> Snapshot {
        Snapshot {
            packets_in: self.packets_in.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            sent: self.sent.load(Ordering::Relaxed),
            reconnects: self.reconnects.load(Ordering::Relaxed),
            queue_depth: self.queue_depth.load(Ordering::Relaxed),
            last_send_age_ms: self.last_send_age_ms(now_ms),
        }
    }
}

impl Snapshot {
    /// Renders stable `key value` lines, one per metric.
    pub fn render(&self) -> String {
        let age = self
            .last_send_age_ms
            .map_or_else(|| "never".to_string(), |a| a.to_string());
        format!(
            "packets_in {}\ndropped {}\nsent {}\nreconnects {}\nqueue_depth {}\nlast_send_age_ms {}\n",
            self.packets_in, self.dropped, self.sent, self.reconnects, self.queue_depth, age
        )
    }
}

