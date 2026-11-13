//! Structured logging levels and logger traits.

pub mod stdio;
pub use stdio::{format_record, StdioLogger};

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

impl fmt::Display for ParseLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid log level: '{}'", self.0)
    }
}

impl std::error::Error for ParseLevelError {}

impl Level {
    /// Returns lowercase static string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("error") {
            Ok(Self::Error)
        } else if trimmed.eq_ignore_ascii_case("warn") || trimmed.eq_ignore_ascii_case("warning") {
            Ok(Self::Warn)
        } else if trimmed.eq_ignore_ascii_case("info") {
            Ok(Self::Info)
        } else if trimmed.eq_ignore_ascii_case("debug") {
            Ok(Self::Debug)
        } else if trimmed.eq_ignore_ascii_case("trace") {
            Ok(Self::Trace)
        } else {
            Err(ParseLevelError(s.to_string()))
        }
    }
}

/// A structured log record with metadata, message, and key-value fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Record<'a> {
    pub ts_ms: u64,
    pub level: Level,
    pub target: &'a str,
    pub message: &'a str,
    pub fields: &'a [(&'a str, &'a str)],
}

impl<'a> Record<'a> {
    /// Constructs a new log record.
    pub fn new(
        ts_ms: u64,
        level: Level,
        target: &'a str,
        message: &'a str,
        fields: &'a [(&'a str, &'a str)],
    ) -> Self {
        Self {
            ts_ms,
            level,
            target,
            message,
            fields,
        }
    }

    /// Looks up a field value by key name.
    pub fn get_field(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

/// Trait for receiving structured log records.
pub trait Logger: Send + Sync {
    /// Returns the maximum enabled log level.
    fn level(&self) -> Level;

    /// Checks if a record with the given level would be logged.
    fn enabled(&self, level: Level) -> bool {
        level <= self.level()
    }

    /// Logs a record if its level is enabled.
    fn log(&self, record: &Record<'_>);
}

impl<L: Logger + ?Sized> Logger for &L {
    fn level(&self) -> Level {
        (**self).level()
    }

    fn enabled(&self, level: Level) -> bool {
        (**self).enabled(level)
    }

    fn log(&self, record: &Record<'_>) {
        (**self).log(record);
    }
}

impl<L: Logger + ?Sized> Logger for Arc<L> {
    fn level(&self) -> Level {
        (**self).level()
    }

    fn enabled(&self, level: Level) -> bool {
        (**self).enabled(level)
    }

    fn log(&self, record: &Record<'_>) {
        (**self).log(record);
    }
}
