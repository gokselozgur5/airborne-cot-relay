//! Prometheus telemetry metrics exporter for Cursor-on-Target relay.
//!
//! Formats relay counters, gauges, and queue telemetry according to the
//! Prometheus 0.0.4 text exposition specification.

/// Standard Prometheus text format media type.
pub const PROMETHEUS_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

/// Prometheus metric classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically increasing counter.
    Counter,
    /// Value that can arbitrarily go up and down.
    Gauge,
}

impl MetricType {
    /// Returns exposition format type string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
        }
    }
}
