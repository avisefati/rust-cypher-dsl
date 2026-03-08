# Implementation Plan: Rust Cypher DSL

## 1. Project Scaffolding and Core Types

- [x] **1.1 Set up project structure and module skeleton**
  - Fix `Cargo.toml` edition to `"2021"`, add `pretty_assertions` dev-dependency
  - Create all module files under `src/` (empty `mod` declarations): `prelude.rs`, `macros.rs`, `cypher.rs`, `builder.rs`, `statement.rs`, `types/mod.rs`, `types/node.rs`, `types/relationship.rs`, `types/property.rs`, `types/expression.rs`, `types/literal.rs`, `types/parameter.rs`, `types/pattern.rs`, `types/condition.rs`, `types/operator.rs`, `clauses/mod.rs`, `functions/mod.rs`, `renderer/mod.rs`, `catalog.rs`
  - Configure clippy pedantic lints in `Cargo.toml` or `lib.rs`
  - Replace placeholder `add()` function in `lib.rs` with module declarations
  - Ref: Req 19.1 (clippy), Req 19.5 (zero deps)

- [x] **1.2 Implement `Expression` enum with literal variants and basic methods**
  - Define `Expression` enum with: `StringLiteral`, `IntegerLiteral`, `FloatLiteral`, `BooleanLiteral`, `NullLiteral`, `ListLiteral`, `MapLiteral`, `Parameter`, `Property`, `SymbolicName`, `Asterisk`, `RawExpression`
  - Derive `Debug`, `Clone`, `PartialEq`; use `Cow<'static, str>` for string data; wrap in `Rc` for cheap cloning
  - Implement `From<i32>`, `From<i64>`, `From<f64>`, `From<bool>`, `From<&str>`, `From<String>` for `Expression`
  - Write tests: each `From` conversion produces correct variant, `clone()` is cheap (Rc)
  - Ref: Req 2.1–2.8 (literals), Req 19.2 (traits), Req 19.3 (Cow), Req 19.5 (zero deps)

- [x] **1.3 Implement `Operator` enums and `Expression` comparison/arithmetic methods**
  - Define `ComparisonOp`, `BooleanOp`, `MathOp`, `StringPredicateOp`, `Operator` enums
  - Add `Operation` variant to `Expression`
  - Implement methods on `Expression`: `eq()`, `ne()`, `lt()`, `lte()`, `gt()`, `gte()`, `add()`, `subtract()`, `multiply()`, `divide()`, `remainder()`, `pow()`
  - Implement `as_alias()` method returning `Aliased` variant
  - Write tests: `lit(5).eq(3)` produces correct `Operation`, `as_alias()` wraps correctly
  - Ref: Req 3.1 (comparison operators), Req 2.6 (property access)

- [x] **1.4 Implement `Condition` enum with composition methods**
  - Define `Condition` enum with all variants: `Comparison`, `Compound`, `Not`, `IsNull`, `IsNotNull`, `StringPredicate`, `HasLabels`, `In`, `PatternCondition`, `ExistentialSubquery`, `ExpressionCondition`, `IsTrue`, `IsFalse`, `RegexMatch`, `TypePredicate`, `IsNormalized`, `NoCondition`
  - Implement `.and()`, `.or()`, `.xor()`, `.not()` composition methods
  - Implement `NoCondition` collapsing: `NoCondition.and(x)` returns `x`
  - Add condition-producing methods to `Expression`: `is_null()`, `is_not_null()`, `starts_with()`, `ends_with()`, `contains()`, `matches()`, `regex_match()`, `in_list()`, `is_type()`, `is_normalized()`, `is_not_normalized()`
  - Implement `From<Condition>` for `Expression`
  - Write tests: boolean composition, `NoCondition` collapsing, string predicates, regex
  - Ref: Req 3.1–3.12 (all condition types)

