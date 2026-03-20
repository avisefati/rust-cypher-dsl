//! Cypher clause types: MATCH, RETURN, CREATE, MERGE, SET, DELETE, etc.
//!
//! Each clause represents a single segment of a Cypher query.
//! Clauses are composed into a [`SinglePartQuery`](crate::statement::SinglePartQuery)
//! to form a complete statement.

use std::borrow::Cow;

use crate::types::condition::Condition;
use crate::types::expression::{Expression, SortExpression};

/// Returns `true` if the name is a valid dotted Cypher identifier.
///
/// Allows `[a-zA-Z_][a-zA-Z0-9_]*` segments separated by dots.
fn is_valid_dotted_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.split('.').all(|segment| {
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        (first.is_ascii_alphabetic() || first == '_')
            && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}
use crate::types::pattern::Pattern;
use crate::types::property::Property;

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
    /// `WITH expr1, expr2, ...` with optional DISTINCT.
    With(WithClause),
    /// `UNWIND expr AS alias`.
    Unwind(UnwindClause),
    /// `CREATE pattern`.
    Create(CreateClause),
    /// `MERGE pattern [ON CREATE SET ...] [ON MATCH SET ...]`.
    Merge(MergeClause),
    /// `SET item1, item2, ...`.
    Set(SetClause),
    /// `DELETE expr1, expr2, ...` or `DETACH DELETE`.
    Delete(DeleteClause),
    /// `REMOVE item1, item2, ...`.
    Remove(RemoveClause),
    /// `FOREACH (var IN list | clauses)`.
    Foreach(ForeachClause),
    /// `CALL proc(args) [YIELD ...]`.
    Call(CallClause),
    /// `CALL { subquery } [IN TRANSACTIONS]`.
    InQueryCall(InQueryCallClause),
    /// `LOAD CSV [WITH HEADERS] FROM url AS alias`.
    LoadCsv(LoadCsvClause),
    /// `USE graphName` or `USE graph.byName(...)`.
    Use(UseClause),
    /// `USING INDEX var:Label(prop)` or `USING INDEX SEEK var:Label(prop)`.
    UsingIndex(UsingIndexClause),
    /// `USING SCAN var:Label`.
    UsingScan(UsingScanClause),
    /// `USING JOIN ON var`.
    UsingJoin(UsingJoinClause),
    /// `USING PERIODIC COMMIT [size]`.
    UsingPeriodicCommit(UsingPeriodicCommitClause),
    // ── Cypher 25 ──
    /// `FILTER condition` — filters rows without introducing a new scope.
    Filter(FilterClause),
    /// `LET var = expr` — binds a variable without WITH semantics.
    Let(LetClause),
    /// `FINISH` — terminates a query without returning results.
    Finish,
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

/// A WITH clause: `WITH expr1 AS a, expr2 AS b`.
///
/// Projects intermediate results, optionally with DISTINCT.
/// Expressions should typically include aliases via `alias()`.
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    /// Whether to apply DISTINCT.
    pub(crate) distinct: bool,
    /// The expressions to project.
    pub(crate) expressions: Vec<Expression>,
}

/// An UNWIND clause: `UNWIND expr AS alias`.
///
/// Expands a list into individual rows.
#[derive(Debug, Clone, PartialEq)]
pub struct UnwindClause {
    /// The expression to unwind (typically a list or parameter).
    pub(crate) expression: Expression,
}

impl WithClause {
    /// Creates a WITH clause with the given expressions.
    pub const fn new(expressions: Vec<Expression>) -> Self {
        Self {
            distinct: false,
            expressions,
        }
    }

    /// Creates a WITH DISTINCT clause.
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

    /// Returns the projected expressions.
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl UnwindClause {
    /// Creates an UNWIND clause.
    ///
    /// The expression should be aliased via `alias()`,
    /// e.g. `Expression::symbolic_name("list").alias("x")`.
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    /// Returns the unwind expression.
    pub const fn expression(&self) -> &Expression {
        &self.expression
    }
}

/// A CREATE clause: `CREATE pattern`.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateClause {
    /// The pattern to create.
    pub(crate) pattern: Pattern,
}

/// A MERGE clause: `MERGE pattern [ON CREATE SET ...] [ON MATCH SET ...]`.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeClause {
    /// The pattern to merge.
    pub(crate) pattern: Pattern,
    /// Optional merge actions (ON CREATE SET, ON MATCH SET).
    pub(crate) actions: Vec<MergeAction>,
}

