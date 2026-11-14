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

impl CallsignTemplate {
    pub fn parse(template: &str) -> Result<Self, CallsignTemplateError> {
        let mut tokens = Vec::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            if start > 0 {
                tokens.push(TemplateToken::Literal(rest[..start].to_string()));
            }
            let after = &rest[start + 1..];
            let end = after
                .find('}')
                .ok_or(CallsignTemplateError::UnclosedBrace)?;
            tokens.push(Self::parse_token(&after[..end])?);
            rest = &after[end + 1..];
        }
        if !rest.is_empty() {
            tokens.push(TemplateToken::Literal(rest.to_string()));
        }
        Ok(Self {
            tokens,
            raw: template.to_string(),
        })
    }
}

impl CallsignTemplate {
    pub fn format(&self, prefix: &str, class: &str, id: u64) -> String {
        let mut out = String::new();
        for token in &self.tokens {
            match token {
                TemplateToken::Literal(s) => out.push_str(s),
                TemplateToken::Prefix => out.push_str(prefix),
                TemplateToken::Class => out.push_str(class),
                TemplateToken::Id => out.push_str(&id.to_string()),
            }
        }
        out
    }
}

impl CallsignTemplate {
    pub fn build_contact(&self, prefix: &str, class: &str, id: u64) -> DetailElement {
        DetailElement::new("contact").attr("callsign", self.format(prefix, class, id))
    }
}

impl Default for CallsignTemplate {
    fn default() -> Self {
        Self::parse("{prefix}-{class}-{id}").expect("valid default template")
    }
}
