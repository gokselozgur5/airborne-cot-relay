//! Pipeline throughput and packet drop rate tracker.

/// Tracks pipeline packet throughput and drop rates.
#[derive(Debug, Clone)]
pub struct PipelineStatsTracker {
    time_window_sec: f64,
    total_received: u64,
    total_dropped: u64,
}

impl PipelineStatsTracker {
    /// Creates a new tracker with a given time window in seconds.
    pub fn new(time_window_sec: f64) -> Self {
        Self {
            time_window_sec: if time_window_sec > 0.0 { time_window_sec } else { 1.0 },
            total_received: 0,
            total_dropped: 0,
        }
    }
}
