//! Structured logging levels and logger traits.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

/// Severity level for structured log records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

/// Error returned when parsing an unrecognized log level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLevelError(pub String);
