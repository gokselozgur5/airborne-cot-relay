//! Pipeline throughput and packet drop rate tracker.

use std::collections::VecDeque;
use std::time::Instant;

/// Tracks pipeline packet throughput and drop rates.
#[derive(Debug, Clone)]
pub struct PipelineStatsTracker {
    time_window_sec: f64,
    total_received: u64,
    total_dropped: u64,
    history: VecDeque<(Instant, u64)>,
}

impl PipelineStatsTracker {
    /// Creates a new tracker with a given time window in seconds.
    pub fn new(time_window_sec: f64) -> Self {
        Self {
            time_window_sec: if time_window_sec > 0.0 {
                time_window_sec
            } else {
                1.0
            },
            total_received: 0,
            total_dropped: 0,
            history: VecDeque::new(),
        }
    }
}

impl PipelineStatsTracker {
    /// Records received packets count at a specific timestamp.
    pub fn record_received_at(&mut self, count: u64, at: Instant) {
        self.total_received = self.total_received.saturating_add(count);
        self.history.push_back((at, count));
    }

    /// Records received packets count.
    pub fn record_received(&mut self, count: u64) {
        self.record_received_at(count, Instant::now());
    }

    /// Records dropped packets count.
    pub fn record_dropped(&mut self, count: u64) {
        self.total_dropped = self.total_dropped.saturating_add(count);
    }

    /// Resets all counters to zero.
    pub fn reset(&mut self) {
        self.total_received = 0;
        self.total_dropped = 0;
        self.history.clear();
    }

    /// Returns the configured time window in seconds.
    pub fn time_window_sec(&self) -> f64 {
        self.time_window_sec
    }
}

impl PipelineStatsTracker {
    /// Returns total received packet count.
    pub fn total_received(&self) -> u64 {
        self.total_received
    }

    /// Returns total dropped packet count.
    pub fn total_dropped(&self) -> u64 {
        self.total_dropped
    }

    /// Computes packet drop rate between 0.0 and 1.0.
    pub fn drop_rate(&self) -> f64 {
        if self.total_received == 0 {
            0.0
        } else {
            self.total_dropped as f64 / self.total_received as f64
        }
    }
}

impl PipelineStatsTracker {
    /// Computes packets per second throughput over the configured time window at a specific timestamp.
    pub fn throughput_pps_at(&mut self, now: Instant) -> f64 {
        if self.time_window_sec <= 0.0 {
            return 0.0;
        }
        let window = std::time::Duration::from_secs_f64(self.time_window_sec);
        while let Some(&(ts, _)) = self.history.front() {
            if now.saturating_duration_since(ts) > window {
                self.history.pop_front();
            } else {
                break;
            }
        }
        let window_received: u64 = self.history.iter().map(|(_, c)| c).sum();
        window_received as f64 / self.time_window_sec
    }

    /// Computes packets per second throughput over the configured time window.
    pub fn throughput_pps(&mut self) -> f64 {
        self.throughput_pps_at(Instant::now())
    }
}
