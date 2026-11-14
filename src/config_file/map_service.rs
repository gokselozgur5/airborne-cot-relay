//! Maps `[http]` and `[log]` sections from configuration file.

use crate::config::RelayError;
use crate::config_file::lexer::{IniSection, ParsedIni};
use crate::log::Level;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

/// Output formatting style for structured log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

impl LogFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LogFormat {
    type Err = RelayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" | "plain" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(RelayError::InvalidConfig(format!(
                "invalid log format: '{other}'"
            ))),
        }
    }
}

/// Configuration for the HTTP status and health service.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HttpServiceConfig {
    pub bind: Option<SocketAddr>,
}

impl HttpServiceConfig {
    pub fn is_enabled(&self) -> bool {
        self.bind.is_some()
    }
}

/// Configuration for the logging service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogServiceConfig {
    pub level: Level,
    pub format: LogFormat,
}

impl Default for LogServiceConfig {
    fn default() -> Self {
        Self {
            level: Level::Info,
            format: LogFormat::Text,
        }
    }
}

fn parse_http_bind(value: &str) -> Result<Option<SocketAddr>, RelayError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    trimmed.parse().map(Some).map_err(|_| {
        RelayError::InvalidConfig(format!("invalid socket address for http.bind: '{value}'"))
    })
}

pub(crate) fn apply_http_section(
    c: &mut HttpServiceConfig,
    sec: &IniSection,
) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "bind" => c.bind = parse_http_bind(&e.value)?,
            k => {
                return Err(RelayError::InvalidConfig(format!(
                    "unknown key '{k}' in section 'http'"
                )))
            }
        }
    }
    Ok(())
}

/// Maps `[http]` section from `ini` onto `base`.
pub fn map_http_config(
    ini: &ParsedIni,
    base: &HttpServiceConfig,
) -> Result<HttpServiceConfig, RelayError> {
    let mut config = base.clone();
    if let Some(sec) = ini.section("http") {
        apply_http_section(&mut config, sec)?;
    }
    Ok(config)
}

fn parse_log_level(value: &str) -> Result<Level, RelayError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "trace" => Ok(Level::Trace),
        "debug" => Ok(Level::Debug),
        "info" => Ok(Level::Info),
        "warn" => Ok(Level::Warn),
        "error" => Ok(Level::Error),
        other => Err(RelayError::InvalidConfig(format!("invalid log level: '{other}'"))),
    }
}

pub(crate) fn apply_log_section(
    c: &mut LogServiceConfig,
    sec: &IniSection,
) -> Result<(), RelayError> {
    for e in &sec.entries {
        match e.key.as_str() {
            "level" => c.level = parse_log_level(&e.value)?,
            "format" => c.format = e.value.parse()?,
            k => return Err(RelayError::InvalidConfig(format!("unknown key '{k}' in section 'log'"))),
        }
    }
    Ok(())
}
