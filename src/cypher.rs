//! `Cypher` entry point for constructing statements.
//!
//! Provides static methods that return builder types for fluent query construction.
//! Each method transitions into the appropriate typestate builder.

use crate::builder::{
    IntoReturnExprs, IntoSubqueryClauses, OngoingInQueryCall, OngoingLoadCsv, OngoingMatch,
    OngoingMerge, OngoingPeriodicCommit, OngoingStandaloneCall, OngoingUnwind, OngoingUpdate,
    OngoingWith,
};
use crate::clauses::{
    CallClause, Clause, CreateClause, InQueryCallClause, MatchClause, MergeClause,
    UsingPeriodicCommitClause, WithClause,
};
use crate::types::expression::Expression;
use crate::types::pattern::IntoPattern;

/// Entry point for building Cypher statements.
///
/// Use the associated functions to begin constructing a query:
/// ```rust
/// use rust_cypher_dsl::prelude::*;
///
/// let stmt = Cypher::match_(node("Person").named("n"))
///     .returning(name("n"))
///     .build();
/// ```
#[derive(Debug)]
pub struct Cypher;

impl Cypher {
    /// Begins a `MATCH` query with the given pattern.
    pub fn match_(pattern: impl IntoPattern) -> OngoingMatch {
        OngoingMatch::new(vec![Clause::Match(MatchClause::new(
            pattern.into_pattern(),
        ))])
    }

