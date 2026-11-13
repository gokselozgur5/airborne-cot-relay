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
