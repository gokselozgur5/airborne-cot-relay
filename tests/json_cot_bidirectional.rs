use airborne_cot_relay::json_cot_codec::CotEventDto;

#[test]
fn test_dto_xml_roundtrip() {
    let mut dto = CotEventDto::new(
        "TRACK-0001",
        "a-f-G-E-V-C",
        "2026-11-16T08:00:00.000Z",
        35.1234567,
        139.1234567,
    );
    dto.hae = 250.5;
    dto.course = Some(45.0);
    dto.speed = Some(15.2);
    dto.callsign = Some("VIPER-1".to_string());

    let xml = dto.to_xml();
    assert!(xml.contains(r#"uid="TRACK-0001""#));
    assert!(xml.contains(r#"course="45.00""#));

    let parsed = CotEventDto::from_xml(&xml).expect("parse xml succeeds");
    assert_eq!(parsed.uid, dto.uid);
    assert_eq!(parsed.cot_type, dto.cot_type);
    assert!((parsed.lat - dto.lat).abs() < 1e-6);
    assert!((parsed.lon - dto.lon).abs() < 1e-6);
    assert_eq!(parsed.course, dto.course);
    assert_eq!(parsed.callsign, dto.callsign);
}

#[test]
fn test_dto_json_roundtrip() {
    let mut dto = CotEventDto::new(
        "DRONE-42",
        "a-u-A",
        "2026-11-16T08:05:00.000Z",
        41.0123456,
        28.9876543,
    );
    dto.hae = 120.0;
    dto.speed = Some(22.4);

    let json = dto.to_json();
    assert!(json.contains(r#""uid":"DRONE-42""#));
    assert!(json.contains(r#""speed":22.40"#));

    let parsed = CotEventDto::from_json(&json).expect("parse json succeeds");
    assert_eq!(parsed.uid, dto.uid);
    assert_eq!(parsed.cot_type, dto.cot_type);
    assert!((parsed.lat - dto.lat).abs() < 1e-6);
    assert_eq!(parsed.speed, dto.speed);
}

#[test]
fn test_direct_json_xml_roundtrip() {
    let json_in = r#"{"uid":"HELO-9","type":"a-f-A","how":"m-g","time":"2026-11-16T08:10:00Z","start":"2026-11-16T08:10:00Z","stale":"2026-11-16T08:10:30Z","lat":39.5000000,"lon":-104.9000000,"hae":1600.00,"ce":12.00,"le":18.00,"course":180.00,"speed":45.00}"#;

    let xml = airborne_cot_relay::json_cot_codec::json_to_cot_xml(json_in).expect("json to xml succeeds");
    assert!(xml.contains(r#"uid="HELO-9""#));
    assert!(xml.contains(r#"lat="39.5000000""#));

    let json_out = airborne_cot_relay::json_cot_codec::cot_xml_to_json(&xml).expect("xml to json succeeds");
    assert!(json_out.contains(r#""uid":"HELO-9""#));
    assert!(json_out.contains(r#""course":180.00"#));
}
