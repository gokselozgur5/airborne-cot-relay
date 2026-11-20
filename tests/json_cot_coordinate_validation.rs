use airborne_cot_relay::json_to_cot_xml;

#[test]
fn non_finite_and_out_of_range_coordinates_are_rejected() {
    for coords in [
        r#""lat":NaN,"lon":2.0"#,
        r#""lat":1e999,"lon":2.0"#,
        r#""lat":123.0,"lon":999.0"#,
    ] {
        let json =
            format!(r#"{{"uid":"U1","type":"a-f-G","time":"2026-11-16T08:00:00Z",{coords}}}"#);
        let out = json_to_cot_xml(&json);
        assert!(out.is_err(), "{coords} produced {out:?}");
    }
}
