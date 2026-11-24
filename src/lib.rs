pub mod app;
pub mod bearing;
pub mod clock;
pub mod config;
pub mod config_file;
pub mod enrich;
pub mod error;
pub mod health;
pub mod http;
pub mod ingest;
pub mod log;
pub mod metrics;
pub mod output;
pub mod pipeline;
pub mod relay;
pub mod resilience;
pub mod runtime;
pub mod targeter;
pub mod telemetry;
pub mod track_filter;
pub mod xml_sanitize;

pub use config::{CameraIntrinsics, RelayConfig, RelayError};
pub use pipeline::Pipeline;
pub use relay::{classification_to_cot_type, CotStreamer, TrackSummary};
pub use targeter::{GroundIntersection, TargetDetection, Targeter, TrackedObject};
pub use telemetry::{PlatformPose, TelemetryIngest};

#[cfg(feature = "experimental")]
#[path = "experimental_a.rs"]
pub mod experimental_a;
#[cfg(feature = "experimental")]
pub use experimental_a::*;

#[cfg(feature = "experimental")]
#[path = "experimental_b.rs"]
pub mod experimental_b;
#[cfg(feature = "experimental")]
pub use experimental_b::*;
