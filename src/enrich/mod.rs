//! Cursor-on-Target detail element builder.

pub mod callsign;

pub use callsign::{CallsignTemplate, CallsignTemplateError};

/// Single child XML element inside a `<detail>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailElement {
    tag: String,
    attributes: Vec<(String, String)>,
}

use sightline_cot::cot::escape_xml_attr;

impl DetailElement {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
        }
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((name.into(), value.into()));
        self
    }
}

impl DetailElement {
    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn attributes(&self) -> &[(String, String)] {
        &self.attributes
    }

    pub fn to_xml(&self) -> String {
        let mut out = format!("<{}", self.tag);
        for (k, v) in &self.attributes {
            out.push_str(&format!(" {k}=\"{}\"", escape_xml_attr(v)));
        }
        out.push_str("/>");
        out
    }
}
