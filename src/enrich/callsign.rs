//! Callsign template parser and <contact> element generator.

use crate::enrich::DetailElement;
use std::fmt;

/// Errors when parsing a callsign template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallsignTemplateError {
    UnknownPlaceholder(String),
    UnclosedBrace,
}

impl fmt::Display for CallsignTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlaceholder(p) => write!(f, "unknown placeholder: {{{p}}}"),
            Self::UnclosedBrace => write!(f, "unclosed placeholder brace"),
        }
    }
}

impl std::error::Error for CallsignTemplateError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateToken {
    Literal(String),
    Prefix,
    Class,
    Id,
}

/// Callsign template supporting {prefix}, {class}, and {id} placeholders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallsignTemplate {
    pub(crate) tokens: Vec<TemplateToken>,
    pub(crate) raw: String,
}

impl CallsignTemplate {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    fn parse_token(name: &str) -> Result<TemplateToken, CallsignTemplateError> {
        match name {
            "prefix" => Ok(TemplateToken::Prefix),
            "class" => Ok(TemplateToken::Class),
            "id" => Ok(TemplateToken::Id),
            _ => Err(CallsignTemplateError::UnknownPlaceholder(name.to_string())),
        }
    }
}
