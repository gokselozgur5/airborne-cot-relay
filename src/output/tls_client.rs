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
