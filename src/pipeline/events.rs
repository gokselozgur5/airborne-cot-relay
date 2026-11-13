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

/// Turns per-call observations into edge-triggered `PipelineEvent`s.
#[derive(Debug)]
pub struct EventTracker {
    degrade_after: u32,
    failures: u32,
    degraded: bool,
    telemetry_stale: bool,
    pending: Vec<PipelineEvent>,
}

impl EventTracker {
    /// Marks the sink degraded after `degrade_after` consecutive failures (at least 1).
    pub fn new(degrade_after: u32) -> Self {
        Self {
            degrade_after: degrade_after.max(1),
            failures: 0,
            degraded: false,
            telemetry_stale: false,
            pending: Vec::new(),
        }
    }

    /// Returns and clears the pending events in emission order.
    pub fn drain(&mut self) -> Vec<PipelineEvent> {
        std::mem::take(&mut self.pending)
    }
}

impl EventTracker {
    /// Records one send result, emitting `SinkDegraded` or `SinkRecovered` on a transition.
    pub fn record_send(&mut self, ok: bool) {
        if ok {
            self.failures = 0;
            if std::mem::replace(&mut self.degraded, false) {
                self.pending.push(PipelineEvent::SinkRecovered);
            }
            return;
        }
        self.failures = self.failures.saturating_add(1);
        if !self.degraded && self.failures >= self.degrade_after {
            self.degraded = true;
            let failures = self.failures;
            self.pending.push(PipelineEvent::SinkDegraded { failures });
        }
    }
}

use crate::config::RelayError;

impl EventTracker {
    /// Records the telemetry age, emitting `TelemetryStale` once per stale period.
    pub fn record_telemetry_age(&mut self, age_ms: u64, stale_after_ms: u64) {
        let stale = age_ms > stale_after_ms;
        if stale && !self.telemetry_stale {
            self.pending.push(PipelineEvent::TelemetryStale { age_ms });
        }
        self.telemetry_stale = stale;
    }

    /// Records one decode failure as a `DecodeError` event.
    pub fn record_decode_error(&mut self, error: &RelayError) {
        let message = error.to_string();
        self.pending.push(PipelineEvent::DecodeError { message });
    }
}
