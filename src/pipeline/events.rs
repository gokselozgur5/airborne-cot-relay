//! Edge-triggered pipeline events for operators, logs and recovery loops.

/// Notable pipeline state changes, each emitted once per transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineEvent {
    /// The sink failed `failures` times in a row.
    SinkDegraded { failures: u32 },
    /// The sink accepted a send after being degraded.
    SinkRecovered,
    /// The newest telemetry is `age_ms` old, past the stale limit.
    TelemetryStale { age_ms: u64 },
    /// A telemetry frame failed to decode.
    DecodeError { message: String },
}
