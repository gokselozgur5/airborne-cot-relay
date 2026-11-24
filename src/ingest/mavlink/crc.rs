//! Zero-dependency MAVLink v2 CRC-16 (X.25/MCRF4XX) implementation.

pub fn x25_crc_accumulate(byte: u8, mut crc: u16) -> u16 {
    let mut tmp = byte ^ (crc as u8);
    tmp ^= tmp << 4;
    crc = (crc >> 8) ^ ((tmp as u16) << 8) ^ ((tmp as u16) << 3) ^ ((tmp as u16) >> 4);
    crc
}

pub fn x25_crc_calculate(buffer: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &b in buffer {
        crc = x25_crc_accumulate(b, crc);
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_crc_calculation() {
        let test_data = b"123456789";
        let crc = x25_crc_calculate(test_data);
        assert_eq!(crc, 0x6F91);
    }
}
