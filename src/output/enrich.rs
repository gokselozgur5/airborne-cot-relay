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

/// Builder for arbitrary CoT `<detail>` child elements with stable attribute ordering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CotDetail {
    tag: String,
    attributes: Vec<(String, String)>,
    text: Option<String>,
}

impl CotDetail {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into(), attributes: Vec::new(), text: None }
    }
}

impl CotDetail {
    pub fn with_attr(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.attributes.push((key.into(), val.into()));
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl CotDetail {
    pub fn to_xml(&self) -> String {
        let mut out = format!("<{}", self.tag);
        for (k, v) in &self.attributes {
            out.push_str(&format!(" {k}=\"{}\"", escape_xml_attr(v)));
        }
        if let Some(t) = &self.text {
            out.push('>');
            out.push_str(&escape_xml_attr(t));
            out.push_str(&format!("</{}>", self.tag));
        } else {
            out.push_str("/>");
        }
        out
    }
}