- [x] **1.5 Implement `Parameter` and `Property` types**
  - Define `Parameter` struct with name and optional bound value
  - Define `Property` struct with container expression and property name chain
  - Implement `From<Parameter>` and `From<Property>` for `Expression`
  - Add `property()` method to `Expression` for chained property access
  - Implement comparison/arithmetic delegation on `Property` (via `Into<Expression>`)
  - Write tests: parameter renders `$name`, property access chains correctly
  - Ref: Req 2.5 (parameters), Req 2.6 (property access)

## 2. Node, Relationship, and Pattern Types

- [x] **2.1 Implement `Node` type with labels, naming, and properties**
  - Define `Node` struct with `Rc<NodeInner>` wrapping: symbolic name, labels, properties, label expression
  - Define `NodeLabel` and `LabelExpression` enum (`Label`, `And`, `Or`, `Not`, `Wildcard`)
  - Implement `node()` free function, `.named()`, `.with_properties()`, `.property()`, `.has_labels()`
  - Implement `From<Node>` for `Expression`
  - Write tests: named node, anonymous node, multi-label node, label expressions (`&`, `|`, `!`, `%`), node with properties via `props!{}`
  - Ref: Req 1.1–1.2 (node creation), Req 1.8 (label expressions)

- [x] **2.2 Implement `props!{}` macro**
  - Define `props!{}` macro in `src/macros.rs` producing `Expression::MapLiteral`
  - Export via `#[macro_export]` and re-export in prelude
  - Write tests: empty props, single entry, multiple entries, mixed literal and param values
  - Ref: Req 2.8 (map literals), design 3.14

- [x] **2.3 Implement `RelationshipDetail` and `Relationship` types with method syntax**
  - Define `RelationshipDetail` struct (types, name, length, properties)
  - Define `Relationship` struct (left, right, direction, details)
  - Define `RelationshipBuilder` (knows left node + detail, awaits `.to()`/`.from()`/`.between()`)
  - Define `Direction` and `RelationshipLength` enums
  - Implement `rel()` free function, `.named()`, `.with_properties()`, `.min()`, `.max()`, `.unbounded()` on `RelationshipDetail`
  - Implement `.rel()` on `Node` returning `RelationshipBuilder`
  - Implement `.to()`, `.from()`, `.between()` on `RelationshipBuilder` returning `Relationship`
  - Implement `.inverse()` and `.property()` on `Relationship`
  - Write tests: outgoing, incoming, undirected, with properties, with variable-length, inverse
  - Ref: Req 1.3–1.5 (relationship creation, direction, properties, variable-length)

- [x] **2.4 Implement relationship chaining**
  - Implement `.rel()` on `Relationship` returning a new `RelationshipBuilder` from the right node
  - Define `RelationshipChain` for multi-hop patterns
  - Write tests: `a.rel("R1").to(b).rel("R2").to(c)` produces 3-node chain, mixed directions in chain
  - Ref: Req 1.6 (relationship chaining)

- [x] **2.5 Implement `>>` / `<<` operator overloading for relationships**
  - Define `OutgoingHalf` and `IncomingHalf` intermediate types
  - Implement `Shr<RelationshipDetail> for Node` → `OutgoingHalf`
  - Implement `Shr<Node> for OutgoingHalf` → `Relationship`
  - Implement `Shl<RelationshipDetail> for Node` → `IncomingHalf`
  - Implement `Shl<Node> for IncomingHalf` → `Relationship`
  - Implement chaining: `Shr<RelationshipDetail> for Relationship` and `Shl<RelationshipDetail> for Relationship`
  - Write tests: `a >> rel("R") >> b` equals `a.rel("R").to(b)`, mixed `>>` and `<<`, pre-built relationship details with operators
  - Ref: Req 19.8 (operator syntax)

- [x] **2.6 Implement `Pattern`, `PatternElement`, and `NamedPath`**
  - Define `Pattern`, `PatternElement` enum, `NamedPath`
  - Define `IntoPattern` trait; implement for `Node`, `Relationship`, `RelationshipChain`, tuples, `Vec`
  - Implement named path: `path("p").defined_by(pattern)` or similar
  - Write tests: single-node pattern, multi-element pattern (comma-separated), named path `p = (a)-->(b)`
  - Ref: Req 1.7 (named paths)

