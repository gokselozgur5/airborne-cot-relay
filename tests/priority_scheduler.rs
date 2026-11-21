use airborne_cot_relay::priority_scheduler::*;

#[test]
fn test_scheduler_emergency_preemption_scenario() {
    let mut sched = PriorityPacketScheduler::new(4);
    sched
        .push_cot(r#"<event version="2.0" type="a-f-G"/>"#, 1000)
        .unwrap();
    sched
        .push_cot(r#"<event version="2.0" type="a-u-G"/>"#, 1001)
        .unwrap();
    sched
        .push_cot(r#"<event version="2.0" type="b-a-o-tbl"/>"#, 1002)
        .unwrap();

    let first = sched.pop().expect("emergency packet expected");
    assert_eq!(first.priority, CotPriority::Emergency);
    assert_eq!(first.sequence, 3);
}

#[test]
fn test_scheduler_stats_tracking() {
    let mut sched = PriorityPacketScheduler::new(3);
    sched.push(b"p1".to_vec(), CotPriority::Low, 1).unwrap();
    sched.push(b"p2".to_vec(), CotPriority::Normal, 2).unwrap();
    sched
        .push(b"p3".to_vec(), CotPriority::Critical, 3)
        .unwrap();
    sched
        .push(b"p4".to_vec(), CotPriority::Emergency, 4)
        .unwrap();

    let stats = sched.stats();
    assert_eq!(stats.dropped[CotPriority::Low.as_tier()], 1);
    assert_eq!(stats.enqueued[CotPriority::Emergency.as_tier()], 1);
}

#[test]
fn test_scheduler_exhaustion_and_clear() {
    let mut sched = PriorityPacketScheduler::new(5);
    sched
        .push(b"item".to_vec(), CotPriority::Normal, 1)
        .unwrap();
    assert_eq!(sched.len(), 1);
    assert!(!sched.is_empty());

    sched.clear();
    assert_eq!(sched.len(), 0);
    assert!(sched.is_empty());
    assert!(sched.pop().is_none());
}

#[test]
fn rejected_packet_is_counted_as_dropped() {
    let mut scheduler = PriorityPacketScheduler::new(1);
    scheduler.push(vec![1], CotPriority::Normal, 0).unwrap();
    assert_eq!(
        scheduler.push(vec![2], CotPriority::Normal, 1),
        Err(SchedulerError::QueueFull)
    );
    assert_eq!(scheduler.stats().dropped[CotPriority::Normal.as_tier()], 1);
}

#[test]
fn test_emergency_push_into_full_emergency_queue_evicts_oldest_emergency() {
    let mut sched = PriorityPacketScheduler::new(2);
    sched
        .push(b"emerg1".to_vec(), CotPriority::Emergency, 1)
        .unwrap();
    sched
        .push(b"emerg2".to_vec(), CotPriority::Emergency, 2)
        .unwrap();

    let seq3 = sched
        .push(b"emerg3".to_vec(), CotPriority::Emergency, 3)
        .expect("Emergency packet push must succeed even when queue is full of emergency packets");

    assert_eq!(sched.len(), 2);
    assert_eq!(sched.stats().dropped[CotPriority::Emergency.as_tier()], 1);
    assert_eq!(sched.stats().enqueued[CotPriority::Emergency.as_tier()], 3);

    let first = sched.pop().unwrap();
    assert_eq!(first.payload, b"emerg2");
    let second = sched.pop().unwrap();
    assert_eq!(second.payload, b"emerg3");
    assert_eq!(second.sequence, seq3);
    assert!(sched.pop().is_none());
}
