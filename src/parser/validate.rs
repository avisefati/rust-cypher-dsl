//! Clause ordering validation for parsed queries.
//!
//! Enforces the same rules that the typestate builder encodes at compile time,
//! but at runtime for parser-produced clause sequences.

#![allow(dead_code, reason = "validation will be wired into parse() incrementally")]

use super::error::ParseError;
use crate::clauses::Clause;

/// Parser state tracking which clauses are valid next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    /// Initial state: no clauses parsed yet.
    Start,
    /// After a MATCH or OPTIONAL MATCH clause.
    AfterMatch,
    /// After a WHERE clause.
    AfterWhere,
    /// After a WITH clause.
    AfterWith,
    /// After a RETURN clause.
    AfterReturn,
    /// After a writing clause (CREATE, SET, DELETE, REMOVE, MERGE).
    AfterWrite,
    /// After an ORDER BY clause.
    AfterOrderBy,
    /// After a SKIP clause.
    AfterSkip,
    /// After a LIMIT clause (terminal for return-tail).
    AfterLimit,
    /// After UNWIND clause.
    AfterUnwind,
    /// After FINISH clause (terminal).
    AfterFinish,
}

/// Returns a short label for a clause, used in error messages.
#[allow(clippy::missing_const_for_fn, reason = "match guard on m.optional prevents const")]
fn clause_label(clause: &Clause) -> &'static str {
    match clause {
        Clause::Match(m) if m.optional => "OPTIONAL MATCH",
        Clause::Match(_) => "MATCH",
        Clause::Where(_) => "WHERE",
        Clause::Return(_) => "RETURN",
        Clause::OrderBy(_) => "ORDER BY",
        Clause::Skip(_) => "SKIP",
        Clause::Limit(_) => "LIMIT",
        Clause::With(_) => "WITH",
        Clause::Unwind(_) => "UNWIND",
        Clause::Create(_) => "CREATE",
        Clause::Merge(_) => "MERGE",
        Clause::Set(_) => "SET",
        Clause::Delete(_) => "DELETE",
        Clause::Remove(_) => "REMOVE",
        Clause::Foreach(_) => "FOREACH",
        Clause::Call(_) => "CALL",
        Clause::InQueryCall(_) => "CALL {}",
        Clause::LoadCsv(_) => "LOAD CSV",
        Clause::Use(_) => "USE",
        Clause::UsingIndex(_) => "USING INDEX",
        Clause::UsingScan(_) => "USING SCAN",
        Clause::UsingJoin(_) => "USING JOIN",
        Clause::UsingPeriodicCommit(_) => "USING PERIODIC COMMIT",
        Clause::Filter(_) => "FILTER",
        Clause::Let(_) => "LET",
        Clause::Finish => "FINISH",
    }
}

/// Returns the expected clause names for a given state.
fn expected_for_state(state: ParserState) -> Vec<String> {
    match state {
        ParserState::Start => vec![
            "MATCH".to_owned(),
            "OPTIONAL MATCH".to_owned(),
            "CREATE".to_owned(),
            "MERGE".to_owned(),
            "UNWIND".to_owned(),
            "CALL".to_owned(),
            "LOAD CSV".to_owned(),
            "WITH".to_owned(),
            "RETURN".to_owned(),
        ],
        ParserState::AfterMatch => vec![
            "WHERE".to_owned(),
            "RETURN".to_owned(),
            "WITH".to_owned(),
            "MATCH".to_owned(),
            "OPTIONAL MATCH".to_owned(),
            "CREATE".to_owned(),
            "MERGE".to_owned(),
            "SET".to_owned(),
            "DELETE".to_owned(),
            "REMOVE".to_owned(),
            "FINISH".to_owned(),
        ],
        ParserState::AfterWhere | ParserState::AfterWrite => vec![
            "RETURN".to_owned(),
            "WITH".to_owned(),
            "CREATE".to_owned(),
            "MERGE".to_owned(),
            "SET".to_owned(),
            "DELETE".to_owned(),
            "REMOVE".to_owned(),
            "FINISH".to_owned(),
        ],
        ParserState::AfterWith => vec![
            "MATCH".to_owned(),
            "OPTIONAL MATCH".to_owned(),
            "WHERE".to_owned(),
            "RETURN".to_owned(),
            "UNWIND".to_owned(),
            "CALL".to_owned(),
        ],
        ParserState::AfterReturn => vec![
            "ORDER BY".to_owned(),
            "SKIP".to_owned(),
            "LIMIT".to_owned(),
        ],
        ParserState::AfterOrderBy => vec![
            "SKIP".to_owned(),
            "LIMIT".to_owned(),
        ],
        ParserState::AfterSkip => vec![
            "LIMIT".to_owned(),
        ],
        ParserState::AfterLimit | ParserState::AfterFinish => vec![],
        ParserState::AfterUnwind => vec![
            "MATCH".to_owned(),
            "OPTIONAL MATCH".to_owned(),
            "RETURN".to_owned(),
            "WITH".to_owned(),
            "CREATE".to_owned(),
            "MERGE".to_owned(),
        ],
    }
}

