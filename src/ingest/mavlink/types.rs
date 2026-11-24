//! MAVLink v2 message definitions for attitude and global position.

pub const MAVLINK_V2_MAGIC: u8 = 0xFD;
pub const MSG_ID_ATTITUDE: u32 = 30;
pub const MSG_ID_GLOBAL_POSITION_INT: u32 = 33;

pub const CRC_EXTRA_ATTITUDE: u8 = 39;
pub const CRC_EXTRA_GLOBAL_POSITION_INT: u8 = 104;

#[derive(Debug, Clone, PartialEq)]
pub enum MavlinkMessage {
    Attitude {
        time_boot_ms: u32,
        roll: f32,
        pitch: f32,
        yaw: f32,
    },
    GlobalPositionInt {
        time_boot_ms: u32,
        lat: i32,
        lon: i32,
        alt: i32,
        relative_alt: i32,
        vx: i16,
        vy: i16,
        vz: i16,
        hdg: u16,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MavlinkPacket {
    pub seq: u8,
    pub sysid: u8,
    pub compid: u8,
    pub msgid: u32,
    pub message: MavlinkMessage,
}
