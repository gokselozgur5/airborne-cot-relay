use airborne_cot_relay::CotEventDto;

#[test]
fn from_json_decodes_escaped_quotes() {
    let json = r#"{"uid":"U1","type":"a-f-G","time":"2026-11-16T08:00:00Z","lat":1.0,"lon":2.0,"callsign":"Alpha \"Two\""}"#;
    let dto = CotEventDto::from_json(json).unwrap();
    assert_eq!(dto.callsign.as_deref(), Some(r#"Alpha "Two""#));
}

#[test]
fn to_json_escapes_quotes_and_backslashes() {
    let mut dto = CotEventDto::new("U1", "a-f-G", "2026-11-16T08:00:00Z", 1.0, 2.0);
    dto.callsign = Some(r#"Alpha "Two" C:\x"#.to_string());
    let json = dto.to_json();
    assert!(
        json.contains(r#""callsign":"Alpha \"Two\" C:\\x""#),
        "{json}"
    );
}

#[test]
fn round_trip_keeps_control_characters() {
    let mut dto = CotEventDto::new("U1", "a-f-G", "2026-11-16T08:00:00Z", 1.0, 2.0);
    dto.callsign = Some("a\"b\\c\nd\u{1}".to_string());
    let back = CotEventDto::from_json(&dto.to_json()).unwrap();
    assert_eq!(back.callsign, dto.callsign);
}
