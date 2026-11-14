//! Telemetry freshness gate controlling outbound CoT sending.


/// State of the telemetry staleness gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleGateState {
    /// No pose has been received yet.
    Uninitialized,
    /// Latest pose is fresh within the max age limit.
    Active,
    /// Latest pose is older than the max age limit.
    Paused,
}

/// Gate deciding whether output sending is allowed based on telemetry freshness.
#[derive(Debug, Clone)]
pub struct StaleGate {
    max_age_ms: u64,
    last_pose_ts_ms: Option<u64>,
}

impl StaleGate {
    /// Creates a new StaleGate with the given maximum age in milliseconds.
    pub fn new(max_age_ms: u64) -> Self {
        Self {
            max_age_ms,
            last_pose_ts_ms: None,
        }
    }

    /// Returns configured max age in milliseconds.
    pub fn max_age_ms(&self) -> u64 {
        self.max_age_ms
    }
}
