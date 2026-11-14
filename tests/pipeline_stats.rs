use airborne_cot_relay::pipeline::stats::PipelineStats;

#[test]
fn test_pipeline_stats_initial_zero() {
    let stats = PipelineStats::new();
    assert_eq!(stats.received(), 0);
    assert_eq!(stats.dropped(), 0);
    assert_eq!(stats.drop_rate(), 0.0);
}

#[test]
fn test_pipeline_stats_record_and_drop_rate() {
    let mut stats = PipelineStats::new();
    stats.record_received(100);
    stats.record_dropped(15);
    assert_eq!(stats.received(), 100);
    assert_eq!(stats.dropped(), 15);
    assert!((stats.drop_rate() - 0.15).abs() < 1e-9);
}
