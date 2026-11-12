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
