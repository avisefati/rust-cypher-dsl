//! Top-level grammar and token stream for parsing Cypher statements.

#![allow(dead_code, reason = "parser functions will be used incrementally")]
#![allow(clippy::too_many_lines, reason = "parser logic requires comprehensive match coverage")]

use super::clauses::parse_clauses;
use super::error::ParseError;
use super::lexer::SpannedToken;
use super::tokens::{Keyword, Token};
use super::validate::validate_clause_ordering;
use crate::statement::{SinglePartQuery, Statement};

/// A token stream wrapper that maintains position for parsing.
pub struct TokenStream<'input, 'tokens> {
    /// The token array.
    tokens: &'tokens [SpannedToken<'input>],
    /// Current position in the token array.
    pos: usize,
    /// Original input string (for error reporting).
    input: &'input str,
}

impl<'input, 'tokens> TokenStream<'input, 'tokens> {
    /// Creates a new token stream from tokens and input.
    pub const fn new(tokens: &'tokens [SpannedToken<'input>], input: &'input str) -> Self {
        Self {
            tokens,
            pos: 0,
            input,
        }
    }

    /// Peeks at the current token without advancing.
    pub fn peek(&self) -> Option<&Token<'input>> {
        self.tokens.get(self.pos).map(|st| &st.token)
    }

    /// Advances and returns the current token.
    pub fn advance(&mut self) -> Option<&'tokens SpannedToken<'input>> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    /// Returns true if the current token matches the given keyword.
    pub fn at_keyword(&self, kw: Keyword) -> bool {
        matches!(self.peek(), Some(Token::Keyword(k)) if *k == kw)
    }

    /// Returns true if the current token matches the given token type.
    pub fn at_token(&self, tok: &Token<'_>) -> bool {
        self.peek().is_some_and(|current| std::mem::discriminant(current) == std::mem::discriminant(tok))
    }

    /// Expects and consumes the given keyword, or returns an error.
    pub fn expect_keyword(&mut self, kw: Keyword) -> Result<(), ParseError> {
        if self.at_keyword(kw) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(
                vec![format!("{kw}")],
                vec![format!("expected keyword {kw}")],
            ))
        }
    }

    /// Expects a token that matches the given pattern (by discriminant), or returns an error.
    pub fn expect_token(&mut self, expected: &Token<'_>) -> Result<(), ParseError> {
        if self.at_token(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(
                vec![format!("{expected}")],
                vec![format!("expected {expected}")],
            ))
        }
    }

    /// Returns the current byte offset (for error reporting).
    pub fn offset(&self) -> usize {
        self.tokens.get(self.pos).map_or_else(
            || self.input.len(),
            |st| st.offset,
        )
    }

    /// Returns true if there are no more tokens.
    pub const fn is_empty(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Creates a parse error at the current position.
    pub fn error(&self, expected: Vec<String>, context: Vec<String>) -> ParseError {
        ParseError::from_offset(self.input, self.offset(), expected, context)
    }
}

/// Parses a complete Cypher statement from a token stream.
pub fn parse_statement(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    // Check for EXPLAIN or PROFILE prefix
    let is_explain = if stream.at_keyword(Keyword::Explain) {
        stream.advance();
        true
    } else {
        false
    };

    let is_profile = if stream.at_keyword(Keyword::Profile) {
        stream.advance();
        true
    } else {
        false
    };

    // Parse the main statement
    let mut stmt = parse_single_part_or_union(stream)?;

    // Wrap in EXPLAIN/PROFILE if needed
    if is_explain {
        stmt = stmt.explain();
    }
    if is_profile {
        stmt = stmt.profile();
    }

    Ok(stmt)
}

/// Parses a single-part query or UNION composition.
fn parse_single_part_or_union(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    let mut left = parse_single_part_query(stream)?;

    while stream.at_keyword(Keyword::Union) {
        stream.advance();
        let is_all = if stream.at_keyword(Keyword::All) {
            stream.advance();
            true
        } else {
            false
        };

        let right = parse_single_part_query(stream)?;

        left = if is_all {
            left.union_all(right)
        } else {
            left.union(right)
        };
    }

    Ok(left)
}

/// Parses a single-part query (a sequence of clauses).
///
/// After syntactic parsing, runs clause ordering validation to ensure
/// the clause sequence follows legal Cypher ordering rules.
fn parse_single_part_query(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    let clauses = parse_clauses(stream)?;
    validate_clause_ordering(&clauses)?;
    Ok(Statement::SinglePart(SinglePartQuery::new(clauses)))
}
