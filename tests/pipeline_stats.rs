use airborne_cot_relay::pipeline::stats::PipelineStats;

#[test]
fn test_pipeline_stats_initial_zero() {
    let stats = PipelineStats::new();
    assert_eq!(stats.received(), 0);
    assert_eq!(stats.dropped(), 0);
    assert_eq!(stats.drop_rate(), 0.0);
}
