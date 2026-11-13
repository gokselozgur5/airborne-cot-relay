use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::pipeline::queue_stage::QueueStage;
use airborne_cot_relay::relay::DropPolicy;

fn fill(stage: &mut QueueStage<u32>, metrics: &Metrics, n: u32) {
    for i in 0..n {
        stage.push(i, metrics);
    }
}

fn drain(stage: &mut QueueStage<u32>, metrics: &Metrics) -> Vec<u32> {
    std::iter::from_fn(|| stage.pop(metrics)).collect()
}

#[test]
fn drop_oldest_keeps_newest_and_counts_overflow() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(3, DropPolicy::DropOldest);
    fill(&mut stage, &metrics, 5);
    assert_eq!(stage.dropped(), 2);
    assert_eq!(metrics.snapshot(0).dropped, 2);
    assert_eq!(drain(&mut stage, &metrics), vec![2, 3, 4]);
}

#[test]
fn drop_newest_keeps_oldest_and_counts_overflow() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(2, DropPolicy::DropNewest);
    fill(&mut stage, &metrics, 4);
    assert_eq!(metrics.snapshot(0).dropped, 2);
    assert_eq!(drain(&mut stage, &metrics), vec![0, 1]);
}

#[test]
fn depth_gauge_follows_push_and_pop() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(4, DropPolicy::DropOldest);
    fill(&mut stage, &metrics, 3);
    assert_eq!(metrics.snapshot(0).queue_depth, 3);
    stage.pop(&metrics);
    assert_eq!(metrics.snapshot(0).queue_depth, 2);
    drain(&mut stage, &metrics);
    assert!(stage.is_empty());
    assert_eq!(metrics.snapshot(0).queue_depth, 0);
}