/// An action within a MERGE clause.
#[derive(Debug, Clone, PartialEq)]
pub enum MergeAction {
    /// `ON CREATE SET item1, item2, ...`.
    OnCreate(Vec<SetItem>),
    /// `ON MATCH SET item1, item2, ...`.
    OnMatch(Vec<SetItem>),
}

/// A single item in a SET clause or merge action.
#[derive(Debug, Clone, PartialEq)]
pub enum SetItem {
    /// `property = value` — sets a single property.
    Property {
        /// The property to set.
        property: Property,
        /// The value to assign.
        value: Expression,
    },
    /// `node:Label1:Label2` — adds labels to a node.
    Label {
        /// The node expression.
        node: Expression,
        /// The labels to add.
        labels: Vec<Cow<'static, str>>,
    },
    /// `target += {map}` — merges properties from a map.
    Mutate {
        /// The target node/relationship.
        target: Expression,
        /// The map of properties to merge.
        value: Expression,
    },
    /// `target = value` — replaces all properties.
    ReplaceAll {
        /// The target node/relationship.
        target: Expression,
        /// The map of properties to set.
        value: Expression,
    },
}

impl CreateClause {
    /// Creates a CREATE clause for the given pattern.
    pub fn new(pattern: impl Into<Pattern>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }

    /// Returns the pattern.
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }
}

impl MergeClause {
    /// Creates a MERGE clause for the given pattern.
    pub fn new(pattern: impl Into<Pattern>) -> Self {
        Self {
            pattern: pattern.into(),
            actions: Vec::new(),
        }
    }

    /// Creates a MERGE clause with merge actions.
    pub fn with_actions(
        pattern: impl Into<Pattern>,
        actions: Vec<MergeAction>,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            actions,
        }
    }

    /// Returns the pattern.
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Returns the merge actions.
    pub fn actions(&self) -> &[MergeAction] {
        &self.actions
    }
}

impl SetItem {
    /// Creates a property-set item: `property = value`.
    pub fn property(property: Property, value: impl Into<Expression>) -> Self {
        Self::Property {
            property,
            value: value.into(),
        }
    }

    /// Creates a label-set item: `node:Label1:Label2`.
    pub fn label(
        node: impl Into<Expression>,
        labels: Vec<Cow<'static, str>>,
    ) -> Self {
        Self::Label {
            node: node.into(),
            labels,
        }
    }

    /// Creates a mutate item: `target += {map}`.
    pub fn mutate(
        target: impl Into<Expression>,
        value: impl Into<Expression>,
    ) -> Self {
        Self::Mutate {
            target: target.into(),
            value: value.into(),
        }
    }

    /// Creates a replace-all item: `target = value`.
    pub fn replace_all(
        target: impl Into<Expression>,
        value: impl Into<Expression>,
    ) -> Self {
        Self::ReplaceAll {
            target: target.into(),
            value: value.into(),
        }
    }
}

/// A SET clause: `SET item1, item2, ...`.
#[derive(Debug, Clone, PartialEq)]
pub struct SetClause {
    /// The items to set.
    pub(crate) items: Vec<SetItem>,
}

/// A DELETE clause: `DELETE expr1, expr2, ...` or `DETACH DELETE`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteClause {
    /// Whether to use DETACH DELETE.
    pub(crate) detach: bool,
    /// The expressions to delete.
    pub(crate) expressions: Vec<Expression>,
}

/// A single item in a REMOVE clause.
#[derive(Debug, Clone, PartialEq)]
pub enum RemoveItem {
    /// `REMOVE node.property` — removes a property.
    Property(Property),
    /// `REMOVE node:Label1:Label2` — removes labels.
    Label {
        /// The node expression.
        node: Expression,
        /// The labels to remove.
        labels: Vec<Cow<'static, str>>,
    },
}

/// A REMOVE clause: `REMOVE item1, item2, ...`.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveClause {
    /// The items to remove.
    pub(crate) items: Vec<RemoveItem>,
}

impl SetClause {
    /// Creates a SET clause from items.
    pub const fn new(items: Vec<SetItem>) -> Self {
        Self { items }
    }

    /// Returns the set items.
    pub fn items(&self) -> &[SetItem] {
        &self.items
    }
}

