//! Standard I/O logger with ANSI color and timestamp formatting.

use super::{Level, Logger, Record};

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_BLUE: &str = "\x1b[34m";
const COLOR_MAGENTA: &str = "\x1b[35m";

fn level_color(level: Level) -> &'static str {
    match level {
        Level::Error => COLOR_RED,
        Level::Warn => COLOR_YELLOW,
        Level::Info => COLOR_GREEN,
        Level::Debug => COLOR_BLUE,
        Level::Trace => COLOR_MAGENTA,
    }
}

/// Formats a record into a human-readable log line.
pub fn format_record(record: &Record<'_>, colored: bool) -> String {
    let level_str = record.level.as_str().to_ascii_uppercase();
    let level_formatted = if colored {
        format!("{}{}{}", level_color(record.level), level_str, COLOR_RESET)
    } else {
        level_str
    };
    let mut out = format!(
        "[{}] {} [{}] {}",
        record.ts_ms, level_formatted, record.target, record.message
    );
    for (k, v) in record.fields {
        out.push(' ');
        out.push_str(k);
        out.push('=');
        out.push_str(v);
    }
    out
}

/// Standard I/O logger printing formatted lines to standard error.
#[derive(Debug, Clone)]
pub struct StdioLogger {
    level: Level,
    colored: bool,
}

impl StdioLogger {
    /// Constructs a new stdio logger.
    pub fn new(level: Level, colored: bool) -> Self {
        Self { level, colored }
    }

    /// Returns the maximum enabled log level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns whether ANSI coloring is enabled.
    pub fn is_colored(&self) -> bool {
        self.colored
    }

    /// Formats a record using the logger's color configuration.
    pub fn format(&self, record: &Record<'_>) -> String {
        format_record(record, self.colored)
    }
}

impl Logger for StdioLogger {
    fn level(&self) -> Level {
        self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.level) {
            eprintln!("{}", self.format(record));
        }
    }
}
