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
/// Parse an HTTP/1.1 request line from a raw string slice.
pub fn parse_request_line(raw: &str) -> Result<RequestLine, HttpError> {
    if raw.len() > MAX_REQUEST_LINE_BYTES {
        return Err(HttpError::OversizedRequestLine);
    }
    let trimmed = raw.trim_end_matches(['\r', '\n']);
    let parts: Vec<&str> = trimmed.split(' ').collect();
    if parts.len() != 3 {
        return Err(HttpError::MalformedRequestLine("expected 3 tokens".into()));
    }
    let (method_raw, path, version) = (parts[0], parts[1], parts[2]);
    if method_raw.is_empty() || path.is_empty() || version.is_empty() {
        return Err(HttpError::MalformedRequestLine("empty token".into()));
    }
    if !path.starts_with('/') {
        return Err(HttpError::MalformedRequestLine("path must start with /".into()));
    }
    if !version.starts_with("HTTP/") {
        return Err(HttpError::MalformedRequestLine("invalid version prefix".into()));
    }
    let method = Method::from_str_slice(method_raw);
    Ok(RequestLine::new(method, path, version))
}
/// HTTP status code with numeric value and reason phrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK: StatusCode = StatusCode(200);
    pub const BAD_REQUEST: StatusCode = StatusCode(400);
    pub const NOT_FOUND: StatusCode = StatusCode(404);
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode(405);
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode(503);

    /// Get standard HTTP reason phrase.
    pub fn reason_phrase(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }
}
/// HTTP response representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// Response status code.
    pub status: StatusCode,
    /// Optional Content-Type header.
    pub content_type: Option<String>,
    /// Response payload bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a generic response.
    pub fn new(status: StatusCode, body: impl Into<Vec<u8>>, content_type: Option<&str>) -> Self {
        Self {
            status,
            content_type: content_type.map(String::from),
            body: body.into(),
        }
    }

    /// Helper for 200 OK response.
    pub fn ok(body: impl Into<Vec<u8>>, content_type: &str) -> Self {
        Self::new(StatusCode::OK, body, Some(content_type))
    }
}
