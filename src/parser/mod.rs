//! Cypher query string parser.
//!
//! Converts Cypher query strings into the existing AST types
//! ([`Statement`](Statement), [`Clause`](clauses::Clause),
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

mod admin;
mod clauses;
mod conditions;
mod error;
mod expressions;
mod grammar;
mod lexer;
mod patterns;
#[cfg(test)]
mod replay;
mod tokens;
mod validate;

pub use error::ParseError;

use crate::statement::Statement;

/// Parses a Cypher query string into a [`Statement`].
///
/// # Errors
///
/// Returns [`ParseError`] with position and context on invalid input.
pub fn parse(input: &str) -> Result<Statement, ParseError> {
    let tokens = lexer::tokenize(input)?;
    let mut stream = grammar::TokenStream::new(&tokens, input);
    grammar::parse_statement(&mut stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_match_return() {
        let result = parse("MATCH (n) RETURN n");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_empty_input_errors() {
        let result = parse("");
        assert!(result.is_err());
    }
}
