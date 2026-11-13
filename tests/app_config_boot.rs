use airborne_cot_relay::app::config_boot::boot_config;
use airborne_cot_relay::{RelayConfig, RelayError};

fn only(key: &'static str, value: &'static str) -> impl Fn(&str) -> Option<String> {
    move |k| (k == key).then(|| value.to_string())
}

#[test]
fn empty_environment_yields_defaults() {
    assert_eq!(boot_config(|_| None), Ok(RelayConfig::default()));
}

#[test]
fn non_numeric_stale_timeout_is_invalid_config() {
    let res = boot_config(only("RELAY_STALE_TIMEOUT_MS", "abc"));
    assert!(matches!(res, Err(RelayError::InvalidConfig(_))));
}

#[test]
fn malformed_cot_address_is_invalid_config() {
    let res = boot_config(only("RELAY_COT_ADDR", "nonsense"));
    assert!(matches!(res, Err(RelayError::InvalidConfig(_))));
}

#[test]
fn valid_override_is_applied() {
    let cfg = boot_config(only("RELAY_COT_ADDR", "127.0.0.1:7000")).unwrap();
    assert_eq!(cfg.cot_broadcast_addr, "127.0.0.1:7000");
}
