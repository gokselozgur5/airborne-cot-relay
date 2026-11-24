//! MAVLink v2 protocol support and telemetry source.

pub mod crc;
pub mod frame;
pub mod source;
pub mod synth;
pub mod types;

pub use crc::{x25_crc_accumulate, x25_crc_calculate};
pub use frame::{
    decode_mavlink_v2_frame, encode_mavlink_v2_attitude, encode_mavlink_v2_global_position,
};
pub use source::MavlinkUdpSource;
pub use synth::MavlinkPoseSynthesizer;
pub use types::{
    MavlinkMessage, MavlinkPacket, CRC_EXTRA_ATTITUDE, CRC_EXTRA_GLOBAL_POSITION_INT,
    MAVLINK_V2_MAGIC, MSG_ID_ATTITUDE, MSG_ID_GLOBAL_POSITION_INT,
};
