//! SHOW command types: `ShowCommand`, `ShowYield`, `ExecutableFilter`.

use std::borrow::Cow;

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

/// A SHOW command (indexes, constraints, functions, procedures, transactions).
///
/// The specific kind is determined by the `AdminCommand` variant that
/// wraps this struct. The SHOW command itself only models the common
/// YIELD/WHERE/RETURN tail.
#[derive(Debug, Clone, PartialEq)]
pub struct ShowCommand {
    /// Optional type filter (e.g., "ALL", "RANGE", "BUILT IN").
    pub(crate) type_filter: Option<Cow<'static, str>>,
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
    pub(crate) fn with_type_filter(mut self, filter: impl Into<Cow<'static, str>>) -> Self {
        self.type_filter = Some(filter.into());
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
    pub fn type_filter(&self) -> Option<&str> {
        self.type_filter.as_deref()
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
        let sc = ShowCommand::new().with_type_filter("RANGE");
        assert_eq!(sc.type_filter(), Some("RANGE"));
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
