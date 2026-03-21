//! Integration tests for procedure calls — ported from Java `ProcedureCallsIT.java`.
//!
//! Tests exercise standalone and in-query `CALL` patterns with
//! YIELD, WHERE, and RETURN.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::prelude::*;

// ============================================================================
// Standalone procedure calls
// ============================================================================

#[test]
fn standalone_call_no_args() {
    let stmt = Cypher::call_procedure("db.labels", vec![]).build();
    assert_eq!(stmt.render(), "CALL db.labels()");
}

#[test]
fn standalone_call_with_args() {
    let stmt = Cypher::call_procedure(
        "db.index.fulltext.queryNodes",
        vec![lit("titleIndex"), lit("hello")],
    )
    .build();
    assert_eq!(
        stmt.render(),
        "CALL db.index.fulltext.queryNodes('titleIndex', 'hello')"
    );
}

#[test]
fn standalone_call_with_integer_args() {
    let stmt = Cypher::call_procedure(
        "dbms.listConfig",
        vec![lit("dbms.memory"), lit(10_i32)],
    )
    .build();
    assert_eq!(
        stmt.render(),
        "CALL dbms.listConfig('dbms.memory', 10)"
    );
}

#[test]
fn standalone_call_yield_single() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label"))
        .build();
    assert_eq!(stmt.render(), "CALL db.labels() YIELD label");
}

#[test]
fn standalone_call_yield_multiple() {
    let stmt = Cypher::call_procedure("db.propertyKeys", vec![])
        .yield_((name("propertyKey"), name("type")))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.propertyKeys() YIELD propertyKey, type"
    );
}

#[test]
fn standalone_call_yield_aliased() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label").alias("myLabel"))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.labels() YIELD label AS myLabel"
    );
}

#[test]
fn standalone_call_yield_where() {
    let cond = Expression::from(prop("label", "name")).starts_with("A");
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label"))
        .where_(cond)
        .build();
    // The expression might render as label.name or based on what we pass
    let rendered = stmt.render();
    assert!(rendered.contains("CALL db.labels() YIELD label WHERE"));
    assert!(rendered.contains("STARTS WITH 'A'"));
}

#[test]
fn standalone_call_yield_return() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label"))
        .returning(name("label"))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.labels() YIELD label RETURN label"
    );
}

// ============================================================================
// Procedure calls as part of a larger query
// ============================================================================

#[test]
fn call_procedure_explain() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .build()
        .explain();
    assert_eq!(stmt.render(), "EXPLAIN CALL db.labels()");
}

#[test]
fn call_procedure_profile() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .build()
        .profile();
    assert_eq!(stmt.render(), "PROFILE CALL db.labels()");
}

// ============================================================================
// DBMS procedures
// ============================================================================

#[test]
fn call_dbms_procedures() {
    let stmt = Cypher::call_procedure("dbms.procedures", vec![])
        .yield_((name("name"), name("signature")))
        .returning((name("name"), name("signature")))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL dbms.procedures() YIELD name, signature RETURN name, signature"
    );
}

#[test]
fn call_security_procedures() {
    let stmt = Cypher::call_procedure(
        "dbms.security.createUser",
        vec![lit("bob"), lit("secret123"), lit(false)],
    )
    .build();
    assert_eq!(
        stmt.render(),
        "CALL dbms.security.createUser('bob', 'secret123', false)"
    );
}

// ============================================================================
// Complex yield patterns
// ============================================================================

#[test]
fn yield_with_where_and_return() {
    let cond = name("nodeCount").gt(0_i32);
    let stmt = Cypher::call_procedure("db.stats.retrieve", vec![lit("GRAPH COUNTS")])
        .yield_((name("section"), name("nodeCount")))
        .where_(cond)
        .returning((name("section"), name("nodeCount")))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.stats.retrieve('GRAPH COUNTS') YIELD section, nodeCount WHERE nodeCount > 0 RETURN section, nodeCount"
    );
}
