use airborne_cot_relay::frame_codec::FrameDecoder;

#[test]
fn test_multiple_concatenated_frames() {
    let mut decoder = FrameDecoder::new(1024);
    let p1 = b"FIRST_PACKET";
    let p2 = b"SECOND_PACKET";
    let p3 = b"THIRD_PACKET";

    let mut stream = Vec::new();
    stream.extend_from_slice(&decoder.encode_frame(p1).unwrap());
    stream.extend_from_slice(&decoder.encode_frame(p2).unwrap());
    stream.extend_from_slice(&decoder.encode_frame(p3).unwrap());

    let frames = decoder.decode_all(&stream).unwrap();
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0], p1);
    assert_eq!(frames[1], p2);
    assert_eq!(frames[2], p3);
    assert_eq!(decoder.buffer_len(), 0);
}
