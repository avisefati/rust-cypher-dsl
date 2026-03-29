//! Schema operation integration tests (Req 5).
//!
//! Tests CREATE/DROP INDEX and CONSTRAINT commands against Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 5.1: CREATE INDEX, then SHOW INDEXES confirms it exists.
#[tokio::test]
async fn create_and_show_index() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::create_index("test_idx")
        .for_node("n", "Person", vec!["name"])
        .build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    // Verify the index appears in SHOW INDEXES.
    let mut result = graph
        .execute(neo4rs::query(
            "SHOW INDEXES YIELD name WHERE name = 'test_idx' RETURN name",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("name").unwrap(), "test_idx");
}

/// Req 5.2: DROP INDEX removes it from SHOW INDEXES.
#[tokio::test]
async fn drop_index() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create then drop.
    let create = Cypher::create_index("drop_me")
        .for_node("n", "Person", vec!["name"])
        .build();
    graph.run(neo4rs::query(&create.render())).await.unwrap();

    let drop = Cypher::drop_index("drop_me");
    graph.run(neo4rs::query(&drop.render())).await.unwrap();

    // Verify gone.
    let mut result = graph
        .execute(neo4rs::query(
            "SHOW INDEXES YIELD name WHERE name = 'drop_me' RETURN name",
        ))
        .await
        .unwrap();

    assert!(result.next().await.unwrap().is_none());
}

/// Req 5.3: CREATE INDEX IF NOT EXISTS twice does not error.
#[tokio::test]
async fn create_index_if_not_exists() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::create_index_if_not_exists("idempotent_idx")
        .for_node("n", "Person", vec!["name"])
        .build();

    let cypher = stmt.render();

    // Run twice — second invocation must not error.
    graph.run(neo4rs::query(&cypher)).await.unwrap();
    graph.run(neo4rs::query(&cypher)).await.unwrap();
}

/// Req 5.4: DROP INDEX IF EXISTS on a nonexistent index does not error.
#[tokio::test]
async fn drop_index_if_exists_nonexistent() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::drop_index_if_exists("does_not_exist");
    graph.run(neo4rs::query(&stmt.render())).await.unwrap();
}

/// Req 5.5: CREATE CONSTRAINT (UNIQUE), then SHOW CONSTRAINTS confirms it.
#[tokio::test]
async fn create_unique_constraint() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::create_constraint("unique_email")
        .for_node("n", "Person")
        .is_unique(vec!["email"]);

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "SHOW CONSTRAINTS YIELD name WHERE name = 'unique_email' RETURN name",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("name").unwrap(), "unique_email");
}

/// Req 5.6: Unique constraint violation is rejected by Neo4j.
#[tokio::test]
async fn unique_constraint_violation() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let constraint = Cypher::create_constraint("unique_name")
        .for_node("n", "Person")
        .is_unique(vec!["name"]);

    graph
        .run(neo4rs::query(&constraint.render()))
        .await
        .unwrap();

    // Insert first node — succeeds.
    graph
        .run(neo4rs::query("CREATE (:Person {name: 'Alice'})"))
        .await
        .unwrap();

    // Insert duplicate — must fail.
    let err = graph
        .run(neo4rs::query("CREATE (:Person {name: 'Alice'})"))
        .await;

    assert!(err.is_err(), "expected constraint violation error");
}

/// Req 5.7: DROP CONSTRAINT removes it from SHOW CONSTRAINTS.
#[tokio::test]
async fn drop_constraint() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let create = Cypher::create_constraint("to_drop")
        .for_node("n", "Person")
        .is_unique(vec!["name"]);

    graph.run(neo4rs::query(&create.render())).await.unwrap();

    let drop = Cypher::drop_constraint("to_drop");
    graph.run(neo4rs::query(&drop.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "SHOW CONSTRAINTS YIELD name WHERE name = 'to_drop' RETURN name",
        ))
        .await
        .unwrap();

    assert!(result.next().await.unwrap().is_none());
}
