# Design: Neo4j Integration Tests

## 1. Overview

Add end-to-end integration tests that execute DSL-generated Cypher against a real Neo4j 5.26 instance. A Docker container is managed automatically via `testcontainers`, queries are sent through the `neo4rs` Bolt driver, and results are asserted in Rust.

Everything is gated behind a Cargo feature flag (`neo4j-tests`) so the existing `cargo test` workflow is unaffected. Tests are opt-in for maintainers. When the feature is enabled but Docker is unavailable, the test binary fails fast at container startup.

---

## 2. Dependencies

New dependencies are **dev-only**. Cargo does not support `optional = true` on dev-dependencies, so they are unconditional `[dev-dependencies]`. The feature flag gates the test **source code** (via `#![cfg(feature = "neo4j-tests")]`), not the dependency resolution. When `neo4j-tests` is not enabled, the test binary is skipped entirely by the cfg gate so these crates are never linked, though they may be downloaded/compiled as part of the dependency graph.

```toml
[features]
neo4j-tests = []

[dev-dependencies]
neo4j_testcontainers = "0.3"
neo4rs = "0.8"
tokio = { version = "1", features = ["full"] }
```

### Why these crates

| Crate | Role | Notes |
|-------|------|-------|
| `neo4j_testcontainers` | Starts/stops Neo4j Docker container | Re-exports `testcontainers` and `testcontainers-modules`; provides `bolt_uri_ipv4()`, `user()`, `password()` helpers. Default image: Neo4j 5, user `neo4j`, password `neo`. We pin the image tag via `Neo4j::from_env()` with `NEO4J_VERSION_TAG=5.26` or by constructing with an explicit tag. |
| `neo4rs` | Bolt protocol driver | Async, Tokio-based. `Graph::new(uri, user, pass)`, `graph.execute(query(...).param(...))`, row extraction via `row.get::<T>(col)`. Supports Node, Relation, Path, and primitive types. |
| `tokio` | Async runtime | Required by both `neo4rs` and `testcontainers` async runners. |

### Why `neo4j_testcontainers` over raw `testcontainers-modules`

`neo4j_testcontainers` wraps the community Neo4j module and adds convenience methods (`bolt_uri_ipv4()`, `user()`, `password()`) that avoid manual port-mapping boilerplate. It re-exports everything needed so we don't depend on `testcontainers` or `testcontainers-modules` directly.

---

## 3. Architecture

```
tests/
  neo4j_it/
    main.rs            <-- cfg gate, mod declarations, shared container init
    helpers.rs         <-- shared container setup + Graph + clean_db() + test lock
    read.rs            <-- read query tests (Req 2)
    write.rs           <-- write query tests (Req 3)
    params.rs          <-- parameter binding tests (Req 4)
    schema.rs          <-- index/constraint tests (Req 5)
    functions.rs       <-- aggregation + built-in function tests (Req 6)
    show.rs            <-- SHOW command tests (Req 7)
    escaping.rs        <-- reserved keyword round-trip tests (Req 8)
    patterns.rs        <-- complex pattern tests (Req 9)
    errors.rs          <-- error case tests (Req 10)
```

Using a directory-based test binary (`tests/neo4j_it/main.rs`) instead of separate
top-level `.rs` files ensures all modules compile into **one test binary**. This is
critical because each top-level `tests/*.rs` file becomes its own binary, which would
start a separate Neo4j container per file. The directory approach lets all submodules
share the single container (serialized via `--test-threads=1`).

### 3.1 Shared Container (one per test binary)

Neo4j takes 15-30s to start. Spinning up per-test is unacceptable. We use `tokio::sync::OnceCell` to lazily create one container + driver that lives for the entire test binary:

