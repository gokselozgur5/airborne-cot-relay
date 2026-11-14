use airborne_cot_relay::output::build_remarks;
use sightline_cot::format_mgrs;

#[test]
fn remarks_carry_mgrs_sensor_and_confidence() {
    let mgrs = format_mgrs(37.7749, -122.4194, 5).unwrap();
    let xml = build_remarks(37.7749, -122.4194, Some("eo1"), Some(0.88)).unwrap();
    let expected = format!(
        "<remarks>mgrs={} sensor=eo1 conf=0.88</remarks>",
        mgrs.as_str()
    );
    assert_eq!(xml, expected);
}

#[test]
fn remarks_omit_optional_fields() {
    let xml = build_remarks(0.0, 3.0, None, None).unwrap();
    assert_eq!(xml, "<remarks>mgrs=31NEA0000000000</remarks>");
}

#[test]
fn remarks_reject_polar_latitude() {
    assert!(build_remarks(89.0, 0.0, None, None).is_err());
}
