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

impl std::fmt::Display for PipelineEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SinkDegraded { failures } => {
                write!(f, "sink degraded after {} consecutive failures", failures)
            }
            Self::SinkRecovered => write!(f, "sink recovered"),
            Self::TelemetryStale { age_ms } => write!(f, "telemetry stale: age {} ms", age_ms),
            Self::DecodeError { message } => write!(f, "decode error: {}", message),
        }
    }
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

    /// Configured failure count required to mark sink degraded.
    pub fn degrade_after(&self) -> u32 {
        self.degrade_after
    }

    /// Number of consecutive send failures observed so far.
    pub fn consecutive_failures(&self) -> u32 {
        self.failures
    }

    /// True if the sink is currently marked degraded.
    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    /// True if the latest telemetry observation was stale.
    pub fn is_telemetry_stale(&self) -> bool {
        self.telemetry_stale
    }

    /// Returns a slice of events currently queued.
    pub fn pending(&self) -> &[PipelineEvent] {
        &self.pending
    }

    /// Number of events currently queued.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// True if there are no pending events.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Clears all pending events and resets failure counters and state.
    pub fn reset(&mut self) {
        self.failures = 0;
        self.degraded = false;
        self.telemetry_stale = false;
        self.pending.clear();
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
