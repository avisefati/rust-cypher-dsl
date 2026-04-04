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
  - Implement `alias()` method returning `Aliased` variant
  - Write tests: `lit(5).eq(3)` produces correct `Operation`, `alias()` wraps correctly
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

- [x] **5.4 Implement `Cypher::unwind()`, `Cypher::call_procedure()`, `Cypher::call()`**
  - Define `OngoingUnwind` with `.as_()` method transitioning to `OngoingWith`
  - Define `OngoingStandaloneCall` with `.yield_()`, `.where_()`, `.build()`
  - Define `OngoingInQueryCall` with `.in_transactions()`, linking back to reading/writing builders
  - Implement `Cypher::unwind()`, `Cypher::call_procedure()`, `Cypher::call()`
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

- [x] **10.1 Implement pretty-printing renderer**
  - Define `PrettyRenderer` in `src/renderer/pretty.rs`
  - Implement indented, multi-line output with configurable indent string
  - Implement `Statement::render_with(config)` accepting `RenderConfig`
  - Write tests: multi-clause query renders with newlines and indentation, configurable indent width
  - Ref: Req 14.2

## 11. Statement Catalog

- [x] **11.1 Implement `StatementCatalog` for introspection**
  - Define `StatementCatalog` struct (labels, relationship types, properties, parameters)
  - Define `CatalogProperty` struct
  - Implement AST walker that collects metadata from a `Statement`
  - Implement `Statement::catalog()` method
  - Implement `Statement::get_parameter_names()` and `Statement::get_parameters()`
  - Write tests: catalog from simple query lists labels, types, properties, params correctly; catalog from complex multi-clause query
  - Ref: Req 16.1–16.4

## 12. Cypher 25 Clauses

- [x] **12.1 Implement `FINISH`, `FILTER`, `LET` clauses**
  - Define `FilterClause`, `LetClause` structs and `Finish` variant in `Clause` enum
  - Implement rendering: `FINISH`, `FILTER predicate`, `LET var = expr`
  - Wire into builder: `.filter()`, `.let_()`, `.finish()` methods on appropriate builder states
  - Write tests: each clause renders correctly in a complete statement
  - Ref: Req 17.1–17.3

- [x] **12.2 Implement `WHEN` and `NEXT` composed query support**
  - Extend `Statement` enum or add new composition types for conditional (`WHEN`) and sequential (`NEXT`) queries
  - Implement rendering for `WHEN` conditional branching and `NEXT` sequential chaining
  - Wire into `Cypher` entry point or statement composition API
  - Write tests: conditional query with WHEN, sequential query with NEXT
  - Ref: Req 17.4–17.5

## 13. Integration Test Porting

- [x] **13.1 Port core tests from Java `CypherIT.java`**
  - Create `tests/cypher_it.rs` with header tracking Java test count
  - Port all node/relationship/pattern-related tests
  - Port all clause-related tests (match, return, where, with, unwind, create, merge, set, delete, remove)
  - Port all builder-related tests (multi-part queries, mixed read/write)
  - Each test uses `use rust_cypher_dsl::prelude::*` and `pretty_assertions::assert_eq`
  - Ref: Req 18.1

- [x] **13.2 Port function, expression, subquery, and procedure tests**
  - Create `tests/functions_it.rs` porting from `FunctionsIT.java`
  - Create `tests/expressions_it.rs` porting from `ExpressionsIT.java`
  - Create `tests/subqueries_it.rs` porting from `SubqueriesIT.java`
  - Create `tests/procedure_calls_it.rs` porting from `ProcedureCallsIT.java`
  - Ref: Req 18.2–18.5

- [x] **13.3 Write tests for Cypher features beyond Java DSL**
  - Create `tests/patterns_it.rs` for QPP, quantified relationships, path selectors
  - Create `tests/load_csv_it.rs` for LOAD CSV
  - Create `tests/query_hints_it.rs` for USING INDEX/SCAN/JOIN
  - Create `tests/cypher25_it.rs` for FINISH, FILTER, LET, WHEN, NEXT
  - Create `tests/renderer_tests.rs` for rendering config variations
  - Create `tests/catalog_tests.rs` for statement catalog introspection
  - Ref: Req 18.6–18.8

## 14. Documentation and Final Polish

- [x] **14.1 Add doc comments to all public items**
  - Add `//!` crate-level documentation to `lib.rs` with usage examples
  - Add `///` doc comments to every public struct, enum, trait, function, and method
  - Ensure at least one `# Examples` doc-test per major public entry point
  - Run `cargo doc --no-deps` and verify no warnings
  - Ref: Req 19.7

- [x] **14.2 Final clippy and test audit**
  - Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all issues
  - Run `cargo test` and verify all tests pass
  - Verify zero `todo!()`, `unimplemented!()`, or `bail!("not yet implemented")` in any source file
  - Ref: Req 19.1

## 15. Security Hardening — Cypher Injection Prevention

- [x] **15.1 Add identifier validation helper and escape all identifier positions**
  - Add a `validate_identifier(name: &str) -> bool` helper that accepts `[a-zA-Z_][a-zA-Z0-9_]*`
  - Add an `escape_or_validate_identifier` helper that backtick-escapes identifiers that don't match the safe pattern
  - Apply escaping to: symbolic names (`SymbolicName` rendering), property names (in `write_property`), alias names (in `write_aliased`), map literal keys (in `write_map_literal`), map projection keys, LOAD CSV alias, FOREACH/UNWIND/LET variable names
  - Write tests: identifier with spaces gets escaped, identifier with backticks gets doubled, clean identifier passes through unchanged, injection payloads in each position are neutralized
  - Ref: Security audit finding #2–#6, #9

- [x] **15.2 Validate parameter names**
  - Add validation to `Parameter::new()` (and `param()` / `param_with_value()`) rejecting names that don't match `[a-zA-Z_][a-zA-Z0-9_]*`
  - Return a descriptive error or panic with a clear message on invalid parameter names
  - Write tests: valid names accepted, names with spaces/special chars rejected, injection payloads rejected
  - Ref: Security audit finding #3

