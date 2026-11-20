use airborne_cot_relay::{CotToJsonStream, JsonToCotStream};

#[test]
fn undelimited_input_is_rejected_instead_of_buffered_forever() {
    let chunk = "x".repeat(1 << 20);
    let mut json = JsonToCotStream::new();
    let mut xml = CotToJsonStream::new();
    let (mut json_err, mut xml_err) = (false, false);
    for _ in 0..32 {
        json.feed(&chunk);
        json_err |= json.decode_next().is_err();
        xml.feed(&chunk);
        xml_err |= xml.decode_next().is_err();
    }
    assert!(
        json_err && xml_err,
        "32 MiB buffered per stream without a delimiter or an error"
    );
}
