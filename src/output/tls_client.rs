use crate::config::RelayError;
use crate::relay::{Backoff, CotSink};
use std::io::Write;

/// Configuration for an outbound TLS client connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsClientConfig {
    pub server_name: String,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_peer: bool,
    pub connect_timeout_ms: u64,
    pub write_timeout_ms: u64,
}

impl TlsClientConfig {
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            verify_peer: true,
            connect_timeout_ms: 5_000,
            write_timeout_ms: 5_000,
        }
    }

    pub fn with_ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    pub fn with_client_identity(mut self, cert: impl Into<String>, key: impl Into<String>) -> Self {
        self.client_cert_path = Some(cert.into());
        self.client_key_path = Some(key.into());
        self
    }

    pub fn with_verify_peer(mut self, verify: bool) -> Self {
        self.verify_peer = verify;
        self
    }

    pub fn validate(&self) -> Result<(), RelayError> {
        if self.server_name.is_empty() {
            return Err(RelayError::InvalidConfig("empty TLS server name".into()));
        }
        if self.client_cert_path.is_some() != self.client_key_path.is_some() {
            return Err(RelayError::InvalidConfig(
                "client cert and key must be set together".into(),
            ));
        }
        Ok(())
    }
}

/// Establishes an encrypted stream from a validated configuration.
pub trait TlsConnector {
    fn connect(&mut self, cfg: &TlsClientConfig) -> Result<Box<dyn Write + Send>, RelayError>;
}

/// TLS client sink that reconnects with exponential backoff.
pub struct TlsClientSink<C: TlsConnector> {
    cfg: TlsClientConfig,
    connector: C,
    stream: Option<Box<dyn Write + Send>>,
    backoff: Backoff,
    last_delay_ms: u64,
}

impl<C: TlsConnector> TlsClientSink<C> {
    pub fn new(cfg: TlsClientConfig, connector: C, backoff: Backoff) -> Result<Self, RelayError> {
        cfg.validate()?;
        Ok(Self { cfg, connector, stream: None, backoff, last_delay_ms: 0 })
    }

    /// Delay the caller should wait before the next send after a failure.
    pub fn retry_delay_ms(&self) -> u64 {
        self.last_delay_ms
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}
