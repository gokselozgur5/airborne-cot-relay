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
