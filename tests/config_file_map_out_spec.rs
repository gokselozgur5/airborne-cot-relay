use airborne_cot_relay::config_file::map_outputs::{OutputFormat, OutputSpec};

#[test]
fn test_output_spec_validation() {
    let valid = OutputSpec::new("tak", "udp://127.0.0.1:8087", OutputFormat::Xml);
    assert!(valid.validate().is_ok());

    let empty_name = OutputSpec::new("", "udp://127.0.0.1:8087", OutputFormat::Xml);
    assert!(empty_name.validate().is_err());

    let empty_url = OutputSpec::new("tak", "  ", OutputFormat::Xml);
    assert!(empty_url.validate().is_err());
}
