//! Write query integration tests (Req 3).
//!
//! Tests CREATE, MERGE, SET, DELETE, and DETACH DELETE against Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 3.1: CREATE node with properties, read back, assert labels + properties.
#[tokio::test]
async fn create_node_with_properties() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::create(
        node("Person")
            .named("n")
            .with_properties(props! { "name" => param("name"), "age" => param("age") }),
    )
    .build();

    graph
        .run(
            neo4rs::query(&stmt.render())
                .param("name", "Alice")
                .param("age", 30_i64),
        )
        .await
        .unwrap();

    // Read back via raw Cypher to verify independently.
    let mut result = graph
        .execute(neo4rs::query("MATCH (n:Person) RETURN n"))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let n: neo4rs::Node = row.get("n").unwrap();
    assert!(n.labels().contains(&"Person"));
    assert_eq!(n.get::<String>("name").unwrap(), "Alice");
    assert_eq!(n.get::<i64>("age").unwrap(), 30);
    assert!(result.next().await.unwrap().is_none());
}

/// Req 3.2: CREATE relationship, read back, assert type + properties.
#[tokio::test]
async fn create_relationship() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let a = node("Person").named("a").with_properties(props! { "name" => lit("Alice") });
    let b = node("Person").named("b").with_properties(props! { "name" => lit("Bob") });
    let pattern = a.rel(rel("KNOWS").named("r").with_properties(props! { "since" => lit(2020_i64) })).to(b);

    let stmt = Cypher::create(pattern).build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    // Read back.
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (a)-[r:KNOWS]->(b) RETURN type(r) AS t, r.since AS since, a.name AS a_name, b.name AS b_name",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("t").unwrap(), "KNOWS");
    assert_eq!(row.get::<i64>("since").unwrap(), 2020);
    assert_eq!(row.get::<String>("a_name").unwrap(), "Alice");
    assert_eq!(row.get::<String>("b_name").unwrap(), "Bob");
}

/// Req 3.3: MERGE creates when the node is missing.
#[tokio::test]
async fn merge_creates_when_missing() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::merge(
        node("Person")
            .named("n")
            .with_properties(props! { "name" => param("name") }),
    )
    .build();

    graph
        .run(neo4rs::query(&stmt.render()).param("name", "Alice"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query("MATCH (n:Person) RETURN count(n) AS c"))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 1);
}

/// Req 3.4: MERGE matches when node already exists — no duplication.
#[tokio::test]
async fn merge_matches_when_existing() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::merge(
        node("Person")
            .named("n")
            .with_properties(props! { "name" => param("name") }),
    )
    .build();

    let query_str = stmt.render();

    // Run MERGE twice with the same data.
    graph
        .run(neo4rs::query(&query_str).param("name", "Alice"))
        .await
        .unwrap();
    graph
        .run(neo4rs::query(&query_str).param("name", "Alice"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query("MATCH (n:Person) RETURN count(n) AS c"))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 1);
}

/// Req 3.5: MERGE ON CREATE / ON MATCH fires the correct branch.
#[tokio::test]
async fn merge_on_create_on_match() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => param("name") });

    let stmt = Cypher::merge(n)
        .on_create(SetItem::property(
            Property::new(name("n"), "source"),
            lit("created"),
        ))
        .on_match(SetItem::property(
            Property::new(name("n"), "source"),
            lit("matched"),
        ))
        .build();

    let query_str = stmt.render();

    // First run — ON CREATE fires.
    graph
        .run(neo4rs::query(&query_str).param("name", "Alice"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Person {name: 'Alice'}) RETURN n.source AS s",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("s").unwrap(), "created");

    // Second run — ON MATCH fires.
    graph
        .run(neo4rs::query(&query_str).param("name", "Alice"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Person {name: 'Alice'}) RETURN n.source AS s",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("s").unwrap(), "matched");
}

/// Req 3.6: SET property updates a value on an existing node.
#[tokio::test]
async fn set_property() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query("CREATE (:Person {name: 'Alice', age: 30})"))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Person").named("n"))
        .where_(prop("n", "name").eq(param("name")))
        .set(SetItem::property(
            Property::new(name("n"), "age"),
            param("newAge"),
        ))
        .build();

    graph
        .run(
            neo4rs::query(&stmt.render())
                .param("name", "Alice")
                .param("newAge", 31_i64),
        )
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Person {name: 'Alice'}) RETURN n.age AS age",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("age").unwrap(), 31);
}

/// Req 3.7: DELETE removes a node.
#[tokio::test]
async fn delete_node() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query("CREATE (:Temp {name: 'gone'})"))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Temp").named("n"))
        .delete(name("n"))
        .build();

    graph.run(neo4rs::query(&stmt.render())).await.unwrap();

    let mut result = graph
        .execute(neo4rs::query("MATCH (n:Temp) RETURN count(n) AS c"))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 0);
}

/// Req 3.8: DETACH DELETE removes a node and its relationships.
#[tokio::test]
async fn detach_delete() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
        ))
        .await
        .unwrap();

    // DETACH DELETE Alice — should remove Alice and the KNOWS relationship.
    let stmt = Cypher::match_(node("Person").named("n"))
        .where_(prop("n", "name").eq(param("name")))
        .detach_delete(name("n"))
        .build();

    graph
        .run(neo4rs::query(&stmt.render()).param("name", "Alice"))
        .await
        .unwrap();

    // Alice is gone.
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Person {name: 'Alice'}) RETURN count(n) AS c",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 0);

    // Bob still exists but has no relationships.
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Person {name: 'Bob'}) OPTIONAL MATCH (n)-[r]-() RETURN count(n) AS node_count, count(r) AS rel_count",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("node_count").unwrap(), 1);
    assert_eq!(row.get::<i64>("rel_count").unwrap(), 0);
}
