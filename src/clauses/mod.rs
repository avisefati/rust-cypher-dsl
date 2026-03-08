//! Cypher clause types: MATCH, RETURN, CREATE, MERGE, SET, DELETE, etc.
//!
//! Each clause represents a single segment of a Cypher query.
//! Clauses are composed into a [`SinglePartQuery`](crate::statement::SinglePartQuery)
//! to form a complete statement.

use crate::types::condition::Condition;
use crate::types::expression::Expression;
use crate::types::pattern::Pattern;

/// A single clause in a Cypher query.
#[derive(Debug, Clone, PartialEq)]
pub enum Clause {
    /// `MATCH pattern` or `OPTIONAL MATCH pattern`.
    Match(MatchClause),
    /// `WHERE condition`.
    Where(WhereClause),
    /// `RETURN expr1, expr2, ...` with optional DISTINCT.
    Return(ReturnClause),
}

/// A MATCH or OPTIONAL MATCH clause.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchClause {
    /// Whether this is an OPTIONAL MATCH.
    pub(crate) optional: bool,
    /// The pattern to match.
    pub(crate) pattern: Pattern,
}

/// A WHERE clause.
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    /// The filter condition.
    pub(crate) condition: Condition,
}

/// A RETURN clause.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    /// Whether to return distinct results.
    pub(crate) distinct: bool,
    /// The expressions to return.
    pub(crate) expressions: Vec<Expression>,
}

impl MatchClause {
    /// Creates a MATCH clause for the given pattern.
    pub fn new(pattern: impl Into<Pattern>) -> Self {
        Self {
            optional: false,
            pattern: pattern.into(),
        }
    }

    /// Creates an OPTIONAL MATCH clause for the given pattern.
    pub fn optional(pattern: impl Into<Pattern>) -> Self {
        Self {
            optional: true,
            pattern: pattern.into(),
        }
    }

    /// Returns whether this is an OPTIONAL MATCH.
    pub const fn is_optional(&self) -> bool {
        self.optional
    }

    /// Returns the pattern.
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }
}

impl WhereClause {
    /// Creates a WHERE clause with the given condition.
    pub const fn new(condition: Condition) -> Self {
        Self { condition }
    }

    /// Returns the filter condition.
    pub const fn condition(&self) -> &Condition {
        &self.condition
    }
}

impl ReturnClause {
    /// Creates a RETURN clause with the given expressions.
    pub const fn new(expressions: Vec<Expression>) -> Self {
        Self {
            distinct: false,
            expressions,
        }
    }

    /// Creates a RETURN DISTINCT clause.
    pub const fn distinct(expressions: Vec<Expression>) -> Self {
        Self {
            distinct: true,
            expressions,
        }
    }

    /// Returns whether DISTINCT is applied.
    pub const fn is_distinct(&self) -> bool {
        self.distinct
    }

    /// Returns the expressions.
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::node;

    #[test]
    fn match_clause_creates_non_optional() {
        let n = node("Person").named("n");
        let clause = MatchClause::new(n);
        assert!(!clause.is_optional());
        assert_eq!(clause.pattern().elements().len(), 1);
    }

    #[test]
    fn optional_match_clause() {
        let n = node("Person").named("n");
        let clause = MatchClause::optional(n);
        assert!(clause.is_optional());
    }

    #[test]
    fn where_clause_holds_condition() {
        let cond = Expression::symbolic_name("n").is_null();
        let clause = WhereClause::new(cond.clone());
        assert_eq!(*clause.condition(), cond);
    }

    #[test]
    fn return_clause_non_distinct() {
        let clause = ReturnClause::new(vec![Expression::symbolic_name("n")]);
        assert!(!clause.is_distinct());
        assert_eq!(clause.expressions().len(), 1);
    }

    #[test]
    fn return_clause_distinct() {
        let clause = ReturnClause::distinct(vec![Expression::symbolic_name("n")]);
        assert!(clause.is_distinct());
    }
}
