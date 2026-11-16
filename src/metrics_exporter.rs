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

/// Individual metric sample with key-value labels and float value.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricSample {
    /// Dimension label pairs.
    pub labels: Vec<(String, String)>,
    /// Sample value.
    pub value: f64,
}

impl MetricSample {
    /// Creates a new metric sample.
    pub fn new(labels: Vec<(String, String)>, value: f64) -> Self {
        Self { labels, value }
    }
}

/// Metric family containing metadata and samples.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricFamily {
    /// Metric base identifier.
    pub name: String,
    /// Human-readable description.
    pub help: String,
    /// Metric type.
    pub metric_type: MetricType,
    /// Samples collected under this family.
    pub samples: Vec<MetricSample>,
}

impl MetricFamily {
    /// Creates a new metric family with empty samples.
    pub fn new(name: impl Into<String>, help: impl Into<String>, metric_type: MetricType) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            metric_type,
            samples: Vec::new(),
        }
    }
}

/// Configuration for the Prometheus telemetry exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExporterConfig {
    /// Metric namespace prefix (e.g. "airborne_cot").
    pub namespace: Option<String>,
    /// Metric subsystem prefix (e.g. "relay").
    pub subsystem: Option<String>,
    /// Global labels appended to all rendered metrics.
    pub common_labels: Vec<(String, String)>,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            namespace: Some("airborne_cot".to_string()),
            subsystem: Some("relay".to_string()),
            common_labels: Vec::new(),
        }
    }
}
