//! Cypher query string parser.
//!
//! Converts Cypher query strings into the existing AST types
//! ([`Statement`](crate::statement::Statement), [`Clause`](crate::clauses::Clause),
//! [`Expression`](crate::types::expression::Expression), etc.).
//!
//! This module is gated behind the `parser` cargo feature flag.
//!
//! # Example
//!
//! ```rust,no_run
//! use rust_cypher_dsl::parser::parse;
//!
//! let stmt = parse("MATCH (n) RETURN n").unwrap();
//! assert_eq!(stmt.render(), "MATCH (n) RETURN n");
//! ```

mod clauses;
mod conditions;
mod error;
mod expressions;
mod grammar;
mod lexer;
mod patterns;
mod tokens;

pub use error::ParseError;

use crate::statement::Statement;

/// Parses a Cypher query string into a [`Statement`].
///
/// # Errors
///
/// Returns [`ParseError`] with position and context on invalid input.
pub fn parse(input: &str) -> Result<Statement, ParseError> {
    let _ = input;
    Err(ParseError::not_yet_implemented())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_returns_error_for_empty_input() {
        let result = parse("");
        assert!(result.is_err());
    }
}
