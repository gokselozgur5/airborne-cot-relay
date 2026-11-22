use airborne_cot_relay::fragmentation::*;
use airborne_cot_relay::*;

#[test]
fn test_repro_370_glob_imports_no_ambiguity_and_single_protocol() {
    let header = FragmentHeader::new(100, 0, 1, 0x1234).unwrap();
    assert_eq!(header.msg_id, 100);
    assert_eq!(HEADER_SIZE, 16);
}
