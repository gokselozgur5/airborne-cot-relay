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
