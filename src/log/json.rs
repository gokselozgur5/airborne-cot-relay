//! JSON structured logger emitting newline-delimited JSON.

use super::{Level, Logger, Record};
use std::io::{self, Write};
use std::sync::Mutex;

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
