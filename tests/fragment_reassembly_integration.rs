//! End-to-end integration test for UDP fragment reassembly.

use airborne_cot_relay::fragment_reassembly::{
    fragment_payload, FragmentReassembler, HEADER_SIZE,
};

#[test]
fn test_raw_packet_stream_reassembly() {
    let mut reassembler = FragmentReassembler::new(10, 5000);
    let payload = b"<event uid=\"drone-1\" type=\"a-f-G-U-C\" lat=\"35.1\" lon=\"42.2\"/>";
    let mtu = HEADER_SIZE + 16;
    let fragments = fragment_payload(1234, payload, mtu).expect("fragmentation successful");
    assert!(fragments.len() > 1);
