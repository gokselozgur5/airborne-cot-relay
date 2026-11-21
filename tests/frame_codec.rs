use airborne_cot_relay::frame_codec::FrameDecoder;

#[test]
fn decode_all_preserves_buffer_on_error() {
    let mut decoder = FrameDecoder::new(32);
    let mut input = vec![0, 0, 0, 5];
    input.extend_from_slice(b"HELLO");
    input.extend_from_slice(&64u32.to_be_bytes());
    let err = decoder.decode_all(&input);
    assert!(err.is_err());
    assert_eq!(decoder.buffer_len(), 0);
}
