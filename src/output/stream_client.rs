use crate::config::RelayError;
use crate::relay::{Backoff, CotSink};
use std::io::Write;
use std::time::{Duration, Instant};

/// Configuration for an outbound generic stream client connection.
///
/// Encryption, peer certificate validation, and socket timeouts are the
/// responsibility of the [`StreamConnector`] implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamClientConfig {
    pub endpoint: String,
}

impl StreamClientConfig {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub fn validate(&self) -> Result<(), RelayError> {
        if self.endpoint.trim().is_empty() {
            return Err(RelayError::InvalidConfig("empty stream endpoint".into()));
        }
        Ok(())
    }
}

/// Establishes a stream connection from a validated configuration.
///
/// Transport encryption, certificate validation, and socket timeouts are the
/// responsibility of the connector implementation.
pub trait StreamConnector {
    fn connect(&mut self, cfg: &StreamClientConfig) -> Result<Box<dyn Write + Send>, RelayError>;
}

/// Generic stream client sink that reconnects with exponential backoff.
///
/// Transport encryption, peer authentication, and underlying timeouts are
/// delegated entirely to the supplied [`StreamConnector`].
pub struct StreamClientSink<C: StreamConnector> {
    cfg: StreamClientConfig,
    connector: C,
    stream: Option<Box<dyn Write + Send>>,
    backoff: Backoff,
    last_delay_ms: u64,
    next_connect_deadline: Option<Instant>,
}

impl<C: StreamConnector> StreamClientSink<C> {
    pub fn new(
        cfg: StreamClientConfig,
        connector: C,
        backoff: Backoff,
    ) -> Result<Self, RelayError> {
        cfg.validate()?;
        Ok(Self {
            cfg,
            connector,
            stream: None,
            backoff,
            last_delay_ms: 0,
            next_connect_deadline: None,
        })
    }

    /// Delay the caller should wait before the next send after a failure.
    pub fn retry_delay_ms(&self) -> u64 {
        self.last_delay_ms
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

impl<C: StreamConnector> CotSink for StreamClientSink<C> {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        if self.stream.is_none() {
            let now = Instant::now();
            if let Some(deadline) = self.next_connect_deadline {
                if now < deadline {
                    return Err(RelayError::IoError("Reconnect backoff active".to_string()));
                }
            }
            match self.connector.connect(&self.cfg) {
                Ok(s) => {
                    self.stream = Some(s);
                }
                Err(e) => {
                    let delay = self.backoff.next_delay_ms();
                    self.last_delay_ms = delay;
                    self.next_connect_deadline = Some(now + Duration::from_millis(delay));
                    return Err(e);
                }
            }
        }
        let stream = self.stream.as_mut().expect("connected");
        match stream.write_all(payload).and_then(|_| stream.flush()) {
            Ok(()) => {
                self.backoff.reset();
                self.last_delay_ms = 0;
                self.next_connect_deadline = None;
                Ok(payload.len())
            }
            Err(e) => {
                self.stream = None;
                let delay = self.backoff.next_delay_ms();
                self.last_delay_ms = delay;
                self.next_connect_deadline = Some(Instant::now() + Duration::from_millis(delay));
                Err(RelayError::from(e))
            }
        }
    }
}
