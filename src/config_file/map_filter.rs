//! Maps `[filter]` section to a `FilterConfig`.

use crate::config::RelayError;
use crate::config_file::lexer::{IniSection, ParsedIni};
use crate::track_filter::FilterConfig;

pub(crate) fn apply_filter_section(_c: &mut FilterConfig, _sec: &IniSection) -> Result<(), RelayError> {
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
