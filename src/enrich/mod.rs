//! Cursor-on-Target detail element builder.

pub mod callsign;

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

    pub(crate) fn write_xml(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.tag);
        for (k, v) in &self.attributes {
            out.push_str(&format!(" {k}=\"{}\"", escape_xml_attr(v)));
        }
        if self.text.is_none() && self.children.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        if let Some(ref text) = self.text {
            out.push_str(&escape_xml_attr(text));
        }
        for child in &self.children {
            child.write_xml(out);
        }
        out.push_str(&format!("</{}>", self.tag));
    }
}

/// Builder for Cursor-on-Target `<detail>` element and child elements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CotDetail {
    elements: Vec<DetailElement>,
}

impl CotDetail {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: DetailElement) -> &mut Self {
        self.elements.push(element);
        self
    }

    pub fn elements(&self) -> &[DetailElement] {
        &self.elements
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Renders the complete <detail>...</detail> block.
    pub fn render(&self) -> String {
        let mut out = String::from("<detail>");
        for el in &self.elements {
            el.write_xml(&mut out);
        }
        out.push_str("</detail>");
        out
    }
}

impl std::fmt::Display for CotDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}
