//! Neo4j integration tests for the Cypher DSL.
//!
//! These tests execute DSL-generated Cypher against a real Neo4j 5.26 instance
//! managed automatically via Docker (`testcontainers`).
//!
//! # Running
//!
//! ```bash
//! cargo test --features neo4j-tests -- --test-threads=1
//! ```
//!
//! `--test-threads=1` is **required** because all tests share a single Neo4j
//! database and must run serially to avoid data races.
//!
//! Docker must be running. If it is not, the test binary will fail fast at
//! container startup.
#![cfg(feature = "neo4j-tests")]
#![expect(clippy::unwrap_used, reason = "integration tests - panics are the failure mode")]
#![expect(clippy::expect_used, reason = "integration tests use expect for clear failure messages")]
#![expect(clippy::panic, reason = "clean_db uses explicit panic on iteration errors")]

mod helpers;
mod read;
mod write;

#[tokio::test]
async fn smoke_return_one() {
    let graph = helpers::graph().await;
    helpers::clean_db(graph).await;

    let mut result = graph
        .execute(neo4rs::query("RETURN 1 AS n"))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("n").unwrap();
    assert_eq!(value, 1);
}
