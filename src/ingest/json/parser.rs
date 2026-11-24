//! Zero-dependency JSON-lines parser for target detections.

use crate::config::RelayError;
use crate::targeter::TargetDetection;

pub fn extract_field_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pattern1 = format!("\"{}\":\"", key);
    let pattern2 = format!("\"{}\": \"", key);

    let start = if let Some(idx) = json.find(&pattern1) {
        idx + pattern1.len()
    } else if let Some(idx) = json.find(&pattern2) {
        idx + pattern2.len()
    } else {
        return None;
    };

    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub fn extract_field_f64(json: &str, key: &str) -> Option<f64> {
    let p1 = format!("\"{}\":", key);
    let p2 = format!("\"{}\": ", key);

    let start = if let Some(idx) = json.find(&p1) {
        idx + p1.len()
    } else if let Some(idx) = json.find(&p2) {
        idx + p2.len()
    } else {
        return None;
    };

    let rest = json[start..].trim_start();
    let end = rest
        .find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().ok()
}

pub fn parse_json_detection(line: &str) -> Result<TargetDetection, RelayError> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(RelayError::DecodeError(
            "json object must start with { and end with }".into(),
        ));
    }

    let u = extract_field_f64(trimmed, "u")
        .ok_or_else(|| RelayError::DecodeError("missing u".into()))?;
    let v = extract_field_f64(trimmed, "v")
        .ok_or_else(|| RelayError::DecodeError("missing v".into()))?;

    let x1 = extract_field_f64(trimmed, "x1")
        .or_else(|| extract_field_f64(trimmed, "x"))
        .ok_or_else(|| RelayError::DecodeError("missing x1/x".into()))?;
    let y1 = extract_field_f64(trimmed, "y1")
        .or_else(|| extract_field_f64(trimmed, "y"))
        .ok_or_else(|| RelayError::DecodeError("missing y1/y".into()))?;

    let x2 = extract_field_f64(trimmed, "x2")
        .or_else(|| extract_field_f64(trimmed, "w").map(|w| x1 + w))
        .ok_or_else(|| RelayError::DecodeError("missing x2/w".into()))?;
    let y2 = extract_field_f64(trimmed, "y2")
        .or_else(|| extract_field_f64(trimmed, "h").map(|h| y1 + h))
        .ok_or_else(|| RelayError::DecodeError("missing y2/h".into()))?;

    let class = extract_field_str(trimmed, "class")
        .unwrap_or("unknown")
        .to_string();

    let conf = extract_field_f64(trimmed, "conf")
        .or_else(|| extract_field_f64(trimmed, "confidence"))
        .unwrap_or(1.0) as f32;

    Ok(TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (x1, y1, x2, y2),
        classification: class,
        confidence: conf,
        embedding: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_detection_valid() {
        let line = r#"{"u": 640.0, "v": 360.0, "x1": 600.0, "y1": 320.0, "x2": 680.0, "y2": 400.0, "class": "vehicle", "conf": 0.92}"#;
        let det = parse_json_detection(line).expect("parses json detection");
        assert_eq!(det.pixel_u, 640.0);
        assert_eq!(det.pixel_v, 360.0);
        assert_eq!(det.bbox, (600.0, 320.0, 680.0, 400.0));
        assert_eq!(det.classification, "vehicle");
        assert!((det.confidence - 0.92).abs() < 1e-4);
    }

    #[test]
    fn test_parse_json_detection_alt_keys() {
        let line = r#"{"u":100,"v":200,"x":90,"y":190,"w":20,"h":20,"class":"drone"}"#;
        let det = parse_json_detection(line).expect("parses with x,y,w,h");
        assert_eq!(det.pixel_u, 100.0);
        assert_eq!(det.pixel_v, 200.0);
        assert_eq!(det.bbox, (90.0, 190.0, 110.0, 210.0));
        assert_eq!(det.classification, "drone");
        assert_eq!(det.confidence, 1.0);
    }
}
