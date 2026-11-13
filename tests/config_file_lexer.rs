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
