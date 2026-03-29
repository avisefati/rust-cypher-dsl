# Feature Name: Neo4j Integration Tests

## Introduction

End-to-end integration tests that verify the Cypher DSL produces queries a real Neo4j server accepts and executes with correct semantics. A Neo4j instance is managed automatically via Docker (`testcontainers`), and queries are executed through the `neo4rs` Bolt driver.

These tests complement the existing string-comparison test suite (1,700+ tests) by proving the rendered Cypher is not just syntactically plausible but actually accepted by Neo4j's parser and returns the expected data.

The integration test suite is gated behind a Cargo feature flag (`neo4j-tests`) so that `cargo test` continues to work without Docker. The tests are opt-in for maintainers; CI may enable them when Docker is available.

---

## Requirements

### 1. Test Infrastructure

1. **User Story:** As a library maintainer, I want a Neo4j Docker container to start automatically before integration tests and stop after, so that no manual setup is required.
   - Acceptance Criteria:
     1. When `cargo test --features neo4j-tests` is run and Docker is available, the system shall start a Neo4j container (pinned to image `neo4j:5.26`) via `testcontainers`.
     2. When all integration tests complete, the container shall be removed automatically.
     3. When `cargo test` is run without `--features neo4j-tests`, no Neo4j-related code shall compile and the existing test suite shall pass unchanged.
     4. When `--features neo4j-tests` is enabled but Docker is not running, the test binary shall fail fast with a clear error from `testcontainers` (container start panic). No special runtime skip logic is required.
     5. When multiple tests run in the same test binary, they shall share a single Neo4j container to avoid repeated startup cost.

2. **User Story:** As a library maintainer, I want each test to start from a clean database state, so that tests are isolated and order-independent.
   - Acceptance Criteria:
     1. When a test begins, any data from previous tests shall be cleared (e.g., `MATCH (n) DETACH DELETE n`).
     2. When a test creates indexes or constraints, they shall be dropped during cleanup so subsequent tests are unaffected.

3. **User Story:** As a library maintainer, I want tests to run without data races despite sharing a single database, so that the suite is not flaky.
   - Acceptance Criteria:
     1. Tests shall execute serially (not in parallel) to prevent one test's cleanup from interfering with another test's assertions.
     2. Serial execution shall be enforced by running the test binary with `--test-threads=1`. The canonical invocation is `cargo test --features neo4j-tests -- --test-threads=1`.

### 2. Read Queries

1. **User Story:** As a library consumer, I want confidence that DSL-built `MATCH` queries return the correct data from Neo4j, so that I can trust the rendered Cypher is semantically correct.
   - Acceptance Criteria:
     1. When a `MATCH` query with a label filter is executed, Neo4j shall return only nodes with that label.
     2. When a `WHERE` clause with a property comparison is used, Neo4j shall return only matching nodes.
     3. When `OPTIONAL MATCH` is used for a non-existent pattern, Neo4j shall return `null` for the unmatched bindings.
     4. When `ORDER BY`, `SKIP`, and `LIMIT` are used, Neo4j shall return results in the specified order and quantity.
     5. When relationship patterns are matched (single-hop, multi-hop, variable-length), Neo4j shall return the expected paths.

### 3. Write Queries

1. **User Story:** As a library consumer, I want confidence that DSL-built write queries (`CREATE`, `MERGE`, `SET`, `DELETE`) execute correctly, so that I can trust mutations work as intended.
   - Acceptance Criteria:
     1. When a `CREATE` query creates a node with labels and properties, the node shall be readable back with those exact labels and properties.
     2. When a `CREATE` query creates a relationship between two nodes, the relationship shall be readable back with the correct type and properties.
     3. When a `MERGE` query targets a node that does not exist, the node shall be created.
     4. When a `MERGE` query targets a node that already exists, the existing node shall be returned without duplication.
     5. When `ON CREATE SET` and `ON MATCH SET` are used with `MERGE`, the correct branch shall execute.
     6. When `SET` updates a property, the property value shall be changed in the database.
     7. When `DELETE` removes a node, the node shall no longer be retrievable.
     8. When `DETACH DELETE` removes a node with relationships, both the node and its relationships shall be removed.

### 4. Parameterized Queries

1. **User Story:** As a library consumer, I want DSL-built parameterized queries (`$param`) to bind correctly through the Neo4j driver, so that parameters work end-to-end.
   - Acceptance Criteria:
     1. When a query uses `param("name")` in the DSL and a value is bound via the driver, Neo4j shall receive and use the bound value.
     2. When multiple parameters are used in a single query, all shall bind correctly.
     3. When parameter types include strings, integers, floats, booleans, and lists, each shall be handled correctly by the driver.

