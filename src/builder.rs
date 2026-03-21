//! Typestate statement builder for fluent query construction.
//!
//! Each struct represents a distinct state in the query-building lifecycle.
//! Methods consume `self` and return the next state, enforcing valid clause
//! ordering at compile time.

use crate::clauses::{
    Clause, CreateClause, DeleteClause, FilterClause, InQueryCallClause, LetClause, LimitClause,
    LoadCsvClause, MatchClause, MergeAction, MergeClause, OrderByClause, RemoveClause,
    ReturnClause, SetClause, SetItem, SkipClause, UnwindClause, UsingIndexClause, UsingJoinClause,
    UsingScanClause, WhereClause, WithClause,
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

/// Converts a value into one or more SET items.
pub trait IntoSetItems {
    /// Produces the list of set items.
    fn into_set_items(self) -> Vec<SetItem>;
}

impl IntoSetItems for SetItem {
    fn into_set_items(self) -> Vec<SetItem> {
        vec![self]
    }
}

impl IntoSetItems for Vec<SetItem> {
    fn into_set_items(self) -> Vec<SetItem> {
        self
    }
}

impl<const N: usize> IntoSetItems for [SetItem; N] {
    fn into_set_items(self) -> Vec<SetItem> {
        self.into()
    }
}

/// Converts a value into one or more delete expressions.
pub trait IntoDeleteExprs {
    /// Produces the list of expressions to delete.
    fn into_delete_exprs(self) -> Vec<Expression>;
}

impl IntoDeleteExprs for Expression {
    fn into_delete_exprs(self) -> Vec<Expression> {
        vec![self]
    }
}

impl IntoDeleteExprs for Vec<Expression> {
    fn into_delete_exprs(self) -> Vec<Expression> {
        self
    }
}

impl<const N: usize> IntoDeleteExprs for [Expression; N] {
    fn into_delete_exprs(self) -> Vec<Expression> {
        self.into()
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
    pub fn match_(mut self, pattern: impl IntoPattern) -> Self {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        self
    }

    /// Chains another `MATCH` clause.
    #[must_use]
    #[deprecated(since = "0.2.0", note = "Use `match_()` instead")]
    pub fn match_node(self, pattern: impl IntoPattern) -> Self {
        self.match_(pattern)
    }

    /// Chains an `OPTIONAL MATCH` clause.
    #[must_use]
    pub fn optional_match(mut self, pattern: impl IntoPattern) -> Self {
        self.clauses
            .push(Clause::Match(MatchClause::optional(pattern.into_pattern())));
        self
    }

    /// Chains a `CREATE` clause from a match state (mixed read/write).
    pub fn create(mut self, pattern: impl IntoPattern) -> OngoingUpdate {
        self.clauses.push(Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Chains a `MERGE` clause from a match state (mixed read/write).
    pub fn merge(mut self, pattern: impl IntoPattern) -> OngoingMerge {
        self.clauses.push(Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        )));
        OngoingMerge::new(self.clauses)
    }

    /// Adds a `DELETE` clause from a match state.
    pub fn delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::new(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `DETACH DELETE` clause from a match state.
    pub fn detach_delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::detach(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `SET` clause from a match state.
    pub fn set(mut self, items: impl IntoSetItems) -> OngoingUpdate {
        self.clauses
            .push(Clause::Set(SetClause::new(items.into_set_items())));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `REMOVE` clause from a match state.
    pub fn remove(mut self, items: Vec<crate::clauses::RemoveItem>) -> OngoingUpdate {
        self.clauses
            .push(Clause::Remove(RemoveClause::new(items)));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FOREACH` clause from a match state.
    pub fn foreach(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        list: impl Into<Expression>,
        update_clauses: Vec<Clause>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Foreach(crate::clauses::ForeachClause::new(
                variable,
                list,
                update_clauses,
            )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FILTER` clause (Cypher 25).
    pub fn filter(mut self, condition: impl Into<Condition>) -> OngoingReadingWithWhere {
        self.clauses
            .push(Clause::Filter(FilterClause::new(condition.into())));
        OngoingReadingWithWhere {
            clauses: self.clauses,
        }
    }

    /// Adds a `LET` clause (Cypher 25).
    pub fn let_(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        expression: impl Into<Expression>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Let(LetClause::new(variable, expression)));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FINISH` clause (Cypher 25), terminating the query.
    pub fn finish(mut self) -> OngoingFinished {
        self.clauses.push(Clause::Finish);
        OngoingFinished {
            clauses: self.clauses,
        }
    }

    /// Chains an in-query `CALL { subquery }` from a match state.
    pub fn call_subquery(mut self, subquery_clauses: Vec<Clause>) -> OngoingInQueryCall {
        self.clauses.push(Clause::InQueryCall(InQueryCallClause::new(
            subquery_clauses,
        )));
        OngoingInQueryCall::new(self.clauses)
    }

    /// Adds a `USING INDEX var:Label(prop)` hint.
    #[must_use]
    pub fn using_index(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        label: impl Into<std::borrow::Cow<'static, str>>,
        property: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        self.clauses.push(Clause::UsingIndex(UsingIndexClause::new(
            variable, label, property,
        )));
        self
    }

    /// Adds a `USING INDEX SEEK var:Label(prop)` hint.
    #[must_use]
    pub fn using_index_seek(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        label: impl Into<std::borrow::Cow<'static, str>>,
        property: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        self.clauses.push(Clause::UsingIndex(UsingIndexClause::seek(
            variable, label, property,
        )));
        self
    }

    /// Adds a `USING SCAN var:Label` hint.
    #[must_use]
    pub fn using_scan(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        label: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        self.clauses.push(Clause::UsingScan(UsingScanClause::new(
            variable, label,
        )));
        self
    }

    /// Adds a `USING JOIN ON var` hint.
    #[must_use]
    pub fn using_join(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
    ) -> Self {
        self.clauses
            .push(Clause::UsingJoin(UsingJoinClause::new(variable)));
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

    /// Chains a `CREATE` clause from a reading-with-where state.
    pub fn create(mut self, pattern: impl IntoPattern) -> OngoingUpdate {
        self.clauses.push(Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Chains a `MERGE` clause from a reading-with-where state.
    pub fn merge(mut self, pattern: impl IntoPattern) -> OngoingMerge {
        self.clauses.push(Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        )));
        OngoingMerge::new(self.clauses)
    }

    /// Adds a `DELETE` clause from a reading-with-where state.
    pub fn delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::new(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `DETACH DELETE` clause from a reading-with-where state.
    pub fn detach_delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::detach(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `SET` clause from a reading-with-where state.
    pub fn set(mut self, items: impl IntoSetItems) -> OngoingUpdate {
        self.clauses
            .push(Clause::Set(SetClause::new(items.into_set_items())));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `REMOVE` clause from a reading-with-where state.
    pub fn remove(mut self, items: Vec<crate::clauses::RemoveItem>) -> OngoingUpdate {
        self.clauses
            .push(Clause::Remove(RemoveClause::new(items)));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FOREACH` clause from a reading-with-where state.
    pub fn foreach(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        list: impl Into<Expression>,
        update_clauses: Vec<Clause>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Foreach(crate::clauses::ForeachClause::new(
                variable,
                list,
                update_clauses,
            )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FILTER` clause (Cypher 25).
    #[must_use]
    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        self.clauses
            .push(Clause::Filter(FilterClause::new(condition.into())));
        self
    }

    /// Adds a `LET` clause (Cypher 25).
    pub fn let_(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        expression: impl Into<Expression>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Let(LetClause::new(variable, expression)));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FINISH` clause (Cypher 25), terminating the query.
    pub fn finish(mut self) -> OngoingFinished {
        self.clauses.push(Clause::Finish);
        OngoingFinished {
            clauses: self.clauses,
        }
    }

    /// Chains an in-query `CALL { subquery }` from a reading-with-where state.
    pub fn call_subquery(mut self, subquery_clauses: Vec<Clause>) -> OngoingInQueryCall {
        self.clauses.push(Clause::InQueryCall(InQueryCallClause::new(
            subquery_clauses,
        )));
        OngoingInQueryCall::new(self.clauses)
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
    pub fn match_(mut self, pattern: impl IntoPattern) -> OngoingMatch {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        OngoingMatch::new(self.clauses)
    }

    /// Chains a `MATCH` clause after WITH.
    #[deprecated(since = "0.2.0", note = "Use `match_()` instead")]
    pub fn match_node(self, pattern: impl IntoPattern) -> OngoingMatch {
        self.match_(pattern)
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

    /// Chains a `CREATE` clause after WITH.
    pub fn create(mut self, pattern: impl IntoPattern) -> OngoingUpdate {
        self.clauses.push(Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Chains a `MERGE` clause after WITH.
    pub fn merge(mut self, pattern: impl IntoPattern) -> OngoingMerge {
        self.clauses.push(Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        )));
        OngoingMerge::new(self.clauses)
    }

    /// Adds a `DELETE` clause after WITH.
    pub fn delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::new(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `DETACH DELETE` clause after WITH.
    pub fn detach_delete(mut self, expressions: impl IntoDeleteExprs) -> OngoingUpdate {
        self.clauses.push(Clause::Delete(DeleteClause::detach(
            expressions.into_delete_exprs(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `SET` clause after WITH.
    pub fn set(mut self, items: impl IntoSetItems) -> OngoingUpdate {
        self.clauses
            .push(Clause::Set(SetClause::new(items.into_set_items())));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `LET` clause (Cypher 25) after WITH.
    pub fn let_(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        expression: impl Into<Expression>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Let(LetClause::new(variable, expression)));
        OngoingUpdate::new(self.clauses)
    }

    /// Adds a `FINISH` clause (Cypher 25), terminating the query.
    pub fn finish(mut self) -> OngoingFinished {
        self.clauses.push(Clause::Finish);
        OngoingFinished {
            clauses: self.clauses,
        }
    }

    /// Chains a `LOAD CSV FROM url` clause after WITH.
    pub fn load_csv(self, url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(self.clauses, url.into(), false)
    }

    /// Chains a `LOAD CSV WITH HEADERS FROM url` clause after WITH.
    pub fn load_csv_with_headers(self, url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(self.clauses, url.into(), true)
    }

    /// Adds an `UNWIND` clause after WITH.
    pub fn unwind(self, expression: impl Into<Expression>) -> OngoingUnwind {
        OngoingUnwind::new(self.clauses, expression.into())
    }

    /// Adds a `FOREACH` clause after WITH.
    pub fn foreach(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        list: impl Into<Expression>,
        update_clauses: Vec<Clause>,
    ) -> OngoingUpdate {
        self.clauses
            .push(Clause::Foreach(crate::clauses::ForeachClause::new(
                variable,
                list,
                update_clauses,
            )));
        OngoingUpdate::new(self.clauses)
    }
}

/// State after a writing clause (CREATE, SET, DELETE, REMOVE): can continue
/// with more mutations, RETURN, or WITH.
#[derive(Debug)]
pub struct OngoingUpdate {
    clauses: Vec<Clause>,
}

impl OngoingUpdate {
    /// Creates a new update builder with existing clauses.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Adds a `SET` clause.
    #[must_use]
    pub fn set(mut self, items: impl IntoSetItems) -> Self {
        self.clauses
            .push(Clause::Set(SetClause::new(items.into_set_items())));
        self
    }

    /// Adds a `DELETE` clause.
    #[must_use]
    pub fn delete(mut self, expressions: impl IntoDeleteExprs) -> Self {
        self.clauses.push(Clause::Delete(DeleteClause::new(
            expressions.into_delete_exprs(),
        )));
        self
    }

    /// Adds a `DETACH DELETE` clause.
    #[must_use]
    pub fn detach_delete(mut self, expressions: impl IntoDeleteExprs) -> Self {
        self.clauses.push(Clause::Delete(DeleteClause::detach(
            expressions.into_delete_exprs(),
        )));
        self
    }

    /// Adds a `REMOVE` clause.
    #[must_use]
    pub fn remove(mut self, items: Vec<crate::clauses::RemoveItem>) -> Self {
        self.clauses
            .push(Clause::Remove(RemoveClause::new(items)));
        self
    }

    /// Adds a `CREATE` clause.
    #[must_use]
    pub fn create(mut self, pattern: impl IntoPattern) -> Self {
        self.clauses.push(Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        )));
        self
    }

    /// Adds a `MERGE` clause.
    pub fn merge(mut self, pattern: impl IntoPattern) -> OngoingMerge {
        self.clauses.push(Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        )));
        OngoingMerge {
            clauses: self.clauses,
        }
    }

    /// Adds a `FILTER` clause (Cypher 25).
    pub fn filter(mut self, condition: impl Into<Condition>) -> OngoingReadingWithWhere {
        self.clauses
            .push(Clause::Filter(FilterClause::new(condition.into())));
        OngoingReadingWithWhere {
            clauses: self.clauses,
        }
    }

    /// Adds a `LET` clause (Cypher 25).
    #[must_use]
    pub fn let_(
        mut self,
        variable: impl Into<std::borrow::Cow<'static, str>>,
        expression: impl Into<Expression>,
    ) -> Self {
        self.clauses
            .push(Clause::Let(LetClause::new(variable, expression)));
        self
    }

    /// Adds a `FINISH` clause (Cypher 25), terminating the query.
    pub fn finish(mut self) -> OngoingFinished {
        self.clauses.push(Clause::Finish);
        OngoingFinished {
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

    /// Adds a `WITH` clause.
    pub fn with(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Builds the final `Statement` (for write-only queries without RETURN).
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after a `MERGE` clause: can add ON CREATE/ON MATCH actions,
/// then continue with SET, RETURN, etc.
#[derive(Debug)]
pub struct OngoingMerge {
    clauses: Vec<Clause>,
}

impl OngoingMerge {
    /// Creates a new merge builder with existing clauses.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Adds `ON CREATE SET` actions to the merge clause.
    #[must_use]
    pub fn on_create(mut self, items: impl IntoSetItems) -> Self {
        let items = items.into_set_items();
        if let Some(Clause::Merge(merge)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::Merge(_))
        }) {
            merge.actions.push(MergeAction::OnCreate(items));
        }
        self
    }

    /// Adds `ON MATCH SET` actions to the merge clause.
    #[must_use]
    pub fn on_match(mut self, items: impl IntoSetItems) -> Self {
        let items = items.into_set_items();
        if let Some(Clause::Merge(merge)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::Merge(_))
        }) {
            merge.actions.push(MergeAction::OnMatch(items));
        }
        self
    }

    /// Adds a `SET` clause after merge.
    pub fn set(self, items: impl IntoSetItems) -> OngoingUpdate {
        let mut update = OngoingUpdate::new(self.clauses);
        update.clauses
            .push(Clause::Set(SetClause::new(items.into_set_items())));
        update
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

    /// Adds a `WITH` clause.
    pub fn with(mut self, expressions: impl IntoReturnExprs) -> OngoingWith {
        self.clauses.push(Clause::With(WithClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingWith {
            clauses: self.clauses,
        }
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// Terminal state after `FINISH`: can only build.
#[derive(Debug)]
pub struct OngoingFinished {
    clauses: Vec<Clause>,
}

impl OngoingFinished {
    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `UNWIND expr`: needs `.as_()` to complete the unwind alias.
#[derive(Debug)]
pub struct OngoingUnwind {
    clauses: Vec<Clause>,
    expression: Expression,
}

impl OngoingUnwind {
    /// Creates a new unwind builder.
    pub(crate) const fn new(clauses: Vec<Clause>, expression: Expression) -> Self {
        Self {
            clauses,
            expression,
        }
    }

    /// Sets the alias for the unwound variable, completing the UNWIND clause.
    ///
    /// Transitions to `OngoingWith`-like state where MATCH, RETURN, etc. can follow.
    #[allow(clippy::wrong_self_convention, reason = "builder method mirrors Cypher AS syntax, consumes self to advance builder state")]
    pub fn as_(mut self, alias: impl Into<std::borrow::Cow<'static, str>>) -> OngoingWith {
        let aliased = self.expression.alias(alias);
        self.clauses
            .push(Clause::Unwind(UnwindClause::new(aliased)));
        OngoingWith {
            clauses: self.clauses,
        }
    }
}

/// State after `CALL procedure(args)`: can add YIELD, WHERE, or build.
#[derive(Debug)]
pub struct OngoingStandaloneCall {
    clauses: Vec<Clause>,
}

impl OngoingStandaloneCall {
    /// Creates a new standalone call builder.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Adds YIELD fields to the procedure call.
    pub fn yield_(mut self, items: impl IntoReturnExprs) -> OngoingStandaloneCallWithYield {
        let yield_items = items.into_return_exprs();
        // Modify the last Call clause to add yield items
        if let Some(Clause::Call(call)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::Call(_))
        }) {
            call.yield_items = yield_items;
        }
        OngoingStandaloneCallWithYield {
            clauses: self.clauses,
        }
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `CALL ... YIELD`: can add WHERE or build.
#[derive(Debug)]
pub struct OngoingStandaloneCallWithYield {
    clauses: Vec<Clause>,
}

impl OngoingStandaloneCallWithYield {
    /// Adds a WHERE condition after YIELD.
    #[must_use]
    pub fn where_(mut self, condition: impl Into<Condition>) -> Self {
        let cond = condition.into();
        if let Some(Clause::Call(call)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::Call(_))
        }) {
            call.where_condition = Some(cond);
        }
        self
    }

    /// Adds a `RETURN` clause after YIELD.
    pub fn returning(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `CALL { subquery }`: can add IN TRANSACTIONS or continue.
#[derive(Debug)]
pub struct OngoingInQueryCall {
    clauses: Vec<Clause>,
}

impl OngoingInQueryCall {
    /// Creates a new in-query call builder.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Marks this call as IN TRANSACTIONS.
    #[must_use]
    pub fn in_transactions(mut self) -> OngoingInQueryCallInTransactions {
        if let Some(Clause::InQueryCall(call)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::InQueryCall(_))
        }) {
            call.in_transactions = true;
        }
        OngoingInQueryCallInTransactions {
            clauses: self.clauses,
        }
    }

    /// Adds a `RETURN` clause after subquery call.
    pub fn returning(mut self, expressions: impl IntoReturnExprs) -> OngoingReturn {
        self.clauses.push(Clause::Return(ReturnClause::new(
            expressions.into_return_exprs(),
        )));
        OngoingReturn {
            clauses: self.clauses,
        }
    }

    /// Adds a `MATCH` clause after subquery call.
    pub fn match_(mut self, pattern: impl IntoPattern) -> OngoingMatch {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        OngoingMatch::new(self.clauses)
    }

    /// Adds a `MATCH` clause after subquery call.
    #[deprecated(since = "0.2.0", note = "Use `match_()` instead")]
    pub fn match_node(self, pattern: impl IntoPattern) -> OngoingMatch {
        self.match_(pattern)
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `CALL { subquery } IN TRANSACTIONS`: can set batch size or continue.
#[derive(Debug)]
pub struct OngoingInQueryCallInTransactions {
    clauses: Vec<Clause>,
}

impl OngoingInQueryCallInTransactions {
    /// Sets the batch size: `OF n ROWS`.
    #[must_use]
    pub fn of_rows(mut self, size: impl Into<Expression>) -> Self {
        if let Some(Clause::InQueryCall(call)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::InQueryCall(_))
        }) {
            call.batch_size = Some(size.into());
        }
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

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `LOAD CSV`: needs `.as_()` to set the row alias.
#[derive(Debug)]
pub struct OngoingLoadCsv {
    clauses: Vec<Clause>,
    url: Expression,
    with_headers: bool,
}

impl OngoingLoadCsv {
    /// Creates a new load CSV builder.
    pub(crate) const fn new(clauses: Vec<Clause>, url: Expression, with_headers: bool) -> Self {
        Self {
            clauses,
            url,
            with_headers,
        }
    }

    /// Sets the alias for the loaded row, completing the LOAD CSV clause.
    #[allow(clippy::wrong_self_convention, reason = "builder method mirrors Cypher AS syntax, consumes self to advance builder state")]
    pub fn as_(mut self, alias: impl Into<std::borrow::Cow<'static, str>>) -> OngoingLoadCsvReady {
        let mut clause = LoadCsvClause::new(self.url, alias);
        if self.with_headers {
            clause = clause.with_headers();
        }
        self.clauses.push(Clause::LoadCsv(clause));
        OngoingLoadCsvReady {
            clauses: self.clauses,
        }
    }
}

/// State after `LOAD CSV ... AS alias`: can set field terminator, chain MATCH/CREATE, or RETURN.
#[derive(Debug)]
pub struct OngoingLoadCsvReady {
    clauses: Vec<Clause>,
}

impl OngoingLoadCsvReady {
    /// Sets a custom field terminator on the LOAD CSV clause.
    #[must_use]
    pub fn field_terminator(mut self, terminator: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        if let Some(Clause::LoadCsv(csv)) = self.clauses.iter_mut().rev().find(|c| {
            matches!(c, Clause::LoadCsv(_))
        }) {
            csv.field_terminator = Some(terminator.into());
        }
        self
    }

    /// Chains a `MATCH` clause after LOAD CSV.
    pub fn match_(mut self, pattern: impl IntoPattern) -> OngoingMatch {
        self.clauses
            .push(Clause::Match(MatchClause::new(pattern.into_pattern())));
        OngoingMatch::new(self.clauses)
    }

    /// Chains a `MATCH` clause after LOAD CSV.
    #[deprecated(since = "0.2.0", note = "Use `match_()` instead")]
    pub fn match_node(self, pattern: impl IntoPattern) -> OngoingMatch {
        self.match_(pattern)
    }

    /// Chains a `CREATE` clause after LOAD CSV.
    pub fn create(mut self, pattern: impl IntoPattern) -> OngoingUpdate {
        self.clauses.push(Clause::Create(CreateClause::new(
            pattern.into_pattern(),
        )));
        OngoingUpdate::new(self.clauses)
    }

    /// Chains a `MERGE` clause after LOAD CSV.
    pub fn merge(mut self, pattern: impl IntoPattern) -> OngoingMerge {
        self.clauses.push(Clause::Merge(MergeClause::new(
            pattern.into_pattern(),
        )));
        OngoingMerge::new(self.clauses)
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

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::SinglePart(SinglePartQuery::new(self.clauses))
    }
}

/// State after `USING PERIODIC COMMIT`: must follow with `.load_csv()`.
#[derive(Debug)]
pub struct OngoingPeriodicCommit {
    clauses: Vec<Clause>,
}

impl OngoingPeriodicCommit {
    /// Creates a new periodic commit builder.
    pub(crate) const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Chains a `LOAD CSV FROM url` clause.
    pub fn load_csv(self, url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(self.clauses, url.into(), false)
    }

    /// Chains a `LOAD CSV WITH HEADERS FROM url` clause.
    pub fn load_csv_with_headers(self, url: impl Into<Expression>) -> OngoingLoadCsv {
        OngoingLoadCsv::new(self.clauses, url.into(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::Cypher;
    use crate::types::expression::Expression;
    use crate::types::node::node;
    use crate::types::relationship::rel;

    #[test]
    fn match_return_simple() {
        // Cypher::match_(n).returning(n) => MATCH (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn match_return_multiple_expressions() {
        // MATCH (n:`Person`) RETURN n, n.name
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
            .returning_distinct(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN DISTINCT n");
    }

    #[test]
    fn match_where_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 RETURN n
        let n = node("Person").named("n");
        let cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let stmt = Cypher::match_(n)
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
        let age_cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let name_cond = Expression::symbolic_name("n").property("name").eq("Alice");
        let stmt = Cypher::match_(n)
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
        let age_cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let name_cond = Expression::symbolic_name("n").property("name").eq("Alice");
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
            .returning(Expression::symbolic_name("n"))
            .skip(10_i32)
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n SKIP 10");
    }

    #[test]
    fn match_return_limit() {
        // MATCH (n:`Person`) RETURN n LIMIT 25
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .returning(Expression::symbolic_name("n"))
            .limit(25_i32)
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n LIMIT 25");
    }

    #[test]
    fn match_return_order_by_skip_limit() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name SKIP 5 LIMIT 10
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(r)
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
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n").alias("person"))
            .match_(r)
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
        let stmt = Cypher::match_(n)
            .with_distinct(
                Expression::from(Expression::symbolic_name("n").property("city")).alias("city"),
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
        let cond = Expression::symbolic_name("person").property("age").gt(21_i32);
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n").alias("person"))
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
        let stmt = Cypher::match_(a)
            .match_(b)
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
        let stmt = Cypher::match_(a)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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
        let stmt = Cypher::match_(n)
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

    // --- CREATE builder tests ---

    #[test]
    fn create_node() {
        // CREATE (n:`Person`)
        let n = node("Person").named("n");
        let stmt = Cypher::create(n).build();
        assert_eq!(stmt.render(), "CREATE (n:`Person`)");
    }

    #[test]
    fn create_node_return() {
        // CREATE (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::create(n)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "CREATE (n:`Person`) RETURN n");
    }

    #[test]
    fn create_relationship() {
        // CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Cypher::create(r).build();
        assert_eq!(
            stmt.render(),
            "CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)"
        );
    }

    #[test]
    fn create_set_return() {
        // CREATE (n:`Person`) SET n.name = 'Alice' RETURN n
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Cypher::create(n)
            .set(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "name"),
                Expression::from("Alice"),
            ))
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE (n:`Person`) SET n.name = 'Alice' RETURN n"
        );
    }

    // --- MERGE builder tests ---

    #[test]
    fn merge_simple() {
        // MERGE (n:`Person`)
        let n = node("Person").named("n");
        let stmt = Cypher::merge(n).build();
        assert_eq!(stmt.render(), "MERGE (n:`Person`)");
    }

    #[test]
    fn merge_on_create() {
        // MERGE (n:`Person`) ON CREATE SET n.created = true RETURN n
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Cypher::merge(n)
            .on_create(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "created"),
                Expression::from(true),
            ))
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person`) ON CREATE SET n.created = true RETURN n"
        );
    }

    #[test]
    fn merge_on_match() {
        // MERGE (n:`Person`) ON MATCH SET n.updated = true RETURN n
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Cypher::merge(n)
            .on_match(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "updated"),
                Expression::from(true),
            ))
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person`) ON MATCH SET n.updated = true RETURN n"
        );
    }

    #[test]
    fn merge_on_create_and_on_match() {
        // MERGE (n:`Person`) ON CREATE SET n.created = true ON MATCH SET n.updated = true
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Cypher::merge(n)
            .on_create(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "created"),
                Expression::from(true),
            ))
            .on_match(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "updated"),
                Expression::from(true),
            ))
            .build();
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person`) ON CREATE SET n.created = true ON MATCH SET n.updated = true"
        );
    }

    #[test]
    fn match_create_return() {
        // MATCH (a:`Person`) CREATE (b:`Movie`) RETURN a, b
        // Using OngoingMatch -> create() -> OngoingUpdate -> returning()
        // (This uses the chaining from Task 5.7, but we add .create() to OngoingMatch here too)
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let stmt = Cypher::create(a)
            .create(b)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE (a:`Person`) CREATE (b:`Movie`) RETURN a, b"
        );
    }

    #[test]
    fn create_delete() {
        // CREATE (n:`Temp`) DELETE n
        let n = node("Temp").named("n");
        let stmt = Cypher::create(n)
            .delete(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "CREATE (n:`Temp`) DELETE n");
    }

    #[test]
    fn create_detach_delete() {
        // CREATE (n:`Temp`) DETACH DELETE n
        let n = node("Temp").named("n");
        let stmt = Cypher::create(n)
            .detach_delete(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "CREATE (n:`Temp`) DETACH DELETE n");
    }

    // --- UNWIND builder tests ---

    #[test]
    fn unwind_as_return() {
        // UNWIND [1, 2, 3] AS x RETURN x
        let stmt = Cypher::unwind(Expression::list_literal(vec![
            Expression::from(1_i32),
            Expression::from(2_i32),
            Expression::from(3_i32),
        ]))
        .as_("x")
        .returning(Expression::symbolic_name("x"))
        .build();
        assert_eq!(stmt.render(), "UNWIND [1, 2, 3] AS x RETURN x");
    }

    #[test]
    fn unwind_as_match_return() {
        // UNWIND $names AS name MATCH (n:`Person` {name: name}) RETURN n
        use crate::types::parameter::Parameter;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! {
                "name" => Expression::symbolic_name("name"),
            });
        let stmt = Cypher::unwind(Expression::from(Parameter::new("names")))
            .as_("name")
            .match_(n)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "UNWIND $names AS name MATCH (n:`Person` {name: name}) RETURN n"
        );
    }

    // --- CALL procedure builder tests ---

    #[test]
    fn call_procedure_simple() {
        // CALL db.labels()
        let stmt = Cypher::call_procedure("db.labels", vec![]).build();
        assert_eq!(stmt.render(), "CALL db.labels()");
    }

    #[test]
    fn call_procedure_with_args() {
        // CALL db.index.fulltext.queryNodes('titleIndex', 'hello')
        let stmt = Cypher::call_procedure(
            "db.index.fulltext.queryNodes",
            vec![
                Expression::from("titleIndex"),
                Expression::from("hello"),
            ],
        )
        .build();
        assert_eq!(
            stmt.render(),
            "CALL db.index.fulltext.queryNodes('titleIndex', 'hello')"
        );
    }

    #[test]
    fn call_procedure_yield() {
        // CALL db.labels() YIELD label RETURN label
        let stmt = Cypher::call_procedure("db.labels", vec![])
            .yield_(Expression::symbolic_name("label"))
            .returning(Expression::symbolic_name("label"))
            .build();
        assert_eq!(
            stmt.render(),
            "CALL db.labels() YIELD label RETURN label"
        );
    }

    #[test]
    fn call_procedure_yield_where() {
        // CALL db.labels() YIELD label WHERE label STARTS WITH 'P'
        use crate::types::operator::StringPredicateOp;
        let cond = Condition::StringPredicate {
            left: Expression::symbolic_name("label"),
            predicate: StringPredicateOp::StartsWith,
            right: Expression::from("P"),
        };
        let stmt = Cypher::call_procedure("db.labels", vec![])
            .yield_(Expression::symbolic_name("label"))
            .where_(cond)
            .build();
        assert_eq!(
            stmt.render(),
            "CALL db.labels() YIELD label WHERE label STARTS WITH 'P'"
        );
    }

    // --- CALL subquery builder tests ---

    #[test]
    fn call_subquery_simple() {
        // CALL { MATCH (n:`Person`) RETURN n }
        let n = node("Person").named("n");
        let stmt = Cypher::call_subquery(vec![
            Clause::Match(MatchClause::new(n.into_pattern())),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ])
        .build();
        assert_eq!(
            stmt.render(),
            "CALL { MATCH (n:`Person`) RETURN n }"
        );
    }

    #[test]
    fn call_subquery_in_transactions() {
        // CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS
        let n = node("Person").named("n");
        let stmt = Cypher::call_subquery(vec![
            Clause::Match(MatchClause::new(n.into_pattern())),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ])
        .in_transactions()
        .build();
        assert_eq!(
            stmt.render(),
            "CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS"
        );
    }

    // --- UNION, UNION ALL, EXPLAIN, PROFILE builder tests ---

    #[test]
    fn union_two_match_returns() {
        // MATCH (n:`Person`) RETURN n UNION MATCH (n:`Movie`) RETURN n
        let left = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let right = Cypher::match_(node("Movie").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let stmt = left.union(right);
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n UNION MATCH (n:`Movie`) RETURN n"
        );
    }

    #[test]
    fn union_all_two_match_returns() {
        // MATCH (n:`Person`) RETURN n UNION ALL MATCH (n:`Movie`) RETURN n
        let left = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let right = Cypher::match_(node("Movie").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let stmt = left.union_all(right);
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n UNION ALL MATCH (n:`Movie`) RETURN n"
        );
    }

    #[test]
    fn explain_match_return() {
        // EXPLAIN MATCH (n:`Person`) RETURN n
        let stmt = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build()
            .explain();
        assert_eq!(stmt.render(), "EXPLAIN MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn profile_match_return() {
        // PROFILE MATCH (n:`Person`) RETURN n
        let stmt = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build()
            .profile();
        assert_eq!(stmt.render(), "PROFILE MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn explain_union() {
        // EXPLAIN MATCH (n:`Person`) RETURN n UNION MATCH (n:`Movie`) RETURN n
        let left = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let right = Cypher::match_(node("Movie").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let stmt = left.union(right).explain();
        assert_eq!(
            stmt.render(),
            "EXPLAIN MATCH (n:`Person`) RETURN n UNION MATCH (n:`Movie`) RETURN n"
        );
    }

    #[test]
    fn profile_union_all() {
        // PROFILE MATCH (n:`Person`) RETURN n UNION ALL MATCH (n:`Movie`) RETURN n
        let left = Cypher::match_(node("Person").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let right = Cypher::match_(node("Movie").named("n"))
            .returning(Expression::symbolic_name("n"))
            .build();
        let stmt = left.union_all(right).profile();
        assert_eq!(
            stmt.render(),
            "PROFILE MATCH (n:`Person`) RETURN n UNION ALL MATCH (n:`Movie`) RETURN n"
        );
    }

    #[test]
    fn call_subquery_in_transactions_with_batch_size() {
        // CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS OF 1000 ROWS
        let n = node("Person").named("n");
        let stmt = Cypher::call_subquery(vec![
            Clause::Match(MatchClause::new(n.into_pattern())),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ])
        .in_transactions()
        .of_rows(1000_i32)
        .build();
        assert_eq!(
            stmt.render(),
            "CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS OF 1000 ROWS"
        );
    }

    // --- LOAD CSV builder tests ---

    #[test]
    fn load_csv_return() {
        // LOAD CSV FROM 'file:///data.csv' AS row RETURN row
        let stmt = Cypher::load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    #[test]
    fn load_csv_with_headers_return() {
        // LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row
        let stmt = Cypher::load_csv_with_headers(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    #[test]
    fn load_csv_field_terminator() {
        // LOAD CSV FROM 'file:///data.csv' AS row FIELDTERMINATOR ';' RETURN row
        let stmt = Cypher::load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .field_terminator(";")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "LOAD CSV FROM 'file:///data.csv' AS row FIELDTERMINATOR ';' RETURN row"
        );
    }

    #[test]
    fn load_csv_create() {
        // LOAD CSV FROM 'file:///data.csv' AS row CREATE (:`Person` {name: row})
        let n = node("Person").with_properties(crate::props! {
            "name" => Expression::symbolic_name("row"),
        });
        let stmt = Cypher::load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .create(n)
            .build();
        assert_eq!(
            stmt.render(),
            "LOAD CSV FROM 'file:///data.csv' AS row CREATE (:`Person` {name: row})"
        );
    }

    #[test]
    fn periodic_commit_load_csv() {
        // USING PERIODIC COMMIT 1000 LOAD CSV FROM 'file:///data.csv' AS row RETURN row
        let stmt = Cypher::using_periodic_commit(Some(1000))
            .load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "USING PERIODIC COMMIT 1000 LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    #[test]
    fn periodic_commit_no_size_load_csv() {
        // USING PERIODIC COMMIT LOAD CSV FROM 'file:///data.csv' AS row RETURN row
        let stmt = Cypher::using_periodic_commit(None)
            .load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "USING PERIODIC COMMIT LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    // --- Mixed read/write chaining tests ---

    #[test]
    fn match_then_create_return() {
        // MATCH (a:`Person`) CREATE (b:`Movie`) RETURN a, b
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let stmt = Cypher::match_(a)
            .create(b)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`) CREATE (b:`Movie`) RETURN a, b"
        );
    }

    #[test]
    fn match_where_create_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 CREATE (m:`Adult`) RETURN n, m
        let n = node("Person").named("n");
        let m = node("Adult").named("m");
        let cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let stmt = Cypher::match_(n)
            .where_(cond)
            .create(m)
            .returning((Expression::symbolic_name("n"), Expression::symbolic_name("m")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 CREATE (m:`Adult`) RETURN n, m"
        );
    }

    #[test]
    fn match_set_return() {
        // MATCH (n:`Person`) SET n.active = true RETURN n
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .set(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "active"),
                Expression::from(true),
            ))
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) SET n.active = true RETURN n"
        );
    }

    #[test]
    fn match_delete() {
        // MATCH (n:`Temp`) DELETE n
        let n = node("Temp").named("n");
        let stmt = Cypher::match_(n)
            .delete(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Temp`) DELETE n");
    }

    #[test]
    fn match_detach_delete() {
        // MATCH (n:`Temp`) DETACH DELETE n
        let n = node("Temp").named("n");
        let stmt = Cypher::match_(n)
            .detach_delete(Expression::symbolic_name("n"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n:`Temp`) DETACH DELETE n");
    }

    #[test]
    fn match_merge_return() {
        // MATCH (a:`Person`) MERGE (b:`Movie`) RETURN a, b
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let stmt = Cypher::match_(a)
            .merge(b)
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`) MERGE (b:`Movie`) RETURN a, b"
        );
    }

    #[test]
    fn match_where_delete() {
        // MATCH (n:`Temp`) WHERE n.expired = true DELETE n
        let n = node("Temp").named("n");
        let cond = Expression::symbolic_name("n").property("expired").eq(true);
        let stmt = Cypher::match_(n)
            .where_(cond)
            .delete(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Temp`) WHERE n.expired = true DELETE n"
        );
    }

    #[test]
    fn match_with_create_return() {
        // MATCH (n:`Person`) WITH n CREATE (m:`Clone`) RETURN n, m
        let n = node("Person").named("n");
        let m = node("Clone").named("m");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .create(m)
            .returning((Expression::symbolic_name("n"), Expression::symbolic_name("m")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n CREATE (m:`Clone`) RETURN n, m"
        );
    }

    #[test]
    fn match_with_unwind_return() {
        // MATCH (n:`Person`) WITH n UNWIND [1, 2] AS x RETURN n, x
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .unwind(Expression::list_literal(vec![
                Expression::from(1_i32),
                Expression::from(2_i32),
            ]))
            .as_("x")
            .returning((
                Expression::symbolic_name("n"),
                Expression::symbolic_name("x"),
            ))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n UNWIND [1, 2] AS x RETURN n, x"
        );
    }

    #[test]
    fn match_optional_match_where_create_return() {
        // Complex mixed: MATCH (a) OPTIONAL MATCH (a)-[r]->(b) WHERE b.x = 1 CREATE (c:`New`) RETURN a, c
        let a = crate::types::node::any_node_named("a");
        let a2 = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a2.rel(crate::types::relationship::untyped_rel().named("r")).to(b);
        let cond = Expression::symbolic_name("b").property("x").eq(1_i32);
        let c = node("New").named("c");
        let stmt = Cypher::match_(a)
            .optional_match(r)
            .where_(cond)
            .create(c)
            .returning((
                Expression::symbolic_name("a"),
                Expression::symbolic_name("c"),
            ))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a) OPTIONAL MATCH (a)-[r]->(b) WHERE b.x = 1 CREATE (c:`New`) RETURN a, c"
        );
    }

    #[test]
    fn match_foreach() {
        // MATCH (n:`Person`) FOREACH (x IN [1, 2] | CREATE (:`Temp`))
        use crate::types::pattern::IntoPattern;
        let n = node("Person").named("n");
        let temp = node("Temp");
        let stmt = Cypher::match_(n)
            .foreach(
                "x",
                Expression::list_literal(vec![
                    Expression::from(1_i32),
                    Expression::from(2_i32),
                ]),
                vec![Clause::Create(CreateClause::new(temp.into_pattern()))],
            )
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) FOREACH (x IN [1, 2] | CREATE (:`Temp`))"
        );
    }

    // --- FILTER builder tests ---

    #[test]
    fn match_filter_return() {
        // MATCH (n:`Person`) FILTER n.age > 21 RETURN n
        let n = node("Person").named("n");
        let cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let stmt = Cypher::match_(n)
            .filter(cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) FILTER n.age > 21 RETURN n"
        );
    }

    #[test]
    fn match_where_filter_return() {
        // MATCH (n:`Person`) WHERE n.active = true FILTER n.age > 21 RETURN n
        let n = node("Person").named("n");
        let where_cond = Expression::symbolic_name("n").property("active").eq(true);
        let filter_cond = Expression::symbolic_name("n").property("age").gt(21_i32);
        let stmt = Cypher::match_(n)
            .where_(where_cond)
            .filter(filter_cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.active = true FILTER n.age > 21 RETURN n"
        );
    }

    // --- LET builder tests ---

    #[test]
    fn match_let_return() {
        // MATCH (n) LET x = n.age RETURN x
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .let_("x", Expression::from(Expression::symbolic_name("n").property("age")))
            .returning(Expression::symbolic_name("x"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n) LET x = n.age RETURN x");
    }

    #[test]
    fn match_where_let_return() {
        // MATCH (n) WHERE n.active = true LET x = n.age RETURN x
        let n = crate::types::node::any_node_named("n");
        let cond = Expression::symbolic_name("n").property("active").eq(true);
        let stmt = Cypher::match_(n)
            .where_(cond)
            .let_("x", Expression::from(Expression::symbolic_name("n").property("age")))
            .returning(Expression::symbolic_name("x"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) WHERE n.active = true LET x = n.age RETURN x"
        );
    }

    #[test]
    fn match_with_let_return() {
        // MATCH (n) WITH n LET x = 42 RETURN x
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .let_("x", Expression::from(42_i32))
            .returning(Expression::symbolic_name("x"))
            .build();
        assert_eq!(stmt.render(), "MATCH (n) WITH n LET x = 42 RETURN x");
    }

    // --- FINISH builder tests ---

    #[test]
    fn match_finish() {
        // MATCH (n) FINISH
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n).finish().build();
        assert_eq!(stmt.render(), "MATCH (n) FINISH");
    }

    #[test]
    fn match_where_finish() {
        // MATCH (n) WHERE n.active = true FINISH
        let n = crate::types::node::any_node_named("n");
        let cond = Expression::symbolic_name("n").property("active").eq(true);
        let stmt = Cypher::match_(n).where_(cond).finish().build();
        assert_eq!(stmt.render(), "MATCH (n) WHERE n.active = true FINISH");
    }

    #[test]
    fn match_set_finish() {
        // MATCH (n) SET n.x = 1 FINISH
        use crate::types::property::Property;
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .set(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "x"),
                Expression::from(1_i32),
            ))
            .finish()
            .build();
        assert_eq!(stmt.render(), "MATCH (n) SET n.x = 1 FINISH");
    }

    // --- USING hints builder tests ---

    #[test]
    fn match_using_index_return() {
        // MATCH (n:`Person`) USING INDEX n:Person(name) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .using_index("n", "Person", "name")
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) USING INDEX n:`Person`(name) RETURN n"
        );
    }

    #[test]
    fn match_using_index_seek_return() {
        // MATCH (n:`Person`) USING INDEX SEEK n:Person(name) RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .using_index_seek("n", "Person", "name")
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) USING INDEX SEEK n:`Person`(name) RETURN n"
        );
    }

    #[test]
    fn match_using_scan_return() {
        // MATCH (n:`Person`) USING SCAN n:Person RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .using_scan("n", "Person")
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) USING SCAN n:`Person` RETURN n"
        );
    }

    #[test]
    fn match_using_join_return() {
        // MATCH (a)-->(b) USING JOIN ON b RETURN a, b
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::untyped_rel()).to(b);
        let stmt = Cypher::match_(r)
            .using_join("b")
            .returning((Expression::symbolic_name("a"), Expression::symbolic_name("b")))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (a)-->(b) USING JOIN ON b RETURN a, b"
        );
    }

    #[test]
    fn match_multiple_hints_return() {
        // MATCH (n:`Person`) USING INDEX n:Person(name) USING SCAN n:Person RETURN n
        let n = node("Person").named("n");
        let stmt = Cypher::match_(n)
            .using_index("n", "Person", "name")
            .using_scan("n", "Person")
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) USING INDEX n:`Person`(name) USING SCAN n:`Person` RETURN n"
        );
    }

    // --- CALL chaining builder tests ---

    // --- LOAD CSV after WITH builder tests ---

    #[test]
    fn match_with_load_csv_return() {
        // MATCH (n) WITH n LOAD CSV FROM 'file:///data.csv' AS row RETURN row
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .load_csv(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) WITH n LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    #[test]
    fn match_with_load_csv_with_headers_return() {
        // MATCH (n) WITH n LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .load_csv_with_headers(Expression::from("file:///data.csv"))
            .as_("row")
            .returning(Expression::symbolic_name("row"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) WITH n LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row"
        );
    }

    #[test]
    fn match_call_subquery_return() {
        // MATCH (n) CALL { RETURN 1 } RETURN n
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .call_subquery(vec![Clause::Return(ReturnClause::new(vec![
                Expression::from(1_i32),
            ]))])
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) CALL { RETURN 1 } RETURN n"
        );
    }

    #[test]
    fn match_where_call_subquery_return() {
        // MATCH (n) WHERE n.active = true CALL { RETURN 1 } RETURN n
        let n = crate::types::node::any_node_named("n");
        let cond = Expression::symbolic_name("n").property("active").eq(true);
        let stmt = Cypher::match_(n)
            .where_(cond)
            .call_subquery(vec![Clause::Return(ReturnClause::new(vec![
                Expression::from(1_i32),
            ]))])
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) WHERE n.active = true CALL { RETURN 1 } RETURN n"
        );
    }

    #[test]
    fn match_hint_then_where_return() {
        // MATCH (n:`Person`) USING INDEX n:Person(name) WHERE n.name = 'Alice' RETURN n
        let n = node("Person").named("n");
        let cond = Expression::symbolic_name("n").property("name").eq("Alice");
        let stmt = Cypher::match_(n)
            .using_index("n", "Person", "name")
            .where_(cond)
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) USING INDEX n:`Person`(name) WHERE n.name = 'Alice' RETURN n"
        );
    }

    #[test]
    fn match_with_finish() {
        // MATCH (n) WITH n FINISH
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .with(Expression::symbolic_name("n"))
            .finish()
            .build();
        assert_eq!(stmt.render(), "MATCH (n) WITH n FINISH");
    }

    #[test]
    fn match_set_let_return() {
        // MATCH (n) SET n.x = 1 LET y = n.x RETURN y
        use crate::types::property::Property;
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .set(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "x"),
                Expression::from(1_i32),
            ))
            .let_("y", Expression::from(Expression::symbolic_name("n").property("x")))
            .returning(Expression::symbolic_name("y"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) SET n.x = 1 LET y = n.x RETURN y"
        );
    }

    #[test]
    fn match_set_filter_return() {
        // MATCH (n) SET n.x = 1 FILTER n.y > 0 RETURN n
        use crate::types::property::Property;
        let n = crate::types::node::any_node_named("n");
        let stmt = Cypher::match_(n)
            .set(SetItem::property(
                Property::new(Expression::symbolic_name("n"), "x"),
                Expression::from(1_i32),
            ))
            .filter(Expression::symbolic_name("n").property("y").gt(0_i32))
            .returning(Expression::symbolic_name("n"))
            .build();
        assert_eq!(
            stmt.render(),
            "MATCH (n) SET n.x = 1 FILTER n.y > 0 RETURN n"
        );
    }

    #[test]
    fn periodic_commit_load_csv_with_headers() {
        // USING PERIODIC COMMIT 500 LOAD CSV WITH HEADERS FROM $url AS row CREATE (:`Person` {name: row})
        use crate::types::parameter::Parameter;
        let n = node("Person").with_properties(crate::props! {
            "name" => Expression::symbolic_name("row"),
        });
        let stmt = Cypher::using_periodic_commit(Some(500))
            .load_csv_with_headers(Expression::from(Parameter::new("url")))
            .as_("row")
            .create(n)
            .build();
        assert_eq!(
            stmt.render(),
            "USING PERIODIC COMMIT 500 LOAD CSV WITH HEADERS FROM $url AS row CREATE (:`Person` {name: row})"
        );
    }
}