```rust
// tests/neo4j_it/helpers.rs

use neo4j_testcontainers::Neo4j;
use neo4rs::Graph;
use tokio::sync::OnceCell;

/// Container + Graph, lazily initialized once.
struct TestEnv {
    /// Kept alive so the container isn't dropped.
    _container: neo4j_testcontainers::ContainerAsync<Neo4j>,
    graph: Graph,
}

static ENV: OnceCell<TestEnv> = OnceCell::const_new();

async fn init_env() -> TestEnv {
    use neo4j_testcontainers::runners::AsyncRunner;
    let container = Neo4j::default()
        .start()
        .await
        .expect("Docker must be running to execute neo4j-tests");
    let uri = container.image().bolt_uri_ipv4();
    let user = container.image().user();
    let pass = container.image().password();
    let graph = Graph::new(uri, user, pass)
        .await
        .expect("failed to connect to Neo4j");
    TestEnv { _container: container, graph }
}

/// Returns a connected `Graph` backed by the shared container.
pub async fn graph() -> &'static Graph {
    &ENV.get_or_init(init_env).await.graph
}
```

The `_container` field keeps the `ContainerAsync` alive. When the process exits, the static is dropped and `testcontainers` removes the container.

### 3.2 Test Serialization (Req 1.3)

Rust runs tests in parallel by default. Since all tests share one database, concurrent cleanup and writes would cause flaky failures. The test binary must be invoked with `--test-threads=1`:

```bash
cargo test --features neo4j-tests -- --test-threads=1
```

This is simpler than a code-level mutex and appropriate because *every* test in this binary requires serial access. The canonical command is documented in `CLAUDE.md` and in a module-level doc comment in `main.rs`.

### 3.3 Test Isolation (cleanup between tests)

Since tests share a container, each test must clean up before running. A helper runs cleanup Cypher:

```rust
/// Clears all data and schema from the database.
pub async fn clean_db(graph: &Graph) {
    // Drop all constraints first (some block index deletion)
    let mut result = graph
        .execute(neo4rs::query("SHOW CONSTRAINTS YIELD name"))
        .await.unwrap();
    while let Ok(Some(row)) = result.next().await {
        let name: String = row.get("name").unwrap();
        graph.run(neo4rs::query(&format!("DROP CONSTRAINT `{name}` IF EXISTS")))
            .await.unwrap();
    }

    // Drop all user-created indexes
    let mut result = graph
        .execute(neo4rs::query(
            "SHOW INDEXES YIELD name, type WHERE type <> 'LOOKUP' RETURN name"
        )).await.unwrap();
    while let Ok(Some(row)) = result.next().await {
        let name: String = row.get("name").unwrap();
        graph.run(neo4rs::query(&format!("DROP INDEX `{name}` IF EXISTS")))
            .await.unwrap();
    }

    // Delete all nodes and relationships
    graph.run(neo4rs::query("MATCH (n) DETACH DELETE n"))
        .await.unwrap();
}
```

### 3.4 Test Binary Entry Point

`main.rs` declares the feature gate and wires up all submodules:

```rust
// tests/neo4j_it/main.rs
#![cfg(feature = "neo4j-tests")]
#![expect(clippy::unwrap_used, reason = "integration tests — panics are the failure mode")]
#![expect(clippy::panic, reason = "tests use assert macros")]

mod helpers;
mod read;
mod write;
mod params;
mod schema;
mod functions;
mod show;
mod escaping;
mod patterns;
mod errors;
```

### 3.5 Test Example

Each submodule contains focused tests that follow the clean-act-assert pattern:

