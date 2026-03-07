# Feature Name: Rust Cypher DSL

## Introduction

A Rust library that provides a type-safe, idiomatic way to programmatically construct Neo4j Cypher queries. The library mirrors the feature set of the [Java Neo4j Cypher-DSL](https://github.com/neo4j/cypher-dsl), adapted to Rust idioms (ownership, traits, enums, builder patterns). The core library has zero required runtime dependencies and produces valid Cypher query strings from an in-memory AST representation.

Feature parity is validated by porting the Java library's test suite (~180+ integration tests from `CypherIT.java` and additional test files) as the source of truth for expected Cypher output.

---

## Requirements

### 1. Core AST Types

1. **User Story:** As a library consumer, I want to represent Cypher graph elements (nodes, relationships, paths) as Rust types, so that I can construct patterns programmatically with compile-time safety.
   - Acceptance Criteria:
     1. When a node is created with one or more labels, the system shall produce a `Node` value that renders as `(name:\`Label1\`:\`Label2\`)`.
     2. When a node is created without a name, the system shall render it as an anonymous node `(:\`Label\`)`.
     3. When a relationship is created with a type and direction, the system shall render it with the correct arrow notation (`-->`, `<--`, `--`).
     4. When a relationship is created with variable-length bounds (min, max, unbounded), the system shall render `[*min..max]` syntax correctly.
     5. When a relationship is created with inline properties, the system shall render `{key: value}` syntax within the relationship brackets.
     6. When relationships are chained, the system shall render multi-hop patterns (e.g., `(a)-[:R1]->(b)-[:R2]->(c)`).
     7. When a path is named, the system shall render `p = (a)-[]->(b)` syntax.

2. **User Story:** As a library consumer, I want to represent Cypher expressions (literals, parameters, properties, operations, function calls) as Rust types, so that I can compose complex expressions safely.
   - Acceptance Criteria:
     1. When a string literal is created, the system shall render it with single-quote escaping (e.g., `'hello'`).
     2. When an integer or float literal is created, the system shall render the numeric value directly.
     3. When a boolean literal is created, the system shall render `true` or `false`.
     4. When a null literal is created, the system shall render `NULL`.
     5. When a named parameter is created, the system shall render `$paramName`.
     6. When a property is accessed on a node or relationship, the system shall render `variable.property`.
     7. When a list literal is created, the system shall render `[elem1, elem2, ...]`.
     8. When a map literal is created, the system shall render `{key1: val1, key2: val2}`.

3. **User Story:** As a library consumer, I want to represent Cypher conditions (comparisons, boolean logic, predicates) as Rust types, so that I can build WHERE clauses compositionally.
   - Acceptance Criteria:
     1. When comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`) are used, the system shall render the correct infix syntax.
     2. When conditions are combined with `AND`, `OR`, `XOR`, the system shall render infix boolean operators with correct parenthesization.
     3. When a condition is negated with `NOT`, the system shall render `NOT (condition)`.
     4. When `IS NULL` or `IS NOT NULL` is used, the system shall render the postfix syntax correctly.
     5. When `STARTS WITH`, `ENDS WITH`, `CONTAINS` are used, the system shall render the string predicate syntax.
     6. When `IN` is used with a list, the system shall render `expression IN [list]`.
     7. When a `hasLabels` condition is used, the system shall render label-check syntax.
     8. When empty/no-op conditions are combined, they shall collapse and not appear in rendered output.
     9. When conditions are nested with parentheses, grouping shall be preserved correctly in the output.

### 2. Cypher Clauses

4. **User Story:** As a library consumer, I want to construct reading clauses (`MATCH`, `OPTIONAL MATCH`, `WHERE`, `WITH`, `UNWIND`), so that I can express data retrieval patterns.
   - Acceptance Criteria:
     1. When `MATCH` is used with one or more patterns, the system shall render `MATCH pattern1, pattern2`.
     2. When `OPTIONAL MATCH` is used, the system shall render the `OPTIONAL MATCH` keyword.
     3. When `WHERE` is used after `MATCH` or `WITH`, the system shall render the condition inline.
     4. When `WITH` is used, the system shall render intermediate result projection with optional aliases.
     5. When `UNWIND` is used with an expression and alias, the system shall render `UNWIND expr AS alias`.
     6. When multiple `MATCH` clauses are chained, the system shall render them sequentially.
     7. When `WHERE` conditions use inline pattern expressions, the system shall render them correctly.

5. **User Story:** As a library consumer, I want to construct return clauses (`RETURN`, `ORDER BY`, `SKIP`, `LIMIT`, `DISTINCT`), so that I can control query output.
   - Acceptance Criteria:
     1. When `RETURN` is used with expressions, the system shall render `RETURN expr1, expr2`.
     2. When `RETURN` uses aliases, the system shall render `RETURN expr AS alias`.
     3. When `RETURN DISTINCT` is used, the system shall render the `DISTINCT` keyword.
     4. When `ORDER BY` is used, the system shall render sort items with optional `ASC`/`DESC`.
     5. When `SKIP` is used with a number or parameter, the system shall render `SKIP n`.
     6. When `LIMIT` is used with a number or parameter, the system shall render `LIMIT n`.
     7. When `SKIP` and `LIMIT` are combined, the system shall render both in correct order.
     8. When null is passed for SKIP or LIMIT, the clause shall be omitted from output.
     9. When a wildcard return (`*`) is used, the system shall render `RETURN *`.

6. **User Story:** As a library consumer, I want to construct writing clauses (`CREATE`, `MERGE`, `SET`, `DELETE`, `REMOVE`, `FOREACH`), so that I can express data mutation queries.
   - Acceptance Criteria:
     1. When `CREATE` is used with a pattern, the system shall render `CREATE pattern`.
     2. When `MERGE` is used with a pattern, the system shall render `MERGE pattern`.
     3. When `MERGE` uses `ON CREATE SET` or `ON MATCH SET` actions, the system shall render them in order.
     4. When multiple `ON CREATE`/`ON MATCH` actions are chained, the system shall preserve their order.
     5. When `SET` is used for property assignment, the system shall render `SET n.prop = value`.
     6. When `SET` is used for label assignment, the system shall render `SET n:Label`.
     7. When `SET` is used with `+=` (mutating set), the system shall render `SET n += {map}`.
     8. When `DELETE` is used, the system shall render `DELETE expr`.
     9. When `DETACH DELETE` is used, the system shall render `DETACH DELETE expr`.
     10. When `REMOVE` is used for properties, the system shall render `REMOVE n.prop`.
     11. When `REMOVE` is used for labels, the system shall render `REMOVE n:Label`.
     12. When `FOREACH` is used, the system shall render `FOREACH (var IN list | update-clauses)`.

7. **User Story:** As a library consumer, I want to call stored procedures and use subqueries, so that I can integrate with Neo4j's procedural capabilities.
   - Acceptance Criteria:
     1. When a standalone procedure call is made, the system shall render `CALL proc.name(args)`.
     2. When `YIELD` is used with a procedure call, the system shall render `CALL proc.name() YIELD field1, field2`.
     3. When a procedure result is filtered with `WHERE`, the system shall render the filter after YIELD.
     4. When an in-query `CALL { subquery }` is used, the system shall render the subquery block.
     5. When `IN TRANSACTIONS` is used with a subquery, the system shall render the batching clause.

### 3. Statement Builder (Fluent API)

8. **User Story:** As a library consumer, I want a fluent builder API starting from a `Cypher` entry point, so that I can construct queries in a readable, chainable style.
   - Acceptance Criteria:
     1. When `Cypher::match_node()` is called, a builder shall be returned that accepts `.where_()`, `.returning()`, `.with()`, etc.
     2. When the builder is used, clause ordering shall be enforced (e.g., `RETURN` cannot precede `MATCH`).
     3. When `.build()` is called, the system shall produce a valid `Statement` value.
     4. When `Cypher::create()` is called, a builder for write queries shall be returned.
     5. When `Cypher::merge()` is called, a builder exposing `on_create()` / `on_match()` shall be returned.
     6. When `Cypher::unwind()` is called, a builder for UNWIND-based queries shall be returned.
     7. When `Cypher::call()` is called, a builder for procedure calls shall be returned.
     8. When the builder pattern is used, intermediate states shall be immutable (each method consumes and returns a new builder).
     9. When `UNION` or `UNION ALL` is used, the system shall combine multiple statements correctly.
     10. When `EXPLAIN` or `PROFILE` prefixes are used, the system shall render them before the query.

### 4. Built-in Functions

9. **User Story:** As a library consumer, I want access to Cypher's built-in functions (aggregation, scalar, string, math, list, temporal, spatial, predicate), so that I can use them within expressions.
   - Acceptance Criteria:
     1. When aggregation functions (`count`, `sum`, `avg`, `min`, `max`, `collect`) are used, the system shall render `functionName(expr)`.
     2. When `count` is used with `DISTINCT`, the system shall render `count(DISTINCT expr)`.
     3. When scalar functions (`id`, `elementId`, `type`, `coalesce`, `timestamp`, `size`, `head`, `last`) are used, the system shall render them correctly.
     4. When string functions (`toLower`, `toUpper`, `trim`, `replace`, `substring`, `left`, `right`) are used, the system shall render them correctly.
     5. When math functions (`abs`, `ceil`, `floor`, `round`, `sqrt`, `log`, `sign`) are used, the system shall render them correctly.
     6. When list functions (`range`, `keys`, `labels`, `nodes`, `relationships`, `tail`, `reverse`) are used, the system shall render them correctly.
     7. When temporal functions (`datetime`, `date`, `time`, `duration`) are used, the system shall render them correctly.
     8. When spatial functions (`point`, `distance`) are used, the system shall render them correctly.
     9. When predicate functions (`exists`, `all`, `any`, `none`, `single`) are used, the system shall render them correctly.
     10. When a custom/user-defined function is invoked by name, the system shall render `functionName(args)`.

### 5. Renderer

10. **User Story:** As a library consumer, I want to render a `Statement` into a Cypher string, so that I can send it to a Neo4j driver or inspect it.
    - Acceptance Criteria:
      1. When a statement is rendered with the default renderer, the system shall produce a single-line Cypher string.
      2. When a statement is rendered with the pretty-printing renderer, the system shall produce indented, multi-line output.
      3. When name escaping is configured as "always", the system shall backtick-escape all identifiers.
      4. When name escaping is configured as "as-needed", the system shall only escape identifiers that require it.
      5. When the `Display` trait is used on a `Statement`, it shall delegate to the default renderer.

### 6. Advanced Expressions

11. **User Story:** As a library consumer, I want to use advanced Cypher expressions (CASE, list comprehensions, pattern comprehensions, map projections, existential subqueries), so that I can express complex logic.
    - Acceptance Criteria:
      1. When a simple `CASE` expression is used, the system shall render `CASE expr WHEN value THEN result END`.
      2. When a generic `CASE` expression is used, the system shall render `CASE WHEN cond THEN result ELSE default END`.
      3. When a list comprehension is used, the system shall render `[var IN list WHERE cond | expr]`.
      4. When a pattern comprehension is used, the system shall render `[(pattern) WHERE cond | expr]`.
      5. When a map projection is used, the system shall render `variable {.prop1, .prop2, key: expr}`.
      6. When an existential subquery is used, the system shall render `EXISTS { MATCH pattern }`.
      7. When `COUNT { pattern }` is used, the system shall render the count subquery expression.

### 7. Statement Catalog (Introspection)

12. **User Story:** As a library consumer, I want to inspect the metadata of a built statement (labels, relationship types, properties, parameters used), so that I can perform analysis or validation before execution.
    - Acceptance Criteria:
      1. When a statement is built, the system shall expose all node labels referenced in the query.
      2. When a statement is built, the system shall expose all relationship types referenced in the query.
      3. When a statement is built, the system shall expose all properties referenced in the query, associated with their containers.
      4. When a statement is built, the system shall expose all named parameters and their optional bound values.

### 8. Feature Parity Validation

13. **User Story:** As a library maintainer, I want the Rust test suite to be derived from the Java Cypher-DSL test suite, so that feature parity is systematically validated.
    - Acceptance Criteria:
      1. When tests are written for each feature, they shall use expected Cypher strings ported from the Java `CypherIT.java` test suite.
      2. When tests are written for functions, they shall use expected Cypher strings from `FunctionsIT.java` and `FunctionsTests.java`.
      3. When tests are written for subqueries, they shall use expected Cypher strings from `SubqueriesIT.java`.
      4. When tests are written for expressions, they shall use expected Cypher strings from `ExpressionsIT.java`.
      5. When tests are written for procedures, they shall use expected Cypher strings from `ProcedureCallsIT.java`.
      6. When tests are written for rendering, they shall use expected output from the Java renderer test files.
      7. When a feature area reaches completion, the number of passing Rust tests shall match or exceed the corresponding Java test count for that area.

### 9. Idiomatic Rust Design

14. **User Story:** As a Rust developer, I want the library to follow Rust idioms and conventions, so that it feels natural to use alongside other Rust crates.
    - Acceptance Criteria:
      1. When the library is compiled, zero `clippy` warnings shall be emitted under pedantic lints.
      2. When AST types are used, they shall implement `Clone`, `Debug`, and `PartialEq`.
      3. When string data is stored, the library shall use `Cow<'_, str>` or similar to avoid unnecessary allocations for static strings.
      4. When errors occur during building, the system shall return `Result<Statement, BuildError>` with descriptive error variants.
      5. When the library is used, it shall have zero required runtime dependencies in its core.
      6. When the builder API is used, invalid clause ordering shall be caught at compile time via typestate patterns where feasible.
      7. When the library's public API is used, all public items shall have doc comments.

### 10. Parser (Optional, Future Scope)

15. **User Story:** As a library consumer, I want to optionally parse Cypher query strings into the AST, so that I can analyze, validate, or transform existing queries.
    - Acceptance Criteria:
      1. When the `parser` feature flag is enabled, the system shall expose a `parse()` function that accepts a Cypher string.
      2. When a valid Cypher query is parsed, the system shall produce a `Statement` value equivalent to one built via the DSL.
      3. When an invalid Cypher query is parsed, the system shall return a descriptive parse error.
      4. This requirement is deferred to a later phase and is NOT in scope for initial implementation.
