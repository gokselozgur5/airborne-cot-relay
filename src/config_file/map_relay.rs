//! Maps `[relay]` and `[camera]` sections to a `RelayConfig`.

use crate::config::{RelayConfig, RelayError};
use crate::config_file::lexer::IniSection;

pub(crate) fn apply_relay_section(c: &mut RelayConfig, sec: &IniSection) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "telemetry_listen_addr" => c.telemetry_listen_addr = e.value.clone(),
            "cot_broadcast_addr" => c.cot_broadcast_addr = e.value.clone(),
            "ground_elevation_m" => {
                c.ground_elevation_m = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!("invalid float for relay.ground_elevation_m: '{}'", e.value))
                })?;
            }
            "target_stale_timeout_ms" => {
                c.target_stale_timeout_ms = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!("invalid integer for relay.target_stale_timeout_ms: '{}'", e.value))
                })?;
            }
            k => return Err(RelayError::InvalidConfig(format!("unknown key '{k}' in section 'relay'"))),
        }
    }
    Ok(())
}

fn parse_float(sec: &str, key: &str, val: &str) -> Result<f64, RelayError> {
    val.parse().map_err(|_| {
        RelayError::InvalidConfig(format!("invalid float for {sec}.{key}: '{val}'"))
    })
}

fn parse_u32(sec: &str, key: &str, val: &str) -> Result<u32, RelayError> {
    val.parse().map_err(|_| {
        RelayError::InvalidConfig(format!("invalid integer for {sec}.{key}: '{val}'"))
    })
}
