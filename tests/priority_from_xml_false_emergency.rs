use airborne_cot_relay::CotPriority;

#[test]
fn routine_event_with_911_in_a_number_is_not_an_emergency() {
    let xml = r#"<event version="2.0" uid="UAV-7" type="a-u-G" how="m-g" time="2026-11-16T10:00:00.911Z" start="2026-11-16T10:00:00.911Z" stale="2026-11-16T10:01:00.911Z"><point lat="34.5" lon="-118.2" hae="100.0" ce="10.0" le="10.0"/></event>"#;
    assert_eq!(CotPriority::from_xml_str(xml), CotPriority::Normal);
}

#[test]
fn remark_mentioning_the_word_is_not_an_emergency() {
    let xml = r#"<event version="2.0" uid="UAV-7" type="a-u-G"><detail><remarks>no emergency</remarks></detail></event>"#;
    assert_eq!(CotPriority::from_xml_str(xml), CotPriority::Normal);
}
