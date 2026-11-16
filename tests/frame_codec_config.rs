use airborne_cot_relay::frame_codec::{Endianness, FrameDecoder, LengthField};

#[test]
fn test_u16_little_endian_codec() {
    let mut decoder = FrameDecoder::new(4096)
        .with_length_field(LengthField::U16)
        .with_endianness(Endianness::LittleEndian);

    assert_eq!(decoder.length_field(), LengthField::U16);
    assert_eq!(decoder.endianness(), Endianness::LittleEndian);

    let payload = b"CUSTOM_CONFIG_FRAME";
    let encoded = decoder.encode_frame(payload).expect("encode succeeds");
    assert_eq!(encoded.len(), 2 + payload.len());
    let prefix = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert_eq!(prefix, payload.len());

    let frames = decoder.decode_all(&encoded).expect("decode succeeds");
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0], payload);
}