### 5. Schema Operations (Indexes and Constraints)

1. **User Story:** As a library consumer, I want DSL-built index and constraint commands to execute successfully against Neo4j, so that I can manage schema programmatically.
   - Acceptance Criteria:
     1. When `CREATE INDEX` is rendered and executed, the index shall appear in `SHOW INDEXES`.
     2. When `DROP INDEX` is rendered and executed, the index shall no longer appear in `SHOW INDEXES`.
     3. When `CREATE INDEX IF NOT EXISTS` is executed for an existing index, Neo4j shall not error.
     4. When `DROP INDEX IF EXISTS` is executed for a non-existent index, Neo4j shall not error.
     5. When `CREATE CONSTRAINT` (uniqueness) is rendered and executed, the constraint shall appear in `SHOW CONSTRAINTS`.
     6. When a uniqueness constraint exists and a duplicate value is inserted, Neo4j shall reject the write.
     7. When `DROP CONSTRAINT` is rendered and executed, the constraint shall no longer appear in `SHOW CONSTRAINTS`.

### 6. Aggregation and Functions

1. **User Story:** As a library consumer, I want DSL-built queries that use built-in functions and aggregations to return correct results, so that I can trust function rendering.
   - Acceptance Criteria:
     1. When `count()` is used, Neo4j shall return the correct count.
     2. When `collect()` is used, Neo4j shall return a list of the collected values.
     3. When `sum()`, `avg()`, `min()`, `max()` are used on numeric properties, Neo4j shall return the correct aggregate value.
     4. When string functions (`toLower`, `toUpper`, `substring`, `replace`) are used, Neo4j shall return the correct string result.
     5. When `DISTINCT` is combined with aggregation, duplicates shall be excluded.

### 7. SHOW Commands

1. **User Story:** As a library consumer, I want DSL-built `SHOW` commands to execute and return meaningful results, so that I can verify admin command rendering.
   - Acceptance Criteria:
     1. When `SHOW INDEXES` is executed, Neo4j shall return a result set (possibly empty if no user indexes exist).
     2. When `SHOW CONSTRAINTS` is executed, Neo4j shall return a result set.
     3. When `SHOW FUNCTIONS` is executed, Neo4j shall return built-in function metadata.
     4. When `SHOW PROCEDURES` is executed, Neo4j shall return procedure metadata.
     5. When `SHOW TRANSACTIONS` is executed, Neo4j shall return at least the current transaction.
     6. When a `YIELD` clause is appended to a SHOW command, only the yielded columns shall be returned.

### 8. Backtick-Escaped Identifiers

1. **User Story:** As a library consumer, I want Neo4j to accept queries with backtick-escaped reserved keywords as identifiers, so that the global keyword-escaping behaviour is validated end-to-end.
   - Acceptance Criteria:
     1. When a property named with a Cypher reserved keyword (e.g., `MATCH`, `RETURN`, `ORDER`) is created and read back via DSL-generated queries, the round-trip shall succeed.
     2. When a node label that is a reserved keyword (e.g., `INDEX`) is used, Neo4j shall accept the backtick-escaped form.
     3. When a relationship type that is a reserved keyword (e.g., `MATCH`) is used, Neo4j shall accept the backtick-escaped form.
     4. When an index is created on a property whose name is a reserved keyword, the index shall be created successfully.

### 9. Complex Patterns

1. **User Story:** As a library consumer, I want DSL-built queries with complex patterns (multi-hop, variable-length, UNWIND, FOREACH) to execute correctly, so that I can trust advanced pattern rendering.
   - Acceptance Criteria:
     1. When a multi-hop relationship pattern is matched, Neo4j shall return the correct intermediate and end nodes.
     2. When a variable-length path `[*min..max]` is matched, Neo4j shall return paths within the specified bounds.
     3. When `UNWIND` is used with a list parameter, Neo4j shall iterate over each element.
     4. When `FOREACH` is used to create multiple nodes, all nodes shall be created.

### 10. Error Cases

1. **User Story:** As a library maintainer, I want to verify that syntactically invalid queries (e.g., if `raw_unchecked` is misused) are rejected by Neo4j, so that we can document the boundary between DSL safety and user responsibility.
   - Acceptance Criteria:
     1. When a representative set of queries built through the safe DSL API is executed against Neo4j 5.26, none shall produce a syntax error. (This validates the tested subset, not an absolute guarantee across all Neo4j versions.)
     2. When `raw_unchecked` is used with malformed Cypher, Neo4j shall return a client error, demonstrating the escape hatch carries risk.
