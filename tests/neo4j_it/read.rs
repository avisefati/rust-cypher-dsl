//! Read query integration tests (Req 2).
//!
//! Tests MATCH, WHERE, OPTIONAL MATCH, ORDER BY, SKIP, LIMIT,
//! relationship patterns, and variable-length paths against Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 2.1: MATCH by label returns only nodes with that label.
#[tokio::test]
async fn match_by_label() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create two nodes with different labels.
    graph
        .run(neo4rs::query(
            "CREATE (:Person {name: 'Alice'}), (:Animal {name: 'Rex'})",
        ))
        .await
        .unwrap();

    // MATCH only Person nodes via DSL.
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let mut names: Vec<String> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => {
                let n: neo4rs::Node = row.get("n").unwrap();
                names.push(n.get::<String>("name").unwrap());
            }
            Ok(None) => break,
            Err(e) => panic!("failed to iterate match_by_label results: {e}"),
        }
    }

    assert_eq!(names, vec!["Alice"]);
}

/// Req 2.2: MATCH with WHERE property filter returns correct subset.
#[tokio::test]
async fn match_where_property() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Person {name: 'Alice', age: 30}), (:Person {name: 'Bob', age: 20})",
        ))
        .await
        .unwrap();

    // Match persons older than 25.
    let stmt = Cypher::match_(node("Person").named("n"))
        .where_(prop("n", "age").gt(lit(25_i64)))
        .returning(prop("n", "name").alias("result_name"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let found_name: String = row.get("result_name").unwrap();
    assert_eq!(found_name, "Alice");

    // No more rows.
    assert!(result.next().await.unwrap().is_none());
}

/// Req 2.3: OPTIONAL MATCH on a non-existent pattern yields null.
#[tokio::test]
async fn optional_match_missing() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create a Person with no relationships.
    graph
        .run(neo4rs::query("CREATE (:Person {name: 'Alice'})"))
        .await
        .unwrap();

    // OPTIONAL MATCH a relationship that does not exist.
    // Return b IS NULL as a boolean so we assert null directly
    // rather than relying on driver decode behavior.
    let a = node("Person").named("a");
    let b = any_node_named("b");
    let stmt = Cypher::match_(a.clone())
        .optional_match(a.rel(rel("KNOWS")).to(b))
        .returning((
            prop("a", "name").alias("a_name"),
            raw_unchecked("b IS NULL").alias("b_is_null"),
        ))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("a_name").unwrap(), "Alice");
    assert!(row.get::<bool>("b_is_null").unwrap());
}

/// Req 2.4: ORDER BY + SKIP + LIMIT returns correct subset in order.
#[tokio::test]
async fn order_by_skip_limit() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create 5 nodes with sequential values.
    graph
        .run(neo4rs::query(
            "UNWIND [1, 2, 3, 4, 5] AS i CREATE (:Item {val: i})",
        ))
        .await
        .unwrap();

    // ORDER BY val ASC, SKIP 1, LIMIT 2 → expect [2, 3].
    let stmt = Cypher::match_(node("Item").named("n"))
        .returning(prop("n", "val").alias("v"))
        .order_by(Expression::from(prop("n", "val")).ascending())
        .skip(1_i64)
        .limit(2_i64)
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let mut values: Vec<i64> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => values.push(row.get::<i64>("v").unwrap()),
            Ok(None) => break,
            Err(e) => panic!("failed to iterate order_by_skip_limit results: {e}"),
        }
    }

    assert_eq!(values, vec![2, 3]);
}

/// Req 2.5: MATCH relationship pattern returns connected nodes.
#[tokio::test]
async fn match_relationship() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
        ))
        .await
        .unwrap();

    // Match the KNOWS relationship via DSL.
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let pattern = a.rel(rel("KNOWS")).to(b);

    let stmt = Cypher::match_(pattern)
        .returning((
            prop("a", "name").alias("from"),
            prop("b", "name").alias("to"),
        ))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("from").unwrap(), "Alice");
    assert_eq!(row.get::<String>("to").unwrap(), "Bob");
}

/// Req 2.6: Variable-length path matches paths within bounds.
#[tokio::test]
async fn match_variable_length_path() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create a chain: a->b->c->d.
    graph
        .run(neo4rs::query(
            "CREATE (a:Node {name: 'a'})-[:R]->(b:Node {name: 'b'})-[:R]->(c:Node {name: 'c'})-[:R]->(d:Node {name: 'd'})",
        ))
        .await
        .unwrap();

    // Match paths of length 2..3 starting from 'a'.
    let start = node("Node").named("start");
    let finish = node("Node").named("finish");
    let pattern = start.rel(rel("R").min(2).max(3)).to(finish);

    let stmt = Cypher::match_(pattern)
        .where_(prop("start", "name").eq(lit("a")))
        .returning(prop("finish", "name").alias("end_name"))
        .order_by(Expression::from(prop("finish", "name")).ascending())
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let mut end_names: Vec<String> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => end_names.push(row.get::<String>("end_name").unwrap()),
            Ok(None) => break,
            Err(e) => panic!("failed to iterate variable_length_path results: {e}"),
        }
    }

    // From 'a', paths of length 2 reach 'c', paths of length 3 reach 'd'.
    assert_eq!(end_names, vec!["c", "d"]);
}
