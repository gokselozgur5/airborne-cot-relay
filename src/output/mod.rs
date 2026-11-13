//! Outbound sinks delivering CoT events to network destinations.

pub mod tcp_client;
pub mod udp_broadcast;
pub mod udp_multicast;

pub use tcp_client::TcpClientConfig;
pub use udp_broadcast::{UdpBroadcastConfig, UdpBroadcastSink};
pub use udp_multicast::{UdpMulticastConfig, UdpMulticastSink};
