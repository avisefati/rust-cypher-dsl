//! Shared test infrastructure for Neo4j integration tests.
//!
//! Provides a lazily-initialized Neo4j Docker container and Bolt driver
//! that is shared across all tests in this binary. Also provides a
//! `clean_db()` helper for test isolation.

use neo4j_testcontainers::prelude::*;
use neo4j_testcontainers::{runners::AsyncRunner, Neo4j, Neo4jImage};
use neo4rs::Graph;
use testcontainers::ContainerAsync;
use tokio::sync::OnceCell;

/// Pinned Neo4j image version to avoid CI drift.
const NEO4J_VERSION: &str = "5.26";

/// Container + Graph, lazily initialized once per test binary.
struct TestEnv {
    /// Kept alive so the container is not dropped until the process exits.
    _container: ContainerAsync<Neo4jImage>,
    graph: Graph,
}

/// Single shared environment for the entire test binary.
static ENV: OnceCell<TestEnv> = OnceCell::const_new();

/// Starts a Neo4j container and connects via Bolt.
///
/// # Panics
///
/// Panics if Docker is not running or the connection fails.
async fn init_env() -> TestEnv {
    let container = Neo4j::from_env()
        .with_version(NEO4J_VERSION)
        .start()
        .await;

    let uri = container.image().bolt_uri_ipv4();
    let user = container
        .image()
        .user()
        .expect("Neo4j user must be set")
        .to_owned();
    let pass = container
        .image()
        .password()
        .expect("Neo4j password must be set")
        .to_owned();

    let graph = Graph::new(uri, user, pass)
        .await
        .expect("failed to connect to Neo4j");

    TestEnv {
        _container: container,
        graph,
    }
}

/// Returns a connected [`Graph`] backed by the shared Neo4j container.
///
/// The container is started lazily on first call and reused for all
/// subsequent calls within this test binary.
pub async fn graph() -> &'static Graph {
    &ENV.get_or_init(init_env).await.graph
}

/// Clears all data and schema from the database for test isolation.
///
/// Drops all constraints, then all user-created indexes, then deletes
/// all nodes and relationships. Must be called at the start of each
/// test to ensure a clean slate.
///
/// # Panics
///
/// Panics if any cleanup query fails.
pub async fn clean_db(graph: &Graph) {
    // Drop all constraints first (some block index deletion).
    let mut result = graph
        .execute(neo4rs::query("SHOW CONSTRAINTS YIELD name"))
        .await
        .unwrap();
    loop {
        match result.next().await {
            Ok(Some(row)) => {
                let name: String = row.get("name").unwrap();
                graph
                    .run(neo4rs::query(&format!(
                        "DROP CONSTRAINT `{name}` IF EXISTS"
                    )))
                    .await
                    .unwrap();
            }
            Ok(None) => break,
            Err(e) => panic!("failed to iterate SHOW CONSTRAINTS: {e}"),
        }
    }

    // Drop all user-created indexes (skip internal LOOKUP indexes).
    let mut result = graph
        .execute(neo4rs::query(
            "SHOW INDEXES YIELD name, type WHERE type <> 'LOOKUP' RETURN name",
        ))
        .await
        .unwrap();
    loop {
        match result.next().await {
            Ok(Some(row)) => {
                let name: String = row.get("name").unwrap();
                graph
                    .run(neo4rs::query(&format!("DROP INDEX `{name}` IF EXISTS")))
                    .await
                    .unwrap();
            }
            Ok(None) => break,
            Err(e) => panic!("failed to iterate SHOW INDEXES: {e}"),
        }
    }

    // Delete all nodes and relationships.
    graph
        .run(neo4rs::query("MATCH (n) DETACH DELETE n"))
        .await
        .unwrap();
}
