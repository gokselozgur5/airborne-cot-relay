use airborne_cot_relay::{CotSignatureValidator, CotSigner, CotSignerConfig};

#[test]
fn replayed_frame_is_rejected() {
    let key = b"mission-key-0123456789abcdef".to_vec();
    let mut signer = CotSigner::new(CotSignerConfig {
        key_id: 1,
        secret_key: key.clone(),
        max_clock_skew_secs: 300,
    });
    let frame = signer.create_signed_frame(b"<event uid=\"A\"/>", 1000);
    let mut validator = CotSignatureValidator::new(300);
    validator.add_key(1, key);
    assert!(validator.verify_frame(&frame, 1000).is_ok());
    let again = validator.verify_frame(&frame, 1200);
    assert!(again.is_err(), "captured frame accepted again 200 s later");
}