## 3. Basic Renderer

- [x] **3.1 Implement default renderer for expressions and conditions**
  - Define `Renderer` trait, `RenderConfig`, `EscapeMode` in `src/renderer/mod.rs`
  - Implement `DefaultRenderer` in `src/renderer/default.rs`
  - Implement rendering for all `Expression` variants: literals (string escaping with single quotes, numeric, boolean, NULL), parameters (`$name`), properties (`container.prop`), operations (infix with parens), function invocations, aliases (`AS`), lists, maps, asterisk, symbolic names, raw
  - Implement rendering for all `Condition` variants: comparisons, AND/OR/XOR/NOT, IS NULL, string predicates, regex `=~`, type predicates `IS :: TYPE`, IS NORMALIZED, HasLabels, IN
  - Write tests: one test per expression variant, one per condition variant, verifying rendered strings match expected Cypher
  - Ref: Req 14.1 (single-line), Req 14.3–14.4 (escaping)

- [x] **3.2 Implement renderer for nodes, relationships, and patterns**
  - Render `Node`: `(name:\`Label\`)`, anonymous `(:\`Label\`)`, with properties `{key: value}`, with label expressions
  - Render `Relationship`: arrow direction (`-->`, `<--`, `--`), type, name, properties, variable-length `[*min..max]`
  - Render `RelationshipChain`: multi-hop concatenation
  - Render `NamedPath`: `p = (pattern)`
  - Render `Pattern`: comma-separated elements
  - Write tests ported from Java `CypherIT`: `unrelatedNodes`, `asteriskShouldWork`, `simpleRelationship`, `simpleRelationshipWithProperties`, `simpleRelationshipWithReturn`
  - Ref: Req 1.1–1.8 (all node/relationship rendering)

- [x] **3.3 Implement `Display` for `Statement` and `Statement::render()`**
  - Define `Statement` enum and `SinglePartQuery` struct
  - Implement `Statement::render()` delegating to `DefaultRenderer`
  - Implement `Display` trait for `Statement`
  - Wire up clause rendering (initially just MATCH + RETURN for basic end-to-end testing)
  - Write tests: build a simple statement, verify `render()` and `format!("{}")` produce same output
  - Ref: Req 14.5 (Display trait)

## 4. Clauses

- [x] **4.1 Implement `MatchClause` and `WhereClause`**
  - Define `Clause` enum with `Match`, `OptionalMatch`, `Where` variants
  - Define `MatchClause` struct (optional flag, pattern) and `WhereClause` struct (condition)
  - Implement rendering: `MATCH pattern`, `OPTIONAL MATCH pattern`, `WHERE condition`
  - Write tests: simple match, optional match, match with where, multiple match clauses
  - Ref: Req 4.1–4.3, 4.6–4.7

- [x] **4.2 Implement `ReturnClause` with ORDER BY, SKIP, LIMIT**
  - Define `ReturnClause` (distinct, expressions), `OrderByClause` (sort items with direction), `SkipClause`, `LimitClause`
  - Define `SortItem` and `SortDirection` types
  - Add `ascending()` / `descending()` methods to `Expression`
  - Implement rendering: `RETURN expr AS alias`, `RETURN DISTINCT`, `RETURN *`, `ORDER BY`, `SKIP n`, `LIMIT n`
  - Write tests: simple return, aliased return, distinct, order by with direction, skip + limit, wildcard
  - Ref: Req 5.1–5.9

- [x] **4.3 Implement `WithClause` and `UnwindClause`**
  - Define `WithClause` and `UnwindClause` structs
  - Implement rendering: `WITH expr AS alias`, `UNWIND expr AS alias`
  - Write tests: with single expression, with multiple, unwind list
  - Ref: Req 4.4–4.5

