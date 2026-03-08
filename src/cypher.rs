//! `Cypher` entry point for constructing statements.
//!
//! Provides static methods that return builder types for fluent query construction.
//! Each method transitions into the appropriate typestate builder.

use crate::builder::{
    OngoingInQueryCall, OngoingMatch, OngoingMerge, OngoingStandaloneCall, OngoingUnwind,
    OngoingUpdate,
};
use crate::clauses::{
    CallClause, Clause, CreateClause, InQueryCallClause, MatchClause, MergeClause,
};
use crate::types::expression::Expression;
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

    /// Begins a `CREATE` query with the given pattern.
    pub fn create(pattern: impl IntoPattern) -> OngoingUpdate {
        OngoingUpdate::new(vec![Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        ))])
    }

    /// Begins a `MERGE` query with the given pattern.
    pub fn merge(pattern: impl IntoPattern) -> OngoingMerge {
        OngoingMerge::new(vec![Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        ))])
    }

    /// Begins an `UNWIND` clause. Call `.as_("alias")` to complete it.
    pub fn unwind(expression: impl Into<Expression>) -> OngoingUnwind {
        OngoingUnwind::new(Vec::new(), expression.into())
    }

    /// Begins a standalone `CALL procedure(args)` query.
    pub fn call_procedure(
        name: impl Into<std::borrow::Cow<'static, str>>,
        args: Vec<Expression>,
    ) -> OngoingStandaloneCall {
        OngoingStandaloneCall::new(vec![Clause::Call(CallClause::new(name, args))])
    }

    /// Begins an in-query `CALL { subquery }` clause.
    ///
    /// The `subquery_fn` receives a builder to construct the subquery,
    /// or you can pass pre-built clauses directly.
    pub fn call_subquery(subquery_clauses: Vec<Clause>) -> OngoingInQueryCall {
        OngoingInQueryCall::new(vec![Clause::InQueryCall(InQueryCallClause::new(
            subquery_clauses,
        ))])
    }
}
