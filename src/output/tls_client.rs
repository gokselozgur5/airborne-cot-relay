use crate::config::RelayError;

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
}
