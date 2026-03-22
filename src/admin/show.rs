//! SHOW command types: `ShowCommand`, `ShowYield`, `ExecutableFilter`, and filter enums.

use std::borrow::Cow;
use std::fmt;

use crate::types::condition::Condition;
use crate::types::expression::Expression;

/// YIELD clause for SHOW commands.
#[derive(Debug, Clone, PartialEq)]
pub enum ShowYield {
    /// `YIELD *`
    All,
    /// `YIELD field1, field2, ...`
    Fields(Vec<Expression>),
}

/// EXECUTABLE filter for SHOW FUNCTIONS / SHOW PROCEDURES.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableFilter {
    /// `EXECUTABLE BY CURRENT USER`
    CurrentUser,
    /// `EXECUTABLE BY username`
    User(Cow<'static, str>),
}

// ---------------------------------------------------------------------------
// Typed filter enums
// ---------------------------------------------------------------------------

/// Filter for `SHOW ... INDEXES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexFilter {
    /// `SHOW RANGE INDEXES`
    Range,
    /// `SHOW TEXT INDEXES`
    Text,
    /// `SHOW POINT INDEXES`
    Point,
    /// `SHOW FULLTEXT INDEXES`
    Fulltext,
    /// `SHOW VECTOR INDEXES`
    Vector,
    /// `SHOW LOOKUP INDEXES`
    Lookup,
}

impl fmt::Display for IndexFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Range => f.write_str("RANGE"),
            Self::Text => f.write_str("TEXT"),
            Self::Point => f.write_str("POINT"),
            Self::Fulltext => f.write_str("FULLTEXT"),
            Self::Vector => f.write_str("VECTOR"),
            Self::Lookup => f.write_str("LOOKUP"),
        }
    }
}

/// Filter for `SHOW ... CONSTRAINTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstraintFilter {
    /// `SHOW UNIQUE CONSTRAINTS`
    Unique,
    /// `SHOW UNIQUENESS CONSTRAINTS`
    Uniqueness,
    /// `SHOW EXISTS CONSTRAINTS`
    Exists,
    /// `SHOW NOT NULL CONSTRAINTS`
    NotNull,
    /// `SHOW NODE KEY CONSTRAINTS`
    NodeKey,
    /// `SHOW RELATIONSHIP KEY CONSTRAINTS`
    RelationshipKey,
    /// `SHOW PROPERTY TYPE CONSTRAINTS`
    PropertyType,
}

impl fmt::Display for ConstraintFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unique => f.write_str("UNIQUE"),
            Self::Uniqueness => f.write_str("UNIQUENESS"),
            Self::Exists => f.write_str("EXISTS"),
            Self::NotNull => f.write_str("NOT NULL"),
            Self::NodeKey => f.write_str("NODE KEY"),
            Self::RelationshipKey => f.write_str("RELATIONSHIP KEY"),
            Self::PropertyType => f.write_str("PROPERTY TYPE"),
        }
    }
}

/// Filter for `SHOW ... FUNCTIONS` and `SHOW ... PROCEDURES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallableFilter {
    /// `SHOW BUILT IN FUNCTIONS/PROCEDURES`
    BuiltIn,
    /// `SHOW USER DEFINED FUNCTIONS/PROCEDURES`
    UserDefined,
    /// `SHOW ALL FUNCTIONS/PROCEDURES`
    All,
}

impl fmt::Display for CallableFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuiltIn => f.write_str("BUILT IN"),
            Self::UserDefined => f.write_str("USER DEFINED"),
            Self::All => f.write_str("ALL"),
        }
    }
}

/// Wrapper enum that holds the typed filter for any SHOW command.
///
/// Preserves type information from the builder through storage
/// and rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShowTypeFilter {
    /// Index filter (RANGE, TEXT, POINT, etc.)
    Index(IndexFilter),
    /// Constraint filter (UNIQUE, NOT NULL, NODE KEY, etc.)
    Constraint(ConstraintFilter),
    /// Callable filter (BUILT IN, USER DEFINED, ALL)
    Callable(CallableFilter),
}

impl fmt::Display for ShowTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(v) => v.fmt(f),
            Self::Constraint(v) => v.fmt(f),
            Self::Callable(v) => v.fmt(f),
        }
    }
}

