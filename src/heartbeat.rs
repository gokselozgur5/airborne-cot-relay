//! Airborne telemetry heartbeat generator with platform status and radio metrics.

use crate::PlatformPose;
use sightline_cot::cot::{escape_xml_attr, format_iso8601_utc};

/// Operational state of the airborne platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlatformOperationalState {
    #[default]
    Nominal,
    Degraded,
    Emergency,
    Standby,
    Fault,
}