/// Returns true if the given clause is valid in the given state.
#[allow(clippy::missing_const_for_fn, reason = "matches! macro prevents const")]
fn is_valid_transition(state: ParserState, clause: &Clause) -> bool {
    match state {
        ParserState::Start => matches!(
            clause,
            Clause::Match(_)
                | Clause::Create(_)
                | Clause::Merge(_)
                | Clause::Unwind(_)
                | Clause::Call(_)
                | Clause::InQueryCall(_)
                | Clause::LoadCsv(_)
                | Clause::With(_)
                | Clause::Return(_)
        ),
        ParserState::AfterMatch => matches!(
            clause,
            Clause::Where(_)
                | Clause::Return(_)
                | Clause::With(_)
                | Clause::Match(_)
                | Clause::Create(_)
                | Clause::Merge(_)
                | Clause::Set(_)
                | Clause::Delete(_)
                | Clause::Remove(_)
                | Clause::Finish
        ),
        ParserState::AfterWhere => matches!(
            clause,
            Clause::Return(_)
                | Clause::With(_)
                | Clause::Create(_)
                | Clause::Merge(_)
                | Clause::Set(_)
                | Clause::Delete(_)
                | Clause::Remove(_)
                | Clause::Finish
        ),
        ParserState::AfterWith => matches!(
            clause,
            Clause::Match(_)
                | Clause::Where(_)
                | Clause::Return(_)
                | Clause::Unwind(_)
                | Clause::Call(_)
                | Clause::InQueryCall(_)
        ),
        ParserState::AfterReturn => matches!(
            clause,
            Clause::OrderBy(_) | Clause::Skip(_) | Clause::Limit(_)
        ),
        ParserState::AfterWrite => matches!(
            clause,
            Clause::Return(_)
                | Clause::With(_)
                | Clause::Create(_)
                | Clause::Merge(_)
                | Clause::Set(_)
                | Clause::Delete(_)
                | Clause::Remove(_)
                | Clause::Finish
        ),
        ParserState::AfterOrderBy => matches!(
            clause,
            Clause::Skip(_) | Clause::Limit(_)
        ),
        ParserState::AfterSkip => matches!(clause, Clause::Limit(_)),
        ParserState::AfterLimit | ParserState::AfterFinish => false,
        ParserState::AfterUnwind => matches!(
            clause,
            Clause::Match(_)
                | Clause::Return(_)
                | Clause::With(_)
                | Clause::Create(_)
                | Clause::Merge(_)
        ),
    }
}

/// Returns the new state after processing a clause.
#[allow(clippy::missing_const_for_fn, reason = "match on enum with data prevents const")]
fn next_state(clause: &Clause) -> ParserState {
    match clause {
        Clause::Match(_) => ParserState::AfterMatch,
        Clause::Where(_) | Clause::Filter(_) => ParserState::AfterWhere,
        Clause::Return(_) => ParserState::AfterReturn,
        Clause::OrderBy(_) => ParserState::AfterOrderBy,
        Clause::Skip(_) => ParserState::AfterSkip,
        Clause::Limit(_) => ParserState::AfterLimit,
        Clause::With(_) => ParserState::AfterWith,
        Clause::Unwind(_) => ParserState::AfterUnwind,
        Clause::Finish => ParserState::AfterFinish,
        Clause::Create(_)
        | Clause::Merge(_)
        | Clause::Set(_)
        | Clause::Delete(_)
        | Clause::Remove(_)
        | Clause::Foreach(_)
        | Clause::Call(_)
        | Clause::InQueryCall(_)
        | Clause::LoadCsv(_)
        | Clause::Use(_)
        | Clause::UsingIndex(_)
        | Clause::UsingScan(_)
        | Clause::UsingJoin(_)
        | Clause::UsingPeriodicCommit(_)
        | Clause::Let(_) => ParserState::AfterWrite,
    }
}

/// Validates that a sequence of parsed clauses follows legal Cypher ordering.
///
/// Enforces the same rules that the typestate builder encodes at compile time:
/// - Reading clauses (MATCH, UNWIND, CALL) before writing clauses (CREATE, SET, DELETE)
/// - WHERE must follow MATCH or WITH
/// - RETURN/WITH position constraints
/// - ORDER BY, SKIP, LIMIT must follow RETURN
///
/// # Errors
///
/// Returns [`ParseError`] with the offending clause name and valid alternatives.
pub fn validate_clause_ordering(clauses: &[Clause]) -> Result<(), ParseError> {
    let mut state = ParserState::Start;

    for (i, clause) in clauses.iter().enumerate() {
        if !is_valid_transition(state, clause) {
            let label = clause_label(clause);
            let prev_label = if i == 0 {
                "start of query".to_owned()
            } else {
                clause_label(&clauses[i - 1]).to_owned()
            };
            let expected = expected_for_state(state);

            return Err(ParseError::validation_error(
                label,
                &prev_label,
                expected,
            ));
        }

        state = next_state(clause);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, reason = "tests use unwrap and assert macros")]
