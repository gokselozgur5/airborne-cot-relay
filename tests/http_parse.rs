use airborne_cot_relay::http::{parse_request_line, Method};

#[test]
fn test_parse_metrics_request_line() {
    let raw = "GET /metrics HTTP/1.1";
    let req = parse_request_line(raw).expect("should parse valid request");
    assert_eq!(req.method, Method::Get);
    assert_eq!(req.path, "/metrics");
    assert_eq!(req.version, "HTTP/1.1");
}

#[test]
fn test_parse_with_crlf_and_other_paths() {
    let raw = "GET /healthz HTTP/1.1\r\n";
    let req = parse_request_line(raw).expect("should strip crlf");
    assert_eq!(req.method, Method::Get);
    assert_eq!(req.path, "/healthz");
    assert_eq!(req.version, "HTTP/1.1");
}

use airborne_cot_relay::http::{HttpError, MAX_REQUEST_LINE_BYTES};

#[test]
fn test_oversized_request_line_returns_400() {
    let large = format!("GET /{} HTTP/1.1", "a".repeat(MAX_REQUEST_LINE_BYTES + 1));
    let err = parse_request_line(&large).unwrap_err();
    assert_eq!(err, HttpError::OversizedRequestLine);
    assert_eq!(err.status_code(), 400);
}

#[test]
fn test_malformed_request_lines_return_400() {
    let bad_lines = ["GET /metrics", "GET metrics HTTP/1.1", "GET /metrics FTP/1.0", ""];
    for line in bad_lines {
        let err = parse_request_line(line).unwrap_err();
        assert_eq!(err.status_code(), 400);
    }
}
