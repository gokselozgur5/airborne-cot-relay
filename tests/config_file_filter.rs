use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::config_file::lexer::ParsedIni;
use airborne_cot_relay::config_file::map_filter_config;
use airborne_cot_relay::track_filter::FilterConfig;

#[test]
fn test_map_filter_clean() {
    let ini = ParsedIni::parse(
        "[filter]\nmin_interval_ms = 500\nmin_move_m = 2.5\nstale_after_ms = 10000\n",
    )
    .unwrap();
    let base = FilterConfig::default();
    let cfg = map_filter_config(&ini, &base).unwrap();
    assert_eq!(cfg.min_interval_ms, 500);
    assert_eq!(cfg.min_move_m, 2.5);
    assert_eq!(cfg.stale_after_ms, 10_000);
}

#[test]
fn test_map_filter_invalid_and_unknown() {
    let bad_int = ParsedIni::parse("[filter]\nmin_interval_ms = abc\n").unwrap();
    assert!(matches!(
        map_filter_config(&bad_int, &FilterConfig::default()),
        Err(RelayError::InvalidConfig(_))
    ));

    let unknown = ParsedIni::parse("[filter]\nunknown_key = 123\n").unwrap();
    assert!(matches!(
        map_filter_config(&unknown, &FilterConfig::default()),
        Err(RelayError::InvalidConfig(_))
    ));
}