```rust
// tests/neo4j_it/read.rs

use rust_cypher_dsl::prelude::*;

#[tokio::test]
async fn match_returns_created_node() {
    let graph = crate::helpers::graph().await;
    crate::helpers::clean_db(graph).await;

    // CREATE via DSL
    let create_stmt = Cypher::create(
        node("Person").named("n").with_properties(props!("name" => param("name")))
    ).returning(name("n")).build();

    graph.run(
        neo4rs::query(&create_stmt.render()).param("name", "Alice")
    ).await.unwrap();

    // READ via DSL
    let read_stmt = Cypher::match_(node("Person").named("n"))
        .where_(prop("n", "name").eq(param("name")))
        .returning(name("n"))
        .build();

    let mut result = graph.execute(
        neo4rs::query(&read_stmt.render()).param("name", "Alice")
    ).await.unwrap();

    let row = result.next().await.unwrap().unwrap();
    let node: neo4rs::Node = row.get("n").unwrap();
    assert_eq!(node.get::<String>("name").unwrap(), "Alice");
}
```

---

## 4. Test Categories and Coverage

### 4.1 Read Queries (Req 2)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `match_by_label` | `Cypher::match_()`, `node()` | Returns only nodes with the target label |
| `match_where_property` | `.where_()`, `prop().eq()` | Returns only matching nodes |
| `optional_match_missing` | `Cypher::optional_match()` | Unmatched binding is null |
| `order_by_skip_limit` | `.order_by()`, `.skip()`, `.limit()` | Results in correct order and count |
| `match_relationship` | `node().rel().to()` | Returns connected nodes |
| `match_variable_length_path` | `rel("T").min(2).max(3)` | Returns paths within bounds |

### 4.2 Write Queries (Req 3)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `create_node_with_properties` | `Cypher::create()`, `.with_properties(props!(...))` | Node readable with exact labels + properties |
| `create_relationship` | `CREATE` with relationship pattern | Relationship readable with correct type |
| `merge_creates_when_missing` | `Cypher::merge()` | Node created; count = 1 |
| `merge_matches_when_existing` | `Cypher::merge()` (duplicate call) | No duplication; count still = 1 |
| `merge_on_create_on_match` | `.on_create()`, `.on_match()` | Correct branch sets property |
| `set_property` | `.set(SetItem::property(...))` | Property value updated |
| `delete_node` | `.delete()` | Node no longer retrievable |
| `detach_delete` | `.detach_delete()` | Node and relationships removed |

### 4.3 Parameterized Queries (Req 4)

| Test | Assertion |
|------|-----------|
| `param_string` | String parameter binds and returns correctly |
| `param_integer` | Integer parameter binds correctly |
| `param_boolean` | Boolean parameter binds correctly |
| `param_list` | List parameter binds correctly via `UNWIND` |

### 4.4 Schema Operations (Req 5)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `create_and_show_index` | `Cypher::create_index()`, `Cypher::show_indexes()` | Index appears in SHOW INDEXES |
| `drop_index` | `Cypher::drop_index()` | Index disappears from SHOW INDEXES |
| `create_index_if_not_exists` | `Cypher::create_index_if_not_exists()` | No error on duplicate |
| `drop_index_if_exists_nonexistent` | `Cypher::drop_index_if_exists()` | No error |
| `create_unique_constraint` | `Cypher::create_constraint("c").for_node("n", "L").is_unique(vec!["p"])` | Constraint in SHOW CONSTRAINTS |
| `unique_constraint_violation` | INSERT duplicate after constraint | Neo4j rejects the write |
| `drop_constraint` | `Cypher::drop_constraint()` | Constraint disappears |

### 4.5 Aggregation and Functions (Req 6)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `count_nodes` | `count()` | Correct count returned |
| `collect_values` | `collect()` | List of expected values |
| `sum_avg_min_max` | `sum()`, `avg()`, `min()`, `max()` | Correct aggregate values |
| `string_functions` | `to_lower()`, `to_upper()`, `substring()` | Correct string results |
| `count_distinct` | `count_distinct()` | Duplicates excluded |

### 4.6 SHOW Commands (Req 7)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `show_indexes` | `Cypher::show_indexes()` | No syntax error; result set returned |
| `show_constraints` | `Cypher::show_constraints()` | No syntax error; result set returned |
| `show_functions` | `Cypher::show_functions()` | Returns built-in function rows |
| `show_procedures` | `Cypher::show_procedures()` | Returns procedure rows |
| `show_transactions` | `Cypher::show_transactions()` | Returns at least current tx |
| `show_with_yield` | `.yield_fields([...])` on any SHOW | Only yielded columns present |