- [x] **15.3 Validate procedure names**
  - Add validation to `CallClause` construction, allowing only `[a-zA-Z_][a-zA-Z0-9_.]*` (dots permitted for namespaced procedures like `db.index.fulltext.queryNodes`)
  - Return a descriptive error or panic with a clear message on invalid procedure names
  - Write tests: simple name accepted, dotted namespace accepted, injection payload rejected
  - Ref: Security audit finding #8

- [x] **15.4 Escape LOAD CSV field terminator value**
  - In `write_load_csv_clause`, escape single quotes inside the field terminator value (double them, consistent with string literal escaping)
  - Write tests: terminator with single quote is safely escaped, normal terminators unchanged
  - Ref: Security audit finding #7

- [x] **15.5 Rename `raw()` to `raw_unchecked()` and add safety documentation**
  - Rename `Expression::raw()` to `Expression::raw_unchecked()`
  - Rename the `raw()` free function in prelude to `raw_unchecked()`
  - Add prominent `# Safety` doc comment warning that the value is inserted verbatim with zero sanitization, and must never contain user-controlled input
  - Remove `raw_unchecked` from the default prelude re-exports (require explicit import)
  - Update all internal usages and tests
  - Write test: verify the rename compiles and renders identically
  - Ref: Security audit finding #1

- [x] **15.6 Validate function invocation names**
  - Add validation to `Expression::function_invocation()` allowing only `[a-zA-Z_][a-zA-Z0-9_.]*`
  - This covers built-in functions and `custom_function()` calls
  - Write tests: simple name accepted, dotted namespace accepted, injection payload rejected
  - Ref: Security audit finding #8 (extends to all function names)

- [x] **15.7 Final security verification**
  - Create `tests/security_it.rs` with injection-attempt tests for every input position: string literals, identifiers, parameter names, labels, relationship types, property names, aliases, map keys, procedure names, function names, LOAD CSV field terminator, raw_unchecked
  - Each test constructs a query with a crafted injection payload and verifies the rendered output is safe (escaped/rejected/backtick-quoted)
  - Run `cargo build` + `cargo test` + `cargo clippy --all-targets --all-features -- -D warnings`
  - Ref: All security audit findings

## 16. API Readability — Fluent Ergonomics

- [x] **16.1 Make comparison methods return `Condition` for fluent WHERE clauses**
  - Change `Expression::eq/ne/lt/lte/gt/gte` to return `Condition::Comparison` instead of `Expression::Operation`
  - Add comparison convenience methods (`eq/ne/lt/lte/gt/gte`) and `alias()` to `Property`, delegating to `Expression`
  - Replace all 57 verbose `Condition::Comparison { left, operator, right }` struct literals with fluent calls (e.g. `prop("n", "age").gt(21_i32)`)
  - Math operators (`add/subtract/multiply/divide/remainder/pow`) remain returning `Expression`
  - Existing `From<Condition> for Expression` conversion covers the rare `RETURN a > b` use-case
  - Net reduction of ~200 lines across 19 files; all 921 tests pass

- [x] **16.2 Rename `Cypher::match_node()` to `Cypher::match_()`**
  - Add `Cypher::match_()` as the primary entry point (consistent with `where_()` naming)
  - Keep `Cypher::match_node()` as a deprecated alias for backward compatibility
  - Update all tests and internal callers to use `match_()`
  - Update builder methods: `StatementBuilder::match_node()` → add `match_()` alias

- [x] **16.3 Add quantifier shorthand methods**
  - Add `.times(n)` to `RelationshipDetail` — shorthand for `.quantified(Quantifier::Exact(n))`
  - Add `.plus()` to `RelationshipDetail` — shorthand for `.quantified(Quantifier::OneOrMore)`
  - Add `.star()` to `RelationshipDetail` — shorthand for `.quantified(Quantifier::ZeroOrMore)`
  - Add postfix `.times(n)` / `.plus()` / `.star()` on `Relationship` and `RelationshipChain` (applies to last relationship)
  - Existing `.quantified(Quantifier)` remains for custom ranges (`Range(min, max)`)
  - Write tests for all shorthands on `RelationshipDetail`, `Relationship`, and `RelationshipChain`

- [x] **16.4 Add relationship shorthand methods on `Node`**
  - **Typed relationships** (first arg: `impl Into<RelationshipDetail>`):
    - `a.to("R", b)` — creates outgoing relationship `(a)-[:R]->(b)`
    - `a.from("R", b)` — creates incoming relationship `(a)<-[:R]-(b)`
    - `a.linked("R", b)` — creates undirected relationship `(a)-[:R]-(b)`
  - **Untyped relationships** (Option A — distinct names):
    - `a.link_to(b)` — creates untyped outgoing `(a)-->(b)`
    - `a.link_from(b)` — creates untyped incoming `(a)<--(b)`
    - `a.link(b)` — creates untyped undirected `(a)--(b)`
  - Add `impl From<&str> for RelationshipDetail` to enable `a.to("R", b)` syntax
  - Pre-built details work via `Into<RelationshipDetail>`: `a.to(rel("KNOWS").named("r"), b)`
  - Add same methods on `Relationship` and `RelationshipChain` for chaining: `a.to("R1", b).to("R2", c)`
  - Write tests covering typed, untyped, pre-built, and chained patterns

## 17. Cypher Parser — Phase 1 (Core MVP)

- [x] **17.1 Project scaffolding: feature flag, module skeleton, dependencies**
  - Add `parser` feature flag to `Cargo.toml` with `winnow = { version = "0.6", optional = true }` dependency
  - Create `src/parser/mod.rs` with public `parse()` function stub (returns `todo!()`) and `ParseError` type
  - Create empty module files: `error.rs`, `tokens.rs`, `lexer.rs`, `grammar.rs`, `clauses.rs`, `expressions.rs`, `patterns.rs`, `conditions.rs`
  - Gate the `parser` module with `#[cfg(feature = "parser")]` in `src/lib.rs`
  - Verify: `cargo build` (no parser), `cargo build --features parser` (with parser)
  - Ref: Design Phase 15 (module layout, library choice)

