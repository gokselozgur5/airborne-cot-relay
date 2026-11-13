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