- [x] **4.4 Implement `CreateClause` and `MergeClause`**
  - Define `CreateClause`, `MergeClause`, `MergeAction` enum (`OnCreate`, `OnMatch`), `SetItem` enum
  - Implement rendering: `CREATE pattern`, `MERGE pattern ON CREATE SET ... ON MATCH SET ...`
  - Write tests: simple create, merge with on create, merge with on match, multiple actions
  - Ref: Req 6.1–6.4

- [x] **4.5 Implement `SetClause`, `DeleteClause`, `RemoveClause`**
  - Define `SetClause` (items), `DeleteClause` (detach flag, expressions), `RemoveClause`
  - Implement `Property.to(value)` returning `SetItem::Property`
  - Implement rendering: `SET n.prop = val`, `SET n:Label`, `SET n += {map}`, `DELETE expr`, `DETACH DELETE`, `REMOVE n.prop`, `REMOVE n:Label`
  - Write tests: each SET variant, delete, detach delete, remove property, remove label
  - Ref: Req 6.5–6.11

- [x] **4.6 Implement `ForeachClause`**
  - Define `ForeachClause` (variable, list, update clauses)
  - Implement rendering: `FOREACH (var IN list | clauses)`
  - Write tests: foreach with set, foreach with create
  - Ref: Req 6.12

- [x] **4.7 Implement `CallClause` and `InQueryCallClause`**
  - Define `CallClause` (procedure name, args, yield fields, where condition)
  - Define `InQueryCallClause` (subquery statement, in-transactions config)
  - Implement rendering: `CALL proc(args)`, `CALL proc() YIELD f1, f2`, `CALL { subquery }`, `CALL { subquery } IN TRANSACTIONS`
  - Write tests: standalone call, call with yield, call with yield + where, in-query call, in transactions
  - Ref: Req 7.1–7.5

- [x] **4.8 Implement `LoadCsvClause`**
  - Define `LoadCsvClause` (url, alias, with_headers, field_terminator)
  - Implement rendering: `LOAD CSV FROM 'url' AS row`, `WITH HEADERS`, `FIELDTERMINATOR`
  - Write tests: basic load csv, with headers, custom field terminator
  - Ref: Req 9.1–9.3

- [x] **4.9 Implement `UseClause` and query hint clauses**
  - Define `UseClause`, `UsingIndexClause`, `UsingScanClause`, `UsingJoinClause`
  - Implement rendering: `USE graph`, `USING INDEX var:Label(prop)`, `USING INDEX SEEK`, `USING SCAN var:Label`, `USING JOIN ON var`
  - Write tests: use with name, use with function, each hint type
  - Ref: Req 10.1–10.2, Req 11.1–11.4

## 5. Statement Builder (Fluent API)

- [x] **5.1 Implement `Cypher` entry point and `OngoingMatch` → `OngoingReturn` → `Statement` flow**
  - Define `Cypher` struct with `match_node()`, `optional_match()` associated functions
  - Define `OngoingMatch` struct with `.where_()`, `.returning()`, `.with()` methods
  - Define `OngoingReadingWithWhere` with `.and()`, `.or()`, `.returning()`, `.with()`
  - Define `OngoingReturn` with `.order_by()`, `.skip()`, `.limit()`, `.build()`
  - Define `IntoReturnExprs` trait; implement for `Expression`, `Node`, `Property`, tuples (2–6), `Vec<Expression>`
  - Write tests: `Cypher::match_node(n).returning(n).build()` renders correctly, match + where + return, match + return + order by + skip + limit
  - Ref: Req 12.1–12.3, Req 12.8

- [x] **5.2 Implement `OngoingWith` for multi-part queries**
  - Define `OngoingWith` with `.match_node()`, `.optional_match()`, `.where_()`, `.returning()`, `.unwind()`
  - Support chaining: `match → with → match → return` (multi-part queries)
  - Write tests: match-with-match-return, with aliased expressions, with distinct
  - Ref: Req 12.1, design 3.10 (loop-back from OngoingWith)