- [x] **17.2 Implement `ParseError` type with position, context, and Display**
  - Define `ParseError` struct in `src/parser/error.rs`: `offset`, `line`, `column`, `expected`, `context`, `snippet`
  - Implement `std::fmt::Display` with formatted error message showing position, snippet, and context
  - Implement `std::error::Error` for `ParseError`
  - Add helper `from_winnow_error(input: &str, err: ContextError)` to convert winnow errors
  - Write tests: display format, line/column calculation from offset
  - Ref: Design Phase 15 (Public API, Error Handling)

- [x] **17.3 Implement `Token` and `Keyword` enums**
  - Define `Token` enum in `src/parser/tokens.rs` with all variants: keywords, identifiers, literals, operators, punctuation
  - Define `Keyword` enum with all Cypher keywords (MATCH, RETURN, WHERE, WITH, etc.)
  - Implement `Keyword::from_str()` with case-insensitive matching
  - Write tests: keyword lookup is case-insensitive, all keywords recognized
  - Ref: Design Phase 15 (Token Types)

- [x] **17.4 Implement lexer: whitespace, comments, punctuation, operators**
  - Implement whitespace/comment skipping (line comments `//`, block comments `/* */`)
  - Implement single-char operator/punctuation tokenization: `( ) [ ] { } , . : ; | & ! ~ $ * + - / % ^`
  - Implement multi-char operators: `<>`, `<=`, `>=`, `->`, `<-`, `=~`, `+=`, `..`
  - Write tests: each operator tokenizes correctly, whitespace stripped, comments stripped
  - Ref: Design Phase 15 (Architecture — lexer phase)

