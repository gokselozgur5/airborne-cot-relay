use airborne_cot_relay::http::server::{HttpServer, Router};
use airborne_cot_relay::http::HttpResponse;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

fn status_line(server: &mut HttpServer, request: &str) -> String {
    let mut client = TcpStream::connect(server.local_addr().unwrap()).unwrap();
    client.write_all(request.as_bytes()).unwrap();
    sleep(Duration::from_millis(10));
    server.poll_step().unwrap();
    let mut response = String::new();
    let _ = client.read_to_string(&mut response);
    response.lines().next().unwrap_or("").to_string()
}

#[test]
fn query_string_and_head_reach_the_route() {
    let mut router = Router::new();
    router.add_route("/healthz", |_| HttpResponse::ok("ok\n", "text/plain"));
    let mut server = HttpServer::bind("127.0.0.1:0", router).unwrap();
    let with_query = status_line(&mut server, "GET /healthz?probe=1 HTTP/1.1\r\n\r\n");
    let head = status_line(&mut server, "HEAD /healthz HTTP/1.1\r\n\r\n");
    assert_eq!(
        (with_query.as_str(), head.as_str()),
        ("HTTP/1.1 200 OK", "HTTP/1.1 200 OK")
    );
}
