use airborne_cot_relay::config_file::layers::Provenance;

#[test]
fn test_provenance_display_and_ordering() {
    assert_eq!(Provenance::Default.to_string(), "default");
    assert_eq!(Provenance::File.to_string(), "file");
    assert_eq!(Provenance::Env.to_string(), "env");
    assert_eq!(Provenance::Cli.to_string(), "cli");

    assert!(Provenance::Default < Provenance::File);
    assert!(Provenance::File < Provenance::Env);
    assert!(Provenance::Env < Provenance::Cli);
}
