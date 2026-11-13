use airborne_cot_relay::config_file::ParsedIni;

#[test]
fn test_parse_valid_ini_with_sections_and_comments() {
    let input = r#"
# Global comment
[relay]
stale_timeout_ms = 5000
cot_stale_seconds = 30

# Output section
[output.udp1]
url = udp://127.0.0.1:8087
"#;
    let doc = ParsedIni::parse(input).expect("valid INI parses");
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.get("relay", "stale_timeout_ms"), Some("5000"));
    assert_eq!(doc.get("output.udp1", "url"), Some("udp://127.0.0.1:8087"));
}

#[test]
fn test_line_without_equals_fails_with_line_number() {
    let input = "[relay]\nvalid = 1\ncorrupted_line\nanother = 2\n";
    let err = ParsedIni::parse(input).expect_err("should fail");
    assert_eq!(err.line, 3);
    assert!(err.message.contains("missing '='"));
}

#[test]
fn test_duplicate_key_in_section_fails() {
    let input = "[relay]\nkey1 = a\nkey2 = b\nkey1 = c\n";
    let err = ParsedIni::parse(input).expect_err("duplicate key must fail");
    assert_eq!(err.line, 4);
    assert!(err.message.contains("duplicate key 'key1' in section 'relay'"));
}
