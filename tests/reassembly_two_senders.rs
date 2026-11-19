use airborne_cot_relay::fragmentation::{
    FragmenterConfig, PayloadFragmenter, ReassemblyConfig, ReassemblyEngine,
};

#[test]
fn fragments_of_two_senders_are_not_spliced_into_one_message() {
    let cfg = FragmenterConfig { mtu: 24, max_payload_size: 1024 };
    let mut sender_a = PayloadFragmenter::new(cfg.clone()).unwrap();
    let mut sender_b = PayloadFragmenter::new(cfg).unwrap();
    let a = sender_a.fragment(b"AAAAAAAAAAAAAAAA").unwrap();
    let b = sender_b.fragment(b"BBBBBBBBBBBBBBBB").unwrap();
    let mut engine = ReassemblyEngine::new(ReassemblyConfig::default());
    let mut delivered = Vec::new();
    for packet in [&a[0], &b[0], &b[1], &a[1]] {
        if let Ok(Some(message)) = engine.process_bytes(&packet.to_bytes(), 0) {
            delivered.push(String::from_utf8(message).unwrap());
        }
    }
    assert!(!delivered.contains(&"AAAAAAAABBBBBBBB".to_string()), "delivered {delivered:?}");
}
