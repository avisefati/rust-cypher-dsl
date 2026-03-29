# Implementation Plan: Neo4j Integration Tests

## 1. Add dependencies and feature gate

- [x] 1.1 Add `neo4j-tests = []` feature to `Cargo.toml` (empty feature — gates test source, not deps)
- [x] 1.2 Add unconditional dev-dependencies: `neo4j_testcontainers = "0.3"`, `neo4rs = "0.8"`, `tokio = { version = "1", features = ["full"] }`
- [x] 1.3 Verify `cargo test` (without feature) still passes unchanged — no neo4j code compiles
- [x] 1.4 Verify `cargo test --features neo4j-tests` compiles (no tests yet, just dependency resolution)

## 2. Create test binary skeleton and shared helpers

- [x] 2.1 Create `tests/neo4j_it/main.rs` with `#![cfg(feature = "neo4j-tests")]`, lint expects, and `mod helpers;`
- [x] 2.2 Create `tests/neo4j_it/helpers.rs` with `TestEnv` struct (holds `_container` + `Graph`), `OnceCell<TestEnv>` for lazy init, and `graph()` function (Req 1.1, 1.5)
- [x] 2.3 Pin the Neo4j image version (e.g., `neo4j:5.26`) in the container setup to avoid CI drift
- [x] 2.4 Add `clean_db()` function that drops constraints, drops user indexes, and deletes all nodes (Req 2.1, 2.2)
- [x] 2.5 Add module-level doc comment in `main.rs` documenting that `--test-threads=1` is required (Req 1.3)
- [x] 2.6 Add one smoke test in `main.rs` that calls `graph()`, runs `RETURN 1`, and asserts the result
- [x] 2.7 Run `cargo test --features neo4j-tests -- --test-threads=1` with Docker running and confirm the smoke test passes

## 3. Read query tests (`read.rs`) — Req 2

- [x] 3.1 Add `mod read;` to `main.rs`, create `tests/neo4j_it/read.rs`
- [x] 3.2 `match_by_label` — create 2 nodes with different labels, `Cypher::match_()` one label, assert only that node returned
- [x] 3.3 `match_where_property` — create nodes with varying property values, `.where_(prop().eq())`, assert correct subset
- [x] 3.4 `optional_match_missing` — `Cypher::optional_match()` a non-existent pattern, assert null binding
- [x] 3.5 `order_by_skip_limit` — create multiple nodes, `.order_by()` + `.skip()` + `.limit()`, assert correct order and count
- [x] 3.6 `match_relationship` — create `(a)-[:KNOWS]->(b)` via DSL, match the pattern, assert both nodes returned
- [x] 3.7 `match_variable_length_path` — create a chain `a->b->c->d`, match `rel("R").min(2).max(3)`, assert correct paths

## 4. Write query tests (`write.rs`) — Req 3

- [x] 4.1 Add `mod write;` to `main.rs`, create `tests/neo4j_it/write.rs`
- [x] 4.2 `create_node_with_properties` — `Cypher::create(node("L").named("n").with_properties(props!(...)))`, read back, assert labels + properties
- [x] 4.3 `create_relationship` — CREATE two nodes and a typed relationship, read back, assert type + properties
- [x] 4.4 `merge_creates_when_missing` — `Cypher::merge()`, assert count = 1
- [x] 4.5 `merge_matches_when_existing` — MERGE same node twice, assert count still = 1
- [x] 4.6 `merge_on_create_on_match` — `.on_create()` / `.on_match()`, assert correct branch fires
- [x] 4.7 `set_property` — CREATE node, `.set(SetItem::property(...))`, read back, assert updated value
- [x] 4.8 `delete_node` — CREATE node, `.delete()`, MATCH it, assert no results
- [x] 4.9 `detach_delete` — CREATE node with relationship, `.detach_delete()`, assert both gone

## 5. Parameter binding tests (`params.rs`) — Req 4

- [x] 5.1 Add `mod params;` to `main.rs`, create `tests/neo4j_it/params.rs`
- [x] 5.2 `param_string` — bind a string via `param("s")` + `.param("s", "hello")`, assert round-trip
- [x] 5.3 `param_integer` — bind an integer parameter, assert round-trip
- [x] 5.4 `param_boolean` — bind a boolean parameter, assert round-trip
- [x] 5.5 `param_list` — bind a list parameter via `Cypher::unwind()`, assert each element processed

## 6. Schema operation tests (`schema.rs`) — Req 5