- [x] **5.3 Implement `Cypher::create()`, `Cypher::merge()`, write builders**
  - Define `OngoingUpdate` with `.set()`, `.delete()`, `.detach_delete()`, `.remove()`, `.returning()`, `.with()`
  - Define `OngoingMerge` with `.on_create()`, `.on_match()`, `.set()`, `.returning()`
  - Implement `Cypher::create()`, `Cypher::merge()`
  - Write tests: create node, create relationship, merge with on-create/on-match set, create then return
  - Ref: Req 12.4–12.5

- [x] **5.4 Implement `Cypher::unwind()`, `Cypher::call_procedure()`, `Cypher::call_subquery()`**
  - Define `OngoingUnwind` with `.as_()` method transitioning to `OngoingWith`
  - Define `OngoingStandaloneCall` with `.yield_()`, `.where_()`, `.build()`
  - Define `OngoingInQueryCall` with `.in_transactions()`, linking back to reading/writing builders
  - Implement `Cypher::unwind()`, `Cypher::call_procedure()`, `Cypher::call_subquery()`
  - Write tests: unwind list as var, procedure call with yield, in-query call subquery
  - Ref: Req 12.6–12.7, Req 7.1–7.5

- [x] **5.5 Implement `Cypher::union()`, `Cypher::union_all()`, `Cypher::explain()`, `Cypher::profile()`**
  - Implement union/union-all statement composition
  - Implement explain/profile wrapping
  - Write tests: union of two match-returns, union all, explain prefix, profile prefix
  - Ref: Req 12.9–12.10

- [x] **5.6 Implement `Cypher::load_csv()` and `Cypher::using_periodic_commit()`**
  - Define `OngoingLoadCsv` with `.as_()`, `.field_terminator()`, linking to reading builders
  - Define `OngoingPeriodicCommit` linking to load-csv
  - Write tests: load csv flow, with headers, periodic commit + load csv
  - Ref: Req 12.11, Req 9.1–9.4

- [x] **5.7 Wire `OngoingMatch` optional_match and mixed read/write chaining**
  - Add `.optional_match()` to `OngoingMatch`, `OngoingReadingWithWhere`, `OngoingWith`
  - Add `.create()`, `.merge()`, `.delete()`, `.detach_delete()`, `.set()`, `.remove()` to reading states
  - Add `.foreach()` where applicable
  - Write tests: match + optional match + return, match + create + return, complex mixed read/write
  - Ref: Req 4.2, Req 12.1–12.2

## 6. Prelude and Ergonomics

- [x] **6.1 Implement prelude module with all free functions**
  - Create `src/prelude.rs` re-exporting: `node`, `any_node`, `any_node_named`, `lit`, `lit_true`, `lit_false`, `lit_null`, `param`, `param_with_value`, `name`, `rel`, `prop`, `list_of`, `map_of`, `not`, `case`, `list_comprehension`, `sort`, `raw`, `reduce`, `quantified_path`, `shortest`, `all_shortest`, `any_path`, `shortest_groups`, `Cypher`
  - Re-export key types: `Node`, `Expression`, `Condition`, `Statement`, `RelationshipDetail`, `Property`, `Parameter`
  - Write tests: verify all free functions are accessible via `use rust_cypher_dsl::prelude::*`
  - Ref: Design 3.12

## 7. Built-in Functions

- [x] **7.1 Implement aggregation functions**
  - Implement `count`, `count_distinct`, `sum`, `sum_distinct`, `avg`, `avg_distinct`, `min`, `min_distinct`, `max`, `max_distinct`, `collect`, `collect_distinct`, `percentile_cont`, `percentile_disc`, `st_dev`, `st_dev_p`
  - Each returns `Expression::FunctionInvocation` with correct name and distinct flag
  - Write tests: each function renders correctly, distinct variants include `DISTINCT` keyword
  - Ref: Req 13.1–13.2