impl From<IndexFilter> for ShowTypeFilter {
    fn from(f: IndexFilter) -> Self {
        Self::Index(f)
    }
}

impl From<ConstraintFilter> for ShowTypeFilter {
    fn from(f: ConstraintFilter) -> Self {
        Self::Constraint(f)
    }
}

impl From<CallableFilter> for ShowTypeFilter {
    fn from(f: CallableFilter) -> Self {
        Self::Callable(f)
    }
}

/// A SHOW command (indexes, constraints, functions, procedures, transactions).
///
/// The specific kind is determined by the `AdminCommand` variant that
/// wraps this struct. The SHOW command itself only models the common
/// YIELD/WHERE/RETURN tail.
#[derive(Debug, Clone, PartialEq)]
pub struct ShowCommand {
    /// Optional typed filter (e.g., `IndexFilter::Range`, `ConstraintFilter::Unique`).
    pub(crate) type_filter: Option<ShowTypeFilter>,
    /// Optional YIELD fields.
    pub(crate) yield_items: Option<ShowYield>,
    /// Optional WHERE condition (only valid with YIELD).
    pub(crate) where_condition: Option<Condition>,
    /// Optional transaction IDs (for SHOW TRANSACTIONS only).
    pub(crate) transaction_ids: Vec<Cow<'static, str>>,
    /// Optional EXECUTABLE filter (for SHOW FUNCTIONS/PROCEDURES).
    pub(crate) executable: Option<ExecutableFilter>,
}

impl ShowCommand {
    /// Creates a new empty SHOW command.
    pub(crate) const fn new() -> Self {
        Self {
            type_filter: None,
            yield_items: None,
            where_condition: None,
            transaction_ids: Vec::new(),
            executable: None,
        }
    }

    /// Sets the type filter.
    #[must_use]
    pub(crate) const fn with_type_filter(mut self, filter: ShowTypeFilter) -> Self {
        self.type_filter = Some(filter);
        self
    }

    /// Sets YIELD *.
    #[must_use]
    pub(crate) fn with_yield_all(mut self) -> Self {
        self.yield_items = Some(ShowYield::All);
        self
    }

    /// Sets YIELD fields.
    #[must_use]
    pub(crate) fn with_yield_fields(mut self, fields: Vec<Expression>) -> Self {
        self.yield_items = Some(ShowYield::Fields(fields));
        self
    }

    /// Sets the WHERE condition.
    #[must_use]
    pub(crate) fn with_where(mut self, condition: Condition) -> Self {
        self.where_condition = Some(condition);
        self
    }

    /// Sets transaction IDs.
    #[must_use]
    pub(crate) fn with_transaction_ids(mut self, ids: Vec<Cow<'static, str>>) -> Self {
        self.transaction_ids = ids;
        self
    }

    /// Sets the EXECUTABLE filter.
    #[must_use]
    pub(crate) fn with_executable(mut self, filter: ExecutableFilter) -> Self {
        self.executable = Some(filter);
        self
    }

    /// Returns the type filter, if any.
    pub const fn type_filter(&self) -> Option<&ShowTypeFilter> {
        self.type_filter.as_ref()
    }

    /// Returns the YIELD items, if any.
    pub const fn yield_items(&self) -> Option<&ShowYield> {
        self.yield_items.as_ref()
    }

    /// Returns the WHERE condition, if any.
    pub const fn where_condition(&self) -> Option<&Condition> {
        self.where_condition.as_ref()
    }

    /// Returns the transaction IDs.
    pub fn transaction_ids(&self) -> &[Cow<'static, str>] {
        &self.transaction_ids
    }

    /// Returns the EXECUTABLE filter, if any.
    pub const fn executable(&self) -> Option<&ExecutableFilter> {
        self.executable.as_ref()
    }
}

#[cfg(test)]
#[allow(clippy::panic, reason = "tests use assert macros")]
mod tests {
    use super::*;

    // ── Filter enum Display tests ──

    #[test]
    fn index_filter_display() {
        assert_eq!(IndexFilter::Range.to_string(), "RANGE");
        assert_eq!(IndexFilter::Text.to_string(), "TEXT");
        assert_eq!(IndexFilter::Point.to_string(), "POINT");
        assert_eq!(IndexFilter::Fulltext.to_string(), "FULLTEXT");
        assert_eq!(IndexFilter::Vector.to_string(), "VECTOR");
        assert_eq!(IndexFilter::Lookup.to_string(), "LOOKUP");
    }