    /// Begins a `MATCH` query with the given pattern.
    #[deprecated(since = "0.2.0", note = "Use `Cypher::match_()` instead")]
    pub fn match_node(pattern: impl IntoPattern) -> OngoingMatch {
        Self::match_(pattern)
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

    /// Begins a `WITH` clause.
    ///
    /// Primarily useful for building subquery bodies that start with
    /// `WITH var` when passed to [`call()`](Self::call):
    ///
    /// ```rust
    /// use rust_cypher_dsl::prelude::*;
    /// use rust_cypher_dsl::functions::aggregate::count;
    ///
    /// let sub = Cypher::with(name("app"))
    ///     .optional_match(
    ///         node("User").named("u") >> rel("USES") >> any_node_named("app")
    ///     )
    ///     .returning(count(name("u")).alias("userCount"))
    ///     .build();
    /// ```
    pub fn with(expressions: impl IntoReturnExprs) -> OngoingWith {
        OngoingWith::new(vec![Clause::With(WithClause::new(
            expressions.into_return_exprs(),
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
    /// Accepts either raw `Vec<Clause>` or a builder-constructed `Statement`:
    ///
    /// ```rust
    /// use rust_cypher_dsl::prelude::*;
    /// use rust_cypher_dsl::functions::aggregate::count;
    ///
    /// let sub = Cypher::with(name("n"))
    ///     .returning(count(name("n")).alias("cnt"))
    ///     .build();
    ///
    /// let stmt = Cypher::match_(node("Person").named("n"))
    ///     .call(sub)
    ///     .returning(name("cnt"))
    ///     .build();
    /// ```
    pub fn call(subquery: impl IntoSubqueryClauses) -> OngoingInQueryCall {
        OngoingInQueryCall::new(vec![Clause::InQueryCall(InQueryCallClause::new(
            subquery.into_subquery_clauses(),
        ))])
    }

    /// Begins a `LOAD CSV FROM url` clause. Call `.as_("alias")` next.
    pub fn load_csv(url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(Vec::new(), url.into(), false)
    }

    /// Begins a `LOAD CSV WITH HEADERS FROM url` clause. Call `.as_("alias")` next.
    pub fn load_csv_with_headers(url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(Vec::new(), url.into(), true)
    }

    /// Begins a `USING PERIODIC COMMIT [size]` clause. Follow with `.load_csv()`.
    pub fn using_periodic_commit(size: Option<u64>) -> OngoingPeriodicCommit {
        OngoingPeriodicCommit::new(vec![Clause::UsingPeriodicCommit(
            UsingPeriodicCommitClause::new(size),
        )])
    }

    // ── Index management ──

    /// Begins a `CREATE INDEX name` statement.
    ///
    /// Chain with `.text()`, `.point()`, `.fulltext()`, `.vector()`,
    /// or `.lookup()` to set the index type, then `.for_node()` or
    /// `.for_relationship()` to set the target.
    pub fn create_index(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::admin::IndexBuilder {
        crate::admin::IndexBuilder::new(name, false)
    }

    /// Begins a `CREATE INDEX name IF NOT EXISTS` statement.
    pub fn create_index_if_not_exists(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::admin::IndexBuilder {
        crate::admin::IndexBuilder::new(name, true)
    }

    /// Creates a `DROP INDEX name` statement.
    pub fn drop_index(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::statement::Statement {
        crate::statement::Statement::Admin(crate::admin::AdminCommand::DropIndex(
            crate::admin::DropIndex::new(name, false),
        ))
    }

    /// Creates a `DROP INDEX name IF EXISTS` statement.
    pub fn drop_index_if_exists(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::statement::Statement {
        crate::statement::Statement::Admin(crate::admin::AdminCommand::DropIndex(
            crate::admin::DropIndex::new(name, true),
        ))
    }

    /// Begins a `SHOW INDEXES` statement.
    pub const fn show_indexes() -> crate::admin::ShowIndexesBuilder {
        crate::admin::ShowIndexesBuilder::new()
    }

    // ── Constraint management ──

    /// Begins a `CREATE CONSTRAINT name` statement.
    ///
    /// Chain with `.for_node()` or `.for_relationship()` to set the target,
    /// then `.is_unique()`, `.is_not_null()`, etc. to set the constraint type.
    pub fn create_constraint(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::admin::ConstraintBuilder {
        crate::admin::ConstraintBuilder::new(name, false)
    }

    /// Begins a `CREATE CONSTRAINT name IF NOT EXISTS` statement.
    pub fn create_constraint_if_not_exists(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::admin::ConstraintBuilder {
        crate::admin::ConstraintBuilder::new(name, true)
    }

    /// Creates a `DROP CONSTRAINT name` statement.
    pub fn drop_constraint(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::statement::Statement {
        crate::statement::Statement::Admin(crate::admin::AdminCommand::DropConstraint(
            crate::admin::DropConstraint::new(name, false),
        ))
    }

    /// Creates a `DROP CONSTRAINT name IF EXISTS` statement.
    pub fn drop_constraint_if_exists(
        name: impl Into<std::borrow::Cow<'static, str>>,
    ) -> crate::statement::Statement {
        crate::statement::Statement::Admin(crate::admin::AdminCommand::DropConstraint(
            crate::admin::DropConstraint::new(name, true),
        ))
    }

    /// Begins a `SHOW CONSTRAINTS` statement.
    pub const fn show_constraints() -> crate::admin::ShowConstraintsBuilder {
        crate::admin::ShowConstraintsBuilder::new()
    }

    // ── Functions / Procedures ──

    /// Begins a `SHOW FUNCTIONS` statement.
    pub const fn show_functions() -> crate::admin::ShowFunctionsBuilder {
        crate::admin::ShowFunctionsBuilder::new()
    }

    /// Begins a `SHOW PROCEDURES` statement.
    pub const fn show_procedures() -> crate::admin::ShowProceduresBuilder {
        crate::admin::ShowProceduresBuilder::new()
    }

    // ── Transaction management ──

    /// Begins a `SHOW TRANSACTIONS` statement.
    pub const fn show_transactions() -> crate::admin::ShowTransactionsBuilder {
        crate::admin::ShowTransactionsBuilder::new()
    }

    /// Begins a `TERMINATE TRANSACTIONS` statement with the given IDs.
    pub fn terminate_transactions(
        ids: Vec<impl Into<std::borrow::Cow<'static, str>>>,
    ) -> crate::admin::TerminateBuilder {
        crate::admin::TerminateBuilder::new(ids.into_iter().map(Into::into).collect())
    }
}