impl DeleteClause {
    /// Creates a DELETE clause.
    pub const fn new(expressions: Vec<Expression>) -> Self {
        Self {
            detach: false,
            expressions,
        }
    }

    /// Creates a DETACH DELETE clause.
    pub const fn detach(expressions: Vec<Expression>) -> Self {
        Self {
            detach: true,
            expressions,
        }
    }

    /// Returns whether this is a DETACH DELETE.
    pub const fn is_detach(&self) -> bool {
        self.detach
    }

    /// Returns the expressions to delete.
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl RemoveItem {
    /// Creates a property-remove item: `REMOVE node.property`.
    pub const fn property(property: Property) -> Self {
        Self::Property(property)
    }

    /// Creates a label-remove item: `REMOVE node:Label`.
    pub fn label(
        node: impl Into<Expression>,
        labels: Vec<Cow<'static, str>>,
    ) -> Self {
        Self::Label {
            node: node.into(),
            labels,
        }
    }
}

impl RemoveClause {
    /// Creates a REMOVE clause from items.
    pub const fn new(items: Vec<RemoveItem>) -> Self {
        Self { items }
    }

    /// Returns the remove items.
    pub fn items(&self) -> &[RemoveItem] {
        &self.items
    }
}

/// A FOREACH clause: `FOREACH (var IN list | clauses)`.
///
/// Iterates over a list and applies update clauses for each element.
#[derive(Debug, Clone, PartialEq)]
pub struct ForeachClause {
    /// The iteration variable name.
    pub(crate) variable: Cow<'static, str>,
    /// The list expression to iterate over.
    pub(crate) list: Expression,
    /// The update clauses to execute for each element.
    pub(crate) clauses: Vec<Clause>,
}

impl ForeachClause {
    /// Creates a FOREACH clause.
    pub fn new(
        variable: impl Into<Cow<'static, str>>,
        list: impl Into<Expression>,
        clauses: Vec<Clause>,
    ) -> Self {
        Self {
            variable: variable.into(),
            list: list.into(),
            clauses,
        }
    }

    /// Returns the iteration variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the list expression.
    pub const fn list(&self) -> &Expression {
        &self.list
    }

    /// Returns the update clauses.
    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }
}

/// A standalone CALL clause: `CALL proc(args) [YIELD f1, f2 [WHERE cond]]`.
#[derive(Debug, Clone, PartialEq)]
pub struct CallClause {
    /// The procedure name (e.g. `db.labels`).
    pub(crate) procedure: Cow<'static, str>,
    /// Arguments to the procedure.
    pub(crate) arguments: Vec<Expression>,
    /// Optional YIELD fields.
    pub(crate) yield_items: Vec<Expression>,
    /// Optional WHERE condition after YIELD.
    pub(crate) where_condition: Option<Condition>,
}

/// An in-query CALL clause: `CALL { subquery } [IN TRANSACTIONS [OF n ROWS]]`.
#[derive(Debug, Clone, PartialEq)]
pub struct InQueryCallClause {
    /// The subquery clauses.
    pub(crate) subquery: Vec<Clause>,
    /// Whether to run in transactions.
    pub(crate) in_transactions: bool,
    /// Optional batch size for IN TRANSACTIONS.
    pub(crate) batch_size: Option<Expression>,
}

impl CallClause {
    /// Creates a CALL clause for a procedure.
    ///
    /// # Panics
    ///
    /// Panics if `procedure` is not a valid dotted Cypher identifier
    /// (`[a-zA-Z_][a-zA-Z0-9_.]*`).
    pub fn new(
        procedure: impl Into<Cow<'static, str>>,
        arguments: Vec<Expression>,
    ) -> Self {
        let procedure = procedure.into();
        assert!(
            is_valid_dotted_identifier(&procedure),
            "invalid procedure name `{procedure}`: must match [a-zA-Z_][a-zA-Z0-9_.]*"
        );
        Self {
            procedure,
            arguments,
            yield_items: Vec::new(),
            where_condition: None,
        }
    }

    /// Adds YIELD fields to this call.
    #[must_use]
    pub fn yield_items(mut self, items: Vec<Expression>) -> Self {
        self.yield_items = items;
        self
    }

    /// Adds a WHERE condition after YIELD.
    #[must_use]
    pub fn where_condition(mut self, condition: Condition) -> Self {
        self.where_condition = Some(condition);
        self
    }

