//! Cursor-on-Target detail element builder.

pub mod callsign;

pub use callsign::{CallsignTemplate, CallsignTemplateError};

/// Single child XML element inside a `<detail>` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailElement {
    tag: String,
    attributes: Vec<(String, String)>,
}
