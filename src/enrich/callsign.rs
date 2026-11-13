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
    /// Returns the raw template string.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Parses a callsign template string with validation.
    pub fn parse(template: &str) -> Result<Self, CallsignTemplateError> {
        let mut tokens = Vec::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            if start > 0 {
                tokens.push(TemplateToken::Literal(rest[..start].to_string()));
            }
            let after = &rest[start + 1..];
            let end = after.find('}').ok_or(CallsignTemplateError::UnclosedBrace(start))?;
            let name = &after[..end];
            let token = match name {
                "prefix" => TemplateToken::Prefix,
                "class" => TemplateToken::Class,
                "id" => TemplateToken::Id,
                _ => return Err(CallsignTemplateError::UnknownPlaceholder(name.to_string())),
            };
            tokens.push(token);
            rest = &after[end + 1..];
        }
        if !rest.is_empty() {
            tokens.push(TemplateToken::Literal(rest.to_string()));
        }
        Ok(Self { tokens, raw: template.to_string() })
    }
}
