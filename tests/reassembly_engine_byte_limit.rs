use airborne_cot_relay::fragmentation::{
    crc16_ccitt, FragmentHeader, FragmentPacket, ReassemblyConfig, ReassemblyEngine,
};

#[test]
fn reassembly_engine_bounds_the_bytes_buffered_for_one_message() {
    let mut engine = ReassemblyEngine::new(ReassemblyConfig::default());
    let chunk = vec![0x41u8; 1400];
    let crc = crc16_ccitt(&chunk);
    let mut rejected = false;
    for seq in 0..3000u16 {
        let header = FragmentHeader::new(7, seq, u16::MAX, crc).unwrap();
        let packet = FragmentPacket::new(header, chunk.clone()).to_bytes();
        rejected |= engine.process_bytes(&packet, 0).is_err();
    }
    assert!(
        rejected,
        "4.2 MB accepted for one message, sender limit is 1 MiB"
    );
}
