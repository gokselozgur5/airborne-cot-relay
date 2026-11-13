//! Cursor-on-Target detail element builder.

use sightline_cot::cot::escape_xml_attr;

/// Single child XML element inside a `<detail>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailElement {
    tag: String,
    attributes: Vec<(String, String)>,
    text: Option<String>,
    children: Vec<DetailElement>,
}

impl DetailElement {
    /// Creates a new detail element with given tag.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
            text: None,
            children: Vec::new(),
        }
    }
}
