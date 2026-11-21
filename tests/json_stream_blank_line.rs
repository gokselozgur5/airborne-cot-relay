use airborne_cot_relay::json_cot_codec::JsonToCotStream;

#[test]
fn blank_line_does_not_hide_the_next_buffered_event() {
    let mut stream = JsonToCotStream::new();
    stream.feed("\n{\"uid\":\"U1\",\"type\":\"a-f-G\",\"time\":\"t\",\"lat\":1.0,\"lon\":2.0}\n");
    let first = stream.decode_next().unwrap();
    assert!(
        first.is_some(),
        "a complete line is buffered but decode_next returned None"
    );
}
