use airborne_cot_relay::{downgrade_cot_xml, CotEventDto, CotSchemaVersion};

const XML: &str = "<event version='2.0' uid='ANDROID-1' type='a-f-G-U-C' how='h-e' \
  time='2026-11-16T08:00:00Z' start='2026-11-16T08:00:00Z' stale='2026-11-16T08:01:00Z'>\
  <point lat='34.1' lon='-118.2' hae='10.0' ce='5.0' le='5.0'/><detail/></event>";

#[test]
fn from_xml_accepts_single_quoted_attributes() {
    let dto = CotEventDto::from_xml(XML).expect("well-formed XML 1.0");
    assert_eq!(dto.uid, "ANDROID-1");
    assert_eq!(dto.lat, 34.1);
}

#[test]
fn version_is_detected_and_not_duplicated_when_single_quoted() {
    let out = downgrade_cot_xml(XML, CotSchemaVersion::V1_0).unwrap();
    assert_eq!(out.matches("version=").count(), 1, "{out}");
    assert_eq!(
        CotSchemaVersion::detect_from_xml(XML),
        Ok(CotSchemaVersion::V2_0)
    );
}
