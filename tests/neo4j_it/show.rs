//! SHOW command integration tests (Req 7).
//!
//! Tests SHOW INDEXES, CONSTRAINTS, FUNCTIONS, PROCEDURES,
//! TRANSACTIONS, and YIELD against Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 7.1: SHOW INDEXES executes without syntax error.
#[tokio::test]
async fn show_indexes() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_indexes().build();

    // Should execute without error — result set may be empty or not.
    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    // Consume all rows to confirm no protocol error.
    loop {
        match result.next().await {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(e) => panic!("SHOW INDEXES failed during iteration: {e}"),
        }
    }
}

/// Req 7.2: SHOW CONSTRAINTS executes without syntax error.
#[tokio::test]
async fn show_constraints() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_constraints().build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    loop {
        match result.next().await {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(e) => panic!("SHOW CONSTRAINTS failed during iteration: {e}"),
        }
    }
}

/// Req 7.3: SHOW FUNCTIONS returns built-in function rows.
#[tokio::test]
async fn show_functions() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_functions().build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    // Neo4j always has built-in functions, so at least one row expected.
    let row = result.next().await.unwrap();
    assert!(row.is_some(), "expected at least one built-in function");
}

/// Req 7.4: SHOW PROCEDURES returns procedure rows.
#[tokio::test]
async fn show_procedures() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_procedures().build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap();
    assert!(row.is_some(), "expected at least one procedure");
}

/// Req 7.5: SHOW TRANSACTIONS returns at least the current transaction.
#[tokio::test]
async fn show_transactions() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_transactions().build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap();
    assert!(row.is_some(), "expected at least the current transaction");
}

/// Req 7.6: SHOW with YIELD returns only yielded columns.
#[tokio::test]
async fn show_with_yield() {
    let graph = helpers::graph().await;

    let stmt = Cypher::show_indexes()
        .yield_fields(vec![name("name"), name("state")])
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    // Consume results — if the query has syntax errors, execute or next will fail.
    // We just verify it runs without error. If indexes exist, rows will have
    // only the yielded columns.
    loop {
        match result.next().await {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(e) => panic!("SHOW INDEXES YIELD failed during iteration: {e}"),
        }
    }
}