- [x] **17.5 Implement lexer: identifiers, keywords, escaped identifiers**
  - Implement unquoted identifier recognition: `[a-zA-Z_][a-zA-Z0-9_]*`
  - Implement keyword vs identifier disambiguation (try keyword lookup first, fall back to identifier)
  - Implement backtick-escaped identifiers: `` `my identifier` `` with doubled backtick escaping
  - Write tests: `person` → Identifier, `MATCH` → Keyword, `` `my var` `` → EscapedIdentifier, mixed case keywords
  - Ref: Design Phase 15 (Token Types, Key Design Decisions #2)

- [x] **17.6 Implement lexer: string literals, numeric literals, boolean/null**
  - Implement single-quoted string literals with escape sequences (`\'`, `\\`, `\n`, `\t`, `\r`, `\uXXXX`)
  - Implement double-quoted string literals with same escape handling
  - Implement integer literals (decimal)
  - Implement float literals (decimal with `.` and optional exponent)
  - Implement boolean (`true`/`false`) and null (`null`) as keywords
  - Write tests: string escaping, integer parsing, float parsing, edge cases (empty string, negative numbers via unary minus)
  - Ref: Design Phase 15 (Token Types — literals)

- [x] **17.7 Implement full tokenizer: `tokenize(&str) -> Result<Vec<Token>, ParseError>`**
  - Combine all lexer components into a `tokenize()` function that produces a complete token stream
  - Handle end-of-input
  - Produce `ParseError` on unrecognized characters with position info
  - Write tests: full query tokenization (`MATCH (n:Person) WHERE n.age > 21 RETURN n`), error on invalid chars
  - Ref: Design Phase 15 (Architecture)

- [x] **17.8 Implement expression parser: atoms (literals, identifiers, parameters, parenthesized)**
  - Parse integer, float, string, boolean, null literals → `Expression` variants
  - Parse identifiers → `Expression::SymbolicName`
  - Parse parameters (`$name`) → `Expression::Parameter`
  - Parse parenthesized expressions `(expr)`
  - Parse `*` → `Expression::Asterisk`
  - Write tests: each atom type parses correctly
  - Ref: Design Phase 15 (Grammar — Atom)

- [x] **17.9 Implement expression parser: property access and function calls**
  - Parse property access: `n.name`, `n.address.city` → `Expression::Property`
  - Parse function calls: `name(arg1, arg2)`, `count(DISTINCT x)` → `Expression::FunctionInvocation`
  - Parse qualified function names: `coll.sort(list)`, `datetime.realtime()`
  - Write tests: simple property, chained property, function with 0/1/many args, distinct function, qualified names
  - Ref: Design Phase 15 (Grammar — PostfixExpr, Atom)

- [x] **17.10 Implement expression parser: arithmetic and comparison operators (precedence climbing)**
  - Parse arithmetic: `+`, `-`, `*`, `/`, `%`, `^` with correct precedence
  - Parse unary: `-x`, `+x`
  - Parse comparison: `=`, `<>`, `<`, `>`, `<=`, `>=` → `Condition::Comparison`
  - Parse `IS NULL`, `IS NOT NULL` → `Condition::IsNull` / `Condition::IsNotNull`
  - Parse `IN [list]` → `Condition::In`
  - Write tests: precedence (`1 + 2 * 3` = `1 + (2 * 3)`), associativity, comparisons, IS NULL
  - Ref: Design Phase 15 (Grammar — precedence climbing)

- [x] **17.11 Implement expression parser: boolean operators and string predicates**
  - Parse `AND`, `OR`, `XOR`, `NOT` → `Condition::Compound` / `Condition::Not`
  - Parse `STARTS WITH`, `ENDS WITH`, `CONTAINS` → `Condition::StringPredicate`
  - Parse `=~` regex match → `Condition::RegexMatch`
  - Write tests: boolean composition with precedence (AND binds tighter than OR), string predicates, regex
  - Ref: Design Phase 15 (Grammar — OrExpression through NotExpression)

- [x] **17.12 Implement expression parser: aliases (`AS`)**
  - Parse `expression AS identifier` → `Expression::Aliased`
  - Handle alias in return items, with items
  - Write tests: `n.name AS personName`, `count(*) AS total`
  - Ref: Design Phase 15 (Grammar — ReturnItems)

- [x] **17.13 Implement pattern parser: nodes**
  - Parse node patterns: `(n)`, `(:Label)`, `(n:Label)`, `(n:Label {key: value})`
  - Parse multi-label nodes: `(n:A:B)`
  - Parse anonymous nodes: `()`
  - Map to existing `Node` type via `node()`, `.named()`, `.with_properties()`
  - Write tests: all node variants, with properties via map literal parsing
  - Ref: Design Phase 15 (Grammar — NodePattern)

- [x] **17.14 Implement pattern parser: relationships and chains**
  - Parse typed relationships: `(a)-[:R]->(b)`, `(a)<-[:R]-(b)`, `(a)-[:R]-(b)`
  - Parse untyped relationships: `(a)-->(b)`, `(a)<--(b)`, `(a)--(b)`
  - Parse relationship details: named `[r:R]`, with properties `[:R {k: v}]`
  - Parse multi-hop chains: `(a)-[:R1]->(b)-[:R2]->(c)`
  - Map to existing `Relationship`, `RelationshipChain`, `RelationshipDetail` types
  - Write tests: all direction variants, named + typed, multi-hop chains, untyped
  - Ref: Design Phase 15 (Grammar — RelPattern)

- [x] **17.15 Implement pattern parser: named paths**
  - Parse named paths: `p = (a)-[:R]->(b)`
  - Map to existing `NamedPath` / `path()` API
  - Write tests: named path with simple pattern, named path with chain
  - Ref: Design Phase 15 (Grammar — PatternElement)

- [x] **17.16 Implement clause parser: MATCH and OPTIONAL MATCH**
  - Parse `MATCH pattern` → `MatchClause`
  - Parse `OPTIONAL MATCH pattern` → `MatchClause` (optional = true)
  - Handle multiple comma-separated patterns in MATCH
  - Write tests: simple match, optional match, match with multiple patterns
  - Ref: Design Phase 15 (Grammar — Match), Req 4.1–4.3

- [x] **17.17 Implement clause parser: WHERE**
  - Parse `WHERE condition` → `WhereClause`
  - Integrates with the condition/expression parser for the predicate
  - Write tests: where with comparison, where with boolean composition, where with string predicate
  - Ref: Design Phase 15 (Grammar — Match), Req 4.6–4.7

- [x] **17.18 Implement clause parser: RETURN with ORDER BY, SKIP, LIMIT**
  - Parse `RETURN expr1, expr2` → `ReturnClause`
  - Parse `RETURN DISTINCT` → distinct flag
  - Parse `RETURN *` → asterisk
  - Parse `ORDER BY expr ASC/DESC` → `OrderByClause` with `SortItem`
  - Parse `SKIP n` → `SkipClause`
  - Parse `LIMIT n` → `LimitClause`
  - Write tests: simple return, aliased return, distinct, ORDER BY with direction, SKIP + LIMIT
  - Ref: Design Phase 15 (Grammar — Return, OrderBy), Req 5.1–5.9

- [x] **17.19 Implement clause parser: WITH**
  - Parse `WITH expr1 AS alias1, expr2 AS alias2` → `WithClause`
  - Parse `WITH DISTINCT`
  - Support WITH followed by WHERE
  - Write tests: with aliased expressions, with distinct, with + where
  - Ref: Design Phase 15 (Grammar — With), Req 4.4

- [x] **17.20 Implement top-level statement parser and multi-part queries**
  - Implement `parse_single_part_query()`: sequence of reading clauses → optional return
  - Implement `parse_statement()`: handle multi-part queries (multiple WITH-separated parts)
  - Wire everything together in `parse()` public function
  - Handle trailing whitespace/semicolons gracefully
  - Write tests: single-part query, multi-part query (MATCH-WITH-MATCH-RETURN), trailing semicolon
  - Ref: Design Phase 15 (Grammar — Statement level)

- [x] **17.21 Implement clause ordering validation**
  - Implement `validate_clause_ordering(clauses: &[Clause]) -> Result<(), ParseError>` in `src/parser/validate.rs`
  - Enforce state transitions per the design table: Start → MATCH/CREATE/..., After MATCH → WHERE/RETURN/..., etc.
  - Produce clear error messages: "WHERE clause cannot appear after RETURN" with position
  - Integrate into `parse()` — run validation after syntactic parsing, before constructing `Statement`
  - Write tests: valid orderings pass, invalid orderings (RETURN before MATCH, WHERE after RETURN, etc.) produce errors
  - Ref: Design Phase 15 (Clause Ordering Validation)

- [x] **17.22 Round-trip integration tests for Phase 1**
  - Create `tests/parser_roundtrip_it.rs` behind `#[cfg(feature = "parser")]`
  - Add round-trip tests for all existing integration test queries that use Phase 1 features (MATCH, WHERE, RETURN, WITH, ORDER BY, SKIP, LIMIT)
  - Use `assert_roundtrip()` and `assert_roundtrip_normalized()` helpers
  - Target: at least 50 round-trip tests from existing `cypher_it.rs`, `expressions_it.rs`, `functions_it.rs`
  - Verify: `cargo test --features parser`
  - Ref: Design Phase 15 (Testing Strategy #1)

- [x] **17.23 Builder-replay test infrastructure and initial tests**
  - Create `src/parser/replay.rs` behind `#[cfg(test)]`
  - Implement `replay_through_builder(parsed: &Statement) -> Statement` with pattern matching on clauses
  - Support Phase 1 clause types: MATCH, OPTIONAL MATCH, WHERE, RETURN, WITH, ORDER BY, SKIP, LIMIT
  - Write builder-replay tests: at least 10 queries verified via `replay_through_builder()`
  - Ref: Design Phase 15 (Testing Strategy #2)

## 18. Cypher Parser — Phase 2 (Write Clauses)

- [x] **18.1 Implement clause parser: CREATE**
  - Parse `CREATE pattern` → `CreateClause`
  - Write tests: create node, create relationship, create chain
  - Ref: Design Phase 15 (Phase 2 scope), Req 6.1

- [x] **18.2 Implement clause parser: MERGE with ON CREATE SET / ON MATCH SET**
  - Parse `MERGE pattern` → `MergeClause`
  - Parse `ON CREATE SET item1, item2` → `MergeAction::OnCreate`
  - Parse `ON MATCH SET item1, item2` → `MergeAction::OnMatch`
  - Write tests: simple merge, merge with on-create, merge with on-match, merge with both
  - Ref: Design Phase 15 (Phase 2 scope), Req 6.2–6.4

- [x] **18.3 Implement clause parser: SET**
  - Parse `SET n.prop = value` → `SetItem::Property`
  - Parse `SET n:Label` → `SetItem::Label`
  - Parse `SET n += {map}` → `SetItem::Mutate`
  - Parse `SET n = {map}` → `SetItem::Replace`
  - Write tests: each SET variant
  - Ref: Design Phase 15 (Phase 2 scope), Req 6.5–6.8

- [x] **18.4 Implement clause parser: DELETE, REMOVE**
  - Parse `DELETE expr1, expr2` → `DeleteClause` (detach = false)
  - Parse `DETACH DELETE expr` → `DeleteClause` (detach = true)
  - Parse `REMOVE n.prop` → `RemoveClause`
  - Parse `REMOVE n:Label` → `RemoveClause`
  - Write tests: delete, detach delete, remove property, remove label
  - Ref: Design Phase 15 (Phase 2 scope), Req 6.9–6.11

- [x] **18.5 Implement clause parser: UNWIND and FOREACH**
  - Parse `UNWIND expr AS var` → `UnwindClause`
  - Parse `FOREACH (var IN expr | updateClauses)` → `ForeachClause`
  - Write tests: unwind list, foreach with set, foreach with create
  - Ref: Design Phase 15 (Phase 2 scope), Req 4.5, Req 6.12

- [x] **18.6 Update clause ordering validation for write clauses**
  - Extend `validate_clause_ordering()` to handle CREATE, MERGE, SET, DELETE, REMOVE, FOREACH, UNWIND
  - Write tests: valid mixed read/write orderings, invalid sequences
  - Ref: Design Phase 15 (Clause Ordering Validation table)

- [x] **18.7 Round-trip and builder-replay tests for Phase 2**
  - Add round-trip tests for write-clause queries from `cypher_it.rs`
  - Extend `replay_through_builder()` to handle CREATE, MERGE, SET, DELETE, REMOVE, UNWIND, FOREACH
  - Target: at least 30 additional round-trip tests
  - Ref: Design Phase 15 (Testing Strategy)

## 19. Cypher Parser — Phase 3 (Advanced Features)

- [x] **19.1 Implement UNION / UNION ALL parsing**
  - Parse `query1 UNION query2` and `query1 UNION ALL query2`
  - Map to existing `Statement::Union` / `Statement::UnionAll`
  - Write tests: union of two queries, union all, multiple unions
  - Ref: Design Phase 15 (Phase 3 scope), Req 12.9

- [x] **19.2 Implement EXPLAIN / PROFILE prefix parsing**
  - Parse `EXPLAIN query` and `PROFILE query`
  - Map to existing `Statement::Explain` / `Statement::Profile`
  - Write tests: explain match-return, profile match-return
  - Ref: Design Phase 15 (Phase 3 scope), Req 12.10

- [x] **19.3 Implement CASE expression parsing**
  - Parse simple CASE: `CASE expr WHEN val THEN result END`
  - Parse generic CASE: `CASE WHEN cond THEN result ELSE default END`
  - Parse multiple WHEN clauses
  - Map to existing `Expression::Case` type
  - Write tests: simple case, generic case, multiple when, with else
  - Ref: Design Phase 15 (Phase 3 scope), Req 15.1–15.2

- [x] **19.4 Implement list comprehension and pattern comprehension parsing**
  - Parse list comprehension: `[x IN list WHERE cond | expr]`
  - Parse pattern comprehension: `[(a)-->(b) | b.name]`
  - Map to existing `Expression::ListComprehension` / `Expression::PatternComprehension`
  - Write tests: with/without WHERE, with/without projection, pattern comprehension
  - Ref: Design Phase 15 (Phase 3 scope), Req 15.3–15.4

- [x] **19.5 Implement subquery expression parsing (EXISTS, COUNT, COLLECT)**
  - Parse `EXISTS { MATCH ... }` → `Expression::ExistentialSubquery`
  - Parse `COUNT { MATCH ... }` → `Expression::CountSubquery`
  - Parse `COLLECT { MATCH ... }` → `Expression::CollectSubquery`
  - Write tests: exists in WHERE, count as expression, collect subquery
  - Ref: Design Phase 15 (Phase 3 scope), Req 15.6–15.8

- [x] **19.6 Implement CALL procedure and CALL subquery parsing**
  - Parse standalone `CALL proc(args)` → `CallClause`
  - Parse `CALL proc() YIELD f1, f2 WHERE cond` → `CallClause` with yield
  - Parse in-query `CALL { subquery }` → `InQueryCallClause`
  - Parse `CALL { subquery } IN TRANSACTIONS OF n ROWS`
  - Write tests: standalone call, call with yield + where, in-query call, in transactions
  - Ref: Design Phase 15 (Phase 3 scope), Req 7.1–7.5

- [x] **19.7 Implement variable-length relationship parsing**
  - Parse `[*]`, `[*2]`, `[*2..5]`, `[*..5]`, `[*2..]` → `RelationshipLength` variants
  - Write tests: each length variant, combined with type and properties
  - Ref: Design Phase 15 (Phase 3 scope), Req 1.5

- [x] **19.8 Implement quantified relationship and quantified path pattern parsing**
  - Parse quantified relationships: `-[:R]->{2}`, `--+`, `-->*`, `-[:R]->{1,3}`
  - Parse quantified path patterns: `((a)-[:R]->(b)){1,3}`, `((a)-[:R]->(b))+`
  - Map to existing `Quantifier` and `QuantifiedPath` types
  - Write tests: each quantifier type on relationships and path patterns
  - Ref: Design Phase 15 (Phase 3 scope), Req 8.1–8.6

- [x] **19.9 Implement path selector parsing**
  - Parse `SHORTEST 1 (pattern)` → `PathSelector::Shortest(1)`
  - Parse `ALL SHORTEST (pattern)` → `PathSelector::AllShortest`
  - Parse `ANY (pattern)` → `PathSelector::Any`
  - Parse `SHORTEST 2 GROUPS (pattern)` → `PathSelector::ShortestGroups(2)`
  - Write tests: each selector type, combined with named paths
  - Ref: Design Phase 15 (Phase 3 scope), Req 8.7–8.11

- [x] **19.10 Implement LOAD CSV and USING hints parsing**
  - Parse `LOAD CSV FROM 'url' AS row` → `LoadCsvClause`
  - Parse `LOAD CSV WITH HEADERS FROM 'url' AS row FIELDTERMINATOR ';'`
  - Parse `USING INDEX var:Label(prop)`, `USING SCAN`, `USING JOIN ON`
  - Write tests: load csv variants, each hint type
  - Ref: Design Phase 15 (Phase 3 scope), Req 9.1–9.3, Req 11.1–11.4

- [x] **19.11 Implement label expression parsing**
  - Parse `:A&B` → `LabelExpression::And`
  - Parse `:A|B` → `LabelExpression::Or`
  - Parse `:!A` → `LabelExpression::Not`
  - Parse `:%` → `LabelExpression::Wildcard`
  - Parse combinations: `:A&(B|C)` with parenthesized grouping
  - Write tests: each operator, nested combinations
  - Ref: Design Phase 15 (Phase 3 scope), Req 1.8

- [x] **19.12 Implement map projection parsing**
  - Parse `n { .name, .age }` → `Expression::MapProjection` with dot-property entries
  - Parse `n { .name, totalAge: n.age + 1, .* }` with literal entries and all-properties
  - Write tests: dot-property, literal entry, all-properties wildcard, mixed
  - Ref: Design Phase 15 (Phase 3 scope), Req 15.5

- [x] **19.13 Implement Cypher 25 clause parsing (FINISH, FILTER, LET)**
  - Parse `FINISH` → `Clause::Finish`
  - Parse `FILTER condition` → `FilterClause`
  - Parse `LET var = expr` → `LetClause`
  - Write tests: each Cypher 25 clause in a complete statement
  - Ref: Design Phase 15 (Phase 3 scope), Req 17.1–17.3

- [x] **19.14 Implement list literals and map literals parsing**
  - Parse list literals: `[1, 2, 3]`, `['a', 'b']`, nested lists
  - Parse map literals: `{key: value, key2: value2}` → `Expression::MapLiteral`
  - Write tests: empty list, nested lists, map with mixed value types
  - Ref: Design Phase 15 (Phase 1 scope — Atom), Req 2.7–2.8

- [x] **19.15 Update clause ordering validation and replay for Phase 3**
  - Extend `validate_clause_ordering()` for UNION, CALL, LOAD CSV, Cypher 25 clauses
  - Extend `replay_through_builder()` for all Phase 3 clause types
  - Ref: Design Phase 15 (Clause Ordering Validation, Testing Strategy)

- [x] **19.16 Comprehensive round-trip tests for Phase 3**
  - Add round-trip tests for all remaining integration test queries
  - Target: all 250+ existing integration tests verified via round-trip
  - Run full verification: `cargo test --features parser` + `cargo clippy --all-targets --all-features -- -D warnings`
  - Ref: Design Phase 15 (Testing Strategy)

## 20. Fluent API Validation Gaps

Post-parser analysis revealed the typestate builder API does not expose all valid clause orderings that the parser accepts. This phase closes those gaps so the fluent DSL guides users to write correct Cypher without requiring manual `Clause` construction.

- [x] **20.1 Add FILTER clause to the builder API**
  - Add `.filter(condition)` method to `OngoingMatch`, `OngoingReadingWithWhere`, and `OngoingUpdate`
  - FILTER transitions to the same state as WHERE (reading-with-where)
  - Write tests: `Cypher::match_(...).filter(cond).returning(...)`, FILTER after SET
  - Ref: Req 17.2 (FILTER clause), parser `AfterMatch/AfterWhere/AfterWrite → FILTER`

- [x] **20.2 Add LET clause to the builder API**
  - Add `.let_(variable, expression)` method to `OngoingMatch`, `OngoingReadingWithWhere`, `OngoingUpdate`, and `OngoingWith`
  - LET transitions to the same state as other write clauses (OngoingUpdate)
  - Write tests: `Cypher::match_(...).let_("x", lit(42)).returning(...)`, LET after WITH
  - Ref: Req 17.1 (LET clause), parser `AfterMatch/AfterWhere/AfterWrite/AfterWith → LET`

- [x] **20.3 Add FINISH clause to the builder API**
  - Add `.finish()` method to `OngoingMatch`, `OngoingReadingWithWhere`, `OngoingUpdate`, and `OngoingWith`
  - FINISH is terminal — `.finish()` returns a buildable state with no further chaining
  - Write tests: `Cypher::match_(...).finish().build()`, FINISH after SET, FINISH after WITH
  - Ref: Req 17.3 (FINISH clause), parser `AfterMatch/AfterWhere/AfterWrite → FINISH`

- [x] **20.4 Add USING INDEX/SCAN/JOIN hints to the builder API**
  - Add `.using_index(variable, label, property)` to `OngoingMatch`
  - Add `.using_index_seek(variable, label, property)` to `OngoingMatch`
  - Add `.using_scan(variable, label)` to `OngoingMatch`
  - Add `.using_join(variables)` to `OngoingMatch`
  - Hints stay in `OngoingMatch` state (can chain multiple hints)
  - Write tests: single hint, multiple hints, hint before WHERE
  - Ref: Req 11.1–11.4, parser `AfterMatch → USING INDEX/SCAN/JOIN`

- [x] **20.5 Add CALL chaining after MATCH and other read states**
  - Add `.call_procedure(name)` and `.call(inner)` to `OngoingMatch` and `OngoingReadingWithWhere`
  - Standalone CALL after MATCH transitions to standalone-call flow
  - In-query CALL after MATCH transitions to in-query-call flow
  - Write tests: `Cypher::match_(...).call(inner).returning(...)`, CALL after WHERE
  - Ref: Req 7.1–7.5, parser `AfterMatch/AfterWhere → CALL/InQueryCall`

- [x] **20.6 Add LOAD CSV after WITH**
  - Add `.load_csv(url)` and `.load_csv_with_headers(url)` to `OngoingWith`
  - Transitions to `OngoingLoadCsv` flow (existing)
  - Write tests: `Cypher::match_(...).with(...).load_csv(url).as_("row").returning(...)`
  - Ref: Req 9.1–9.3, parser `AfterWith → LOAD CSV`

- [x] **20.7 Verification: full parity audit and round-trip tests**
  - Systematically verify every `is_valid_transition` rule has a corresponding builder method
  - Add round-trip + replay tests exercising all new builder methods
  - Run full verification: `cargo test` + `cargo clippy --all-targets --all-features -- -D warnings`
  - Document any intentional restrictions (e.g., WITH/RETURN at start are valid Cypher but unusual)

- [x] **20.8 Update renderer tests to verify CypherQL spec conformance**
  - Audit all renderer unit tests against the CypherQL specification
  - Ensure rendered output matches canonical Cypher syntax (e.g., quantifier placement, keyword casing, punctuation)
  - Add missing coverage for any rendering paths not yet tested
  - Ref: CypherQL spec compliance

## 21. Administration Commands (Req 16)

Administration commands (index/constraint management, SHOW commands, transaction management) are structurally different from regular Cypher queries. They are modeled as `Statement::Admin(AdminCommand)` — a new `Statement` variant separate from the clause-based query structure.

- [x] **21.1 Create admin module skeleton and `AdminCommand` enum**
  - Create `src/admin/mod.rs` with `AdminCommand` enum (10 variants: `CreateIndex`, `DropIndex`, `ShowIndexes`, `CreateConstraint`, `DropConstraint`, `ShowConstraints`, `ShowFunctions`, `ShowProcedures`, `ShowTransactions`, `TerminateTransactions`)
  - Create empty submodules: `index.rs`, `constraint.rs`, `show.rs`, `transaction.rs`
  - Add `Statement::Admin(AdminCommand)` variant to `Statement` enum
  - Wire up module declarations in `lib.rs`
  - Verify: `cargo build` + `cargo test` + `cargo clippy`
  - Ref: Design Phase 21 (Architecture, Module Layout)

- [x] **21.2 Implement index types: `CreateIndex`, `DropIndex`, `IndexType`, `IndexTarget`**
  - Define `IndexType` enum (Range, Text, Point, Fulltext, Vector, Lookup) in `admin/index.rs`
  - Define `IndexTarget` enum (Node, Relationship, NodeLookup, RelationshipLookup) with variable, labels/types, properties fields
  - Define `CreateIndex` struct (index_type, name, if_not_exists, target, options)
  - Define `DropIndex` struct (name, if_exists)
  - Add constructor methods and accessor methods
  - Write unit tests: construct each index type and target combination, verify fields
  - Ref: Design Phase 21 (Data Models §21.2), Req 16.1–16.2

- [x] **21.3 Implement constraint types: `CreateConstraint`, `DropConstraint`, `ConstraintType`, `ConstraintTarget`**
  - Define `ConstraintType` enum (Unique, Exists, NodeKey, RelationshipKey, PropertyType) in `admin/constraint.rs`
  - Define `ConstraintTarget` enum (Node, Relationship) with variable, label/type fields
  - Define `CreateConstraint` struct (name, if_not_exists, target, properties, constraint_type)
  - Define `DropConstraint` struct (name, if_exists)
  - Add constructor methods and accessor methods
  - Write unit tests: construct each constraint type and target combination, verify fields
  - Ref: Design Phase 21 (Data Models §21.3), Req 16.4–16.5

- [x] **21.4 Implement SHOW command types: `ShowCommand`, `ShowYield`, `ExecutableFilter`, `TerminateTransactions`**
  - Define `ShowCommand` struct (type_filter, yield_items, where_condition, transaction_ids, executable) in `admin/show.rs`
  - Define `ShowYield` enum (All, Fields) and `ExecutableFilter` enum (CurrentUser, User)
  - Define `TerminateTransactions` struct (transaction_ids, yield_items, where_condition) in `admin/transaction.rs`
  - Add constructor methods and accessor methods
  - Write unit tests: construct SHOW commands with various options
  - Ref: Design Phase 21 (Data Models §21.4), Req 16.3, 16.6–16.9

- [x] **21.5 Implement renderer for `AdminCommand`**
  - Add `render_admin_command()` to `DefaultRenderer` dispatching on all 10 variants
  - Implement `render_create_index()`: handles all 6 index types × 4 targets, IF NOT EXISTS, OPTIONS
  - Implement `render_drop_index()`: name + optional IF EXISTS
  - Implement `render_create_constraint()`: all constraint types × 2 targets, composite properties, REQUIRE clause
  - Implement `render_drop_constraint()`: name + optional IF EXISTS
  - Implement `render_show()`: type filter, YIELD (*/fields), WHERE, EXECUTABLE filter, transaction IDs
  - Implement `render_terminate_transactions()`: transaction IDs, optional YIELD/WHERE
  - Handle `Statement::Admin` in `render_statement()`
  - Labels in admin commands rendered **without** backtick escaping by default (matching Neo4j convention)
  - Write renderer unit tests: one test per command type with expected Cypher output
    - CREATE INDEX: range, text, point, fulltext, vector, lookup (nodes and relationships)
    - DROP INDEX: with and without IF EXISTS
    - SHOW INDEXES: plain, with type filter, with YIELD, with WHERE
    - CREATE CONSTRAINT: unique, exists, node key, relationship key, property type
    - DROP CONSTRAINT: with and without IF EXISTS
    - SHOW CONSTRAINTS: plain, with type filter
    - SHOW FUNCTIONS: plain, BUILT IN, EXECUTABLE
    - SHOW PROCEDURES: with YIELD + WHERE
    - SHOW TRANSACTIONS: plain, with IDs
    - TERMINATE TRANSACTIONS: with IDs
  - Ref: Design Phase 21 (Rendering), Req 16.1–16.9

- [x] **21.6 Implement `PrettyRenderer` support for admin commands**
  - Add `render_admin_command()` to `PrettyRenderer` (same structure as default but with newlines/indentation for complex commands like CREATE INDEX with OPTIONS)
  - Write tests: pretty-printed CREATE INDEX with OPTIONS, multi-line SHOW with YIELD
  - Ref: Design Phase 21 (Rendering), Req 14.2

- [x] **21.7 Implement fluent builder API: index management**
  - Add `Cypher::create_index(name)` → `IndexBuilder`
  - Add `Cypher::create_index_if_not_exists(name)` → `IndexBuilder`
  - Implement `IndexBuilder`: `.text()`, `.point()`, `.fulltext()`, `.vector()`, `.lookup()`, `.for_node()`, `.for_relationship()`, `.for_node_lookup()`, `.for_relationship_lookup()` → `IndexBuildable`
  - Implement `IndexBuildable`: `.options(expr)`, `.build()` → `Statement`
  - Add `Cypher::drop_index(name)` → `Statement`
  - Add `Cypher::drop_index_if_exists(name)` → `Statement`
  - Add `Cypher::show_indexes()` → `ShowBuilder`
  - Write builder tests: each index type + target, IF NOT EXISTS, OPTIONS, drop, show
  - Ref: Design Phase 21 (Builder API — Index Builder), Req 16.1–16.3

- [x] **21.8 Implement fluent builder API: constraint management**
  - Add `Cypher::create_constraint(name)` → `ConstraintBuilder`
  - Add `Cypher::create_constraint_if_not_exists(name)` → `ConstraintBuilder`
  - Implement `ConstraintBuilder`: `.for_node()`, `.for_relationship()` → `ConstraintRequire`
  - Implement `ConstraintRequire`: `.is_unique()`, `.is_not_null()`, `.is_node_key()`, `.is_relationship_key()`, `.is_typed()` → `Statement`
  - Add `Cypher::drop_constraint(name)` → `Statement`
  - Add `Cypher::drop_constraint_if_exists(name)` → `Statement`
  - Add `Cypher::show_constraints()` → `ShowBuilder`
  - Write builder tests: each constraint type + target, IF NOT EXISTS, composite properties, drop, show
  - Ref: Design Phase 21 (Builder API — Constraint Builder), Req 16.4–16.6

- [x] **21.9 Implement fluent builder API: SHOW and TERMINATE commands**
  - Implement `ShowBuilder`: `.type_filter()`, `.yield_all()`, `.yield_fields()`, `.where_()`, `.executable_by_current_user()`, `.executable_by()`, `.ids()`, `.build()`
  - Add `Cypher::show_functions()` → `ShowBuilder`
  - Add `Cypher::show_procedures()` → `ShowBuilder`
  - Add `Cypher::show_transactions()` → `ShowBuilder`
  - Add `Cypher::terminate_transactions(ids)` → `TerminateBuilder`
  - Implement `TerminateBuilder`: `.yield_all()`, `.yield_fields()`, `.where_()`, `.build()`
  - Write builder tests: show with filters, show with yield + where, terminate with IDs
  - Ref: Design Phase 21 (Builder API — Show Builder), Req 16.7–16.9

- [x] **21.10 Update StatementCatalog for admin commands**
  - Extend `StatementCatalog::from_statement()` to walk `Statement::Admin`
  - `CreateIndex` contributes labels, types, properties to catalog
  - `CreateConstraint` contributes labels, types, properties to catalog
  - Other admin commands contribute nothing
  - Write tests: catalog from create index, catalog from create constraint
  - Ref: Design Phase 21 (StatementCatalog Integration)

- [x] **21.11 Update prelude and re-exports**
  - Re-export admin builder types in prelude: `IndexBuilder`, `IndexBuildable`, `ConstraintBuilder`, `ConstraintRequire`, `ShowBuilder`, `TerminateBuilder`
  - Re-export admin types for advanced usage: `AdminCommand`, `IndexType`, `ConstraintType`
  - Write prelude smoke tests: verify all admin builder entry points are accessible via `use prelude::*`
  - Ref: Design 3.12 (Prelude)

- [x] **21.12 Integration tests for all admin commands**
  - Create `tests/admin_commands_it.rs`
  - Write end-to-end builder → render tests for every rendering example in the design document (see Rendering table)
  - Test edge cases: unnamed indexes, composite properties, multi-label fulltext, vector with options map
  - Target: ~30 integration tests covering all 9 acceptance criteria
  - Run full verification: `cargo build` + `cargo test` + `cargo clippy --all-targets --all-features -- -D warnings`
  - Ref: Design Phase 21 (Testing Strategy), Req 16.1–16.9

- [x] **21.13 Parser support for admin commands (feature-gated)**
  - Create `src/parser/admin.rs` behind `#[cfg(feature = "parser")]`
  - Parse `CREATE [type] INDEX [name] [IF NOT EXISTS] FOR target ON properties [OPTIONS]` → `AdminCommand::CreateIndex`
  - Parse `DROP INDEX name [IF EXISTS]` → `AdminCommand::DropIndex`
  - Parse `SHOW [filter] INDEXES/CONSTRAINTS/FUNCTIONS/PROCEDURES/TRANSACTIONS [YIELD] [WHERE]`
  - Parse `CREATE CONSTRAINT [name] [IF NOT EXISTS] FOR target REQUIRE specification`
  - Parse `DROP CONSTRAINT name [IF EXISTS]`
  - Parse `TERMINATE TRANSACTIONS ids`
  - Extend `parse_statement()` to try admin commands before query body
  - Add round-trip tests for all admin command types
  - Run full verification: `cargo test --features parser` + `cargo clippy --all-targets --all-features -- -D warnings`
  - Ref: Design Phase 21 (Parser Integration)
