//! Outbound sinks delivering CoT events to network destinations.

pub mod udp_multicast;

pub use udp_multicast::UdpMulticastConfig;
