use crate::config::RelayError;
use crate::relay::{Backoff, CotSink};
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

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

impl TcpClientConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn with_backoff(mut self, base_ms: u64, max_ms: u64) -> Self {
        self.backoff_base_ms = base_ms;
        self.backoff_max_ms = max_ms;
        self
    }
}

/// Outbound TCP sink that connects on demand with reconnect backoff.
#[derive(Debug)]
pub struct TcpClientSink {
    target_addr: SocketAddr,
    config: TcpClientConfig,
    stream: Option<TcpStream>,
    backoff: Backoff,
    next_connect_deadline: Option<Instant>,
}

impl TcpClientSink {
    pub fn new(target_addr: SocketAddr, config: TcpClientConfig) -> Self {
        let backoff = Backoff::new(config.backoff_base_ms, config.backoff_max_ms);
        Self {
            target_addr,
            config,
            stream: None,
            backoff,
            next_connect_deadline: None,
        }
    }

    pub fn with_defaults(target_addr: SocketAddr) -> Self {
        Self::new(target_addr, TcpClientConfig::default())
    }

    pub fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn backoff(&self) -> &Backoff {
        &self.backoff
    }
}

impl TcpClientSink {
    fn ensure_connected(&mut self) -> Result<&mut TcpStream, RelayError> {
        if self.stream.is_none() {
            let now = Instant::now();
            if let Some(deadline) = self.next_connect_deadline {
                if now < deadline {
                    return Err(RelayError::IoError("Reconnect backoff active".to_string()));
                }
            }
            let stream = match self.config.connect_timeout {
                Some(timeout) => TcpStream::connect_timeout(&self.target_addr, timeout),
                None => TcpStream::connect(self.target_addr),
            };
            match stream {
                Ok(s) => {
                    self.backoff.reset();
                    self.next_connect_deadline = None;
                    self.stream = Some(s);
                }
                Err(err) => {
                    let delay = self.backoff.next_delay_ms();
                    self.next_connect_deadline = Some(now + Duration::from_millis(delay));
                    return Err(RelayError::from(err));
                }
            }
        }
        self.stream
            .as_mut()
            .ok_or_else(|| RelayError::IoError("Not connected".to_string()))
    }
}

impl CotSink for TcpClientSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        let stream = self.ensure_connected()?;
        match stream.write_all(payload) {
            Ok(()) => Ok(payload.len()),
            Err(err) => {
                self.stream = None;
                let delay = self.backoff.next_delay_ms();
                self.next_connect_deadline = Some(Instant::now() + Duration::from_millis(delay));
                Err(RelayError::from(err))
            }
        }
    }
}
