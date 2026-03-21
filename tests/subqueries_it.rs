//! Integration tests for subquery patterns — ported from Java `SubqueriesIT.java`.
//!
//! Tests exercise existential subqueries, COUNT/COLLECT subqueries,
//! CALL subquery patterns, and IN TRANSACTIONS.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::clauses::{MatchClause, ReturnClause};
use rust_cypher_dsl::prelude::*;

// ============================================================================
// Existential subquery (WHERE EXISTS { ... })
// ============================================================================

#[test]
fn existential_subquery_in_return() {
    // MATCH (person:`Person`) RETURN EXISTS { MATCH (person)-[:`KNOWS`]->(friend:`Person`) RETURN friend } AS hasFriends
    let person = node("Person").named("person");
    let sub_expr = Expression::existential_subquery(
        Expression::raw_unchecked("MATCH (person)-[:`KNOWS`]->(friend:`Person`) RETURN friend"),
    );
    let stmt = Cypher::match_(person)
        .returning(sub_expr.alias("hasFriends"))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("EXISTS {"));
    assert!(rendered.contains("KNOWS"));
    assert!(rendered.contains("hasFriends"));
}

// ============================================================================
// COUNT subquery
// ============================================================================

#[test]
fn count_subquery_expression() {
    let sub = Expression::raw_unchecked("MATCH (n)-[:`KNOWS`]->(m) RETURN m");
    let expr = Expression::count_subquery(sub);
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr.alias("friendCount"))
        .build();
    let rendered = stmt.render();
    println!("{rendered}");
    assert!(rendered.contains("COUNT {"));
    assert!(rendered.contains("friendCount"));
}

// ============================================================================
// COLLECT subquery
// ============================================================================

#[test]
fn collect_subquery_expression() {
    let sub = Expression::raw_unchecked("MATCH (n)-[:`KNOWS`]->(m) RETURN m.name");
    let expr = Expression::collect_subquery(sub);
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr.alias("friendNames"))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("COLLECT {"));
    assert!(rendered.contains("friendNames"));
}

// ============================================================================
// CALL subquery via builder
// ============================================================================

#[test]
fn call_subquery_then_return() {
    // CALL { MATCH (m:`Movie`) RETURN m } RETURN m
    let m = node("Movie").named("m");
    let stmt = Cypher::call_subquery(vec![
        Clause::Match(MatchClause::new(m)),
        Clause::Return(ReturnClause::new(vec![name("m")])),
    ])
    .returning(name("m"))
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (m:`Movie`) RETURN m } RETURN m"
    );
}

#[test]
fn call_subquery_then_match() {
    // CALL { MATCH (m:`Movie`) RETURN m } MATCH (m)-[:`ACTED_IN`]->(a) RETURN m, a
    let m = node("Movie").named("m");
    let m_ref = any_node_named("m");
    let a = any_node_named("a");
    let pattern = m_ref.rel(rel("ACTED_IN")).to(a);
    let stmt = Cypher::call_subquery(vec![
        Clause::Match(MatchClause::new(m)),
        Clause::Return(ReturnClause::new(vec![name("m")])),
    ])
    .match_(pattern)
    .returning((name("m"), name("a")))
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (m:`Movie`) RETURN m } MATCH (m)-[:`ACTED_IN`]->(a) RETURN m, a"
    );
}

// ============================================================================
// CALL subquery IN TRANSACTIONS
// ============================================================================

#[test]
fn call_subquery_in_transactions_default() {
    let m = node("Movie").named("m");
    let stmt = Cypher::call_subquery(vec![
        Clause::Match(MatchClause::new(m)),
        Clause::Return(ReturnClause::new(vec![name("m")])),
    ])
    .in_transactions()
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS"
    );
}

#[test]
fn call_subquery_in_transactions_with_rows() {
    let m = node("Movie").named("m");
    let stmt = Cypher::call_subquery(vec![
        Clause::Match(MatchClause::new(m)),
        Clause::Return(ReturnClause::new(vec![name("m")])),
    ])
    .in_transactions()
    .of_rows(500_i32)
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS OF 500 ROWS"
    );
}

#[test]
fn call_subquery_in_transactions_return() {
    let m = node("Movie").named("m");
    let stmt = Cypher::call_subquery(vec![
        Clause::Match(MatchClause::new(m)),
        Clause::Return(ReturnClause::new(vec![name("m")])),
    ])
    .in_transactions()
    .of_rows(100_i32)
    .returning(name("m"))
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS OF 100 ROWS RETURN m"
    );
}
