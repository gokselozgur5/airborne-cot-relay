use airborne_cot_relay::json_cot_codec::{cot_xml_to_json, json_to_cot_xml, CotEventDto, JsonCotError};

#[test]
fn test_error_missing_and_malformed() {
    assert_eq!(
        CotEventDto::from_json("not json").unwrap_err(),
        JsonCotError::MalformedJson("expected root JSON object")
    );

    let no_uid = r#"{"type":"a-f-G","time":"2026-11-16T08:00:00Z","lat":10.0,"lon":20.0}"#;
    assert_eq!(
        json_to_cot_xml(no_uid).unwrap_err(),
        JsonCotError::MissingField("uid")
    );

    assert_eq!(
        cot_xml_to_json("<invalid>foo</invalid>").unwrap_err(),
        JsonCotError::MalformedXml("missing event tag")
    );
}
