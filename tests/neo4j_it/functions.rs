//! Aggregation and function integration tests (Req 6).
//!
//! Tests count, collect, sum, avg, min, max, string functions,
//! and `count_distinct` against Neo4j.

use rust_cypher_dsl::functions::aggregate::{avg, collect, count, count_distinct, max, min, sum};
use rust_cypher_dsl::functions::string::{substring, to_lower, to_upper};
use rust_cypher_dsl::prelude::*;

use crate::helpers;

/// Req 6.1: `count()` returns the correct node count.
#[tokio::test]
async fn count_nodes() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "UNWIND [1, 2, 3] AS i CREATE (:Item {val: i})",
        ))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Item").named("n"))
        .returning(count(name("n")).alias("c"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 3);
}

/// Req 6.2: `collect()` returns a list of values.
#[tokio::test]
async fn collect_values() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Item {name: 'a'}), (:Item {name: 'b'}), (:Item {name: 'c'})",
        ))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Item").named("n"))
        .returning(collect(prop("n", "name")).alias("names"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    let mut names: Vec<String> = row.get("names").unwrap();
    names.sort();
    assert_eq!(names, vec!["a", "b", "c"]);
}

/// Req 6.3: sum, avg, min, max return correct aggregates.
#[tokio::test]
async fn sum_avg_min_max() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "UNWIND [10, 20, 30] AS v CREATE (:Item {val: v})",
        ))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Item").named("n"))
        .returning((
            sum(prop("n", "val")).alias("s"),
            avg(prop("n", "val")).alias("a"),
            min(prop("n", "val")).alias("lo"),
            max(prop("n", "val")).alias("hi"),
        ))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("s").unwrap(), 60);
    // avg returns a float in Neo4j.
    let avg_val: f64 = row.get("a").unwrap();
    assert!((avg_val - 20.0).abs() < f64::EPSILON);
    assert_eq!(row.get::<i64>("lo").unwrap(), 10);
    assert_eq!(row.get::<i64>("hi").unwrap(), 30);
}

/// Req 6.4: `to_lower`, `to_upper`, `substring` return correct strings.
#[tokio::test]
async fn string_functions() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let stmt = Cypher::match_(node("Dummy").named("n"))
        .returning((
            to_lower(lit("HELLO")).alias("low"),
            to_upper(lit("hello")).alias("up"),
            substring(lit("hello world"), lit(6_i64), None).alias("sub"),
        ))
        .build();

    // The MATCH will return no rows if there are no Dummy nodes,
    // so use a raw RETURN instead to test the functions directly.
    let mut result = graph
        .execute(neo4rs::query(
            "RETURN toLower('HELLO') AS low, toUpper('hello') AS up, substring('hello world', 6) AS sub",
        ))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("low").unwrap(), "hello");
    assert_eq!(row.get::<String>("up").unwrap(), "HELLO");
    assert_eq!(row.get::<String>("sub").unwrap(), "world");

    // Also verify the DSL renders valid Cypher by executing it with a node present.
    graph
        .run(neo4rs::query("CREATE (:Dummy)"))
        .await
        .unwrap();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>("low").unwrap(), "hello");
    assert_eq!(row.get::<String>("up").unwrap(), "HELLO");
    assert_eq!(row.get::<String>("sub").unwrap(), "world");
}

/// Req 6.5: `count_distinct` excludes duplicates.
#[tokio::test]
async fn count_distinct_test() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    graph
        .run(neo4rs::query(
            "CREATE (:Item {color: 'red'}), (:Item {color: 'red'}), (:Item {color: 'blue'})",
        ))
        .await
        .unwrap();

    let stmt = Cypher::match_(node("Item").named("n"))
        .returning(count_distinct(prop("n", "color")).alias("c"))
        .build();

    let mut result = graph
        .execute(neo4rs::query(&stmt.render()))
        .await
        .unwrap();

    let row = result.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("c").unwrap(), 2);
}
