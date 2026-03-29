//! Parameter binding integration tests (Req 4).
//!
//! Tests that DSL-generated `$param` references bind correctly when
//! passed through the `neo4rs` driver to Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 4.1: String parameter round-trips correctly.
#[tokio::test]
async fn param_string() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // CREATE a node with a parameterised string property via DSL.
    let create_stmt = Cypher::create(
        node("Item")
            .named("n")
            .with_properties(props! { "val" => param("v") }),
    )
    .build();

    graph
        .run(neo4rs::query(&create_stmt.render()).param("v", "hello"))
        .await
        .unwrap();

    // Read back via DSL.
    let read_stmt = Cypher::match_(node("Item").named("n"))
        .where_(prop("n", "val").eq(param("v")))
        .returning(prop("n", "val").alias("result"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&read_stmt.render()).param("v", "hello"))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("result").unwrap(), "hello");
}

/// Req 4.2: Integer parameter round-trips correctly.
#[tokio::test]
async fn param_integer() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let create_stmt = Cypher::create(
        node("Item")
            .named("n")
            .with_properties(props! { "val" => param("v") }),
    )
    .build();

    graph
        .run(neo4rs::query(&create_stmt.render()).param("v", 42_i64))
        .await
        .unwrap();

    let read_stmt = Cypher::match_(node("Item").named("n"))
        .returning(prop("n", "val").alias("result"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&read_stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("result").unwrap(), 42);
}

/// Req 4.3: Boolean parameter round-trips correctly.
#[tokio::test]
async fn param_boolean() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let create_stmt = Cypher::create(
        node("Item")
            .named("n")
            .with_properties(props! { "val" => param("v") }),
    )
    .build();

    graph
        .run(neo4rs::query(&create_stmt.render()).param("v", true))
        .await
        .unwrap();

    let read_stmt = Cypher::match_(node("Item").named("n"))
        .returning(prop("n", "val").alias("result"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&read_stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert!(row.get::<bool>("result").unwrap());
}

/// Req 4.4: List parameter via UNWIND processes each element.
#[tokio::test]
async fn param_list() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // UNWIND $items AS item CREATE (:Item {val: item})
    let stmt = Cypher::unwind(param("items"))
        .as_("item")
        .create(
            node("Item")
                .named("n")
                .with_properties(props! { "val" => name("item") }),
        )
        .build();

    graph
        .run(
            neo4rs::query(&stmt.render()).param("items", vec![10_i64, 20_i64, 30_i64]),
        )
        .await
        .unwrap();

    // Verify all three nodes were created.
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Item) RETURN n.val AS v ORDER BY v",
        ))
        .await
        .unwrap();

    let mut values: Vec<i64> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => values.push(row.get::<i64>("v").unwrap()),
            Ok(None) => break,
            Err(e) => panic!("failed to iterate param_list results: {e}"),
        }
    }

    assert_eq!(values, vec![10, 20, 30]);
}