- [x] **7.2 Implement scalar and type conversion functions**
  - Implement `id`, `element_id`, `type_of`, `coalesce`, `timestamp`, `size`, `head`, `last`, `start_node`, `end_node`, `properties`, `random_uuid`, `null_if`, `value_type`, `char_length`, `length`, `path_length`
  - Implement `to_integer`, `to_integer_or_null`, `to_float`, `to_float_or_null`, `to_boolean`, `to_boolean_or_null`, `to_string_fn`, `to_string_or_null`
  - Write tests: each function renders correctly
  - Ref: Req 13.3–13.4

- [x] **7.3 Implement string functions**
  - Implement `to_lower`, `lower`, `to_upper`, `upper`, `trim`, `btrim`, `ltrim`, `rtrim`, `replace`, `substring`, `left`, `right`, `split`, `reverse_str`, `normalize`
  - Write tests: each function renders correctly
  - Ref: Req 13.5

- [x] **7.4 Implement math functions (numeric, logarithmic, trigonometric)**
  - Implement numeric: `abs`, `ceil`, `ceiling`, `floor`, `round`, `sign`, `rand`, `is_nan`
  - Implement logarithmic: `sqrt`, `log`, `ln`, `log10`, `exp`, `e_const`
  - Implement trigonometric: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `cot`, `cosh`, `sinh`, `tanh`, `coth`, `degrees`, `radians`, `haversin`, `pi`
  - Write tests: representative tests per category
  - Ref: Req 13.6–13.8

- [x] **7.5 Implement list and coll functions**
  - Implement list: `range`, `keys`, `labels_fn`, `nodes_fn`, `relationships_fn`, `tail`, `reverse_list`, `reduce_fn`, `to_boolean_list`, `to_float_list`, `to_integer_list`, `to_string_list`
  - Implement coll namespace: `coll_distinct`, `coll_flatten`, `coll_index_of`, `coll_insert`, `coll_max`, `coll_min`, `coll_remove`, `coll_sort` (rendered as `coll.distinct()`, etc.)
  - Write tests: each function renders with correct qualified name
  - Ref: Req 13.9–13.10

- [x] **7.6 Implement temporal functions**
  - Implement: `datetime_fn`, `localdatetime`, `date_fn`, `localtime`, `time_fn`, `duration_fn`
  - Implement duration utilities: `duration_between`, `duration_in_days`, `duration_in_months`, `duration_in_seconds`
  - Implement epoch: `datetime_from_epoch`, `datetime_from_epoch_millis`
  - Implement: `format_fn`
  - Implement `.realtime()`, `.statement()`, `.transaction()`, `.truncate()` method variants (qualified function names like `datetime.realtime`)
  - Write tests: each function and variant renders correctly
  - Ref: Req 13.11–13.14

- [x] **7.7 Implement spatial, predicate, database, graph, vector, and load-csv functions**
  - Implement spatial: `point`, `point_distance`, `point_within_bbox`
  - Implement predicate: `exists`, `all_fn`, `all_reduce`, `any_fn`, `none_fn`, `single`, `is_empty`
  - Implement database: `db_name_from_element_id`
  - Implement graph: `graph_by_element_id`, `graph_by_name`, `graph_names`, `graph_properties_by_name`
  - Implement vector: `vector_fn`, `vector_similarity_cosine`, `vector_similarity_euclidean`
  - Implement load_csv: `file_fn`, `linenumber`
  - Write tests: representative tests per category
  - Ref: Req 13.15–13.20

- [x] **7.8 Implement custom/user-defined function invocation**
  - Implement `custom_function(name, args)` free function for arbitrary function calls
  - Write tests: custom function with 0, 1, and multiple args renders correctly
  - Ref: Req 13.21

## 8. Advanced Expressions

