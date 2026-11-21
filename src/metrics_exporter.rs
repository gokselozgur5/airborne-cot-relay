//! Prometheus telemetry metrics exporter for Cursor-on-Target relay.
//!
//! Formats relay counters, gauges, and queue telemetry according to the
//! Prometheus 0.0.4 text exposition specification.

use crate::metrics::{Metrics, Snapshot};

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

    fn format_labels(common: &[(String, String)], sample: &[(String, String)]) -> String {
        if common.is_empty() && sample.is_empty() {
            return String::new();
        }
        let mut parts = Vec::with_capacity(common.len() + sample.len());
        for (k, v) in common.iter().chain(sample.iter()) {
            parts.push(format!("{}=\"{}\"", k, Self::escape_label_value(v)));
        }
        format!("{{{}}}", parts.join(","))
    }

    fn render_family(&self, family: &MetricFamily, full_name: &str, out: &mut String) {
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

    fn valid_name(s: &str, colon: bool) -> bool {
        let mut c = s.chars();
        matches!(c.next(), Some(x) if x.is_ascii_alphabetic() || x == '_' || (colon && x == ':'))
            && c.all(|x| x.is_ascii_alphanumeric() || x == '_' || (colon && x == ':'))
    }

    /// Appends a pre-built metric family, merging samples if the family already exists.
    pub fn add_metric_family(&mut self, family: MetricFamily) {
        let labels_valid = family
            .samples
            .iter()
            .all(|s| s.labels.iter().all(|(k, _)| Self::valid_name(k, false)));
        if !Self::valid_name(&family.name, true) || !labels_valid {
            return;
        }
        if let Some(existing) = self
            .custom_families
            .iter_mut()
            .find(|f| f.name == family.name)
        {
            existing.samples.extend(family.samples);
        } else {
            self.custom_families.push(family);
        }
    }

    /// Registers a custom counter metric with given name, description, and value.
    pub fn add_counter(&mut self, name: &str, help: &str, val: f64, labels: &[(&str, &str)]) {
        let mut fam = MetricFamily::new(name, help, MetricType::Counter);
        let lbls = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        fam.samples.push(MetricSample::new(lbls, val));
        self.add_metric_family(fam);
    }

    /// Registers a custom gauge metric with given name, description, and value.
    pub fn add_gauge(&mut self, name: &str, help: &str, val: f64, labels: &[(&str, &str)]) {
        let mut fam = MetricFamily::new(name, help, MetricType::Gauge);
        let lbls = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        fam.samples.push(MetricSample::new(lbls, val));
        self.add_metric_family(fam);
    }

    fn render_single(
        &self,
        base_name: &str,
        help: &str,
        mtype: MetricType,
        val: f64,
        out: &mut String,
    ) {
        let full = self.format_metric_name(base_name);
        let mut fam = MetricFamily::new(base_name, help, mtype);
        fam.samples.push(MetricSample::new(Vec::new(), val));
        self.render_family(&fam, &full, out);
    }

    /// Exports a metrics Snapshot into Prometheus exposition text.
    pub fn export_snapshot(&self, snapshot: &Snapshot) -> String {
        let mut out = String::with_capacity(1024);
        self.render_single(
            "packets_in_total",
            "Total incoming packets.",
            MetricType::Counter,
            snapshot.packets_in as f64,
            &mut out,
        );
        self.render_single(
            "packets_dropped_total",
            "Total packets dropped.",
            MetricType::Counter,
            snapshot.dropped as f64,
            &mut out,
        );
        self.render_single(
            "packets_sent_total",
            "Total packets sent.",
            MetricType::Counter,
            snapshot.sent as f64,
            &mut out,
        );
        self.render_single(
            "reconnects_total",
            "Total reconnect attempts.",
            MetricType::Counter,
            snapshot.reconnects as f64,
            &mut out,
        );
        self.render_single(
            "queue_depth",
            "Current queue depth.",
            MetricType::Gauge,
            snapshot.queue_depth as f64,
            &mut out,
        );
        if let Some(age_ms) = snapshot.last_send_age_ms {
            self.render_single(
                "last_send_age_seconds",
                "Seconds since last send.",
                MetricType::Gauge,
                (age_ms as f64) / 1000.0,
                &mut out,
            );
        }
        for fam in &self.custom_families {
            let full = self.format_metric_name(&fam.name);
            self.render_family(fam, &full, &mut out);
        }
        out
    }

    /// Takes a snapshot of live Metrics and exports to Prometheus format.
    pub fn export_metrics(&self, metrics: &Metrics, now_ms: u64) -> String {
        let snap = metrics.snapshot(now_ms);
        self.export_snapshot(&snap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_help_and_labels() {
        assert_eq!(
            PrometheusExporter::escape_help("line 1\nline 2\\slash"),
            "line 1\\nline 2\\\\slash"
        );
        assert_eq!(
            PrometheusExporter::escape_label_value("foo\"bar\nbaz\\qux"),
            "foo\\\"bar\\nbaz\\\\qux"
        );
    }

    #[test]
    fn test_format_metric_name_combinations() {
        let default_exp = PrometheusExporter::default();
        assert_eq!(
            default_exp.format_metric_name("packets"),
            "airborne_cot_relay_packets"
        );

        let ns_exp = PrometheusExporter::with_namespace("custom");
        assert_eq!(ns_exp.format_metric_name("gauge"), "custom_gauge");

        let empty_exp = PrometheusExporter::new(ExporterConfig::empty());
        assert_eq!(empty_exp.format_metric_name("raw"), "raw");
    }

    #[test]
    fn test_custom_counter_and_gauge() {
        let mut exp = PrometheusExporter::new(ExporterConfig::empty());
        exp.add_common_label("host", "node-1");
        exp.add_counter(
            "http_requests",
            "Total requests",
            10.0,
            &[("method", "GET")],
        );
        exp.add_gauge("cpu_load", "CPU load", 0.75, &[]);

        let snap = Snapshot {
            packets_in: 0,
            dropped: 0,
            sent: 0,
            reconnects: 0,
            queue_depth: 0,
            last_send_age_ms: None,
        };
        let rendered = exp.export_snapshot(&snap);
        assert!(rendered.contains("# HELP http_requests Total requests\n"));
        assert!(rendered.contains("# TYPE http_requests counter\n"));
        assert!(rendered.contains("http_requests{host=\"node-1\",method=\"GET\"} 10\n"));
        assert!(rendered.contains("cpu_load{host=\"node-1\"} 0.75\n"));
    }

    #[test]
    fn test_export_snapshot_full() {
        let exp = PrometheusExporter::default();
        let snap = Snapshot {
            packets_in: 120,
            dropped: 5,
            sent: 115,
            reconnects: 2,
            queue_depth: 3,
            last_send_age_ms: Some(2500),
        };
        let body = exp.export_snapshot(&snap);
        assert!(body.contains("airborne_cot_relay_packets_in_total 120\n"));
        assert!(body.contains("airborne_cot_relay_packets_dropped_total 5\n"));
        assert!(body.contains("airborne_cot_relay_packets_sent_total 115\n"));
        assert!(body.contains("airborne_cot_relay_reconnects_total 2\n"));
        assert!(body.contains("airborne_cot_relay_queue_depth 3\n"));
        assert!(body.contains("airborne_cot_relay_last_send_age_seconds 2.5\n"));
    }

    #[test]
    fn test_export_live_metrics_and_content_type() {
        assert_eq!(
            PROMETHEUS_CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8"
        );
        let m = Metrics::new();
        m.add_packets_in(10);
        m.set_queue_depth(2);
        let exp = PrometheusExporter::default();
        let rendered = exp.export_metrics(&m, 1_000);
        assert!(rendered.contains("airborne_cot_relay_packets_in_total 10\n"));
        assert!(rendered.contains("airborne_cot_relay_queue_depth 2\n"));
        assert!(!rendered.contains("last_send_age_seconds"));
    }
}
