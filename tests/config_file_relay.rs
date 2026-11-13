use airborne_cot_relay::config::RelayConfig;
use airborne_cot_relay::config_file::{map_relay_config, ParsedIni};

#[test]
fn test_all_relay_and_camera_fields_set_from_file() {
    let conf = "[relay]\ntelemetry_listen_addr = 127.0.0.1:15000\ncot_broadcast_addr = 239.2.3.4:7000\nground_elevation_m = 125.5\ntarget_stale_timeout_ms = 8000\n[camera]\nfocal_length_px = 1500.0\nprincipal_u = 960.0\nprincipal_v = 540.0\nimage_width = 1920\nimage_height = 1080\n";
    let ini = ParsedIni::parse(conf).expect("parse INI");
    let res = map_relay_config(&ini, &RelayConfig::default()).expect("map");
    assert_eq!(res.telemetry_listen_addr, "127.0.0.1:15000");
    assert_eq!(res.cot_broadcast_addr, "239.2.3.4:7000");
    assert_eq!(res.ground_elevation_m, 125.5);
    assert_eq!(res.target_stale_timeout_ms, 8000);
    assert_eq!(res.camera.focal_length_px, 1500.0);
    assert_eq!(res.camera.principal_u, 960.0);
    assert_eq!(res.camera.principal_v, 540.0);
    assert_eq!(res.camera.image_width, 1920);
    assert_eq!(res.camera.image_height, 1080);
}

#[test]
fn test_unknown_key_in_relay_section_fails() {
    let conf = "[relay]\nunknown_foo = 1\n";
    let ini = ParsedIni::parse(conf).unwrap();
    let err = map_relay_config(&ini, &RelayConfig::default()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown key 'unknown_foo' in section 'relay'"));
}

#[test]
fn test_unknown_key_in_camera_section_fails() {
    let conf = "[camera]\nbad_param = 1\n";
    let ini = ParsedIni::parse(conf).unwrap();
    let err = map_relay_config(&ini, &RelayConfig::default()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown key 'bad_param' in section 'camera'"));
}

#[test]
fn test_invalid_numeric_values_and_validation() {
    let conf1 = "[relay]\nground_elevation_m = abc\n";
    let ini1 = ParsedIni::parse(conf1).unwrap();
    assert!(map_relay_config(&ini1, &RelayConfig::default()).is_err());

    let conf2 = "[camera]\nfocal_length_px = -10.0\n";
    let ini2 = ParsedIni::parse(conf2).unwrap();
    assert!(map_relay_config(&ini2, &RelayConfig::default()).is_err());
}
