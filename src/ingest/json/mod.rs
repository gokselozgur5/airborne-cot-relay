//! JSON-lines detection parsing and UDP ingestion.

pub mod parser;
pub mod source;

pub use parser::{extract_field_f64, extract_field_str, parse_json_detection};
pub use source::JsonUdpDetectionSource;
