use airborne_cot_relay::http::server::{HttpServer, RouteHandler, Router};
use airborne_cot_relay::http::{HttpResponse, Method, StatusCode};
use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
fn test_router_dispatch_get_success() {
    let mut router = Router::new();
    router.add_route("/healthz", |_req| HttpResponse::ok("healthy\n", "text/plain"));
    let mut server = HttpServer::bind("127.0.0.1:0", router).unwrap();
    let addr = server.local_addr().unwrap();

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.write_all(b"GET /healthz HTTP/1.1\r\n\r\n").unwrap();

    let handled = server.poll_step().unwrap();
    assert!(handled);

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).unwrap();
    let res = String::from_utf8_lossy(&buf[..n]);
    assert!(res.contains("200 OK"));
    assert!(res.contains("healthy\n"));
}

#[test]
fn test_router_dispatch_404_not_found() {
    let router = Router::new();
    let mut server = HttpServer::bind("127.0.0.1:0", router).unwrap();
    let addr = server.local_addr().unwrap();

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.write_all(b"GET /unknown HTTP/1.1\r\n\r\n").unwrap();

    let handled = server.poll_step().unwrap();
    assert!(handled);

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).unwrap();
    let res = String::from_utf8_lossy(&buf[..n]);
    assert!(res.contains("404 Not Found"));
}
