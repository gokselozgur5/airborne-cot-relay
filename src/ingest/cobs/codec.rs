//! Zero-dependency Consistent Overhead Byte Stuffing (COBS) encoder and decoder.

use crate::config::RelayError;

pub fn cobs_decode(src: &[u8]) -> Result<Vec<u8>, RelayError> {
    if src.is_empty() {
        return Ok(Vec::new());
    }
    let mut dest = Vec::with_capacity(src.len());
    let mut idx = 0;
    while idx < src.len() {
        let code = src[idx];
        if code == 0 {
            return Err(RelayError::DecodeError(
                "unexpected zero byte in cobs payload".into(),
            ));
        }
        idx += 1;
        let num_data = (code - 1) as usize;
        if idx + num_data > src.len() {
            return Err(RelayError::DecodeError(
                "cobs block points past end of buffer".into(),
            ));
        }
        dest.extend_from_slice(&src[idx..idx + num_data]);
        idx += num_data;
        if code < 0xFF && idx < src.len() {
            dest.push(0);
        }
    }
    Ok(dest)
}

pub fn cobs_encode(src: &[u8]) -> Vec<u8> {
    let mut dest = Vec::with_capacity(src.len() + src.len() / 254 + 1);
    let mut code_idx = 0;
    dest.push(1);
    let mut code = 1u8;

    for &b in src {
        if b == 0 {
            dest[code_idx] = code;
            code_idx = dest.len();
            dest.push(1);
            code = 1;
        } else {
            dest.push(b);
            code += 1;
            if code == 0xFF {
                dest[code_idx] = code;
                code_idx = dest.len();
                dest.push(1);
                code = 1;
            }
        }
    }
    dest[code_idx] = code;
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cobs_roundtrip() {
        let test_cases: Vec<&[u8]> = vec![
            b"",
            b"hello world",
            b"\x00",
            b"\x00\x00",
            b"foo\x00bar",
            b"\x00abc\x00def\x00",
        ];
        for original in test_cases {
            let encoded = cobs_encode(original);
            assert!(!encoded.contains(&0x00));
            let decoded = cobs_decode(&encoded).expect("cobs decodes");
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn test_cobs_corrupted_frame_handling() {
        let invalid = vec![10, 1, 2];
        assert!(cobs_decode(&invalid).is_err());
    }
}
