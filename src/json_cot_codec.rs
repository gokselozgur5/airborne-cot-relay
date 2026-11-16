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

impl std::fmt::Display for JsonCotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::MalformedJson(msg) => write!(f, "malformed json: {msg}"),
            Self::MalformedXml(msg) => write!(f, "malformed xml: {msg}"),
            Self::NumericParseError(field) => write!(f, "invalid numeric value for: {field}"),
        }
    }
}

impl std::error::Error for JsonCotError {}