- [x] **8.1 Implement CASE expressions**
  - Define `CaseBuilder` with `.when()`, `.then()`, `.else_()` methods
  - Implement `case()` free function returning `CaseBuilder`
  - Implement rendering for simple CASE (`CASE expr WHEN val THEN result END`) and generic CASE (`CASE WHEN cond THEN result ELSE default END`)
  - Write tests: simple case, generic case, multiple when clauses, with else
  - Ref: Req 15.1–15.2

- [x] **8.2 Implement list comprehensions and pattern comprehensions**
  - Define `ListComprehensionBuilder` with `.in_()`, `.where_()`, `.pipe()` methods
  - Add `ListComprehension` and `PatternComprehension` variants to `Expression`
  - Implement rendering: `[var IN list WHERE cond | expr]`, `[(pattern) WHERE cond | expr]`
  - Write tests: list comprehension with/without where, with/without projection, pattern comprehension
  - Ref: Req 15.3–15.4

- [x] **8.3 Implement map projections**
  - Define `MapProjectionEntry` enum (property, all-properties, literal entry)
  - Add `MapProjection` variant to `Expression`
  - Implement rendering: `variable {.prop1, .prop2, key: expr, .*}`
  - Write tests: dot-property entries, literal entries, mixed
  - Ref: Req 15.5

- [x] **8.4 Implement subquery expressions (EXISTS, COUNT, COLLECT)**
  - Implement rendering for `ExistentialSubquery`, `CountSubquery`, `CollectSubquery` Expression variants
  - Wire into condition system: `EXISTS { MATCH ... }` usable in WHERE
  - Write tests: exists subquery in where, count subquery as expression, collect subquery
  - Ref: Req 15.6–15.8, Req 7.6

- [x] **8.5 Implement `reduce()` expression**
  - Add `ReduceExpression` variant to `Expression` (already in enum)
  - Implement `reduce()` free function in prelude
  - Implement rendering: `reduce(acc = init, x IN list | expr)`
  - Write tests: reduce with numeric accumulation, reduce with string
  - Ref: Req 2.9

## 9. Advanced Patterns

- [x] **9.1 Implement quantified path patterns (QPP)**
  - Define `QuantifiedPath` struct (pattern, quantifier, optional where)
  - Define `Quantifier` enum (`Star`, `Plus`, `Exact`, `Range`)
  - Implement `quantified_path()` free function with `.star()`, `.plus()`, `.range(n,m)`, `.where_()` builder methods
  - Implement rendering: `((a)-[:R]->(b)){1,3}`, `((a)-[:R]->(b))+`, `((a)-[:R]->(b))*`
  - Write tests: each quantifier type, QPP with inline where predicate
  - Ref: Req 8.1–8.3, 8.5–8.6

- [x] **9.2 Implement quantified relationships**
  - Define `QuantifiedRelationship` struct
  - Add `.quantified(min, max)` method to `RelationshipDetail` or `RelationshipBuilder`
  - Implement rendering: `(a)-[:R]->{1,5}(b)`, `(a)-[:R]->+(b)`
  - Write tests: quantified relationship with range, with plus, with star
  - Ref: Req 8.4

- [x] **9.3 Implement path selectors**
  - Define `PathSelector` enum (`Shortest`, `AllShortest`, `Any`, `ShortestGroups`)
  - Implement `shortest()`, `all_shortest()`, `any_path()`, `shortest_groups()` free functions
  - Implement rendering: `SHORTEST 1 (pattern)`, `ALL SHORTEST (pattern)`, `ANY (pattern)`, `SHORTEST 2 GROUPS (pattern)`
  - Implement named paths with selectors: `p = SHORTEST 1 (pattern)`
  - Write tests: each selector type, named path with selector
  - Ref: Req 8.7–8.11

## 10. Pretty Renderer

- [ ] **10.1 Implement pretty-printing renderer**
  - Define `PrettyRenderer` in `src/renderer/pretty.rs`
  - Implement indented, multi-line output with configurable indent string
  - Implement `Statement::render_with(config)` accepting `RenderConfig`
  - Write tests: multi-clause query renders with newlines and indentation, configurable indent width
  - Ref: Req 14.2