    /// Returns the procedure name.
    pub fn procedure(&self) -> &str {
        &self.procedure
    }

    /// Returns the arguments.
    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    /// Returns the yield items.
    pub fn yield_fields(&self) -> &[Expression] {
        &self.yield_items
    }

    /// Returns the optional WHERE condition.
    pub const fn where_cond(&self) -> Option<&Condition> {
        self.where_condition.as_ref()
    }
}

impl InQueryCallClause {
    /// Creates an in-query CALL clause with a subquery.
    pub const fn new(subquery: Vec<Clause>) -> Self {
        Self {
            subquery,
            in_transactions: false,
            batch_size: None,
        }
    }

    /// Creates an in-query CALL clause that runs IN TRANSACTIONS.
    pub const fn in_transactions(subquery: Vec<Clause>) -> Self {
        Self {
            subquery,
            in_transactions: true,
            batch_size: None,
        }
    }

    /// Sets the batch size for IN TRANSACTIONS.
    #[must_use]
    pub fn with_batch_size(mut self, size: impl Into<Expression>) -> Self {
        self.batch_size = Some(size.into());
        self
    }

    /// Returns the subquery clauses.
    pub fn subquery(&self) -> &[Clause] {
        &self.subquery
    }

    /// Returns whether this runs in transactions.
    pub const fn is_in_transactions(&self) -> bool {
        self.in_transactions
    }

    /// Returns the optional batch size.
    pub const fn batch_size(&self) -> Option<&Expression> {
        self.batch_size.as_ref()
    }
}

/// A LOAD CSV clause: `LOAD CSV [WITH HEADERS] FROM url AS alias`.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadCsvClause {
    /// The URL expression (typically a string literal or parameter).
    pub(crate) url: Expression,
    /// The alias for each row.
    pub(crate) alias: Cow<'static, str>,
    /// Whether to parse with headers.
    pub(crate) with_headers: bool,
    /// Optional custom field terminator.
    pub(crate) field_terminator: Option<Cow<'static, str>>,
}

impl LoadCsvClause {
    /// Creates a LOAD CSV clause.
    pub fn new(
        url: impl Into<Expression>,
        alias: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            url: url.into(),
            alias: alias.into(),
            with_headers: false,
            field_terminator: None,
        }
    }

    /// Sets WITH HEADERS mode.
    #[must_use]
    pub const fn with_headers(mut self) -> Self {
        self.with_headers = true;
        self
    }

    /// Sets a custom field terminator.
    #[must_use]
    pub fn field_terminator(mut self, terminator: impl Into<Cow<'static, str>>) -> Self {
        self.field_terminator = Some(terminator.into());
        self
    }

    /// Returns the URL expression.
    pub const fn url(&self) -> &Expression {
        &self.url
    }

    /// Returns the alias.
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Returns whether WITH HEADERS is set.
    pub const fn is_with_headers(&self) -> bool {
        self.with_headers
    }

    /// Returns the optional field terminator.
    pub fn field_terminator_value(&self) -> Option<&str> {
        self.field_terminator.as_deref()
    }
}

/// A USE clause: `USE graphName`.
///
/// Specifies the target graph for a query, used with composite databases.
#[derive(Debug, Clone, PartialEq)]
pub struct UseClause {
    /// The graph expression (name or function call).
    pub(crate) graph: Expression,
}

impl UseClause {
    /// Creates a USE clause with the given graph expression.
    pub fn new(graph: impl Into<Expression>) -> Self {
        Self {
            graph: graph.into(),
        }
    }

    /// Returns the graph expression.
    pub const fn graph(&self) -> &Expression {
        &self.graph
    }
}

/// A USING INDEX hint: `USING INDEX [SEEK] var:Label(prop)`.
///
/// Instructs the query planner to use a specific index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingIndexClause {
    /// The variable name.
    pub(crate) variable: Cow<'static, str>,
    /// The label name.
    pub(crate) label: Cow<'static, str>,
    /// The property name.
    pub(crate) property: Cow<'static, str>,
    /// Whether to use INDEX SEEK instead of INDEX.
    pub(crate) seek: bool,
}

