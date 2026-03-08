//! Typestate statement builder for fluent query construction.
//!
//! Each struct represents a distinct state in the query-building lifecycle.
//! Methods consume `self` and return the next state, enforcing valid clause
//! ordering at compile time.

use crate::clauses::{
    Clause, LimitClause, MatchClause, OrderByClause, ReturnClause, SkipClause, WhereClause,
    WithClause,
};
use crate::statement::{SinglePartQuery, Statement};
use crate::types::condition::Condition;
use crate::types::expression::{Expression, SortExpression};
use crate::types::pattern::IntoPattern;

// NOTE: `IntoPattern` is our custom trait, distinct from `Into<Pattern>`.
// We always call `.into_pattern()` explicitly when passing to `MatchClause`.

// ---------------------------------------------------------------------------
// Conversion traits
// ---------------------------------------------------------------------------

/// Converts a value into one or more return/with expressions.
pub trait IntoReturnExprs {
    /// Produces the list of return expressions.
    fn into_return_exprs(self) -> Vec<Expression>;
}

impl IntoReturnExprs for Expression {
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self]
    }
}

impl IntoReturnExprs for Vec<Expression> {
    fn into_return_exprs(self) -> Vec<Expression> {
        self
    }
}

impl<const N: usize> IntoReturnExprs for [Expression; N] {
    fn into_return_exprs(self) -> Vec<Expression> {
        self.into()
    }
}

impl IntoReturnExprs for (Expression, Expression) {
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self.0, self.1]
    }
}

impl IntoReturnExprs for (Expression, Expression, Expression) {
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self.0, self.1, self.2]
    }
}

impl IntoReturnExprs for (Expression, Expression, Expression, Expression) {
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self.0, self.1, self.2, self.3]
    }
}

impl IntoReturnExprs for (Expression, Expression, Expression, Expression, Expression) {
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self.0, self.1, self.2, self.3, self.4]
    }
}

impl
    IntoReturnExprs
    for (
        Expression,
        Expression,
        Expression,
        Expression,
        Expression,
        Expression,
    )
{
    fn into_return_exprs(self) -> Vec<Expression> {
        vec![self.0, self.1, self.2, self.3, self.4, self.5]
    }
}

/// Converts a value into one or more sort expressions.
pub trait IntoSortItems {
    /// Produces the list of sort expressions.
    fn into_sort_items(self) -> Vec<SortExpression>;
}

impl IntoSortItems for SortExpression {
    fn into_sort_items(self) -> Vec<SortExpression> {
        vec![self]
    }
}

impl IntoSortItems for Vec<SortExpression> {
    fn into_sort_items(self) -> Vec<SortExpression> {
        self
    }
}

impl<const N: usize> IntoSortItems for [SortExpression; N] {
    fn into_sort_items(self) -> Vec<SortExpression> {
        self.into()
    }
}

impl IntoSortItems for (SortExpression, SortExpression) {
    fn into_sort_items(self) -> Vec<SortExpression> {
        vec![self.0, self.1]
    }
}

impl IntoSortItems for (SortExpression, SortExpression, SortExpression) {
    fn into_sort_items(self) -> Vec<SortExpression> {
        vec![self.0, self.1, self.2]
    }
}

// ---------------------------------------------------------------------------
// Builder states
// ---------------------------------------------------------------------------

/// State after `MATCH` / `OPTIONAL MATCH`: can add WHERE, RETURN, or WITH.
#[derive(Debug)]
pub struct OngoingMatch {
    clauses: Vec<Clause>,
}