## 11. Statement Catalog

- [ ] **11.1 Implement `StatementCatalog` for introspection**
  - Define `StatementCatalog` struct (labels, relationship types, properties, parameters)
  - Define `CatalogProperty` struct
  - Implement AST walker that collects metadata from a `Statement`
  - Implement `Statement::catalog()` method
  - Implement `Statement::get_parameter_names()` and `Statement::get_parameters()`
  - Write tests: catalog from simple query lists labels, types, properties, params correctly; catalog from complex multi-clause query
  - Ref: Req 16.1–16.4

## 12. Cypher 25 Clauses

- [ ] **12.1 Implement `FINISH`, `FILTER`, `LET` clauses**
  - Define `FilterClause`, `LetClause` structs and `Finish` variant in `Clause` enum
  - Implement rendering: `FINISH`, `FILTER predicate`, `LET var = expr`
  - Wire into builder: `.filter()`, `.let_()`, `.finish()` methods on appropriate builder states
  - Write tests: each clause renders correctly in a complete statement
  - Ref: Req 17.1–17.3

- [ ] **12.2 Implement `WHEN` and `NEXT` composed query support**
  - Extend `Statement` enum or add new composition types for conditional (`WHEN`) and sequential (`NEXT`) queries
  - Implement rendering for `WHEN` conditional branching and `NEXT` sequential chaining
  - Wire into `Cypher` entry point or statement composition API
  - Write tests: conditional query with WHEN, sequential query with NEXT
  - Ref: Req 17.4–17.5

## 13. Integration Test Porting

- [ ] **13.1 Port core tests from Java `CypherIT.java`**
  - Create `tests/cypher_it.rs` with header tracking Java test count
  - Port all node/relationship/pattern-related tests
  - Port all clause-related tests (match, return, where, with, unwind, create, merge, set, delete, remove)
  - Port all builder-related tests (multi-part queries, mixed read/write)
  - Each test uses `use rust_cypher_dsl::prelude::*` and `pretty_assertions::assert_eq`
  - Ref: Req 18.1

- [ ] **13.2 Port function, expression, subquery, and procedure tests**
  - Create `tests/functions_it.rs` porting from `FunctionsIT.java`
  - Create `tests/expressions_it.rs` porting from `ExpressionsIT.java`
  - Create `tests/subqueries_it.rs` porting from `SubqueriesIT.java`
  - Create `tests/procedure_calls_it.rs` porting from `ProcedureCallsIT.java`
  - Ref: Req 18.2–18.5

- [ ] **13.3 Write tests for Cypher features beyond Java DSL**
  - Create `tests/patterns_it.rs` for QPP, quantified relationships, path selectors
  - Create `tests/load_csv_it.rs` for LOAD CSV
  - Create `tests/query_hints_it.rs` for USING INDEX/SCAN/JOIN
  - Create `tests/cypher25_it.rs` for FINISH, FILTER, LET, WHEN, NEXT
  - Create `tests/renderer_tests.rs` for rendering config variations
  - Create `tests/catalog_tests.rs` for statement catalog introspection
  - Ref: Req 18.6–18.8

## 14. Documentation and Final Polish

- [ ] **14.1 Add doc comments to all public items**
  - Add `//!` crate-level documentation to `lib.rs` with usage examples
  - Add `///` doc comments to every public struct, enum, trait, function, and method
  - Ensure at least one `# Examples` doc-test per major public entry point
  - Run `cargo doc --no-deps` and verify no warnings
  - Ref: Req 19.7

- [ ] **14.2 Final clippy and test audit**
  - Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all issues
  - Run `cargo test` and verify all tests pass
  - Verify zero `todo!()`, `unimplemented!()`, or `bail!("not yet implemented")` in any source file
  - Ref: Req 19.1
