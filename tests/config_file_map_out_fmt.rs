use airborne_cot_relay::config_file::map_outputs::OutputFormat;

#[test]
fn test_output_format_parse() {
    assert_eq!(OutputFormat::parse_str("xml").unwrap(), OutputFormat::Xml);
    assert_eq!(
        OutputFormat::parse_str("tak-proto").unwrap(),
        OutputFormat::TakProto
    );
    assert_eq!(
        OutputFormat::parse_str("takproto").unwrap(),
        OutputFormat::TakProto
    );
    assert!(OutputFormat::parse_str("unknown").is_err());
}
