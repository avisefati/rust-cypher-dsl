# Feature Name: Rust Cypher DSL

## Introduction

A Rust library that provides a type-safe, idiomatic way to programmatically construct Neo4j Cypher queries. The library mirrors the feature set of the [Java Neo4j Cypher-DSL](https://github.com/neo4j/cypher-dsl), adapted to Rust idioms (ownership, traits, enums, builder patterns), and extended to cover the full Cypher language specification as documented in the [Neo4j Cypher Manual](https://neo4j.com/docs/cypher-manual/current/).

The core library has zero required runtime dependencies and produces valid Cypher query strings from an in-memory AST representation.

Feature parity is validated by porting the Java library's test suite (~180+ integration tests from `CypherIT.java` and additional test files) as the source of truth for expected Cypher output, supplemented by additional tests for Cypher features not covered by the Java DSL.

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
     8. When a node uses label expressions (`&` for AND, `|` for OR, `!` for NOT, `%` for wildcard), the system shall render them correctly (e.g., `(n:A&B)`, `(n:A|B)`, `(n:!A)`, `(n:%)`).

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
     9. When a `reduce()` expression is used, the system shall render `reduce(acc = init, x IN list | expr)`.

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
     10. When regex matching (`=~`) is used, the system shall render `expression =~ 'pattern'`.
     11. When a type predicate (`IS :: TYPE`) is used, the system shall render the type check expression.
     12. When `IS NORMALIZED` or `IS NOT NORMALIZED` is used, the system shall render the normalization check.

### 2. Cypher Clauses

1. **User Story:** As a library consumer, I want to construct reading clauses (`MATCH`, `OPTIONAL MATCH`, `WHERE`, `WITH`, `UNWIND`), so that I can express data retrieval patterns.
   - Acceptance Criteria:
     1. When `MATCH` is used with one or more patterns, the system shall render `MATCH pattern1, pattern2`.
     2. When `OPTIONAL MATCH` is used, the system shall render the `OPTIONAL MATCH` keyword.
     3. When `WHERE` is used after `MATCH` or `WITH`, the system shall render the condition inline.
     4. When `WITH` is used, the system shall render intermediate result projection with optional aliases.
     5. When `UNWIND` is used with an expression and alias, the system shall render `UNWIND expr AS alias`.
     6. When multiple `MATCH` clauses are chained, the system shall render them sequentially.
     7. When `WHERE` conditions use inline pattern expressions, the system shall render them correctly.

2. **User Story:** As a library consumer, I want to construct return clauses (`RETURN`, `ORDER BY`, `SKIP`, `LIMIT`, `DISTINCT`), so that I can control query output.
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

3. **User Story:** As a library consumer, I want to construct writing clauses (`CREATE`, `MERGE`, `SET`, `DELETE`, `REMOVE`, `FOREACH`), so that I can express data mutation queries.
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

4. **User Story:** As a library consumer, I want to call stored procedures and use subqueries, so that I can integrate with Neo4j's procedural capabilities.
   - Acceptance Criteria:
     1. When a standalone procedure call is made, the system shall render `CALL proc.name(args)`.
     2. When `YIELD` is used with a procedure call, the system shall render `CALL proc.name() YIELD field1, field2`.
     3. When a procedure result is filtered with `WHERE`, the system shall render the filter after YIELD.
     4. When an in-query `CALL { subquery }` is used, the system shall render the subquery block.
     5. When `IN TRANSACTIONS` is used with a subquery, the system shall render the batching clause.
     6. When a `COLLECT { subquery }` expression is used, the system shall render the collect subquery.

### 3. Advanced Patterns

1. **User Story:** As a library consumer, I want to express advanced Cypher path patterns (quantified paths, shortest paths, path selectors), so that I can match complex graph structures.
   - Acceptance Criteria:
     1. When a quantified path pattern is used with `{n,m}` bounds, the system shall render `((a)-[:R]->(b)){n,m}`.
     2. When `+` (one or more) is used as a quantifier, the system shall render the `+` postfix.
     3. When `*` (zero or more) is used as a quantifier, the system shall render the `*` postfix.
     4. When a quantified relationship is used, the system shall render `(a)-[:R]->{n,m}(b)`.
     5. When group variables are used inside quantified patterns, they shall bind to lists and render correctly.
     6. When inline predicates (`WHERE`) are used inside quantified patterns, the system shall render them within the pattern.
     7. When `SHORTEST k` is used as a path selector, the system shall render `SHORTEST k (pattern)`.
     8. When `ALL SHORTEST` is used, the system shall render `ALL SHORTEST (pattern)`.
     9. When `ANY` is used as a path selector, the system shall render `ANY (pattern)`.
     10. When `SHORTEST k GROUPS` is used, the system shall render `SHORTEST k GROUPS (pattern)`.
     11. When a named path with a selector is used, the system shall render `p = SHORTEST 1 (pattern)`.

### 4. Data Import

1. **User Story:** As a library consumer, I want to construct `LOAD CSV` queries, so that I can express data import operations.
   - Acceptance Criteria:
     1. When `LOAD CSV FROM` is used, the system shall render `LOAD CSV FROM 'url' AS row`.
     2. When `WITH HEADERS` is specified, the system shall render `LOAD CSV WITH HEADERS FROM 'url' AS row`.
     3. When `FIELDTERMINATOR` is specified, the system shall render the custom delimiter option.
     4. When `USING PERIODIC COMMIT` is used, the system shall render it before the `LOAD CSV` clause.

### 5. Graph Selection

1. **User Story:** As a library consumer, I want to specify target graphs for queries, so that I can work with composite databases.
   - Acceptance Criteria:
     1. When `USE` is specified with a graph name, the system shall render `USE graphName`.
     2. When `USE` is specified with a function call (e.g., `graph.byName()`), the system shall render it correctly.

### 6. Query Hints

1. **User Story:** As a library consumer, I want to add query hints to my statements, so that I can influence the query planner.
   - Acceptance Criteria:
     1. When `USING INDEX` is specified, the system shall render `USING INDEX variable:Label(property)`.
     2. When `USING INDEX SEEK` is specified, the system shall render `USING INDEX SEEK variable:Label(property)`.
     3. When `USING SCAN` is specified, the system shall render `USING SCAN variable:Label`.
     4. When `USING JOIN` is specified, the system shall render `USING JOIN ON variable`.

### 7. Statement Builder (Fluent API)

1. **User Story:** As a library consumer, I want a fluent builder API starting from a `Cypher` entry point, so that I can construct queries in a readable, chainable style.
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
     11. When `Cypher::load_csv()` is called, a builder for LOAD CSV queries shall be returned.

### 8. Built-in Functions

1. **User Story:** As a library consumer, I want access to Cypher's built-in functions, so that I can use them within expressions.
   - Acceptance Criteria:
     1. When aggregation functions (`count`, `sum`, `avg`, `min`, `max`, `collect`, `percentileCont`, `percentileDisc`, `stDev`, `stDevP`) are used, the system shall render them correctly.
     2. When `count` or other aggregation functions are used with `DISTINCT`, the system shall render `functionName(DISTINCT expr)`.
     3. When scalar functions (`id`, `elementId`, `type`, `coalesce`, `timestamp`, `size`, `head`, `last`, `startNode`, `endNode`, `properties`, `randomUUID`, `nullIf`, `valueType`, `char_length`, `length`, `path_length`) are used, the system shall render them correctly.
     4. When type conversion functions (`toInteger`, `toFloat`, `toBoolean`, `toString` and their `OrNull` variants) are used, the system shall render them correctly.
     5. When string functions (`toLower`/`lower`, `toUpper`/`upper`, `trim`, `btrim`, `ltrim`, `rtrim`, `replace`, `substring`, `left`, `right`, `split`, `reverse`, `normalize`) are used, the system shall render them correctly.
     6. When math numeric functions (`abs`, `ceil`/`ceiling`, `floor`, `round`, `sign`, `rand`, `isNaN`) are used, the system shall render them correctly.
     7. When math logarithmic functions (`sqrt`, `log`, `ln`, `log10`, `exp`, `e`) are used, the system shall render them correctly.
     8. When math trigonometric functions (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `cot`, `cosh`, `sinh`, `tanh`, `coth`, `degrees`, `radians`, `haversin`, `pi`) are used, the system shall render them correctly.
     9. When list functions (`range`, `keys`, `labels`, `nodes`, `relationships`, `tail`, `reverse`, `reduce`, `toBooleanList`, `toFloatList`, `toIntegerList`, `toStringList`) are used, the system shall render them correctly.
     10. When `coll.*` namespace functions (`coll.distinct`, `coll.flatten`, `coll.indexOf`, `coll.insert`, `coll.max`, `coll.min`, `coll.remove`, `coll.sort`) are used, the system shall render them correctly.
     11. When temporal instant functions (`datetime`, `localdatetime`, `date`, `localtime`, `time` and their `.realtime`, `.statement`, `.transaction`, `.truncate` variants) are used, the system shall render them correctly.
     12. When temporal duration functions (`duration`, `duration.between`, `duration.inDays`, `duration.inMonths`, `duration.inSeconds`) are used, the system shall render them correctly.
     13. When the temporal `format()` function is used, the system shall render it correctly.
     14. When `datetime.fromEpoch` and `datetime.fromEpochMillis` are used, the system shall render them correctly.
     15. When spatial functions (`point`, `point.distance`, `point.withinBBox`) are used, the system shall render them correctly.
     16. When predicate functions (`exists`, `all`, `any`, `none`, `single`, `isEmpty`, `allReduce`) are used, the system shall render them correctly.
     17. When database functions (`db.nameFromElementId`) are used, the system shall render them correctly.
     18. When graph functions (`graph.byElementId`, `graph.byName`, `graph.names`, `graph.propertiesByName`) are used, the system shall render them correctly.
     19. When vector functions (`vector`, `vector.similarity.cosine`, `vector.similarity.euclidean`) are used, the system shall render them correctly.
     20. When LOAD CSV functions (`file`, `linenumber`) are used, the system shall render them correctly.
     21. When a custom/user-defined function is invoked by name, the system shall render `functionName(args)`.

### 9. Renderer

1. **User Story:** As a library consumer, I want to render a `Statement` into a Cypher string, so that I can send it to a Neo4j driver or inspect it.
   - Acceptance Criteria:
     1. When a statement is rendered with the default renderer, the system shall produce a single-line Cypher string.
     2. When a statement is rendered with the pretty-printing renderer, the system shall produce indented, multi-line output.
     3. When name escaping is configured as "always", the system shall backtick-escape all identifiers.
     4. When name escaping is configured as "as-needed", the system shall only escape identifiers that require it.
     5. When the `Display` trait is used on a `Statement`, it shall delegate to the default renderer.

### 10. Advanced Expressions

1. **User Story:** As a library consumer, I want to use advanced Cypher expressions (CASE, list comprehensions, pattern comprehensions, map projections, subquery expressions), so that I can express complex logic.
   - Acceptance Criteria:
     1. When a simple `CASE` expression is used, the system shall render `CASE expr WHEN value THEN result END`.
     2. When a generic `CASE` expression is used, the system shall render `CASE WHEN cond THEN result ELSE default END`.
     3. When a list comprehension is used, the system shall render `[var IN list WHERE cond | expr]`.
     4. When a pattern comprehension is used, the system shall render `[(pattern) WHERE cond | expr]`.
     5. When a map projection is used, the system shall render `variable {.prop1, .prop2, key: expr}`.
     6. When an existential subquery is used, the system shall render `EXISTS { MATCH pattern }`.
     7. When `COUNT { pattern }` is used, the system shall render the count subquery expression.
     8. When `COLLECT { subquery }` is used as an expression, the system shall render it correctly.

### 11. Statement Catalog (Introspection)

1. **User Story:** As a library consumer, I want to inspect the metadata of a built statement (labels, relationship types, properties, parameters used), so that I can perform analysis or validation before execution.
   - Acceptance Criteria:
     1. When a statement is built, the system shall expose all node labels referenced in the query.
     2. When a statement is built, the system shall expose all relationship types referenced in the query.
     3. When a statement is built, the system shall expose all properties referenced in the query, associated with their containers.
     4. When a statement is built, the system shall expose all named parameters and their optional bound values.

### 12. Cypher 25 Clauses

1. **User Story:** As a library consumer, I want to use Cypher 25 clauses (`FINISH`, `FILTER`, `LET`, `WHEN`, `NEXT`), so that I can take advantage of newer Cypher language features.
   - Acceptance Criteria:
     1. When `FINISH` is used, the system shall render a query terminator with no result set.
     2. When `FILTER` is used with a predicate, the system shall render `FILTER predicate` as a standalone clause.
     3. When `LET` is used for variable binding, the system shall render `LET var = expr`.
     4. When `WHEN` is used for conditional branching, the system shall render the conditional query composition syntax.
     5. When `NEXT` is used for sequential query composition, the system shall render the sequential query chaining syntax.

### 13. Feature Parity Validation

1. **User Story:** As a library maintainer, I want the Rust test suite to be derived from the Java Cypher-DSL test suite, so that feature parity is systematically validated.
   - Acceptance Criteria:
     1. When tests are written for each feature, they shall use expected Cypher strings ported from the Java `CypherIT.java` test suite.
     2. When tests are written for functions, they shall use expected Cypher strings from `FunctionsIT.java` and `FunctionsTests.java`.
     3. When tests are written for subqueries, they shall use expected Cypher strings from `SubqueriesIT.java`.
     4. When tests are written for expressions, they shall use expected Cypher strings from `ExpressionsIT.java`.
     5. When tests are written for procedures, they shall use expected Cypher strings from `ProcedureCallsIT.java`.
     6. When tests are written for rendering, they shall use expected output from the Java renderer test files.
     7. When a feature area reaches completion, the number of passing Rust tests shall match or exceed the corresponding Java test count for that area.
     8. When tests cover Cypher features beyond the Java DSL (QPPs, path selectors, Cypher 25 clauses), they shall validate against the Cypher manual's documented syntax.

### 14. Idiomatic Rust Design

1. **User Story:** As a Rust developer, I want the library to follow Rust idioms and conventions, so that it feels natural to use alongside other Rust crates.
   - Acceptance Criteria:
     1. When the library is compiled, zero `clippy` warnings shall be emitted under pedantic lints.
     2. When AST types are used, they shall implement `Clone`, `Debug`, and `PartialEq`.
     3. When string data is stored, the library shall use `Cow<'_, str>` or similar to avoid unnecessary allocations for static strings.
     4. When errors occur during building, the system shall return `Result<Statement, BuildError>` with descriptive error variants.
     5. When the library is used, it shall have zero required runtime dependencies in its core.
     6. When the builder API is used, invalid clause ordering shall be caught at compile time via typestate patterns where feasible.
     7. When the library's public API is used, all public items shall have doc comments.
     8. When relationship patterns are constructed, both method syntax (`.rel().to()`) and operator syntax (`>>` / `<<`) shall be available.

### 15. Parser (Optional, Future Phase)

1. **User Story:** As a library consumer, I want to optionally parse Cypher query strings into the AST, so that I can analyze, validate, or transform existing queries.
   - Acceptance Criteria:
     1. When the `parser` feature flag is enabled, the system shall expose a `parse()` function that accepts a Cypher string.
     2. When a valid Cypher query is parsed, the system shall produce a `Statement` value equivalent to one built via the DSL.
     3. When an invalid Cypher query is parsed, the system shall return a descriptive parse error.
     4. This requirement is deferred to a later phase and is NOT in scope for initial implementation.

### 16. Administration Commands

1. **User Story:** As a library consumer, I want to construct Neo4j administration commands (index/constraint management, SHOW commands, transaction management), so that I can manage database schema and operations programmatically.
   - Acceptance Criteria:
     1. When `CREATE INDEX` is used, the system shall render index creation syntax (range, text, point, full-text, vector, lookup).
     2. When `DROP INDEX` is used, the system shall render `DROP INDEX name [IF EXISTS]`.
     3. When `SHOW INDEXES` is used, the system shall render the index listing command with optional type filter, YIELD, and WHERE.
     4. When `CREATE CONSTRAINT` is used, the system shall render constraint creation syntax (uniqueness, existence, node key, relationship key, property type).
     5. When `DROP CONSTRAINT` is used, the system shall render `DROP CONSTRAINT name [IF EXISTS]`.
     6. When `SHOW CONSTRAINTS` is used, the system shall render the constraint listing command with optional type filter, YIELD, and WHERE.
     7. When `SHOW FUNCTIONS` or `SHOW PROCEDURES` is used, the system shall render the listing commands with optional type filter (`ALL`, `BUILT IN`, `USER DEFINED`), EXECUTABLE filter, YIELD, and WHERE.
     8. When `SHOW TRANSACTIONS` is used, the system shall render the transaction listing command with optional transaction IDs, YIELD, and WHERE.
     9. When `TERMINATE TRANSACTIONS` is used, the system shall render `TERMINATE TRANSACTIONS txId1, txId2, ...` with optional YIELD and WHERE.
