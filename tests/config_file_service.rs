use airborne_cot_relay::config_file::lexer::ParsedIni;
use airborne_cot_relay::config_file::map_service::{
    map_http_config, map_log_config, HttpServiceConfig, LogFormat, LogServiceConfig,
};
use airborne_cot_relay::log::Level;

#[test]
fn test_default_service_configs() {
    let http = HttpServiceConfig::default();
    assert!(!http.is_enabled());
    assert_eq!(http.bind, None);

    let log = LogServiceConfig::default();
    assert_eq!(log.level, Level::Info);
    assert_eq!(log.format, LogFormat::Text);
}

#[test]
fn test_parse_http_and_log_service_ini() {
    let ini =
        ParsedIni::parse("[http]\nbind = 127.0.0.1:8080\n[log]\nlevel = debug\nformat = json")
            .unwrap();
    let http = map_http_config(&ini, &HttpServiceConfig::default()).unwrap();
    assert!(http.is_enabled());
    assert_eq!(http.bind, Some("127.0.0.1:8080".parse().unwrap()));

    let log = map_log_config(&ini, &LogServiceConfig::default()).unwrap();
    assert_eq!(log.level, Level::Debug);
    assert_eq!(log.format, LogFormat::Json);
}
