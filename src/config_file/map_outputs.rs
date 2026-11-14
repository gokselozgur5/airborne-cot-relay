//! Maps `[output.NAME]` sections to output destination specifications.

use crate::config::RelayError;
use crate::config_file::lexer::{IniSection, ParsedIni};

/// Output format for CoT streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Xml,
    TakProto,
}

impl OutputFormat {
    /// Parse output format from string representation.
    pub fn parse_str(s: &str) -> Result<Self, RelayError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "xml" => Ok(Self::Xml),
            "tak-proto" | "tak_proto" | "takproto" | "proto" => Ok(Self::TakProto),
            other => Err(RelayError::InvalidConfig(format!(
                "invalid output format: '{other}'"
            ))),
        }
    }
}
