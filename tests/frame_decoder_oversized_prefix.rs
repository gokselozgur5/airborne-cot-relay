use airborne_cot_relay::FrameDecoder;

#[test]
fn decoder_does_not_buffer_without_bound_after_an_oversized_prefix() {
    let mut decoder = FrameDecoder::new(16);
    for _ in 0..32 {
        assert!(decoder.decode_all(&[0xff; 65_536]).is_err());
    }
    assert!(
        decoder.buffer_len() <= 65_536,
        "buffer_len {}",
        decoder.buffer_len()
    );
}
