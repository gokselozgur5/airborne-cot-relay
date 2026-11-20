use airborne_cot_relay::{CotSchemaVersion, DowngradeTransformer};

#[test]
fn downgrade_transformer_never_raises_the_version() {
    let xml = r#"<event version="1.0" uid="A"><detail/></event>"#;
    let keep = DowngradeTransformer::new(CotSchemaVersion::V2_0).with_strip_flow_tags(false);
    assert_eq!(keep.transform(xml).unwrap(), xml);
    let strip = DowngradeTransformer::new(CotSchemaVersion::V2_0);
    assert_eq!(strip.transform(xml).unwrap(), xml);
}
