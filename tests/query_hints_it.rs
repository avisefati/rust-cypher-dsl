//! Integration tests for query hints (USING INDEX, USING SCAN, USING JOIN).

use rust_cypher_dsl::clauses::{Clause, UsingIndexClause, UsingJoinClause, UsingScanClause};
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::statement::SinglePartQuery;

#[test]
fn using_index_hint() {
    let n = node("Person").named("n");
    let hint = UsingIndexClause::new("n", "Person", "name");
    let cond = Condition::Comparison {
        left: Expression::from(prop("n", "name")),
        operator: ComparisonOp::Eq,
        right: lit("Alice"),
    };
    // Build manually with hint clause
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(rust_cypher_dsl::clauses::MatchClause::new(n)),
        Clause::UsingIndex(hint),
        Clause::Where(rust_cypher_dsl::clauses::WhereClause::new(cond)),
        Clause::Return(rust_cypher_dsl::clauses::ReturnClause::new(vec![name("n")])),
    ]));
    let rendered = stmt.render();
    assert!(rendered.contains("USING INDEX n:`Person`(name)"));
}

#[test]
fn using_index_seek_hint() {
    let n = node("Person").named("n");
    let hint = UsingIndexClause::seek("n", "Person", "name");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(rust_cypher_dsl::clauses::MatchClause::new(n)),
        Clause::UsingIndex(hint),
        Clause::Return(rust_cypher_dsl::clauses::ReturnClause::new(vec![name("n")])),
    ]));
    let rendered = stmt.render();
    assert!(rendered.contains("USING INDEX SEEK n:`Person`(name)"));
}

#[test]
fn using_scan_hint() {
    let n = node("Person").named("n");
    let hint = UsingScanClause::new("n", "Person");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(rust_cypher_dsl::clauses::MatchClause::new(n)),
        Clause::UsingScan(hint),
        Clause::Return(rust_cypher_dsl::clauses::ReturnClause::new(vec![name("n")])),
    ]));
    let rendered = stmt.render();
    assert!(rendered.contains("USING SCAN n:`Person`"));
}

#[test]
fn using_join_hint() {
    let n = node("Person").named("n");
    let hint = UsingJoinClause::new("n");
    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::Match(rust_cypher_dsl::clauses::MatchClause::new(n)),
        Clause::UsingJoin(hint),
        Clause::Return(rust_cypher_dsl::clauses::ReturnClause::new(vec![name("n")])),
    ]));
    let rendered = stmt.render();
    assert!(rendered.contains("USING JOIN ON n"));
}