### 4.7 Backtick-Escaped Identifiers (Req 8)

| Test | Assertion |
|------|-----------|
| `reserved_keyword_property_roundtrip` | Create node with property `MATCH` (rendered as `` `MATCH` ``), read back — round-trips correctly |
| `reserved_keyword_label` | Node with label `INDEX` (rendered as `` `INDEX` ``) — accepted by Neo4j |
| `reserved_keyword_rel_type` | Relationship with type `RETURN` (rendered as `` `RETURN` ``) — accepted by Neo4j |
| `reserved_keyword_index_property` | CREATE INDEX on property `ORDER` — accepted |

### 4.8 Complex Patterns (Req 9)

| Test | DSL features exercised | Assertion |
|------|----------------------|-----------|
| `multi_hop_pattern` | `a.rel(rel("R")).to(b).rel(rel("R")).to(c)` | Correct end node returned |
| `variable_length_path` | `rel("R").min(1).max(3)` | Paths within bounds |
| `unwind_list` | `Cypher::unwind()` | Each element processed |
| `foreach_create` | `Cypher::match_().foreach()` | All nodes created |

### 4.9 Error Cases (Req 10)

| Test | Assertion |
|------|-----------|
| `safe_api_no_syntax_error` | A representative set of DSL queries execute against Neo4j 5.26 without syntax errors |
| `raw_unchecked_malformed` | `raw_unchecked("INVALID CYPHER !!!")` triggers a Neo4j client error |

---

## 5. Clippy and Lint Considerations

Integration tests use `.unwrap()` extensively (test code, not production). The test binary entry point needs:

```rust
#![expect(clippy::unwrap_used, reason = "integration tests — panics are the failure mode")]
```

The `neo4rs` API is async, so we need `tokio::test`. The crate's existing `panic = "warn"` lint is handled per-file since tests naturally panic on failure.

---

## 6. CI and Docker Considerations

**Local development:**
```bash
cargo test --features neo4j-tests -- --test-threads=1
```

**Without Docker:** If `--features neo4j-tests` is enabled but Docker is not running, `testcontainers` will panic at container startup with a clear error message. No runtime skip logic is implemented — the feature flag is the opt-in mechanism.

**Without the feature flag:** `cargo test` compiles and runs all existing tests. The `neo4j_it` binary is skipped entirely by the `#![cfg(feature = "neo4j-tests")]` gate. The dev-dependencies (`neo4rs`, `tokio`, etc.) may still be resolved/compiled by Cargo as part of the dependency graph, but no Neo4j-related code is linked.

**CI (GitHub Actions):** If enabled, Docker is pre-installed on `ubuntu-latest` runners:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --features neo4j-tests -- --test-threads=1
```

CI integration is out of scope for the initial implementation.

---

## 7. File Layout Summary

```
Cargo.toml                          # + neo4j-tests feature (empty), 3 dev-deps
tests/
  neo4j_it/
    main.rs                         # cfg gate, mod declarations, lint allows
    helpers.rs                      # TestEnv + Graph + clean_db()
    read.rs                         # 6 read query tests
    write.rs                        # 8 write query tests
    params.rs                       # 4 parameter binding tests
    schema.rs                       # 7 schema operation tests
    functions.rs                    # 5 aggregation + function tests
    show.rs                         # 6 SHOW command tests
    escaping.rs                     # 4 reserved keyword round-trip tests
    patterns.rs                     # 4 complex pattern tests
    errors.rs                       # 2 error case tests
```

**12 new files, ~35-40 tests total, zero production code changes.** The DSL's public API is consumed as-is — the tests just call `.render()` and pass the string to `neo4rs::query()`.