- [ ] 6.1 Add `mod schema;` to `main.rs`, create `tests/neo4j_it/schema.rs`
- [ ] 6.2 `create_and_show_index` — `Cypher::create_index("idx").for_node(...)`, run SHOW INDEXES, assert index appears
- [ ] 6.3 `drop_index` — CREATE then `Cypher::drop_index("idx")`, run SHOW INDEXES, assert index gone
- [ ] 6.4 `create_index_if_not_exists` — `Cypher::create_index_if_not_exists("idx")` twice, assert no error
- [ ] 6.5 `drop_index_if_exists_nonexistent` — `Cypher::drop_index_if_exists("x")`, assert no error
- [ ] 6.6 `create_unique_constraint` — `Cypher::create_constraint("c").for_node("n", "L").is_unique(vec!["p"])`, SHOW CONSTRAINTS, assert present
- [ ] 6.7 `unique_constraint_violation` — create constraint, insert duplicate, assert Neo4j rejects
- [ ] 6.8 `drop_constraint` — CREATE then `Cypher::drop_constraint("c")`, assert disappears

## 7. Aggregation and function tests (`functions.rs`) — Req 6

- [ ] 7.1 Add `mod functions;` to `main.rs`, create `tests/neo4j_it/functions.rs`
- [ ] 7.2 `count_nodes` — create N nodes, `count()`, assert N
- [ ] 7.3 `collect_values` — create nodes with distinct names, `collect()`, assert list contents
- [ ] 7.4 `sum_avg_min_max` — create nodes with numeric property, assert correct aggregates
- [ ] 7.5 `string_functions` — use `to_lower()` / `to_upper()` / `substring()` in RETURN, assert correct results
- [ ] 7.6 `count_distinct` — create nodes with duplicate values, `count_distinct()`, assert duplicates excluded

## 8. SHOW command tests (`show.rs`) — Req 7

- [ ] 8.1 Add `mod show;` to `main.rs`, create `tests/neo4j_it/show.rs`
- [ ] 8.2 `show_indexes` — `Cypher::show_indexes()`, assert no syntax error and result set returned
- [ ] 8.3 `show_constraints` — `Cypher::show_constraints()`, assert no syntax error
- [ ] 8.4 `show_functions` — `Cypher::show_functions()`, assert rows returned (built-ins exist)
- [ ] 8.5 `show_procedures` — `Cypher::show_procedures()`, assert rows returned
- [ ] 8.6 `show_transactions` — `Cypher::show_transactions()`, assert at least one row
- [ ] 8.7 `show_with_yield` — `.yield_fields([...])` on a SHOW command, assert only yielded columns present

## 9. Backtick-escaping tests (`escaping.rs`) — Req 8

- [ ] 9.1 Add `mod escaping;` to `main.rs`, create `tests/neo4j_it/escaping.rs`
- [ ] 9.2 `reserved_keyword_property_roundtrip` — CREATE node with property `MATCH` (DSL renders as `` `MATCH` ``), read back, assert round-trip
- [ ] 9.3 `reserved_keyword_label` — CREATE node with label `INDEX` (rendered as `` `INDEX` ``), read back, assert label present
- [ ] 9.4 `reserved_keyword_rel_type` — CREATE relationship with type `RETURN` (rendered as `` `RETURN` ``), read back, assert type correct
- [ ] 9.5 `reserved_keyword_index_property` — `Cypher::create_index("idx").for_node("n", "L", vec!["ORDER"])`, assert index created

## 10. Complex pattern tests (`patterns.rs`) — Req 9

- [ ] 10.1 Add `mod patterns;` to `main.rs`, create `tests/neo4j_it/patterns.rs`
- [ ] 10.2 `multi_hop_pattern` — create `a->b->c` via `a.rel(rel("R")).to(b).rel(rel("R")).to(c)`, match 2-hop, assert `c` returned
- [ ] 10.3 `variable_length_path` — create chain of 5 nodes, match `rel("R").min(1).max(3)`, assert correct path count
- [ ] 10.4 `unwind_list` — `Cypher::unwind()` a parameter list, CREATE nodes, assert all created
- [ ] 10.5 `foreach_create` — FOREACH over a list, CREATE nodes, assert all created

## 11. Error case tests (`errors.rs`) — Req 10

- [ ] 11.1 Add `mod errors;` to `main.rs`, create `tests/neo4j_it/errors.rs`
- [ ] 11.2 `safe_api_no_syntax_error` — execute a representative set of DSL-built queries (MATCH, CREATE, MERGE, WITH, RETURN, UNWIND, aggregation), assert none produce syntax errors against Neo4j 5.26
- [ ] 11.3 `raw_unchecked_malformed` — execute `raw_unchecked("INVALID CYPHER !!!")`, assert Neo4j returns a client error

## 12. Final verification

- [ ] 12.1 Run `cargo test` (without feature) — all existing 1,722+ tests pass
- [ ] 12.2 Run `cargo test --features neo4j-tests` — all new + existing tests pass
- [ ] 12.3 Run `cargo clippy --all-targets --all-features -- -D warnings` — clean
- [ ] 12.4 Run `cargo doc --no-deps` — no warnings
