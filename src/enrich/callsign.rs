//! Callsign template parser and <contact> element generator.

use std::fmt;

/// Errors when parsing a callsign template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallsignTemplateError {
    UnknownPlaceholder(String),
    UnclosedBrace(usize),
}

impl fmt::Display for CallsignTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlaceholder(p) => write!(f, "unknown placeholder: {{{p}}}"),
            Self::UnclosedBrace(idx) => write!(f, "unclosed placeholder at index {idx}"),
        }
    }
}

impl std::error::Error for CallsignTemplateError {}
