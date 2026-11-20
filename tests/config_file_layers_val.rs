use airborne_cot_relay::config_file::layers::{CameraLayers, LayeredValue, Provenance};

#[test]
fn test_layered_value_and_camera_layers_default() {
    let lv = LayeredValue::new(42, Provenance::Env);
    assert_eq!(lv.value, 42);
    assert_eq!(lv.provenance, Provenance::Env);

    let def = CameraLayers::default();
    assert_eq!(def.focal_length_px.provenance, Provenance::Default);
    assert_eq!(def.principal_u.provenance, Provenance::Default);
    assert_eq!(def.principal_v.provenance, Provenance::Default);
    assert_eq!(def.image_width.provenance, Provenance::Default);
    assert_eq!(def.image_height.provenance, Provenance::Default);
}
