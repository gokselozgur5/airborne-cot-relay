//! Maps `[filter]` section to a `FilterConfig`.

use crate::config::RelayError;
use crate::config_file::lexer::{IniSection, ParsedIni};
use crate::track_filter::FilterConfig;

pub(crate) fn apply_filter_section(c: &mut FilterConfig, sec: &IniSection) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "min_interval_ms" => {
                c.min_interval_ms = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!("invalid integer for filter.min_interval_ms: '{}'", e.value))
                })?;
            }
            "min_move_m" => {
                c.min_move_m = e.value.parse().map_err(|_| {
                    RelayError::InvalidConfig(format!("invalid float for filter.min_move_m: '{}'", e.value))
                })?;
            }
            "stale_after_ms" => {}
            k => return Err(RelayError::InvalidConfig(format!("unknown key '{k}' in section 'filter'"))),
        }
    }
    Ok(())
}

/// Maps `[filter]` section from `ini` onto `base`.
pub fn map_filter_config(ini: &ParsedIni, base: &FilterConfig) -> Result<FilterConfig, RelayError> {
    let mut config = *base;
    if let Some(sec) = ini.section("filter") {
        apply_filter_section(&mut config, sec)?;
    }
    Ok(config)
}
