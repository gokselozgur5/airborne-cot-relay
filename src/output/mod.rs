//! Outbound sinks delivering CoT events to network destinations.

pub mod tls_client;
pub mod udp_multicast;

pub use tls_client::{TlsClientConfig, TlsClientSink, TlsConnector};
pub use udp_multicast::{UdpMulticastConfig, UdpMulticastSink};