impl UsingIndexClause {
    /// Creates a USING INDEX hint.
    pub fn new(
        variable: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
        property: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            variable: variable.into(),
            label: label.into(),
            property: property.into(),
            seek: false,
        }
    }

    /// Creates a USING INDEX SEEK hint.
    pub fn seek(
        variable: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
        property: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            variable: variable.into(),
            label: label.into(),
            property: property.into(),
            seek: true,
        }
    }

    /// Returns the variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the label name.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the property name.
    pub fn property_name(&self) -> &str {
        &self.property
    }

    /// Returns whether this is an INDEX SEEK hint.
    pub const fn is_seek(&self) -> bool {
        self.seek
    }
}

/// A USING SCAN hint: `USING SCAN var:Label`.
///
/// Instructs the query planner to use a label scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingScanClause {
    /// The variable name.
    pub(crate) variable: Cow<'static, str>,
    /// The label name.
    pub(crate) label: Cow<'static, str>,
}

impl UsingScanClause {
    /// Creates a USING SCAN hint.
    pub fn new(
        variable: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            variable: variable.into(),
            label: label.into(),
        }
    }

    /// Returns the variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the label name.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// A USING JOIN hint: `USING JOIN ON var`.
///
/// Instructs the query planner to use a hash join.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingJoinClause {
    /// The variable name to join on.
    pub(crate) variable: Cow<'static, str>,
}

impl UsingJoinClause {
    /// Creates a USING JOIN ON hint.
    pub fn new(variable: impl Into<Cow<'static, str>>) -> Self {
        Self {
            variable: variable.into(),
        }
    }

    /// Returns the variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }
}

/// A USING PERIODIC COMMIT clause: `USING PERIODIC COMMIT [size]`.
///
/// Must precede a `LOAD CSV` clause. Commits every `size` rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingPeriodicCommitClause {
    /// Optional batch size (rows per commit).
    pub(crate) size: Option<u64>,
}

impl UsingPeriodicCommitClause {
    /// Creates a USING PERIODIC COMMIT clause with an optional batch size.
    pub const fn new(size: Option<u64>) -> Self {
        Self { size }
    }

    /// Returns the batch size, if set.
    pub const fn size(&self) -> Option<u64> {
        self.size
    }
}

// ---------------------------------------------------------------------------
// Cypher 25 clause types
// ---------------------------------------------------------------------------

/// A `FILTER condition` clause (Cypher 25).
///
/// Filters rows without introducing a new scope, unlike WHERE which
/// requires a preceding reading clause.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterClause {
    /// The filter predicate.
    pub(crate) condition: Condition,
}

impl FilterClause {
    /// Creates a FILTER clause with the given condition.
    pub fn new(condition: impl Into<Condition>) -> Self {
        Self {
            condition: condition.into(),
        }
    }

    /// Returns the filter condition.
    pub const fn condition(&self) -> &Condition {
        &self.condition
    }
}

/// A `LET var = expr` clause (Cypher 25).
///
/// Binds a variable to an expression without the scope-resetting
/// semantics of WITH.
#[derive(Debug, Clone, PartialEq)]
pub struct LetClause {
    /// The variable name to bind.
    pub(crate) variable: Cow<'static, str>,
    /// The expression to bind to the variable.
    pub(crate) expression: Expression,
}

