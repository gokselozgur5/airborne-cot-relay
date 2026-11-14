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
