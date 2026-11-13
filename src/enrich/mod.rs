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

    /// Appends an attribute with name and value.
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((name.into(), value.into()));
        self
    }

    /// Sets inner text content for the element.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Appends a child detail element.
    pub fn with_child(mut self, child: DetailElement) -> Self {
        self.children.push(child);
        self
    }

    /// Returns the element tag name.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the element attributes.
    pub fn attributes(&self) -> &[(String, String)] {
        &self.attributes
    }

}
