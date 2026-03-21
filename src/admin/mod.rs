//! Administration command types for index, constraint, SHOW, and transaction management.
//!
//! Admin commands are structurally different from regular Cypher queries
//! (which are sequences of clauses like MATCH, WHERE, RETURN). They are
//! standalone statements accessed via [`Statement::Admin`](crate::statement::Statement::Admin).

// Constructors are pub(crate) and will be used by the builder API in later tasks.
#![allow(dead_code, reason = "constructors used by builder API in tasks 21.7-21.9")]

pub mod constraint;
pub mod index;
pub mod show;
pub mod transaction;

// Re-export primary types at the admin module level.
pub use constraint::{ConstraintTarget, ConstraintType, CreateConstraint, DropConstraint};
pub use index::{CreateIndex, DropIndex, IndexTarget, IndexType};
pub use show::{ExecutableFilter, ShowCommand, ShowYield};
pub use transaction::TerminateTransactions;

/// A Neo4j administration command.
///
/// Administration commands have a completely different structure from
/// regular Cypher queries. They are standalone statements that manage
/// database schema (indexes, constraints) or inspect database state
/// (SHOW commands, transaction management).
#[derive(Debug, Clone, PartialEq)]
pub enum AdminCommand {
    /// `CREATE [type] INDEX [name] [IF NOT EXISTS] FOR target ON properties [OPTIONS]`
    CreateIndex(CreateIndex),
    /// `DROP INDEX name [IF EXISTS]`
    DropIndex(DropIndex),
    /// `SHOW [filter] INDEXES [YIELD ...] [WHERE ...]`
    ShowIndexes(ShowCommand),
    /// `CREATE CONSTRAINT [name] [IF NOT EXISTS] FOR target REQUIRE spec`
    CreateConstraint(CreateConstraint),
    /// `DROP CONSTRAINT name [IF EXISTS]`
    DropConstraint(DropConstraint),
    /// `SHOW [filter] CONSTRAINTS [YIELD ...] [WHERE ...]`
    ShowConstraints(ShowCommand),
    /// `SHOW [ALL | BUILT IN | USER DEFINED] FUNCTIONS [YIELD ...] [WHERE ...]`
    ShowFunctions(ShowCommand),
    /// `SHOW PROCEDURES [YIELD ...] [WHERE ...]`
    ShowProcedures(ShowCommand),
    /// `SHOW TRANSACTIONS [ids] [YIELD ...] [WHERE ...]`
    ShowTransactions(ShowCommand),
    /// `TERMINATE TRANSACTIONS id1, id2, ...`
    TerminateTransactions(TerminateTransactions),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_command_create_index_variant() {
        let cmd = AdminCommand::CreateIndex(CreateIndex::new(
            IndexType::Range,
            Some("idx".into()),
            false,
            IndexTarget::Node {
                variable: "n".into(),
                labels: vec!["Person".into()],
                properties: vec!["name".into()],
            },
        ));
        assert!(matches!(cmd, AdminCommand::CreateIndex(_)));
    }

    #[test]
    fn admin_command_drop_index_variant() {
        let cmd = AdminCommand::DropIndex(DropIndex::new("idx", false));
        assert!(matches!(cmd, AdminCommand::DropIndex(_)));
    }

    #[test]
    fn admin_command_show_indexes_variant() {
        let cmd = AdminCommand::ShowIndexes(ShowCommand::new());
        assert!(matches!(cmd, AdminCommand::ShowIndexes(_)));
    }

    #[test]
    fn admin_command_create_constraint_variant() {
        let cmd = AdminCommand::CreateConstraint(CreateConstraint::new(
            Some("c".into()),
            false,
            ConstraintTarget::Node {
                variable: "n".into(),
                label: "Person".into(),
            },
            vec!["email".into()],
            ConstraintType::Unique,
        ));
        assert!(matches!(cmd, AdminCommand::CreateConstraint(_)));
    }

    #[test]
    fn admin_command_drop_constraint_variant() {
        let cmd = AdminCommand::DropConstraint(DropConstraint::new("c", false));
        assert!(matches!(cmd, AdminCommand::DropConstraint(_)));
    }

    #[test]
    fn admin_command_show_constraints_variant() {
        let cmd = AdminCommand::ShowConstraints(ShowCommand::new());
        assert!(matches!(cmd, AdminCommand::ShowConstraints(_)));
    }

    #[test]
    fn admin_command_show_functions_variant() {
        let cmd = AdminCommand::ShowFunctions(ShowCommand::new());
        assert!(matches!(cmd, AdminCommand::ShowFunctions(_)));
    }

    #[test]
    fn admin_command_show_procedures_variant() {
        let cmd = AdminCommand::ShowProcedures(ShowCommand::new());
        assert!(matches!(cmd, AdminCommand::ShowProcedures(_)));
    }

    #[test]
    fn admin_command_show_transactions_variant() {
        let cmd = AdminCommand::ShowTransactions(ShowCommand::new());
        assert!(matches!(cmd, AdminCommand::ShowTransactions(_)));
    }

    #[test]
    fn admin_command_terminate_transactions_variant() {
        let cmd = AdminCommand::TerminateTransactions(TerminateTransactions::new(vec![
            "neo4j-tx-123".into(),
        ]));
        assert!(matches!(cmd, AdminCommand::TerminateTransactions(_)));
    }

    #[test]
    fn admin_command_is_clone_and_debug() {
        let cmd = AdminCommand::ShowIndexes(ShowCommand::new());
        let cloned = cmd.clone();
        assert_eq!(cmd, cloned);
        let debug = format!("{cmd:?}");
        assert!(debug.contains("ShowIndexes"));
    }
}
