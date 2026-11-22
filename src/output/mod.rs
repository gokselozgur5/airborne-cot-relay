//! Outbound sinks delivering CoT events to network destinations.

pub mod enrich;
pub mod stream_client;
pub mod tcp_client;
pub mod udp_broadcast;
pub mod udp_multicast;

pub use enrich::{build_remarks, CotDetail};
pub use stream_client::{StreamClientConfig, StreamClientSink, StreamConnector};
pub use tcp_client::{TcpClientConfig, TcpClientSink};
pub use udp_broadcast::UdpBroadcastSink;
pub use udp_multicast::{UdpMulticastConfig, UdpMulticastSink};
