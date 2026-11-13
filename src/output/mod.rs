//! Outbound sinks delivering CoT events to network destinations.

pub mod udp_broadcast;
pub mod udp_multicast;

pub use udp_broadcast::UdpBroadcastConfig;
pub use udp_multicast::{UdpMulticastConfig, UdpMulticastSink};
