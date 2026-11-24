//! MAVLink v2 frame encoder helpers.

use super::super::crc::x25_crc_accumulate;
use super::super::types::*;

pub fn encode_mavlink_v2_attitude(
    seq: u8,
    sysid: u8,
    compid: u8,
    time_boot_ms: u32,
    roll: f32,
    pitch: f32,
    yaw: f32,
) -> Vec<u8> {
    let mut payload = [0u8; 28];
    payload[0..4].copy_from_slice(&time_boot_ms.to_le_bytes());
    payload[4..8].copy_from_slice(&roll.to_le_bytes());
    payload[8..12].copy_from_slice(&pitch.to_le_bytes());
    payload[12..16].copy_from_slice(&yaw.to_le_bytes());

    build_mavlink_v2_frame(
        seq,
        sysid,
        compid,
        MSG_ID_ATTITUDE,
        CRC_EXTRA_ATTITUDE,
        &payload,
    )
}

pub fn encode_mavlink_v2_global_position(
    seq: u8,
    sysid: u8,
    compid: u8,
    time_boot_ms: u32,
    lat: i32,
    lon: i32,
    alt: i32,
    relative_alt: i32,
    vx: i16,
    vy: i16,
    vz: i16,
    hdg: u16,
) -> Vec<u8> {
    let mut payload = [0u8; 28];
    payload[0..4].copy_from_slice(&time_boot_ms.to_le_bytes());
    payload[4..8].copy_from_slice(&lat.to_le_bytes());
    payload[8..12].copy_from_slice(&lon.to_le_bytes());
    payload[12..16].copy_from_slice(&alt.to_le_bytes());
    payload[16..20].copy_from_slice(&relative_alt.to_le_bytes());
    payload[20..22].copy_from_slice(&vx.to_le_bytes());
    payload[22..24].copy_from_slice(&vy.to_le_bytes());
    payload[24..26].copy_from_slice(&vz.to_le_bytes());
    payload[26..28].copy_from_slice(&hdg.to_le_bytes());

    build_mavlink_v2_frame(
        seq,
        sysid,
        compid,
        MSG_ID_GLOBAL_POSITION_INT,
        CRC_EXTRA_GLOBAL_POSITION_INT,
        &payload,
    )
}

pub fn build_mavlink_v2_frame(
    seq: u8,
    sysid: u8,
    compid: u8,
    msgid: u32,
    crc_extra: u8,
    payload: &[u8],
) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12 + payload.len());
    frame.push(MAVLINK_V2_MAGIC);
    frame.push(payload.len() as u8);
    frame.push(0);
    frame.push(0);
    frame.push(seq);
    frame.push(sysid);
    frame.push(compid);
    frame.push((msgid & 0xFF) as u8);
    frame.push(((msgid >> 8) & 0xFF) as u8);
    frame.push(((msgid >> 16) & 0xFF) as u8);
    frame.extend_from_slice(payload);

    let mut crc = 0xFFFFu16;
    for &b in &frame[1..] {
        crc = x25_crc_accumulate(b, crc);
    }
    crc = x25_crc_accumulate(crc_extra, crc);
    frame.extend_from_slice(&crc.to_le_bytes());
    frame
}
