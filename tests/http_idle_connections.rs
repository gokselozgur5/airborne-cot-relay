use airborne_cot_relay::http::server::{HttpServer, Router};
use std::net::TcpStream;
use std::time::{Duration, Instant};

#[test]
fn idle_connections_do_not_stall_poll_step() {
    let mut server = HttpServer::bind("127.0.0.1:0", Router::new()).unwrap();
    let addr = server.local_addr().unwrap();
    let idle: Vec<TcpStream> = (0..20).map(|_| TcpStream::connect(addr).unwrap()).collect();
    let start = Instant::now();
    for _ in 0..20 {
        server.poll_step().unwrap();
    }
    let elapsed = start.elapsed();
    drop(idle);
    assert!(
        elapsed < Duration::from_millis(200),
        "20 idle clients stalled the caller for {elapsed:?}"
    );
}
