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

use airborne_cot_relay::http::{HttpResponse, StatusCode};

#[test]
fn test_http_response_ok_serialization() {
    let resp = HttpResponse::ok("prometheus_metrics 1\n", "text/plain");
    let bytes = resp.to_bytes();
    let text = String::from_utf8(bytes).expect("valid utf8");
    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Type: text/plain\r\n"));
    assert!(text.contains("Content-Length: 21\r\n"));
    assert!(text.ends_with("\r\n\r\nprometheus_metrics 1\n"));
}

#[test]
fn test_http_response_error_helpers() {
    let bad = HttpResponse::bad_request("bad\n");
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let nf = HttpResponse::not_found();
    assert_eq!(nf.status, StatusCode::NOT_FOUND);
    let na = HttpResponse::method_not_allowed();
    assert_eq!(na.status, StatusCode::METHOD_NOT_ALLOWED);
}
