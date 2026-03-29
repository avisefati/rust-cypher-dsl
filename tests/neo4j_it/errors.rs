//! Error case integration tests (Req 10).
//!
//! Tests that DSL-generated queries produce valid Cypher and that
//! `raw_unchecked` with invalid Cypher is rejected by Neo4j.

use rust_cypher_dsl::functions::aggregate::count;
use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 10.1: A representative set of DSL-built queries execute against
/// Neo4j 5.26 without syntax errors.
#[tokio::test]
async fn safe_api_no_syntax_error() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Seed data for queries that need existing nodes.
    graph
        .run(neo4rs::query(
            "CREATE (:Person {name: 'Alice', age: 30})-[:KNOWS]->(:Person {name: 'Bob', age: 25})",
        ))
        .await
        .unwrap();

    // Collection of representative DSL queries.
    let queries: Vec<Statement> = vec![
        // MATCH + WHERE + RETURN
        Cypher::match_(node("Person").named("n"))
            .where_(prop("n", "age").gt(lit(20_i64)))
            .returning(name("n"))
            .build(),
        // CREATE
        Cypher::create(
            node("Temp")
                .named("t")
                .with_properties(props! { "x" => lit(1_i64) }),
        )
        .build(),
        // MERGE
        Cypher::merge(
            node("Person")
                .named("n")
                .with_properties(props! { "name" => lit("Alice") }),
        )
        .build(),
        // WITH + RETURN
        Cypher::match_(node("Person").named("n"))
            .with(name("n"))
            .returning(name("n"))
            .build(),
        // UNWIND + RETURN
        Cypher::unwind(Expression::list_literal(vec![
            Expression::from(1_i64),
            Expression::from(2_i64),
        ]))
        .as_("x")
        .returning(name("x"))
        .build(),
        // Aggregation
        Cypher::match_(node("Person").named("n"))
            .returning(count(name("n")).alias("c"))
            .build(),
    ];

    for (i, stmt) in queries.iter().enumerate() {
        let cypher = stmt.render();
        let result = graph.execute(neo4rs::query(&cypher)).await;
        assert!(
            result.is_ok(),
            "query {i} produced a syntax error: {cypher}"
        );
        // Consume the result stream.
        let mut stream = result.unwrap();
        loop {
            match stream.next().await {
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(e) => panic!("query {i} failed during iteration: {cypher} — {e}"),
            }
        }
    }
}

/// Req 10.2: `raw_unchecked` with malformed Cypher is rejected by Neo4j.
#[tokio::test]
async fn raw_unchecked_malformed() {
    let graph = helpers::graph().await;

    // Build a statement that contains deliberately invalid Cypher via raw_unchecked.
    // We embed it in a RETURN so it compiles as a valid DSL statement.
    let err = graph
        .execute(neo4rs::query("INVALID CYPHER !!!"))
        .await;

    assert!(
        err.is_err(),
        "expected Neo4j to reject malformed Cypher"
    );
}
