//! Backtick-escaping integration tests (Req 8).
//!
//! Tests that reserved Cypher keywords used as property names, labels,
//! and relationship types are correctly backtick-escaped by the DSL
//! and round-trip through Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 8.1: Property named with a reserved keyword round-trips.
#[tokio::test]
async fn reserved_keyword_property_roundtrip() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // The DSL should render `MATCH` as a backtick-escaped property.
    let stmt = Cypher::create(
        node("Thing")
            .named("n")
            .with_properties(props! { "MATCH" => param("v") }),
    )
    .build();

    graph
        .run(neo4rs::query(&stmt.render()).param("v", "yes"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query("MATCH (n:Thing) RETURN n.`MATCH` AS val"))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("val").unwrap(), "yes");
}

/// Req 8.2: Label that is a reserved keyword is accepted by Neo4j.
#[tokio::test]
async fn reserved_keyword_label() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // `INDEX` is a reserved keyword — DSL should backtick-escape it.
    let stmt = Cypher::create(
        node("INDEX")
            .named("n")
            .with_properties(props! { "id" => lit(1_i64) }),
    )
    .build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query("MATCH (n:`INDEX`) RETURN n.id AS id"))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("id").unwrap(), 1);
}

/// Req 8.3: Relationship type that is a reserved keyword is accepted.
#[tokio::test]
async fn reserved_keyword_rel_type() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // `RETURN` is a reserved keyword.
    let a = node("A").named("a");
    let b = node("B").named("b");
    let pattern = a.rel(rel("RETURN")).to(b);

    let stmt = Cypher::create(pattern).build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (a)-[r:`RETURN`]->(b) RETURN type(r) AS t",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("t").unwrap(), "RETURN");
}

/// Req 8.4: CREATE INDEX on a property that is a reserved keyword.
#[tokio::test]
async fn reserved_keyword_index_property() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // `ORDER` is a reserved keyword.
    let stmt = Cypher::create_index("kw_idx")
        .for_node("n", "Label", vec!["ORDER"])
        .build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "SHOW INDEXES YIELD name WHERE name = 'kw_idx' RETURN name",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("name").unwrap(), "kw_idx");
}
