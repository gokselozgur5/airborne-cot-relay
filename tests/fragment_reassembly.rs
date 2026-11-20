use airborne_cot_relay::fragment_reassembly::{fragment_payload, FragmentReassembler};

#[test]
fn test_process_fragment_prunes_stale_existing_entry() {
    let mut r = FragmentReassembler::new(5, 50);
    let f1 = fragment_payload(88, b"old-abandoned", 10).unwrap();
    let f2 = fragment_payload(88, b"new-payload!!", 10).unwrap();
    assert_eq!(r.process_fragment(f1[0].clone(), 100).unwrap(), None);
    assert_eq!(r.in_flight_count(), 1);
    assert_eq!(r.process_fragment(f2[0].clone(), 200).unwrap(), None);
    assert_eq!(r.stats().stale_dropped_messages, 1);
}
