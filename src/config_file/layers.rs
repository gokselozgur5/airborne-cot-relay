//! Layered configuration precedence and field-level provenance.

/// Provenance of a configuration field value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provenance {
    Default,
    File,
    Env,
    Cli,
}
