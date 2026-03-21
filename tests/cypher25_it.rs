//! Integration tests for Cypher 25 features: FINISH, FILTER, LET, WHEN, NEXT.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::clauses::{Clause, FilterClause, LetClause, MatchClause, ReturnClause};
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::statement::SinglePartQuery;

// ============================================================================
// FINISH clause
// ============================================================================

#[test]
fn finish_clause() {
    let n = node("Person").named("n");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Finish,
    ]));
    assert_eq!(stmt.render(), "MATCH (n:`Person`) FINISH");
}

// ============================================================================
// FILTER clause
// ============================================================================

#[test]
fn filter_clause() {
    let n = node("Person").named("n");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Filter(FilterClause::new(cond)),
        Clause::Return(ReturnClause::new(vec![name("n")])),
    ]));
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) FILTER n.age > 21 RETURN n"
    );
}

// ============================================================================
// LET clause
// ============================================================================

#[test]
fn let_clause_simple() {
    let n = node("Person").named("n");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Let(LetClause::new("x", lit(42_i32))),
        Clause::Return(ReturnClause::new(vec![name("x")])),
    ]));
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) LET x = 42 RETURN x"
    );
}

#[test]
fn let_clause_with_expression() {
    let n = node("Person").named("n");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Let(LetClause::new(
            "fullName",
            Expression::from(prop("n", "firstName"))
                .add(lit(" "))
                .add(Expression::from(prop("n", "lastName"))),
        )),
        Clause::Return(ReturnClause::new(vec![name("fullName")])),
    ]));
    let rendered = stmt.render();
    assert!(rendered.contains("LET fullName = "));
    assert!(rendered.contains("n.firstName"));
    assert!(rendered.contains("n.lastName"));
}

// ============================================================================
// NEXT composition
// ============================================================================

#[test]
fn next_two_queries() {
    let left = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let right = Cypher::match_(node("Movie").named("m"))
        .returning(name("m"))
        .build();
    let stmt = left.next(right);
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n NEXT MATCH (m:`Movie`) RETURN m"
    );
}

// ============================================================================
// WHEN / THEN / ELSE composition
// ============================================================================

#[test]
fn when_then() {
    let body = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let cond = prop("n", "age").gt(18_i32);
    let stmt = body.when(cond);
    let rendered = stmt.render();
    assert!(rendered.contains("WHEN"));
    assert!(rendered.contains("THEN"));
    assert!(rendered.contains("n.age > 18"));
}

#[test]
fn when_then_else() {
    let then_branch = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let else_branch = Cypher::match_(node("Movie").named("m"))
        .returning(name("m"))
        .build();
    let cond = prop("n", "age").gt(18_i32);
    let stmt = then_branch.when_else(cond, else_branch);
    let rendered = stmt.render();
    assert!(rendered.contains("WHEN"));
    assert!(rendered.contains("THEN"));
    assert!(rendered.contains("ELSE"));
}
