use airborne_cot_relay::xml_sanitize::*;

#[test]
fn test_integration_escape_entities() {
    assert_eq!(escape_xml_attr("foo&bar"), "foo&amp;bar");
    assert_eq!(escape_xml_attr("<tag>"), "&lt;tag&gt;");
    assert_eq!(escape_xml_attr("\"quote'"), "&quot;quote&apos;");
}

#[test]
fn test_integration_escape_text() {
    assert_eq!(escape_xml_text("A < B & B > C"), "A &lt; B &amp; B &gt; C");
    assert_eq!(escape_xml_text("keep \"quotes' as is"), "keep \"quotes' as is");
}

#[test]
fn test_integration_sanitize_cot_uid_and_callsign() {
    let raw_uid = " \tVIPER-01\0\x1f \n";
    assert_eq!(sanitize_cot_uid(raw_uid), "VIPER-01");

    let raw_callsign = "HAWK & EAGLE \"LEAD\"";
    assert_eq!(
        sanitize_cot_callsign(raw_callsign),
        "HAWK &amp; EAGLE &quot;LEAD&quot;"
    );
}
