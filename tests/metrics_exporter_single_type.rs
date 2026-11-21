use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::{ExporterConfig, PrometheusExporter};

#[test]
fn each_metric_name_has_a_single_type_line() {
    let mut exporter = PrometheusExporter::new(ExporterConfig::empty());
    exporter.add_counter("sink_sent_total", "Sent per sink", 1.0, &[("sink", "udp")]);
    exporter.add_counter("sink_sent_total", "Sent per sink", 2.0, &[("sink", "tcp")]);
    let body = exporter.export_metrics(&Metrics::new(), 0);
    let type_lines = body.matches("# TYPE sink_sent_total counter\n").count();
    assert_eq!(type_lines, 1, "TYPE lines for sink_sent_total");
}
