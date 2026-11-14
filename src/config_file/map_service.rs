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
