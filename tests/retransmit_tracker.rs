use airborne_cot_relay::retransmit_tracker::{RbtConfig, RetransmitTracker};

#[test]
fn ack_does_not_clear_force_retransmit_from_other_receiver_nack() {
    let mut tracker = RetransmitTracker::new(RbtConfig::default());
    tracker.track_packet(1, vec![1, 2, 3], 1000).unwrap();
    assert!(tracker.process_nack(1, "node-b"));
    assert!(tracker.process_ack(1, "node-a"));
    let pending = tracker.get_pending_retransmissions(1001);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].0, 1);
}
