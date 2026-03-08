//! Renderers that convert AST types to Cypher query strings.
//!
//! The [`default`] renderer produces single-line output. The [`pretty`]
//! renderer produces indented, multi-line output with configurable
//! indent string.

pub mod default;
pub mod pretty;

use std::borrow::Cow;

/// Configuration for rendering.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Whether to backtick-escape identifiers.
    pub escape_names: EscapeMode,
    /// Whether to pretty-print with indentation and newlines.
    pub pretty_print: bool,
    /// Indentation string (default: two spaces).
    pub indent: Cow<'static, str>,
}

/// Controls when identifiers are backtick-escaped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeMode {
    /// Always backtick-escape labels, types, and property names.
    Always,
    /// Only escape names that contain special characters or are reserved words.
    AsNeeded,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            escape_names: EscapeMode::Always,
            pretty_print: false,
            indent: Cow::Borrowed("  "),
        }
    }
}
