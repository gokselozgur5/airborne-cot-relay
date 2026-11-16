use airborne_cot_relay::frame_codec::FrameDecoder;

#[test]
fn test_encode_decode_roundtrip() {
    let mut decoder = FrameDecoder::new(1024);
    let payload = b"COT_TELEMETRY_SAMPLE_PACKET";
    let encoded = decoder.encode_frame(payload).expect("encode succeeds");
    assert_eq!(encoded.len(), 4 + payload.len());

    let frames = decoder.decode_all(&encoded).expect("decode succeeds");
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0], payload);
}
