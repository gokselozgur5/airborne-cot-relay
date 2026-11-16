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

impl ExporterConfig {
    /// Creates an empty configuration with no prefixes or default labels.
    pub fn empty() -> Self {
        Self {
            namespace: None,
            subsystem: None,
            common_labels: Vec::new(),
        }
    }

    /// Creates a configuration with an explicit namespace prefix.
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            subsystem: None,
            common_labels: Vec::new(),
        }
    }
}

/// Prometheus telemetry metrics exporter engine.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrometheusExporter {
    config: ExporterConfig,
    custom_families: Vec<MetricFamily>,
}

impl PrometheusExporter {
    /// Creates a new exporter with given configuration.
    pub fn new(config: ExporterConfig) -> Self {
        Self {
            config,
            custom_families: Vec::new(),
        }
    }

    /// Creates an exporter with specified namespace and no subsystem.
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self::new(ExporterConfig::with_namespace(namespace))
    }

    /// Appends a static common label applied to all exported metrics.
    pub fn add_common_label(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config.common_labels.push((key.into(), value.into()));
    }

    fn format_metric_name(&self, base_name: &str) -> String {
        let mut parts = Vec::with_capacity(3);
        if let Some(ns) = &self.config.namespace {
            parts.push(ns.as_str());
        }
        if let Some(sub) = &self.config.subsystem {
            parts.push(sub.as_str());
        }
        parts.push(base_name);
        parts.join("_")
    }

    fn escape_help(text: &str) -> String {
        text.replace('\\', "\\\\").replace('\n', "\\n")
    }

    fn escape_label_value(val: &str) -> String {
        val.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    fn format_labels(
        common: &[(String, String)],
        sample: &[(String, String)],
    ) -> String {
        if common.is_empty() && sample.is_empty() {
            return String::new();
        }
        let mut parts = Vec::with_capacity(common.len() + sample.len());
        for (k, v) in common.iter().chain(sample.iter()) {
            parts.push(format!("{}=\"{}\"", k, Self::escape_label_value(v)));
        }
        format!("{{{}}}", parts.join(","))
    }

    fn render_family(
        &self,
        family: &MetricFamily,
        full_name: &str,
        out: &mut String,
    ) {
        out.push_str("# HELP ");
        out.push_str(full_name);
        out.push(' ');
        out.push_str(&Self::escape_help(&family.help));
        out.push('\n');

        out.push_str("# TYPE ");
        out.push_str(full_name);
        out.push(' ');
        out.push_str(family.metric_type.as_str());
        out.push('\n');

        for sample in &family.samples {
            let label_str = Self::format_labels(&self.config.common_labels, &sample.labels);
            out.push_str(&format!("{}{} {}\n", full_name, label_str, sample.value));
        }
    }
}
