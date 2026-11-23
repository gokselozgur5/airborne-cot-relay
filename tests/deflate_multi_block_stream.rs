use airborne_cot_relay::payload_compressor::deflate_decompress;

const STREAM: &str =
    "b2492d4bcd2b5128cd4cb15572545228a92c48b5554ad44dd37557b203000000ffffb329c8cfcc2b51c8492cb15532d4335052c8c9cfb3553202b2f4ed6cf453cb52f34aec00";
const PLAIN: &str = r#"<event uid="A" type="a-f-G"><point lat="1.0" lon="2.0"/></event>"#;

#[test]
fn deflate_decompress_reads_every_block_or_reports_an_error() {
    let bytes: Vec<u8> = (0..STREAM.len() / 2)
        .map(|i| u8::from_str_radix(&STREAM[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    if let Ok(out) = deflate_decompress(&bytes) {
        assert_eq!(String::from_utf8_lossy(&out), PLAIN);
    }
}
