//! Configuration boot: defaults, then environment overrides, then validation.
use crate::config::{RelayConfig, RelayError};

/// Process exit status used when the configuration cannot be booted.
pub const CONFIG_ERROR_EXIT: u8 = 2;

/// Builds a validated config from defaults and the given environment lookup.
pub fn boot_config<F>(lookup: F) -> Result<RelayConfig, RelayError>
where
    F: Fn(&str) -> Option<String>,
{
    let config = RelayConfig::default().with_env_overrides(lookup)?;
    config.validate()?;
    Ok(config)
}
