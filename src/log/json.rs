//! JSON structured logger emitting newline-delimited JSON.

use super::{Level, Logger, Record};
use std::io::{self, Write};
use std::sync::Mutex;

fn escape_json_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
}

/// Formats a record into a single-line JSON string.
pub fn format_json(record: &Record<'_>) -> String {
    let mut out = String::with_capacity(128);
    out.push_str("{\"ts\":");
    out.push_str(&record.ts_ms.to_string());
    out.push_str(",\"level\":\"");
    out.push_str(record.level.as_str());
    out.push_str("\",\"target\":\"");
    escape_json_into(record.target, &mut out);
    out.push_str("\",\"message\":\"");
    escape_json_into(record.message, &mut out);
    out.push_str("\"");
    for (k, v) in record.fields {
        out.push_str(",\"");
        escape_json_into(k, &mut out);
        out.push_str("\":\"");
        escape_json_into(v, &mut out);
        out.push('"');
    }
    out.push('}');
    out
}

/// Structured logger writing newline-delimited JSON lines.
pub struct JsonLogger<W: Write + Send + 'static> {
    level: Level,
    writer: Mutex<W>,
}

impl<W: Write + Send + 'static> JsonLogger<W> {
    /// Creates a new JsonLogger.
    pub fn new(level: Level, writer: W) -> Self {
        Self {
            level,
            writer: Mutex::new(writer),
        }
    }
}

impl<W: Write + Send + 'static> Logger for JsonLogger<W> {
    fn level(&self) -> Level {
        self.level
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.level) {
            return;
        }
        let line = format_json(record);
        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{line}");
        }
    }
}
