//! Maps `[relay]` and `[camera]` sections to a `RelayConfig`.

use crate::config::{CameraIntrinsics, RelayConfig, RelayError};
use crate::config_file::lexer::{IniSection, ParsedIni};

pub(crate) fn apply_relay_section(c: &mut RelayConfig, sec: &IniSection) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "telemetry_listen_addr" => c.telemetry_listen_addr = e.value.clone(),
            "cot_broadcast_addr" => c.cot_broadcast_addr = e.value.clone(),
            "ground_elevation_m" => {
                c.ground_elevation_m = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!(
                        "invalid float for relay.ground_elevation_m: '{}'",
                        e.value
                    ))
                })?;
            }
            "target_stale_timeout_ms" => {
                c.target_stale_timeout_ms = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!(
                        "invalid integer for relay.target_stale_timeout_ms: '{}'",
                        e.value
                    ))
                })?;
            }
            k => {
                return Err(RelayError::InvalidConfig(format!(
                    "unknown key '{k}' in section 'relay'"
                )))
            }
        }
    }
    Ok(())
}

fn parse_float(sec: &str, key: &str, val: &str) -> Result<f64, RelayError> {
    val.parse()
        .map_err(|_| RelayError::InvalidConfig(format!("invalid float for {sec}.{key}: '{val}'")))
}

fn parse_u32(sec: &str, key: &str, val: &str) -> Result<u32, RelayError> {
    val.parse()
        .map_err(|_| RelayError::InvalidConfig(format!("invalid integer for {sec}.{key}: '{val}'")))
}

pub(crate) fn apply_camera_section(
    c: &mut CameraIntrinsics,
    sec: &IniSection,
) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "focal_length_px" => c.focal_length_px = parse_float("camera", &e.key, &e.value)?,
            "principal_u" => c.principal_u = parse_float("camera", &e.key, &e.value)?,
            "principal_v" => c.principal_v = parse_float("camera", &e.key, &e.value)?,
            "image_width" => c.image_width = parse_u32("camera", &e.key, &e.value)?,
            "image_height" => c.image_height = parse_u32("camera", &e.key, &e.value)?,
            k => {
                return Err(RelayError::InvalidConfig(format!(
                    "unknown key '{k}' in section 'camera'"
                )))
            }
        }
    }
    Ok(())
}

/// Maps `[relay]` and `[camera]` sections from `ini` onto `base`, validating the result.
pub fn map_relay_config(ini: &ParsedIni, base: &RelayConfig) -> Result<RelayConfig, RelayError> {
    let mut config = base.clone();
    if let Some(sec) = ini.section("relay") {
        apply_relay_section(&mut config, sec)?;
    }
    if let Some(sec) = ini.section("camera") {
        apply_camera_section(&mut config.camera, sec)?;
    }
    config.validate()?;
    Ok(config)
}
