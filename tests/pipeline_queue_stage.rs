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

#[test]
fn capacity_and_policy_accessors() {
    let stage = QueueStage::<u32>::new(5, DropPolicy::DropOldest);
    assert_eq!(stage.capacity(), 5);
    assert_eq!(stage.policy(), DropPolicy::DropOldest);
}

#[test]
fn backpressure_full_and_remaining_capacity() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(3, DropPolicy::DropOldest);
    assert_eq!(stage.remaining_capacity(), 3);
    assert!(!stage.is_full());
    stage.push(10, &metrics);
    assert_eq!(stage.remaining_capacity(), 2);
    stage.push(20, &metrics);
    stage.push(30, &metrics);
    assert_eq!(stage.remaining_capacity(), 0);
    assert!(stage.is_full());
}

#[test]
fn clear_empties_stage_and_resets_depth_gauge() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(3, DropPolicy::DropOldest);
    fill(&mut stage, &metrics, 2);
    assert_eq!(metrics.snapshot(0).queue_depth, 2);
    stage.clear(&metrics);
    assert!(stage.is_empty());
    assert_eq!(stage.len(), 0);
    assert_eq!(metrics.snapshot(0).queue_depth, 0);
}

#[test]
fn drain_returns_all_items_and_resets_gauge() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(3, DropPolicy::DropOldest);
    fill(&mut stage, &metrics, 2);
    let items = stage.drain(&metrics);
    assert_eq!(items, vec![0, 1]);
    assert!(stage.is_empty());
    assert_eq!(metrics.snapshot(0).queue_depth, 0);
}

#[test]
fn push_batch_enqueues_and_returns_discarded() {
    let metrics = Metrics::new();
    let mut stage = QueueStage::new(2, DropPolicy::DropOldest);
    let discarded = stage.push_batch(vec![10, 20, 30], &metrics);
    assert_eq!(discarded, vec![10]);
    assert_eq!(stage.len(), 2);
    assert_eq!(metrics.snapshot(0).dropped, 1);
    assert_eq!(stage.drain(&metrics), vec![20, 30]);
}
