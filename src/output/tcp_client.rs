use crate::config::RelayError;
use crate::relay::{Backoff, CotSink};
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Configuration options for the TCP client sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpClientConfig {
    pub connect_timeout: Option<Duration>,
    pub backoff_base_ms: u64,
    pub backoff_max_ms: u64,
}

impl Default for TcpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Some(Duration::from_millis(500)),
            backoff_base_ms: 100,
            backoff_max_ms: 5000,
        }
    }
}
