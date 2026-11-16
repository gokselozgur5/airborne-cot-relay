use airborne_cot_relay::frame_codec::{FrameCodecError, FrameDecoder};

#[test]
fn test_frame_exceeding_max_len_is_rejected() {
    let mut decoder = FrameDecoder::new(32);
    let payload = vec![0x42u8; 64];

    let err = decoder.encode_frame(&payload).unwrap_err();
    assert_eq!(
        err,
        FrameCodecError::FrameTooLarge {
            frame_len: 64,
            max_len: 32,
        }
    );

    let raw = (64u32).to_be_bytes();
    decoder.feed(&raw);
    let decode_err = decoder.decode_next().unwrap_err();
    assert_eq!(
        decode_err,
        FrameCodecError::FrameTooLarge {
            frame_len: 64,
            max_len: 32,
        }
    );
}
