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
