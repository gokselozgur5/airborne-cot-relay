//! Builds the CoT `<remarks>` element with MGRS, sensor id and confidence.

use sightline_cot::cot::escape_xml_attr;
use sightline_cot::{format_mgrs, GridError};

/// Returns a `<remarks>` XML element for a target position.
pub fn build_remarks(
    lat_deg: f64,
    lon_deg: f64,
    sensor_id: Option<&str>,
    confidence: Option<f32>,
) -> Result<String, GridError> {
    let mgrs = format_mgrs(lat_deg, lon_deg, 5)?;
    let mut text = format!("mgrs={}", mgrs.as_str());
    if let Some(s) = sensor_id {
        text.push_str(&format!(" sensor={s}"));
    }
    if let Some(c) = confidence {
        text.push_str(&format!(" conf={c:.2}"));
    }
    Ok(format!("<remarks>{}</remarks>", escape_xml_attr(&text)))
}
