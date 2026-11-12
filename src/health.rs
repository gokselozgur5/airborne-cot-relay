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
