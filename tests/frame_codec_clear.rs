use airborne_cot_relay::frame_codec::FrameDecoder;

#[test]
fn test_zero_length_payload_and_clear() {
    let mut decoder = FrameDecoder::new(1024);
    let empty = b"";
    let encoded = decoder.encode_frame(empty).expect("encode succeeds");
    assert_eq!(encoded.len(), 4);

    let frames = decoder.decode_all(&encoded).expect("decode succeeds");
    assert_eq!(frames.len(), 1);
    assert!(frames[0].is_empty());

    decoder.feed(b"PARTIAL_HEADER_BYTES");
    assert_eq!(decoder.buffer_len(), 20);
    decoder.clear();
    assert_eq!(decoder.buffer_len(), 0);
    assert_eq!(decoder.decode_next().unwrap(), None);
}