impl OngoingMatch {
    /// Creates a new builder with existing clauses.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Adds a `WHERE` condition.
    pub fn where_(mut self, condition: impl Into<Condition>) -> OngoingReadingWithWhere {
        self.clauses
            .push(Clause::Where(WhereClause::new(condition.into())));
        OngoingReadingWithWhere {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN` clause.
    pub fn returning(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN DISTINCT` clause.
    pub fn returning_distinct(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::distinct(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `WITH` clause for multi-part queries.
    pub fn with(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Adds a `WITH DISTINCT` clause.
    pub fn with_distinct(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::distinct(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Chains another `MATCH` clause.
    #[must_use]
    pub fn match_node(mut self, pattern: impl IntoPattern) -> Self {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        self
    }

    /// Chains an `OPTIONAL MATCH` clause.
    #[must_use]
    pub fn optional_match(mut self, pattern: impl IntoPattern) -> Self {
        self.clauses
            .push(Clause::Match(MatchClause::optional(pattern.into_pattern())));
        self
    }
}

/// State after `WHERE`: can add AND/OR conditions, RETURN, or WITH.
#[derive(Debug)]
pub struct OngoingReadingWithWhere {
    clauses: Vec<Clause>,
}

impl OngoingReadingWithWhere {
    /// Combines the WHERE condition with AND.
    ///
    /// Modifies the existing WHERE clause to use an AND compound.
    #[must_use]
    pub fn and(mut self, condition: impl Into<Condition>) -> Self {
        self.merge_where_condition(condition.into(), Condition::and);
        self
    }

    /// Combines the WHERE condition with OR.
    ///
    /// Modifies the existing WHERE clause to use an OR compound.
    #[must_use]
    pub fn or(mut self, condition: impl Into<Condition>) -> Self {
        self.merge_where_condition(condition.into(), Condition::or);
        self
    }

    /// Adds a `RETURN` clause.
    pub fn returning(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN DISTINCT` clause.
    pub fn returning_distinct(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::distinct(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `WITH` clause for multi-part queries.
    pub fn with(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Adds a `WITH DISTINCT` clause.
    pub fn with_distinct(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::distinct(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Merges a new condition into the last WHERE clause.
    fn merge_where_condition(
        &mut self,
        new_cond: Condition,
        combiner: fn(Condition, Condition) -> Condition,
    ) {
        // Find and update the last WHERE clause
        if let Some(Clause::Where(where_clause)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::Where(_))
        }) {
            let existing = std::mem::replace(&mut where_clause.condition, Condition::NoCondition);
            where_clause.condition = combiner(existing, new_cond);
        }
    }
}

/// State after `RETURN`: can add ORDER BY, SKIP, LIMIT, or build.
#[derive(Debug)]
pub struct OngoingReturn {
    clauses: Vec<Clause>,
}

impl OngoingReturn {
    /// Adds an `ORDER BY` clause.
    #[must_use]
    pub fn order_by(mut self, sort_items: impl IntoSortItems) -> Self {
        self.clauses.push(Clause::OrderBy(OrderByClause::new(
            sort_items.into_sort_items(),
        )));
        self
    }

    /// Adds a `SKIP` clause.
    #[must_use]
    pub fn skip(mut self, n: impl Into<Expression>) -> Self {
        self.clauses.push(Clause::Skip(SkipClause::new(n)));
        self
    }

    /// Adds a `LIMIT` clause.
    #[must_use]
    pub fn limit(mut self, n: impl Into<Expression>) -> Self {
        self.clauses.push(Clause::Limit(LimitClause::new(n)));
        self
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `WITH`: can chain MATCH, WHERE, RETURN, or UNWIND.
#[derive(Debug)]
pub struct OngoingWith {
    clauses: Vec<Clause>,
}

impl OngoingWith {
    /// Chains a `MATCH` clause after WITH.
    pub fn match_node(mut self, pattern: impl IntoPattern) -> OngoingMatch {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        OngoingMatch::new(self.clauses)
    }

    /// Chains an `OPTIONAL MATCH` clause after WITH.
    pub fn optional_match(mut self, pattern: impl IntoPattern) -> OngoingMatch {
        self.clauses
            .push(Clause::Match(MatchClause::optional(pattern.into_pattern())));
        OngoingMatch::new(self.clauses)
    }

    /// Adds a `WHERE` condition after WITH.
    pub fn where_(mut self, condition: impl Into<Condition>) -> OngoingReadingWithWhere {
        self.clauses
            .push(Clause::Where(WhereClause::new(condition.into())));
        OngoingReadingWithWhere {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN` clause after WITH.
    pub fn returning(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN DISTINCT` clause after WITH.
    pub fn returning_distinct(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::distinct(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::Cypher;
    use crate::types::expression::Expression;
    use crate::types::node::node;
    use crate::types::operator::ComparisonOp;
    use crate::types::relationship::rel;

    #[test]
    fn match_return_simple() {
        // Cypher::match_node(n).returning(n) => MATCH (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn match_return_multiple_expressions() {
        // MATCH (n:`Person`) RETURN n, n.name
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(vec![
                Expression::symbolic_name("n"),
                Expression::from(Expression::symbolic_name("n").property("name")),
            ])
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n, n.name");
    }

    #[test]
    fn match_return_tuple() {
        // MATCH (n:`Person`) RETURN n, n.name
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning((
                Expression::symbolic_name("n"),
                Expression::from(Expression::symbolic_name("n").property("name")),
            ))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n, n.name");
    }

    #[test]
    fn match_return_distinct() {
        // MATCH (n:`Person`) RETURN DISTINCT n
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning_distinct(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN DISTINCT n");
    }

    #[test]
    fn match_where_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 RETURN n
        let n = node("Person").named("n");
        let age = Expression::from(Expression::symbolic_name("n").property("age"));
        let cond = Condition::Comparison {
            left: age,
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let stmt = Cypher::match_node(n)
            .where_(cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
        );
    }

    #[test]
    fn match_where_and_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 AND n.name = 'Alice' RETURN n
        let n = node("Person").named("n");
        let age_cond = Condition::Comparison {
            left: Expression::from(Expression::symbolic_name("n").property("age")),
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let name_cond = Condition::Comparison {
            left: Expression::from(Expression::symbolic_name("n").property("name")),
            operator: ComparisonOp::Eq,
            right: Expression::from("Alice"),
        };
        let stmt = Cypher::match_node(n)
            .where_(age_cond)
            .and(name_cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 AND n.name = 'Alice' RETURN n"
        );
    }

    #[test]
    fn match_where_or_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 OR n.name = 'Alice' RETURN n
        let n = node("Person").named("n");
        let age_cond = Condition::Comparison {
            left: Expression::from(Expression::symbolic_name("n").property("age")),
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let name_cond = Condition::Comparison {
            left: Expression::from(Expression::symbolic_name("n").property("name")),
            operator: ComparisonOp::Eq,
            right: Expression::from("Alice"),
        };
        let stmt = Cypher::match_node(n)
            .where_(age_cond)
            .or(name_cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 OR n.name = 'Alice' RETURN n"
        );
    }

    #[test]
    fn match_return_order_by() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by(
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
            )
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name"
        );
    }

    #[test]
    fn match_return_order_by_desc() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.age DESC
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by(
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            )
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.age DESC"
        );
    }

    #[test]
    fn match_return_order_by_multiple() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            ])
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC"
        );
    }

    #[test]
    fn match_return_skip() {
        // MATCH (n:`Person`) RETURN n SKIP 10
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .skip(10_i32)
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n SKIP 10");
    }

    #[test]
    fn match_return_limit() {
        // MATCH (n:`Person`) RETURN n LIMIT 25
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .limit(25_i32)
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n LIMIT 25");
    }

    #[test]
    fn match_return_order_by_skip_limit() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name SKIP 5 LIMIT 10
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by(
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
            )
            .skip(5_i32)
            .limit(10_i32)
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name SKIP 5 LIMIT 10"
        );
    }

    #[test]
    fn optional_match_return() {
        // OPTIONAL MATCH (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::optional_match(n)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "OPTIONAL MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn match_relationship_return() {
        // MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Cypher::match_node(r)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
        );
    }

    #[test]
    fn match_with_match_return() {
        // MATCH (n:`Person`) WITH n AS person MATCH (person)-[:`KNOWS`]->(m) RETURN person, m
        let n = node("Person").named("n");
        let person = crate::types::node::any_node_named("person");
        let m = crate::types::node::any_node_named("m");
        let r = person.rel(rel("KNOWS")).to(m);
        let stmt = Cypher::match_node(n)
            .with(Expression::symbolic_name("n").as_alias("person"))
            .match_node(r)
            .returning((
                Expression::symbolic_name("person"),
                Expression::symbolic_name("m"),
            ))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n AS person MATCH (person)-[:`KNOWS`]->(m) RETURN person, m"
        );
    }

    #[test]
    fn match_with_distinct_return() {
        // MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .with_distinct(
                Expression::from(Expression::symbolic_name("n").property("city")).as_alias("city"),
            )
            .returning(Expression::symbolic_name("city"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city"
        );
    }

    #[test]
    fn match_with_where_return() {
        // MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person
        let n = node("Person").named("n");
        let cond = Condition::Comparison {
            left: Expression::from(Expression::symbolic_name("person").property("age")),
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let stmt = Cypher::match_node(n)
            .with(Expression::symbolic_name("n").as_alias("person"))
            .where_(cond)
            .returning(Expression::symbolic_name("person"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person"
        );
    }

    #[test]
    fn match_match_return() {
        // MATCH (a:`Person`) MATCH (b:`Movie`) RETURN a, b
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let stmt = Cypher::match_node(a)
            .match_node(b)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`) MATCH (b:`Movie`) RETURN a, b"
        );
    }

    #[test]
    fn match_optional_match_return() {
        // MATCH (a:`Person`) OPTIONAL MATCH (a)-[:`KNOWS`]->(b) RETURN a, b
        let a = node("Person").named("a");
        let a2 = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a2.rel(rel("KNOWS")).to(b);
        let stmt = Cypher::match_node(a)
            .optional_match(r)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`) OPTIONAL MATCH (a)-[:`KNOWS`]->(b) RETURN a, b"
        );
    }

    #[test]
    fn into_return_exprs_array() {
        // Test array impl
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning([
                Expression::symbolic_name("n"),
                Expression::from(Expression::symbolic_name("n").property("name")),
            ])
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n, n.name");
    }

    #[test]
    fn into_sort_items_array() {
        // Test array impl for sort items
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by([
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            ])
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC"
        );
    }

    #[test]
    fn into_sort_items_tuple() {
        // Test tuple impl for sort items
        let n = node("Person").named("n");
        let stmt = Cypher::match_node(n)
            .returning(Expression::symbolic_name("n"))
            .order_by((
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            ))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC"
        );
    }
}
