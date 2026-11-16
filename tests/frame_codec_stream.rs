use airborne_cot_relay::frame_codec::FrameDecoder;

#[test]
fn test_chunked_stream_feeding() {
    let mut decoder = FrameDecoder::new(1024);
    let payload = b"STREAMING_FRAGMENTED_FRAME";
    let encoded = decoder.encode_frame(payload).expect("encode succeeds");

    assert_eq!(decoder.decode_all(&encoded[0..1]).unwrap(), Vec::<Vec<u8>>::new());
    assert_eq!(decoder.decode_all(&encoded[1..3]).unwrap(), Vec::<Vec<u8>>::new());
    assert_eq!(decoder.decode_all(&encoded[3..4]).unwrap(), Vec::<Vec<u8>>::new());

    let mid = 4 + payload.len() / 2;
    assert_eq!(decoder.decode_all(&encoded[4..mid]).unwrap(), Vec::<Vec<u8>>::new());

    let res = decoder.decode_all(&encoded[mid..]).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], payload);
}
