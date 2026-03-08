//! `Cypher` entry point for constructing statements.
//!
//! Provides static methods that return builder types for fluent query construction.
//! Each method transitions into the appropriate typestate builder.

use crate::builder::OngoingMatch;
use crate::clauses::{Clause, MatchClause};
use crate::types::pattern::IntoPattern;

/// Entry point for building Cypher statements.
///
/// Use the associated functions to begin constructing a query:
/// ```ignore
/// let stmt = Cypher::match_node(node("Person").named("n"))
///     .returning(Expression::symbolic_name("n"))
///     .build();
/// ```
#[derive(Debug)]
pub struct Cypher;

impl Cypher {
    /// Begins a `MATCH` query with the given pattern.
    pub fn match_node(pattern: impl IntoPattern) -> OngoingMatch {
        OngoingMatch::new(vec![Clause::Match(MatchClause::new(
            pattern.into_pattern(),
        ))])
    }

    /// Begins an `OPTIONAL MATCH` query with the given pattern.
    pub fn optional_match(pattern: impl IntoPattern) -> OngoingMatch {
        OngoingMatch::new(vec![Clause::Match(MatchClause::optional(
            pattern.into_pattern(),
        ))])
    }
}
