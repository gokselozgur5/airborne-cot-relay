//! Relay health classification (std only).
use std::time::Duration;

/// Overall relay health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Degraded,
    Offline,
}

/// Limits used to classify health.
#[derive(Debug, Clone, Copy)]
pub struct HealthThresholds {
    pub degraded_telemetry_age: Duration,
    pub offline_telemetry_age: Duration,
    pub degraded_send_age: Duration,
    pub offline_failures: u32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            degraded_telemetry_age: Duration::from_secs(2),
            offline_telemetry_age: Duration::from_secs(10),
            degraded_send_age: Duration::from_secs(5),
            offline_failures: 5,
        }
    }
}

/// Injectable monotonic time source.
pub trait TimeSource {
    fn now(&self) -> Duration;
}

/// Tracks telemetry and send activity.
pub struct HealthMonitor<T: TimeSource> {
    clock: T,
    thresholds: HealthThresholds,
    last_telemetry: Option<Duration>,
    last_send: Option<Duration>,
    failures: u32,
}

impl<T: TimeSource> HealthMonitor<T> {
    pub fn new(clock: T, thresholds: HealthThresholds) -> Self {
        Self {
            clock,
            thresholds,
            last_telemetry: None,
            last_send: None,
            failures: 0,
        }
    }

    pub fn record_telemetry(&mut self) {
        self.last_telemetry = Some(self.clock.now());
    }
}

impl<T: TimeSource> HealthMonitor<T> {
    pub fn record_send(&mut self, ok: bool) {
        if ok {
            self.last_send = Some(self.clock.now());
            self.failures = 0;
        } else {
            self.failures += 1;
        }
    }
}
