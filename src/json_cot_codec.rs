//! Bidirectional JSON to Cursor-on-Target (CoT) XML converter.

/// Errors occurring during JSON or XML parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonCotError {
    /// Required field was missing from payload.
    MissingField(&'static str),
    /// Malformed JSON payload.
    MalformedJson(&'static str),
    /// Malformed XML payload.
    MalformedXml(&'static str),
    /// Failed to parse numeric value.
    NumericParseError(&'static str),
}
