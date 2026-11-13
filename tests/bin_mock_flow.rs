use std::net::UdpSocket;
use std::process::{Command, Output};
use std::time::Duration;

/// Runs the binary in mock mode with CoT output aimed at `addr`.
fn run_mock_to(addr: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .arg("--mock")
        .env("RELAY_COT_ADDR", addr)
        .output()
        .expect("binary runs")
}

/// Drains every datagram already queued on `rx`.
fn drain(rx: &UdpSocket) -> Vec<String> {
    rx.set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let mut buf = [0u8; 4096];
    let mut out = Vec::new();
    while let Ok(n) = rx.recv(&mut buf) {
        out.push(String::from_utf8_lossy(&buf[..n]).into_owned());
    }
    out
}
