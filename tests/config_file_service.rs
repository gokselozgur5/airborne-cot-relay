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