impl LetClause {
    /// Creates a LET clause binding `variable` to `expression`.
    pub fn new(
        variable: impl Into<Cow<'static, str>>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            variable: variable.into(),
            expression: expression.into(),
        }
    }

    /// Returns the variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the bound expression.
    pub const fn expression(&self) -> &Expression {
        &self.expression
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

    #[test]
    fn with_clause_non_distinct() {
        let clause = WithClause::new(vec![
            Expression::symbolic_name("n").alias("person"),
        ]);
        assert!(!clause.is_distinct());
        assert_eq!(clause.expressions().len(), 1);
    }

    #[test]
    fn with_clause_distinct() {
        let clause = WithClause::distinct(vec![
            Expression::symbolic_name("n").alias("person"),
        ]);
        assert!(clause.is_distinct());
    }

    #[test]
    fn with_clause_multiple_expressions() {
        let clause = WithClause::new(vec![
            Expression::symbolic_name("n").alias("person"),
            Expression::from(Expression::symbolic_name("n").property("age")).alias("age"),
        ]);
        assert_eq!(clause.expressions().len(), 2);
    }

    #[test]
    fn unwind_clause_holds_expression() {
        let clause = UnwindClause::new(
            Expression::symbolic_name("list").alias("x"),
        );
        // The expression should be an aliased expression
        assert!(matches!(
            clause.expression().inner(),
            crate::types::expression::ExpressionInner::Aliased { .. }
        ));
    }

    #[test]
    fn create_clause_holds_pattern() {
        let n = node("Person").named("n");
        let clause = CreateClause::new(n);
        assert_eq!(clause.pattern().elements().len(), 1);
    }

    #[test]
    fn merge_clause_no_actions() {
        let n = node("Person").named("n");
        let clause = MergeClause::new(n);
        assert_eq!(clause.pattern().elements().len(), 1);
        assert!(clause.actions().is_empty());
    }

    #[test]
    fn merge_clause_with_on_create_action() {
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let clause = MergeClause::with_actions(
            n,
            vec![MergeAction::OnCreate(vec![
                SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "created"),
                    true,
                ),
            ])],
        );
        assert_eq!(clause.actions().len(), 1);
        assert!(matches!(&clause.actions()[0], MergeAction::OnCreate(_)));
    }

    #[test]
    fn merge_clause_with_on_match_action() {
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let clause = MergeClause::with_actions(
            n,
            vec![MergeAction::OnMatch(vec![
                SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "updated"),
                    true,
                ),
            ])],
        );
        assert_eq!(clause.actions().len(), 1);
        assert!(matches!(&clause.actions()[0], MergeAction::OnMatch(_)));
    }

    #[test]
    fn set_item_property_variant() {
        use crate::types::property::Property;
        let item = SetItem::property(
            Property::new(Expression::symbolic_name("n"), "name"),
            Expression::from("Alice"),
        );
        assert!(matches!(item, SetItem::Property { .. }));
    }

    #[test]
    fn set_item_label_variant() {
        let item = SetItem::label(
            Expression::symbolic_name("n"),
            vec![Cow::Borrowed("Admin")],
        );
        assert!(matches!(item, SetItem::Label { .. }));
    }

    #[test]
    fn set_item_mutate_variant() {
        let item = SetItem::mutate(
            Expression::symbolic_name("n"),
            Expression::map_literal(vec![(Cow::Borrowed("x"), Expression::from(1_i32))]),
        );
        assert!(matches!(item, SetItem::Mutate { .. }));
    }

    #[test]
    fn set_clause_holds_items() {
        use crate::types::property::Property;
        let clause = SetClause::new(vec![
            SetItem::property(
                Property::new(Expression::symbolic_name("n"), "name"),
                Expression::from("Alice"),
            ),
        ]);
        assert_eq!(clause.items().len(), 1);
    }

    #[test]
    fn delete_clause_non_detach() {
        let clause = DeleteClause::new(vec![Expression::symbolic_name("n")]);
        assert!(!clause.is_detach());
        assert_eq!(clause.expressions().len(), 1);
    }

    #[test]
    fn delete_clause_detach() {
        let clause = DeleteClause::detach(vec![Expression::symbolic_name("n")]);
        assert!(clause.is_detach());
    }

    #[test]
    fn remove_item_property_variant() {
        use crate::types::property::Property;
        let item = RemoveItem::property(
            Property::new(Expression::symbolic_name("n"), "age"),
        );
        assert!(matches!(item, RemoveItem::Property(_)));
    }

    #[test]
    fn remove_item_label_variant() {
        let item = RemoveItem::label(
            Expression::symbolic_name("n"),
            vec![Cow::Borrowed("Admin")],
        );
        assert!(matches!(item, RemoveItem::Label { .. }));
    }

    #[test]
    fn remove_clause_holds_items() {
        use crate::types::property::Property;
        let clause = RemoveClause::new(vec![
            RemoveItem::property(
                Property::new(Expression::symbolic_name("n"), "age"),
            ),
        ]);
        assert_eq!(clause.items().len(), 1);
    }

    #[test]
    fn foreach_clause_holds_components() {
        use crate::types::property::Property;
        let clause = ForeachClause::new(
            "x",
            Expression::symbolic_name("list"),
            vec![Clause::Set(SetClause::new(vec![
                SetItem::property(
                    Property::new(Expression::symbolic_name("x"), "visited"),
                    Expression::from(true),
                ),
            ]))],
        );
        assert_eq!(clause.variable(), "x");
        assert_eq!(clause.clauses().len(), 1);
    }

    #[test]
    fn call_clause_basic() {
        let clause = CallClause::new("db.labels", vec![]);
        assert_eq!(clause.procedure(), "db.labels");
        assert!(clause.arguments().is_empty());
        assert!(clause.yield_fields().is_empty());
        assert!(clause.where_cond().is_none());
    }

    #[test]
    fn call_clause_with_yield() {
        let clause = CallClause::new("db.labels", vec![])
            .yield_items(vec![Expression::symbolic_name("label")]);
        assert_eq!(clause.yield_fields().len(), 1);
    }

    #[test]
    fn call_clause_with_yield_and_where() {
        let clause = CallClause::new("db.labels", vec![])
            .yield_items(vec![Expression::symbolic_name("label")])
            .where_condition(
                Expression::symbolic_name("label").starts_with("A"),
            );
        assert!(clause.where_cond().is_some());
    }

    #[test]
    fn in_query_call_basic() {
        let clause = InQueryCallClause::new(vec![
            Clause::Return(ReturnClause::new(vec![Expression::from(1_i32)])),
        ]);
        assert_eq!(clause.subquery().len(), 1);
        assert!(!clause.is_in_transactions());
        assert!(clause.batch_size().is_none());
    }

    #[test]
    fn in_query_call_in_transactions() {
        let clause = InQueryCallClause::in_transactions(vec![
            Clause::Return(ReturnClause::new(vec![Expression::from(1_i32)])),
        ]);
        assert!(clause.is_in_transactions());
    }

    #[test]
    fn in_query_call_with_batch_size() {
        let clause = InQueryCallClause::in_transactions(vec![
            Clause::Return(ReturnClause::new(vec![Expression::from(1_i32)])),
        ])
        .with_batch_size(1000_i32);
        assert!(clause.batch_size().is_some());
    }

    #[test]
    fn load_csv_basic() {
        let clause = LoadCsvClause::new(Expression::from("file:///data.csv"), "row");
        assert_eq!(clause.alias(), "row");
        assert!(!clause.is_with_headers());
        assert!(clause.field_terminator_value().is_none());
    }

    #[test]
    fn load_csv_with_headers() {
        let clause = LoadCsvClause::new(Expression::from("file:///data.csv"), "row")
            .with_headers();
        assert!(clause.is_with_headers());
    }

    #[test]
    fn load_csv_with_field_terminator() {
        let clause = LoadCsvClause::new(Expression::from("file:///data.csv"), "row")
            .field_terminator(";");
        assert_eq!(clause.field_terminator_value(), Some(";"));
    }

    #[test]
    fn use_clause_with_name() {
        let clause = UseClause::new(Expression::symbolic_name("myGraph"));
        assert!(matches!(
            clause.graph().inner(),
            crate::types::expression::ExpressionInner::SymbolicName(name) if name == "myGraph"
        ));
    }

    #[test]
    fn use_clause_with_function() {
        let clause = UseClause::new(Expression::raw_unchecked("graph.byName('social')"));
        assert!(matches!(
            clause.graph().inner(),
            crate::types::expression::ExpressionInner::RawExpression(_)
        ));
    }

    #[test]
    fn using_index_basic() {
        let clause = UsingIndexClause::new("n", "Person", "name");
        assert_eq!(clause.variable(), "n");
        assert_eq!(clause.label(), "Person");
        assert_eq!(clause.property_name(), "name");
        assert!(!clause.is_seek());
    }

    #[test]
    fn using_index_seek() {
        let clause = UsingIndexClause::seek("n", "Person", "name");
        assert!(clause.is_seek());
    }

    #[test]
    fn using_scan_basic() {
        let clause = UsingScanClause::new("n", "Person");
        assert_eq!(clause.variable(), "n");
        assert_eq!(clause.label(), "Person");
    }

    #[test]
    fn using_join_basic() {
        let clause = UsingJoinClause::new("n");
        assert_eq!(clause.variable(), "n");
    }

    // ── Cypher 25 ──

    #[test]
    fn filter_clause_holds_condition() {
        let cond = Expression::symbolic_name("n").is_null();
        let clause = FilterClause::new(cond.clone());
        assert_eq!(*clause.condition(), cond);
    }

    #[test]
    fn let_clause_holds_variable_and_expression() {
        let clause = LetClause::new("x", Expression::from(42_i32));
        assert_eq!(clause.variable(), "x");
        assert!(matches!(
            clause.expression().inner(),
            crate::types::expression::ExpressionInner::IntegerLiteral(42)
        ));
    }
}
