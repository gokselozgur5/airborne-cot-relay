//! MAVLink v2 frame parser and decoder.

use super::super::crc::x25_crc_accumulate;
use super::super::types::*;
use crate::config::RelayError;

pub fn decode_mavlink_v2_frame(frame: &[u8]) -> Result<MavlinkPacket, RelayError> {
    if frame.len() < 12 {
        return Err(RelayError::DecodeError("mavlink frame too short".into()));
    }
    if frame[0] != MAVLINK_V2_MAGIC {
        return Err(RelayError::DecodeError(format!(
            "invalid mavlink magic 0x{:02X}, expected 0xFD",
            frame[0]
        )));
    }
    let payload_len = frame[1] as usize;
    let incompat_flags = frame[2];
    let is_signed = (incompat_flags & 0x01) != 0;
    let signature_len = if is_signed { 13 } else { 0 };
    let expected_len = 10 + payload_len + 2 + signature_len;
    if frame.len() < expected_len {
        return Err(RelayError::DecodeError(format!(
            "frame len {} shorter than expected {}",
            frame.len(),
            expected_len
        )));
    }

    let seq = frame[4];
    let sysid = frame[5];
    let compid = frame[6];
    let msgid = (frame[7] as u32) | ((frame[8] as u32) << 8) | ((frame[9] as u32) << 16);

    let crc_extra = match msgid {
        MSG_ID_ATTITUDE => CRC_EXTRA_ATTITUDE,
        MSG_ID_GLOBAL_POSITION_INT => CRC_EXTRA_GLOBAL_POSITION_INT,
        _ => {
            return Err(RelayError::DecodeError(format!(
                "unsupported msgid {msgid}"
            )))
        }
    };

    let mut crc = 0xFFFFu16;
    for &b in &frame[1..10 + payload_len] {
        crc = x25_crc_accumulate(b, crc);
    }
    crc = x25_crc_accumulate(crc_extra, crc);

    let rx_crc = (frame[10 + payload_len] as u16) | ((frame[10 + payload_len + 1] as u16) << 8);
    if crc != rx_crc {
        return Err(RelayError::DecodeError(format!(
            "crc mismatch: computed 0x{crc:04X}, received 0x{rx_crc:04X}"
        )));
    }

    let payload = &frame[10..10 + payload_len];
    let message = match msgid {
        MSG_ID_ATTITUDE => {
            if payload.len() > 28 {
                return Err(RelayError::DecodeError(
                    "attitude payload exceeds 28 bytes".into(),
                ));
            }
            let mut padded = [0u8; 28];
            padded[..payload.len()].copy_from_slice(payload);
            let time_boot_ms = u32::from_le_bytes(padded[0..4].try_into().unwrap());
            let roll = f32::from_le_bytes(padded[4..8].try_into().unwrap());
            let pitch = f32::from_le_bytes(padded[8..12].try_into().unwrap());
            let yaw = f32::from_le_bytes(padded[12..16].try_into().unwrap());
            MavlinkMessage::Attitude {
                time_boot_ms,
                roll,
                pitch,
                yaw,
            }
        }
        MSG_ID_GLOBAL_POSITION_INT => {
            if payload.len() > 28 {
                return Err(RelayError::DecodeError(
                    "global_position_int payload exceeds 28 bytes".into(),
                ));
            }
            let mut padded = [0u8; 28];
            padded[..payload.len()].copy_from_slice(payload);
            let time_boot_ms = u32::from_le_bytes(padded[0..4].try_into().unwrap());
            let lat = i32::from_le_bytes(padded[4..8].try_into().unwrap());
            let lon = i32::from_le_bytes(padded[8..12].try_into().unwrap());
            let alt = i32::from_le_bytes(padded[12..16].try_into().unwrap());
            let relative_alt = i32::from_le_bytes(padded[16..20].try_into().unwrap());
            let vx = i16::from_le_bytes(padded[20..22].try_into().unwrap());
            let vy = i16::from_le_bytes(padded[22..24].try_into().unwrap());
            let vz = i16::from_le_bytes(padded[24..26].try_into().unwrap());
            let hdg = u16::from_le_bytes(padded[26..28].try_into().unwrap());
            MavlinkMessage::GlobalPositionInt {
                time_boot_ms,
                lat,
                lon,
                alt,
                relative_alt,
                vx,
                vy,
                vz,
                hdg,
            }
        }
        _ => unreachable!(),
    };

    Ok(MavlinkPacket {
        seq,
        sysid,
        compid,
        msgid,
        message,
    })
}
