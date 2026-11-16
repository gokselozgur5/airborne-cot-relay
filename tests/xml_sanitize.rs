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