mod tests {
    use super::*;
    use crate::clauses::{
        LimitClause, MatchClause, OrderByClause, ReturnClause, SkipClause, WhereClause,
        WithClause,
    };
    use crate::types::condition::Condition;
    use crate::types::expression::Expression;
    use crate::types::node::Node;
    use crate::types::pattern::Pattern;

    fn match_clause() -> Clause {
        Clause::Match(MatchClause::new(Pattern::new(
            crate::types::pattern::PatternElement::Node(Node::any()),
        )))
    }

    fn optional_match_clause() -> Clause {
        Clause::Match(MatchClause::optional(Pattern::new(
            crate::types::pattern::PatternElement::Node(Node::any()),
        )))
    }

    fn where_clause() -> Clause {
        Clause::Where(WhereClause::new(Condition::ExpressionCondition(
            Expression::from(true),
        )))
    }

    fn return_clause() -> Clause {
        Clause::Return(ReturnClause::new(vec![Expression::asterisk()]))
    }

    fn with_clause() -> Clause {
        Clause::With(WithClause::new(vec![Expression::symbolic_name("n")]))
    }

    fn order_by_clause() -> Clause {
        Clause::OrderBy(OrderByClause::new(vec![Expression::symbolic_name("n")
            .ascending()]))
    }

    fn skip_clause() -> Clause {
        Clause::Skip(SkipClause::new(Expression::from(5_i64)))
    }

    fn limit_clause() -> Clause {
        Clause::Limit(LimitClause::new(Expression::from(10_i64)))
    }

    fn create_clause() -> Clause {
        Clause::Create(crate::clauses::CreateClause::new(Pattern::new(
            crate::types::pattern::PatternElement::Node(Node::any()),
        )))
    }

    fn finish_clause() -> Clause {
        Clause::Finish
    }

    // ─── Valid sequences ───

    #[test]
    fn valid_match_return() {
        let clauses = vec![match_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_where_return() {
        let clauses = vec![match_clause(), where_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_with_match_return() {
        let clauses = vec![
            match_clause(),
            with_clause(),
            match_clause(),
            return_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_return_order_by_skip_limit() {
        let clauses = vec![
            match_clause(),
            return_clause(),
            order_by_clause(),
            skip_clause(),
            limit_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_return_order_by_limit() {
        let clauses = vec![
            match_clause(),
            return_clause(),
            order_by_clause(),
            limit_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_return_skip_limit() {
        let clauses = vec![
            match_clause(),
            return_clause(),
            skip_clause(),
            limit_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_return_limit() {
        let clauses = vec![match_clause(), return_clause(), limit_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_optional_match_return() {
        let clauses = vec![optional_match_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_optional_match_return() {
        let clauses = vec![
            match_clause(),
            optional_match_clause(),
            return_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_create_return() {
        let clauses = vec![match_clause(), create_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_where_create_return() {
        let clauses = vec![
            match_clause(),
            where_clause(),
            create_clause(),
            return_clause(),
        ];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_match_finish() {
        let clauses = vec![match_clause(), finish_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_create_return() {
        let clauses = vec![create_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    #[test]
    fn valid_with_where_return() {
        let clauses = vec![with_clause(), where_clause(), return_clause()];
        assert!(validate_clause_ordering(&clauses).is_ok());
    }

    // ─── Invalid sequences ───

    #[test]
    fn invalid_return_before_match() {
        let clauses = vec![return_clause(), match_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("MATCH"), "error should mention MATCH: {msg}");
        assert!(msg.contains("RETURN"), "error should mention RETURN: {msg}");
    }

    #[test]
    fn invalid_where_at_start() {
        let clauses = vec![where_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("WHERE"), "error should mention WHERE: {msg}");
    }

    #[test]
    fn invalid_order_by_before_return() {
        let clauses = vec![match_clause(), order_by_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ORDER BY"),
            "error should mention ORDER BY: {msg}"
        );
    }

    #[test]
    fn invalid_skip_before_return() {
        let clauses = vec![match_clause(), skip_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("SKIP"), "error should mention SKIP: {msg}");
    }

    #[test]
    fn invalid_limit_before_return() {
        let clauses = vec![match_clause(), limit_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("LIMIT"), "error should mention LIMIT: {msg}");
    }

    #[test]
    fn invalid_where_after_return() {
        let clauses = vec![match_clause(), return_clause(), where_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("WHERE"), "error should mention WHERE: {msg}");
    }

    #[test]
    fn invalid_match_after_return() {
        let clauses = vec![return_clause(), match_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("MATCH"), "error should mention MATCH: {msg}");
    }

    #[test]
    fn invalid_clause_after_finish() {
        let clauses = vec![match_clause(), finish_clause(), return_clause()];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("RETURN"),
            "error should mention RETURN: {msg}"
        );
    }

    #[test]
    fn invalid_clause_after_limit() {
        let clauses = vec![
            match_clause(),
            return_clause(),
            limit_clause(),
            match_clause(),
        ];
        let err = validate_clause_ordering(&clauses).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("MATCH"), "error should mention MATCH: {msg}");
    }

    #[test]
    fn empty_clauses_is_ok() {
        // Validation doesn't check for empty (that's the parser's job).
        assert!(validate_clause_ordering(&[]).is_ok());
    }
}
