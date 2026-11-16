use airborne_cot_relay::priority_scheduler::*;

#[test]
fn test_scheduler_emergency_preemption_scenario() {
    let mut sched = PriorityPacketScheduler::new(4);
    sched.push_cot(r#"<event version="2.0" type="a-f-G"/>"#, 1000).unwrap();
    sched.push_cot(r#"<event version="2.0" type="a-u-G"/>"#, 1001).unwrap();
    sched.push_cot(r#"<event version="2.0" type="b-a-o-tbl"/>"#, 1002).unwrap();

    let first = sched.pop().expect("emergency packet expected");
    assert_eq!(first.priority, CotPriority::Emergency);
    assert_eq!(first.sequence, 3);
}
