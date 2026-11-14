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
