//! Transaction management types: `TerminateTransactions`.

use std::borrow::Cow;

use super::show::ShowYield;
use crate::types::condition::Condition;

/// A `TERMINATE TRANSACTIONS` statement.
///
/// Renders as: `TERMINATE TRANSACTIONS txId1, txId2, ...`
#[derive(Debug, Clone, PartialEq)]
pub struct TerminateTransactions {
    /// The transaction IDs to terminate.
    pub(crate) transaction_ids: Vec<Cow<'static, str>>,
    /// Optional YIELD fields.
    pub(crate) yield_items: Option<ShowYield>,
    /// Optional WHERE condition (only valid with YIELD).
    pub(crate) where_condition: Option<Condition>,
}

impl TerminateTransactions {
    /// Creates a new `TerminateTransactions`.
    pub(crate) const fn new(ids: Vec<Cow<'static, str>>) -> Self {
        Self {
            transaction_ids: ids,
            yield_items: None,
            where_condition: None,
        }
    }

    /// Adds YIELD * to the statement.
    #[must_use]
    pub(crate) fn with_yield_all(mut self) -> Self {
        self.yield_items = Some(ShowYield::All);
        self
    }

    /// Adds YIELD fields to the statement.
    #[must_use]
    pub(crate) fn with_yield_fields(
        mut self,
        fields: Vec<crate::types::expression::Expression>,
    ) -> Self {
        self.yield_items = Some(ShowYield::Fields(fields));
        self
    }

    /// Adds WHERE condition (requires YIELD).
    #[must_use]
    pub(crate) fn with_where(mut self, condition: Condition) -> Self {
        self.where_condition = Some(condition);
        self
    }

    /// Returns the transaction IDs.
    pub fn transaction_ids(&self) -> &[Cow<'static, str>] {
        &self.transaction_ids
    }

    /// Returns the YIELD items, if any.
    pub const fn yield_items(&self) -> Option<&ShowYield> {
        self.yield_items.as_ref()
    }

    /// Returns the WHERE condition, if any.
    pub const fn where_condition(&self) -> Option<&Condition> {
        self.where_condition.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminate_transactions_basic() {
        let tt = TerminateTransactions::new(vec!["neo4j-tx-123".into()]);
        assert_eq!(tt.transaction_ids().len(), 1);
        assert_eq!(tt.transaction_ids()[0], "neo4j-tx-123");
        assert!(tt.yield_items().is_none());
        assert!(tt.where_condition().is_none());
    }

    #[test]
    fn terminate_transactions_multiple_ids() {
        let tt = TerminateTransactions::new(vec![
            "neo4j-tx-123".into(),
            "neo4j-tx-456".into(),
        ]);
        assert_eq!(tt.transaction_ids().len(), 2);
    }

    #[test]
    fn terminate_transactions_with_yield() {
        let tt = TerminateTransactions::new(vec!["neo4j-tx-123".into()]).with_yield_all();
        assert!(matches!(tt.yield_items(), Some(ShowYield::All)));
    }
}
