//! Cypher clause types: MATCH, RETURN, CREATE, MERGE, SET, DELETE, etc.
//!
//! Each clause represents a single segment of a Cypher query.
//! Clauses are composed into a [`SinglePartQuery`](crate::statement::SinglePartQuery)
//! to form a complete statement.

use crate::types::condition::Condition;
use crate::types::expression::{Expression, SortExpression};
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
    /// `ORDER BY sortItem1, sortItem2, ...`.
    OrderBy(OrderByClause),
    /// `SKIP n`.
    Skip(SkipClause),
    /// `LIMIT n`.
    Limit(LimitClause),
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

/// An ORDER BY clause: `ORDER BY expr1 ASC, expr2 DESC`.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    /// Sort items in order of priority.
    pub(crate) items: Vec<SortExpression>,
}

/// A SKIP clause: `SKIP n`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkipClause {
    /// The number of results to skip.
    pub(crate) value: Expression,
}

/// A LIMIT clause: `LIMIT n`.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitClause {
    /// The maximum number of results to return.
    pub(crate) value: Expression,
}

impl OrderByClause {
    /// Creates an ORDER BY clause from sort expressions.
    pub const fn new(items: Vec<SortExpression>) -> Self {
        Self { items }
    }

    /// Returns the sort items.
    pub fn items(&self) -> &[SortExpression] {
        &self.items
    }
}

impl SkipClause {
    /// Creates a SKIP clause.
    pub fn new(value: impl Into<Expression>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the skip value expression.
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

impl LimitClause {
    /// Creates a LIMIT clause.
    pub fn new(value: impl Into<Expression>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the limit value expression.
    pub const fn value(&self) -> &Expression {
        &self.value
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

    #[test]
    fn order_by_clause_holds_items() {
        let items = vec![
            Expression::symbolic_name("n").ascending(),
            Expression::symbolic_name("m").descending(),
        ];
        let clause = OrderByClause::new(items);
        assert_eq!(clause.items().len(), 2);
    }

    #[test]
    fn skip_clause_holds_value() {
        let clause = SkipClause::new(5_i32);
        assert!(matches!(
            clause.value().inner(),
            crate::types::expression::ExpressionInner::IntegerLiteral(5)
        ));
    }

    #[test]
    fn limit_clause_holds_value() {
        let clause = LimitClause::new(10_i32);
        assert!(matches!(
            clause.value().inner(),
            crate::types::expression::ExpressionInner::IntegerLiteral(10)
        ));
    }
}
