//! Minimal HTTP/1.1 request-line parser and response writer.
use std::fmt;

/// Supported HTTP methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Other(String),
}

impl Method {
    /// Parse method from raw string slice.
    pub fn from_str_slice(s: &str) -> Self {
        match s {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            other => Self::Other(other.to_string()),
        }
    }
}
/// Maximum accepted length of an HTTP request line in bytes.
pub const MAX_REQUEST_LINE_BYTES: usize = 8192;

/// Parsed HTTP/1.1 request line components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLine {
    /// HTTP method requested.
    pub method: Method,
    /// Target request path.
    pub path: String,
    /// HTTP protocol version string.
    pub version: String,
}

impl RequestLine {
    /// Construct a new request line.
    pub fn new(method: Method, path: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            version: version.into(),
        }
    }
}
/// Errors encountered during HTTP request parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Request line exceeded MAX_REQUEST_LINE_BYTES.
    OversizedRequestLine,
    /// Request line was malformed or missing fields.
    MalformedRequestLine(String),
}

impl HttpError {
    /// Return the corresponding HTTP status code.
    pub fn status_code(&self) -> u16 {
        400
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OversizedRequestLine => write!(f, "request line exceeds maximum limit"),
            Self::MalformedRequestLine(reason) => write!(f, "malformed request line: {reason}"),
        }
    }
}

impl std::error::Error for HttpError {}