    #[test]
    fn constraint_filter_display() {
        assert_eq!(ConstraintFilter::Unique.to_string(), "UNIQUE");
        assert_eq!(ConstraintFilter::Uniqueness.to_string(), "UNIQUENESS");
        assert_eq!(ConstraintFilter::Exists.to_string(), "EXISTS");
        assert_eq!(ConstraintFilter::NotNull.to_string(), "NOT NULL");
        assert_eq!(ConstraintFilter::NodeKey.to_string(), "NODE KEY");
        assert_eq!(ConstraintFilter::RelationshipKey.to_string(), "RELATIONSHIP KEY");
        assert_eq!(ConstraintFilter::PropertyType.to_string(), "PROPERTY TYPE");
    }

    #[test]
    fn callable_filter_display() {
        assert_eq!(CallableFilter::BuiltIn.to_string(), "BUILT IN");
        assert_eq!(CallableFilter::UserDefined.to_string(), "USER DEFINED");
        assert_eq!(CallableFilter::All.to_string(), "ALL");
    }

    #[test]
    fn show_type_filter_display_delegates() {
        assert_eq!(ShowTypeFilter::Index(IndexFilter::Range).to_string(), "RANGE");
        assert_eq!(ShowTypeFilter::Constraint(ConstraintFilter::Unique).to_string(), "UNIQUE");
        assert_eq!(ShowTypeFilter::Callable(CallableFilter::BuiltIn).to_string(), "BUILT IN");
    }

    #[test]
    fn show_type_filter_from_impls() {
        let f: ShowTypeFilter = IndexFilter::Range.into();
        assert_eq!(f, ShowTypeFilter::Index(IndexFilter::Range));

        let f: ShowTypeFilter = ConstraintFilter::NotNull.into();
        assert_eq!(f, ShowTypeFilter::Constraint(ConstraintFilter::NotNull));

        let f: ShowTypeFilter = CallableFilter::UserDefined.into();
        assert_eq!(f, ShowTypeFilter::Callable(CallableFilter::UserDefined));
    }

    // ── ShowCommand tests ──

    #[test]
    fn show_command_default() {
        let sc = ShowCommand::new();
        assert!(sc.type_filter().is_none());
        assert!(sc.yield_items().is_none());
        assert!(sc.where_condition().is_none());
        assert!(sc.transaction_ids().is_empty());
        assert!(sc.executable().is_none());
    }

    #[test]
    fn show_command_with_type_filter() {
        let sc = ShowCommand::new().with_type_filter(ShowTypeFilter::Index(IndexFilter::Range));
        assert_eq!(sc.type_filter(), Some(&ShowTypeFilter::Index(IndexFilter::Range)));
    }

    #[test]
    fn show_command_with_yield_all() {
        let sc = ShowCommand::new().with_yield_all();
        assert!(matches!(sc.yield_items(), Some(ShowYield::All)));
    }

    #[test]
    fn show_command_with_yield_fields() {
        let sc = ShowCommand::new().with_yield_fields(vec![
            Expression::symbolic_name("name"),
            Expression::symbolic_name("state"),
        ]);
        let Some(ShowYield::Fields(fields)) = sc.yield_items() else {
            panic!("Expected Fields yield");
        };
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn show_command_with_where() {
        let cond = Expression::symbolic_name("name").starts_with("db.");
        let sc = ShowCommand::new().with_where(cond);
        assert!(sc.where_condition().is_some());
    }

    #[test]
    fn show_command_with_transaction_ids() {
        let sc = ShowCommand::new()
            .with_transaction_ids(vec!["neo4j-tx-123".into(), "neo4j-tx-456".into()]);
        assert_eq!(sc.transaction_ids().len(), 2);
    }

    #[test]
    fn show_command_with_executable_current_user() {
        let sc = ShowCommand::new().with_executable(ExecutableFilter::CurrentUser);
        assert!(matches!(sc.executable(), Some(ExecutableFilter::CurrentUser)));
    }

    #[test]
    fn show_command_with_executable_user() {
        let sc = ShowCommand::new().with_executable(ExecutableFilter::User("alice".into()));
        assert!(matches!(sc.executable(), Some(ExecutableFilter::User(_))));
    }
}
