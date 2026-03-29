//! Complex pattern integration tests (Req 9).
//!
//! Tests multi-hop relationships, variable-length paths,
//! UNWIND, and FOREACH against Neo4j.

use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 9.1: Multi-hop pattern matches the correct end node.
#[tokio::test]
async fn multi_hop_pattern() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Node {name: 'a'})-[:R]->(:Node {name: 'b'})-[:R]->(:Node {name: 'c'})",
        ))
        .await
        .unwrap();

    // Match a 2-hop pattern: a->b->c via DSL chain.
    let a = node("Node").named("a");
    let b = node("Node").named("b");
    let c = node("Node").named("c");
    let pattern = a.rel(rel("R")).to(b).rel(rel("R")).to(c);

    let stmt = Cypher::match_(pattern)
        .where_(prop("a", "name").eq(lit("a")))
        .returning(prop("c", "name").alias("end_name"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("end_name").unwrap(), "c");
}

/// Req 9.2: Variable-length path with bounds returns correct path count.
#[tokio::test]
async fn variable_length_path() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // Create a chain of 5 nodes: n1->n2->n3->n4->n5.
    graph
        .run(neo4rs::query(
            "CREATE (:N {i: 1})-[:R]->(:N {i: 2})-[:R]->(:N {i: 3})-[:R]->(:N {i: 4})-[:R]->(:N {i: 5})",
        ))
        .await
        .unwrap();

    // Match paths of length 1..3 from node 1.
    let start = node("N").named("s");
    let finish = node("N").named("f");
    let pattern = start.rel(rel("R").min(1).max(3)).to(finish);

    let stmt = Cypher::match_(pattern)
        .where_(prop("s", "i").eq(lit(1_i64)))
        .returning(prop("f", "i").alias("end_i"))
        .order_by(Expression::from(prop("f", "i")).ascending())
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let mut end_values: Vec<i64> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => end_values.push(row.get::<i64>("end_i").unwrap()),
            Ok(None) => break,
            Err(e) => panic!("failed to iterate variable_length_path results: {e}"),
        }
    }

    // Length 1 → node 2, length 2 → node 3, length 3 → node 4.
    assert_eq!(end_values, vec![2, 3, 4]);
}

/// Req 9.3: UNWIND a list and CREATE nodes from each element.
#[tokio::test]
async fn unwind_list() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::unwind(param("items"))
        .as_("x")
        .create(
            node("Created")
                .named("n")
                .with_properties(props! { "val" => name("x") }),
        )
        .build();

    graph
        .run(neo4rs::query(&stmt.render()).param("items", vec![100_i64, 200_i64, 300_i64]))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Created) RETURN n.val AS v ORDER BY v",
        ))
        .await
        .unwrap();

    let mut values: Vec<i64> = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => values.push(row.get::<i64>("v").unwrap()),
            Ok(None) => break,
            Err(e) => panic!("failed to iterate unwind_list results: {e}"),
        }
    }

    assert_eq!(values, vec![100, 200, 300]);
}

/// Req 9.4: FOREACH over a list creates nodes.
#[tokio::test]
async fn foreach_create() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    // We need an anchor node for the MATCH clause.
    graph
        .run(neo4rs::query("CREATE (:Anchor)"))
        .await
        .unwrap();

    // MATCH (anchor) FOREACH (x IN [1,2,3] | CREATE (:Created {val: x}))
    let anchor = node("Anchor").named("anchor");
    let created = node("Created")
        .named("c")
        .with_properties(props! { "val" => name("x") });

    let stmt = Cypher::match_(anchor)
        .foreach(
            "x",
            param("items"),
            vec![Clause::Create(
                rust_cypher_dsl::clauses::CreateClause::new(created),
            )],
        )
        .build();

    graph
        .run(neo4rs::query(&stmt.render()).param("items", vec![1_i64, 2_i64, 3_i64]))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (n:Created) RETURN count(n) AS c",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 3);
}
