# Design Document: Rust Cypher DSL

## Overview

`rust-cypher-dsl` is a Rust library for programmatically constructing Neo4j Cypher queries. It mirrors the feature set of the [Java Neo4j Cypher-DSL](https://github.com/neo4j/cypher-dsl) while adopting idiomatic Rust patterns.

**Core design principles:**

1. **Immutable AST** — All AST nodes are immutable value types. Builder methods consume `self` and return new values.
2. **Enum-based expression tree** — A central `Expression` enum with variants replaces the Java interface hierarchy. This enables pattern matching, `Clone`/`Debug`/`PartialEq` derivation, and avoids trait-object indirection.
3. **Typestate builder** — The fluent builder uses Rust's type system to enforce valid clause ordering at compile time (something Java can only do at runtime).
4. **Zero required dependencies** — The core library has no runtime dependencies.
5. **`Cow<str>` for strings** — String data uses `Cow<'static, str>` so that static labels/types avoid heap allocation while owned strings are supported.
6. **Ergonomic API** — Prelude free functions, implicit conversions, a `props!{}` macro, and `Rc`-backed cheap cloning minimize boilerplate while keeping the API explicit and readable.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     User Code                           │
│                                                         │
│  use rust_cypher_dsl::prelude::*;                        │
│                                                         │
│  let m = node("Movie").named("m");                      │
│  let stmt = Cypher::match_node(m.clone())               │
│      .where_(m.property("title").eq(param("title")))    │
│      .returning(m)                                      │
│      .build()?;                                         │
│  println!("{}", stmt.render());                         │
└──────────────┬──────────────────────────────────────────┘
               │
       ┌───────▼───────┐
       │  Cypher        │  Entry point: associated functions
       │  (cypher.rs)   │  for statement-level operations
       │  + prelude     │  Free functions: node(), param(), lit()
       └───────┬───────┘
               │
       ┌───────▼───────────────┐
       │  StatementBuilder      │  Typestate builder
       │  (builder.rs)          │  States: Matching → Filtering
       │                        │  → Returning → Terminal
       │  OngoingMatch          │  Each state is a separate
       │  OngoingReadingWithWhere│ struct with constrained methods
       │  OngoingReturn         │
       │  ...                   │
       └───────┬───────────────┘
               │ .build()
       ┌───────▼───────┐
       │  Statement     │  Immutable AST root
       │  (statement.rs)│  Contains Vec<Clause>
       └───────┬───────┘
               │
       ┌───────▼───────────┐
       │  Renderer          │  Visitor over AST
       │  (renderer/)       │  Produces Cypher string
       │                    │
       │  DefaultRenderer   │  Single-line output
       │  PrettyRenderer    │  Indented multi-line output
       └───────────────────┘
```

### Module Layout

```
src/
├── lib.rs                    # Crate root, re-exports public API
├── prelude.rs                # Re-exports free functions: node(), param(), lit(), name(), etc.
├── macros.rs                 # props!{} macro
├── cypher.rs                 # Cypher entry point (statement-level associated functions)
├── builder.rs                # StatementBuilder typestate machine
├── statement.rs              # Statement, SinglePartQuery, UnionQuery
├── types/
│   ├── mod.rs
│   ├── node.rs               # Node, NodeLabel
│   ├── relationship.rs       # Relationship, Direction, RelationshipLength
│   ├── property.rs           # Property, PropertyContainer trait
│   ├── expression.rs         # Expression enum (central AST type)
│   ├── literal.rs            # StringLiteral, NumberLiteral, BooleanLiteral, etc.
│   ├── parameter.rs          # Parameter ($name references)
│   ├── pattern.rs            # Pattern, PatternElement, RelationshipChain, NamedPath
│   ├── condition.rs          # Condition enum (Comparison, Compound, HasLabel, etc.)
│   └── operator.rs           # ComparisonOp, MathOp, StringOp, BooleanOp
├── clauses/
│   ├── mod.rs
│   ├── match_clause.rs       # MatchClause (MATCH / OPTIONAL MATCH)
│   ├── where_clause.rs       # WhereClause
│   ├── return_clause.rs      # ReturnClause, ReturnBody
│   ├── with_clause.rs        # WithClause
│   ├── create_clause.rs      # CreateClause
│   ├── merge_clause.rs       # MergeClause, OnCreate, OnMatch
│   ├── set_clause.rs         # SetClause
│   ├── delete_clause.rs      # DeleteClause
│   ├── remove_clause.rs      # RemoveClause
│   ├── unwind_clause.rs      # UnwindClause
│   ├── order_clause.rs       # OrderBy, SortItem, SortDirection
│   ├── skip_limit.rs         # Skip, Limit
│   ├── call_clause.rs        # CallClause (procedures), InQueryCall
│   ├── foreach_clause.rs     # ForeachClause
│   ├── use_clause.rs         # UseClause
│   └── subquery.rs           # Subquery, InTransactions
├── functions/
│   ├── mod.rs
│   ├── aggregate.rs          # count, sum, avg, min, max, collect, percentileCont, etc.
│   ├── scalar.rs             # id, elementId, coalesce, size, head, last, randomUUID, etc.
│   ├── string.rs             # toLower, toUpper, trim, btrim, replace, substring, normalize, etc.
│   ├── math_numeric.rs       # abs, ceil, floor, round, sign, rand, isNaN
│   ├── math_log.rs           # sqrt, log, ln, log10, exp, e
│   ├── math_trig.rs          # sin, cos, tan, cot, cosh, sinh, tanh, coth, haversin, pi, etc.
│   ├── list.rs               # range, keys, labels, nodes, relationships, tail, reduce
│   ├── coll.rs               # coll.distinct, coll.flatten, coll.indexOf, etc.
│   ├── temporal.rs           # datetime, date, time, duration, duration.between, format, etc.
│   ├── spatial.rs            # point, point.distance, point.withinBBox
│   ├── predicate.rs          # exists, all, allReduce, any, none, single, isEmpty
│   ├── database.rs           # db.nameFromElementId
│   ├── graph.rs              # graph.byElementId, graph.byName, graph.names, etc.
│   ├── vector.rs             # vector, vector.similarity.cosine, etc.
│   └── load_csv.rs           # file, linenumber
├── renderer/
│   ├── mod.rs                # Renderer trait, RenderConfig
│   ├── default.rs            # DefaultRenderer (single-line)
│   └── pretty.rs             # PrettyRenderer (indented)
└── catalog.rs                # StatementCatalog (introspection)
```

---

## Components and Interfaces

### 3.1 Expression (Central AST Type)

In the Java DSL, `Expression` is an interface implemented by many classes. In Rust, we model it as an enum — this is the idiomatic equivalent and gives us exhaustive pattern matching, `Clone`/`Debug`/`PartialEq` for free, and no heap-allocated trait objects.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Graph elements
    Node(Node),
    Relationship(Relationship),
    NamedPath(NamedPath),

    // Literals
    StringLiteral(Cow<'static, str>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    ListLiteral(Vec<Expression>),
    MapLiteral(Vec<(Cow<'static, str>, Expression)>),

    // References
    Parameter(Parameter),
    Property(Property),
    SymbolicName(Cow<'static, str>),

    // Operations
    Operation {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },

    // Function calls
    FunctionInvocation {
        name: Cow<'static, str>,
        distinct: bool,
        args: Vec<Expression>,
    },

    // Conditions (also expressions in Cypher)
    Condition(Condition),

    // Aliases
    Aliased {
        delegate: Box<Expression>,
        alias: Cow<'static, str>,
    },

    // Pattern expressions (for use in WHERE, etc.)
    PatternExpression(Pattern),

    // Comprehensions
    ListComprehension {
        variable: Cow<'static, str>,
        list: Box<Expression>,
        where_: Option<Box<Condition>>,
        projection: Option<Box<Expression>>,
    },
    PatternComprehension {
        path_variable: Option<Cow<'static, str>>,
        pattern: Pattern,
        where_: Option<Box<Condition>>,
        projection: Box<Expression>,
    },

    // CASE
    CaseExpression {
        expression: Option<Box<Expression>>,
        when_clauses: Vec<(Expression, Expression)>,
        else_: Option<Box<Expression>>,
    },

    // Map projection
    MapProjection {
        variable: Cow<'static, str>,
        entries: Vec<MapProjectionEntry>,
    },

    // Subquery expressions
    ExistentialSubquery(Box<Statement>),
    CountSubquery(Box<Statement>),
    CollectSubquery(Box<Statement>),

    // Reduce
    ReduceExpression {
        accumulator: Cow<'static, str>,
        init: Box<Expression>,
        variable: Cow<'static, str>,
        list: Box<Expression>,
        expression: Box<Expression>,
    },

    // Aggregate wrapper
    DistinctExpression(Box<Expression>),

    // Raw Cypher (escape hatch)
    RawExpression(Cow<'static, str>),

    // Wildcard (*)
    Asterisk,
}
```

**Expression methods** — `Expression` gets an `impl` block with methods mirroring the Java `Expression` interface:

- Comparison: `eq()`, `ne()`, `lt()`, `lte()`, `gt()`, `gte()`, `in_list()`
- Boolean: `is_null()`, `is_not_null()`, `is_true()`, `is_false()`
- String predicates: `starts_with()`, `ends_with()`, `contains()`, `matches()`, `regex_match()`
- Type checks: `is_type()`, `is_normalized()`, `is_not_normalized()`
- Arithmetic: `add()`, `subtract()`, `multiply()`, `divide()`, `remainder()`, `pow()`
- Utility: `alias()`, `property()`, `ascending()`, `descending()`
- Conversion: `into_condition()` — wraps expression in an `ExpressionCondition`

Each method returns a new `Expression` or `Condition` (immutable pattern).

### 3.2 Node

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub(crate) symbolic_name: Option<Cow<'static, str>>,
    pub(crate) labels: Vec<NodeLabel>,
    pub(crate) properties: Option<Box<Expression>>,  // MapLiteral or Parameter
    pub(crate) label_expression: Option<LabelExpression>, // For label predicates: :A&B, :A|B
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeLabel(pub Cow<'static, str>);

/// Label expressions support AND (&), OR (|), NOT (!), and wildcard (%).
#[derive(Debug, Clone, PartialEq)]
pub enum LabelExpression {
    Label(Cow<'static, str>),
    And(Vec<LabelExpression>),
    Or(Vec<LabelExpression>),
    Not(Box<LabelExpression>),
    Wildcard, // %
}
```

**Key methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `named(name)` | `Node` | Creates a copy with a symbolic name |
| `property(name)` | `Property` | Access a property on this node |
| `rel(type_or_detail)` | `RelationshipBuilder` | Start a relationship (see 3.3) |
| `has_labels(labels)` | `Condition` | Label-check condition |
| `element_id()` | `Expression` | `elementId(n)` function invocation |
| `labels()` | `Expression` | `labels(n)` function invocation |

`Node` also implements `Into<Expression>` (converts to `Expression::Node`), `Into<PatternElement>`, and the `Shr` (`>>`) / `Shl` (`<<`) operator traits for relationship sugar (see 3.17).

### 3.3 Relationship and RelationshipDetail

The relationship system is split into two concerns:

1. **`RelationshipDetail`** — describes a relationship's metadata (type, name, properties, length) *without* knowing which nodes it connects or which direction it goes. Created via the `rel()` free function.
2. **`Relationship`** — a fully resolved pattern element connecting two nodes with a direction. Created when a `RelationshipDetail` is given a direction via `.to()`, `.from()`, `.between()`, or the `>>` / `<<` operators.

```rust
/// Relationship metadata builder. Created by `rel("TYPE")`.
/// Does NOT contain direction or endpoint nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipDetail {
    pub(crate) types: Vec<Cow<'static, str>>,
    pub(crate) symbolic_name: Option<Cow<'static, str>>,
    pub(crate) length: Option<RelationshipLength>,
    pub(crate) properties: Option<Box<Expression>>,
}

/// A fully resolved relationship in a pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub(crate) left: Box<Node>,
    pub(crate) right: Box<Node>,
    pub(crate) direction: Direction,
    pub(crate) details: RelationshipDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Outgoing,   // (left)-[]->(right)
    Incoming,   // (left)<-[]-(right)
    Undirected, // (left)-[]-(right)
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipLength {
    Unbounded,
    Exact(u32),
    Range { min: Option<u32>, max: Option<u32> },
}
```

**`RelationshipDetail` methods (pre-configuration):**

| Method | Returns | Description |
|--------|---------|-------------|
| `named(name)` | `RelationshipDetail` | Assigns a symbolic name |
| `with_properties(props)` | `RelationshipDetail` | Adds inline properties |
| `min(n)` / `max(n)` / `unbounded()` | `RelationshipDetail` | Sets variable-length bounds |

**`RelationshipBuilder` methods (from `node.rel(...)`):**

When `rel()` is called on a `Node`, it returns a `RelationshipBuilder` that knows its left node and relationship details. Direction is set by the terminal method:

| Method | Returns | Description |
|--------|---------|-------------|
| `.to(target)` | `Relationship` | Outgoing: `(self)-[:TYPE]->(target)` |
| `.from(target)` | `Relationship` | Incoming: `(self)<-[:TYPE]-(target)` |
| `.between(target)` | `Relationship` | Undirected: `(self)-[:TYPE]-(target)` |

**`Relationship` methods (chaining):**

| Method | Returns | Description |
|--------|---------|-------------|
| `rel(type_or_detail)` | `RelationshipBuilder` | Start another hop from the right node |
| `inverse()` | `Relationship` | Reverses direction |
| `property(name)` | `Property` | Access a property |

**Prelude free function:**

```rust
/// Creates a RelationshipDetail with one or more types.
pub fn rel(type_name: &str) -> RelationshipDetail { ... }

// With configuration:
rel("ACTED_IN").named("r").with_properties(props! { "role" => lit("Neo") }).min(1).max(3)
```

### 3.4 Property

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub(crate) container: Box<Expression>,  // Node or Relationship (as Expression)
    pub(crate) names: Vec<Cow<'static, str>>,  // Supports nested: a.b.c
}
```

Property implements the same comparison/arithmetic methods as `Expression` (via `Into<Expression>` delegation or a shared trait).

**Key methods:**
- `to(value)` → `SetItem` — for `SET n.prop = value`
- All comparison/arithmetic from `Expression`

### 3.5 Condition

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    // Comparison: left op right
    Comparison {
        left: Box<Expression>,
        operator: ComparisonOp,
        right: Box<Expression>,
    },

    // Boolean composition
    Compound {
        operator: BooleanOp, // And, Or, Xor
        conditions: Vec<Condition>,
    },

    // Negation
    Not(Box<Condition>),

    // Postfix checks
    IsNull(Box<Expression>),
    IsNotNull(Box<Expression>),

    // String predicates
    StringPredicate {
        left: Box<Expression>,
        predicate: StringPredicateOp, // StartsWith, EndsWith, Contains, Matches
        right: Box<Expression>,
    },

    // Label check
    HasLabels {
        node: Box<Expression>,
        labels: Vec<NodeLabel>,
    },

    // IN
    In {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Pattern condition (WHERE (a)-->(b))
    PatternCondition(Pattern),

    // Existential subquery (WHERE EXISTS { ... })
    ExistentialSubquery(Box<Statement>),

    // Expression used as condition (truthy)
    ExpressionCondition(Box<Expression>),

    // Boolean literals
    IsTrue(Box<Expression>),
    IsFalse(Box<Expression>),

    // Regex match: expr =~ 'pattern'
    RegexMatch {
        left: Box<Expression>,
        pattern: Box<Expression>,
    },

    // Type predicate: expr IS :: TYPE
    TypePredicate {
        expression: Box<Expression>,
        type_name: Cow<'static, str>,
    },

    // Normalization: expr IS [NOT] NORMALIZED
    IsNormalized {
        expression: Box<Expression>,
        negated: bool,
    },

    // No-op (collapses when combined)
    NoCondition,
}
```

**Key methods:**
- `and(other)` → `Condition` — composes with AND
- `or(other)` → `Condition` — composes with OR
- `xor(other)` → `Condition` — composes with XOR
- `not()` → `Condition` — negates

`NoCondition` is a sentinel that collapses when combined: `NoCondition.and(x)` → `x`. This matches the Java behavior where empty conditions disappear from output.

### 3.6 Operators

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp { Eq, Ne, Lt, Lte, Gt, Gte }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp { And, Or, Xor }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOp { Add, Subtract, Multiply, Divide, Remainder, Pow }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringPredicateOp { StartsWith, EndsWith, Contains, Matches, RegexMatch }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Comparison(ComparisonOp),
    Math(MathOp),
}
```

### 3.7 Pattern, PatternElement, and Path Selectors

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub(crate) elements: Vec<PatternElement>,
    pub(crate) selector: Option<PathSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternElement {
    Node(Node),
    Relationship(Relationship),
    Chain(RelationshipChain),
    NamedPath(NamedPath),
    QuantifiedPath(QuantifiedPath),
    QuantifiedRelationship(QuantifiedRelationship),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipChain {
    pub(crate) elements: Vec<Relationship>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedPath {
    pub(crate) name: Cow<'static, str>,
    pub(crate) pattern: Box<PatternElement>,
}

/// Quantified Path Pattern (QPP): ((a)-[:R]->(b)){1,3}
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifiedPath {
    pub(crate) pattern: Box<PatternElement>,
    pub(crate) quantifier: Quantifier,
    pub(crate) where_: Option<Box<Condition>>,
}

/// Quantified Relationship: (a)-[:R]->{1,5}(b)
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifiedRelationship {
    pub(crate) left: Box<Node>,
    pub(crate) right: Box<Node>,
    pub(crate) details: RelationshipDetail,
    pub(crate) direction: Direction,
    pub(crate) quantifier: Quantifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    Star,                                    // * (zero or more)
    Plus,                                    // + (one or more)
    Exact(u32),                              // {n}
    Range { min: Option<u32>, max: Option<u32> }, // {n,m}, {n,}, {,m}
}

/// Path selectors: SHORTEST k, ALL SHORTEST, ANY, SHORTEST k GROUPS
#[derive(Debug, Clone, PartialEq)]
pub enum PathSelector {
    Shortest(u32),          // SHORTEST k
    AllShortest,            // ALL SHORTEST
    Any,                    // ANY
    ShortestGroups(u32),    // SHORTEST k GROUPS
}
```

**QPP builder (from the prelude):**

```rust
// Quantified path pattern: ((a)-[:NEXT]->(b)){1,3}
quantified_path(a.rel("NEXT").to(b)).range(1, 3)

// Quantified relationship shorthand: (a)-[:NEXT]->{1,5}(b)
a.rel("NEXT").quantified(1, 5).to(b)

// Path selectors applied to patterns
Cypher::match_node(shortest(1, pattern))  // SHORTEST 1 (pattern)
Cypher::match_node(all_shortest(pattern)) // ALL SHORTEST (pattern)
```

### 3.8 Clauses

Each clause is a struct holding its AST data. All clauses implement a common `Clause` trait:

```rust
pub(crate) trait Renderable {
    fn render(&self, renderer: &mut dyn RendererBackend);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Clause {
    Match(MatchClause),
    OptionalMatch(MatchClause),
    Where(WhereClause),
    Return(ReturnClause),
    With(WithClause),
    Create(CreateClause),
    Merge(MergeClause),
    Set(SetClause),
    Delete(DeleteClause),
    Remove(RemoveClause),
    Unwind(UnwindClause),
    OrderBy(OrderByClause),
    Skip(SkipClause),
    Limit(LimitClause),
    Call(CallClause),
    InQueryCall(InQueryCallClause),
    Foreach(ForeachClause),
    Use(UseClause),
    LoadCsv(LoadCsvClause),
    UsingIndex(UsingIndexClause),
    UsingScan(UsingScanClause),
    UsingJoin(UsingJoinClause),
    // Cypher 25
    Filter(FilterClause),
    Let(LetClause),
    Finish,
}
```

Representative clause structs:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MatchClause {
    pub(crate) optional: bool,
    pub(crate) pattern: Pattern,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub(crate) distinct: bool,
    pub(crate) expressions: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeClause {
    pub(crate) pattern: Pattern,
    pub(crate) actions: Vec<MergeAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeAction {
    OnCreate(Vec<SetItem>),
    OnMatch(Vec<SetItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetItem {
    Property { property: Property, value: Expression },
    Label { node: Expression, labels: Vec<NodeLabel> },
    Mutate { target: Expression, value: Expression },  // SET n += {map}
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadCsvClause {
    pub(crate) url: Expression,
    pub(crate) alias: Cow<'static, str>,
    pub(crate) with_headers: bool,
    pub(crate) field_terminator: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsingIndexClause {
    pub(crate) variable: Cow<'static, str>,
    pub(crate) label: Cow<'static, str>,
    pub(crate) properties: Vec<Cow<'static, str>>,
    pub(crate) seek: bool, // USING INDEX vs USING INDEX SEEK
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsingScanClause {
    pub(crate) variable: Cow<'static, str>,
    pub(crate) label: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsingJoinClause {
    pub(crate) variables: Vec<Cow<'static, str>>,
}

// Cypher 25 clauses
#[derive(Debug, Clone, PartialEq)]
pub struct FilterClause {
    pub(crate) condition: Condition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetClause {
    pub(crate) variable: Cow<'static, str>,
    pub(crate) expression: Expression,
}
```

### 3.9 Statement

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    SinglePart(SinglePartQuery),
    MultiPart(Vec<SinglePartQuery>),
    Union {
        all: bool,
        statements: Vec<Statement>,
    },
    Explained(Box<Statement>),
    Profiled(Box<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinglePartQuery {
    pub(crate) clauses: Vec<Clause>,
}
```

**Key methods on `Statement`:**

| Method | Returns | Description |
|--------|---------|-------------|
| `render()` | `String` | Renders using default renderer |
| `render_with(config)` | `String` | Renders with custom config |
| `catalog()` | `StatementCatalog` | Introspects labels, types, properties, params |
| `get_parameter_names()` | `HashSet<String>` | Lists all `$param` names |
| `get_parameters()` | `HashMap<String, Expression>` | Params with bound values |

`Statement` implements `Display` by delegating to `render()`.

### 3.10 StatementBuilder (Typestate Pattern)

The builder uses separate structs for each state in the query-building lifecycle. Each struct exposes only the methods valid in that state, so invalid clause sequences are caught at compile time.

```
                        ┌─────────────┐
  Cypher::match_node()─►│ OngoingMatch │
                        └──────┬──────┘
                               │
            ┌──────────────────┼───────────────────┐
            │ .where_()        │ .returning()       │ .with()
            ▼                  ▼                    ▼
  ┌─────────────────┐  ┌──────────────┐   ┌───────────────┐
  │ OngoingWhere     │  │ OngoingReturn│   │ OngoingWith    │
  └────────┬────────┘  └──────┬───────┘   └───────┬───────┘
           │                  │                    │
           │ .returning()     │ .order_by()        │ .match_node()
           ▼                  ▼                    │ .where_()
  ┌──────────────┐    ┌──────────────┐             │ .returning()
  │ OngoingReturn│    │ OngoingOrder │             └──► (loops back)
  └──────┬───────┘    └──────┬───────┘
         │                   │
         │ .build()          │ .skip() / .limit() / .build()
         ▼                   ▼
  ┌──────────────┐    ┌──────────────┐
  │ Statement    │    │ Statement    │
  └──────────────┘    └──────────────┘
```

Example builder structs:

```rust
pub struct OngoingMatch {
    clauses: Vec<Clause>,
}

impl OngoingMatch {
    pub fn where_(self, condition: impl Into<Condition>) -> OngoingReadingWithWhere { ... }
    pub fn returning(self, expressions: impl IntoReturnExprs) -> OngoingReturn { ... }
    pub fn with(self, expressions: impl IntoReturnExprs) -> OngoingWith { ... }
    pub fn create(self, pattern: impl IntoPattern) -> OngoingUpdate { ... }
    pub fn merge(self, pattern: impl IntoPattern) -> OngoingMerge { ... }
    pub fn delete(self, expressions: impl IntoDeleteExprs) -> OngoingUpdate { ... }
    pub fn detach_delete(self, expressions: impl IntoDeleteExprs) -> OngoingUpdate { ... }
    pub fn set(self, items: impl IntoSetItems) -> OngoingUpdate { ... }
}

pub struct OngoingReadingWithWhere {
    clauses: Vec<Clause>,
}

impl OngoingReadingWithWhere {
    pub fn and(self, condition: impl Into<Condition>) -> Self { ... }
    pub fn or(self, condition: impl Into<Condition>) -> Self { ... }
    pub fn returning(self, expressions: impl IntoReturnExprs) -> OngoingReturn { ... }
    pub fn with(self, expressions: impl IntoReturnExprs) -> OngoingWith { ... }
    // ...
}

pub struct OngoingReturn {
    clauses: Vec<Clause>,
}

impl OngoingReturn {
    pub fn order_by(self, sort_items: impl IntoSortItems) -> OngoingOrder { ... }
    pub fn skip(self, n: impl Into<Expression>) -> OngoingReturn { ... }
    pub fn limit(self, n: impl Into<Expression>) -> OngoingReturn { ... }
    pub fn build(self) -> Statement { ... }
}
```

**Design rationale**: Java's `StatementBuilder` uses ~30 nested interfaces to model these states. In Rust, we use concrete structs. This is more explicit and lets the compiler enforce transitions — calling `.returning()` on a state that doesn't expose it is a compile error, not a runtime error.

### 3.11 Cypher Entry Point

`Cypher` is a unit struct that exposes only **statement-level** entry points — operations that begin or combine full queries. All element-level construction (nodes, literals, parameters, etc.) lives in the prelude as free functions (see 3.12).

```rust
pub struct Cypher;

impl Cypher {
    // --- Statement builders ---
    pub fn match_node(pattern: impl IntoPattern) -> OngoingMatch { ... }
    pub fn optional_match(pattern: impl IntoPattern) -> OngoingMatch { ... }
    pub fn create(pattern: impl IntoPattern) -> OngoingUpdate { ... }
    pub fn merge(pattern: impl IntoPattern) -> OngoingMerge { ... }
    pub fn unwind(expression: impl Into<Expression>) -> OngoingUnwind { ... }
    pub fn with(expressions: impl IntoReturnExprs) -> OngoingWith { ... }
    pub fn returning(expressions: impl IntoReturnExprs) -> OngoingReturn { ... }

    // --- Procedure calls ---
    pub fn call_procedure(name: &str) -> OngoingStandaloneCall { ... }
    pub fn call_subquery(statement: Statement) -> OngoingInQueryCall { ... }

    // --- Unions ---
    pub fn union(statements: Vec<Statement>) -> Statement { ... }
    pub fn union_all(statements: Vec<Statement>) -> Statement { ... }

    // --- Data import ---
    pub fn load_csv(url: impl Into<Expression>) -> OngoingLoadCsv { ... }
    pub fn load_csv_with_headers(url: impl Into<Expression>) -> OngoingLoadCsv { ... }

    // --- Query decoration ---
    pub fn explain(statement: Statement) -> Statement { ... }
    pub fn profile(statement: Statement) -> Statement { ... }
    pub fn using_periodic_commit(size: Option<u64>) -> OngoingPeriodicCommit { ... }
}
```

**Design rationale**: Statement-level verbs read naturally with the namespace prefix: `Cypher::match_node(...)`, `Cypher::create(...)`. Element-level helpers (`node()`, `param()`, `lit()`) are used far more frequently within a query and benefit from being unqualified free functions.

### 3.12 Prelude and Free Functions

To reduce `Cypher::` prefix noise for frequently used operations, we provide a `prelude` module with free functions. Statement-level entry points (`Cypher::match_node()`, `Cypher::create()`, etc.) remain qualified since they read better with the namespace.

```rust
// src/prelude.rs — re-exported via `use rust_cypher_dsl::prelude::*`

// Node creation
pub fn node(primary_label: &str) -> Node { ... }
pub fn any_node() -> Node { ... }
pub fn any_node_named(name: &str) -> Node { ... }

// Literals (short aliases)
pub fn lit<T: Into<Expression>>(value: T) -> Expression { ... }
pub fn lit_true() -> Expression { Expression::BooleanLiteral(true) }
pub fn lit_false() -> Expression { Expression::BooleanLiteral(false) }
pub fn lit_null() -> Expression { Expression::NullLiteral }

// Parameters
pub fn param(name: &str) -> Parameter { ... }
pub fn param_with_value(name: &str, value: impl Into<Expression>) -> Parameter { ... }

// Symbolic name references
pub fn name(name: &str) -> Expression { Expression::SymbolicName(...) }

// Relationships
pub fn rel(type_name: &str) -> RelationshipDetail { ... }

// Property access
pub fn prop(container: &str, name: &str) -> Property { ... }

// Collections
pub fn list_of(expressions: Vec<Expression>) -> Expression { ... }
pub fn map_of(entries: Vec<(&str, Expression)>) -> Expression { ... }

// Conditions
pub fn not(condition: impl Into<Condition>) -> Condition { ... }

// CASE
pub fn case() -> CaseBuilder { ... }

// Comprehensions
pub fn list_comprehension(variable: &str) -> ListComprehensionBuilder { ... }

// Sorting
pub fn sort(expression: impl Into<Expression>) -> SortItem { ... }

// Quantified path patterns
pub fn quantified_path(pattern: impl IntoPattern) -> QuantifiedPathBuilder { ... }

// Path selectors
pub fn shortest(k: u32, pattern: impl IntoPattern) -> Pattern { ... }
pub fn all_shortest(pattern: impl IntoPattern) -> Pattern { ... }
pub fn any_path(pattern: impl IntoPattern) -> Pattern { ... }
pub fn shortest_groups(k: u32, pattern: impl IntoPattern) -> Pattern { ... }

// Reduce
pub fn reduce(acc: &str, init: impl Into<Expression>, var: &str,
              list: impl Into<Expression>, expr: impl Into<Expression>) -> Expression { ... }

// Raw Cypher
pub fn raw(cypher: &str) -> Expression { ... }

// Re-export Cypher for statement-level entry points
pub use crate::Cypher;
```

**Design rationale**: The most common operations in a query are creating nodes, referencing names, and using literals/parameters. Making these available as free functions eliminates repetitive `Cypher::` prefixes. Statement-level verbs (`match_node`, `create`, `merge`) stay on `Cypher::` because they begin a new statement and the namespace reads naturally: `Cypher::match_node(...)`.

### 3.13 Implicit Conversions (`Into<Expression>`)

Rust primitives and common types implement `From<T> for Expression`, allowing them to be used directly wherever an expression is expected:

```rust
impl From<i32> for Expression { ... }   // IntegerLiteral
impl From<i64> for Expression { ... }   // IntegerLiteral
impl From<f64> for Expression { ... }   // FloatLiteral
impl From<bool> for Expression { ... }  // BooleanLiteral
impl From<&str> for Expression { ... }  // StringLiteral
impl From<String> for Expression { ... } // StringLiteral
impl From<Node> for Expression { ... }  // Node variant
impl From<Property> for Expression { ... } // Property variant
impl From<Parameter> for Expression { ... } // Parameter variant
impl From<Condition> for Expression { ... } // Condition variant
```

This allows natural usage in builder methods:

```rust
// Before: size(name("combined")).eq(Cypher::literal_of(0))
// After:
size(name("combined")).eq(0)

// Before: ("type", Cypher::literal_of("ORPHANED_ACCOUNT"))
// After:
("type", lit("ORPHANED_ACCOUNT"))
```

Builder methods accept `impl Into<Expression>` so conversions happen automatically at call sites.

### 3.14 `props!{}` Macro

A declarative macro for constructing property maps, replacing verbose `map_of(vec![...])` calls:

```rust
/// Creates a map expression from key-value pairs.
///
/// # Example
/// ```rust
/// let properties = props! {
///     "released" => param("year"),
///     "genre"    => lit("Action"),
/// };
/// ```
///
/// Expands to: `Expression::MapLiteral(vec![("workspace_id", ...), ("id", ...)])`
#[macro_export]
macro_rules! props {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {
        $crate::Expression::MapLiteral(vec![
            $( (::std::borrow::Cow::from($key), ::std::convert::Into::into($value)) ),*
        ])
    };
}
```

Usage:

```rust
// Before
node("Movie").named("m").with_properties(Cypher::map_of(vec![
    ("released", Cypher::parameter("year").into()),
    ("genre",    Cypher::literal_of("Action").into()),
]));

// After
node("Movie").named("m").with_properties(props! {
    "released" => param("year"),
    "genre"    => lit("Action"),
});
```

### 3.15 Cheap Cloning via `Rc`

AST nodes (`Node`, `Relationship`, `Expression`, `Condition`, etc.) use `Rc` internally for their heap-allocated data, making `.clone()` a cheap reference count bump rather than a deep copy.

```rust
// Internal representation (user never sees this)
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    inner: Rc<NodeInner>,
}

struct NodeInner {
    symbolic_name: Option<Cow<'static, str>>,
    labels: Vec<NodeLabel>,
    properties: Option<Expression>,
}
```

**Why this matters**: In Cypher DSL usage, the same node variable is typically referenced 3-5 times in a single query (in MATCH, WHERE, WITH, RETURN). Each use requires `.clone()` due to Rust ownership. With `Rc`, these clones are O(1) pointer copies instead of O(n) deep copies.

**Trade-offs:**
- `.clone()` calls are still needed in user code (Rust requires explicitness), but they are cheap
- `Rc` is not `Send`/`Sync`. Since query building is typically single-threaded, this is acceptable. If needed in the future, `Arc` can be swapped in behind a feature flag.
- `PartialEq` compares by value (via `NodeInner`), not by reference identity

### 3.16 Full API Example

A movie database query demonstrating all ergonomic features. This query finds actors, the movies they acted in, the directors of those movies, and counts reviews per movie:

```cypher
MATCH (actor:`Actor`)-[:`ACTED_IN`]->(movie:`Movie`)<-[:`DIRECTED`]-(director:`Director`)
OPTIONAL MATCH (reviewer:`Critic`)-[r:`REVIEWED`]->(movie)
WHERE r.rating > 7
WITH movie, actor, director, collect(DISTINCT reviewer) AS reviewers
RETURN movie.title AS title, actor.name AS lead, director.name AS directedBy,
       size(reviewers) AS reviewCount
ORDER BY reviewCount DESC
LIMIT 10
```

Built with the DSL using **method syntax**:

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::functions::aggregate::collect_distinct;
use rust_cypher_dsl::functions::scalar::size;

let actor    = node("Actor").named("actor");
let movie    = node("Movie").named("movie");
let director = node("Director").named("director");
let reviewer = node("Critic").named("reviewer");
let r        = rel("REVIEWED").named("r");

let statement = Cypher::match_node(
        actor.clone()
            .rel("ACTED_IN").to(movie.clone())
            .rel("DIRECTED").from(director.clone())
    )
    .optional_match(
        reviewer.clone().rel(r.clone()).to(movie.clone())
    )
    .where_(name("r").property("rating").gt(7))
    .with((
        movie.clone(),
        actor.clone(),
        director.clone(),
        collect_distinct(reviewer).alias("reviewers"),
    ))
    .returning((
        movie.property("title").alias("title"),
        actor.property("name").alias("lead"),
        director.property("name").alias("directedBy"),
        size(name("reviewers")).alias("reviewCount"),
    ))
    .order_by(name("reviewCount").descending())
    .limit(10)
    .build()
    .unwrap();
```

Or equivalently using **operator syntax** (`>>` / `<<`):

```rust
let statement = Cypher::match_node(
        actor.clone() >> rel("ACTED_IN") >> movie.clone()
                      << rel("DIRECTED") << director.clone()
    )
    .optional_match(
        reviewer.clone() >> r.clone() >> movie.clone()
    )
    // ... rest is identical
```

Both produce the same rendered Cypher output.

### 3.17 Relationship Syntax (Hybrid: Methods + Operators)

The DSL provides two interchangeable syntaxes for expressing relationships:

#### Method syntax: `.rel().to()` / `.rel().from()` / `.rel().between()`

The primary API. `rel()` starts a relationship builder, and the terminal method (`.to()`, `.from()`, `.between()`) sets direction:

```rust
let actor = node("Actor").named("a");
let movie = node("Movie").named("m");

// Outgoing: (a)-[:ACTED_IN]->(m)
actor.rel("ACTED_IN").to(movie.clone())

// Incoming: (a)<-[:DIRECTED]-(m)
actor.rel("DIRECTED").from(movie.clone())

// Undirected: (a)-[:KNOWS]-(m)
actor.rel("KNOWS").between(movie.clone())
```

Methods handle complex relationships naturally:

```rust
// With properties and variable length
actor.rel("ACTED_IN")
    .named("r")
    .with_properties(props! { "role" => lit("Neo") })
    .min(1).max(3)
    .to(movie)
```

#### Operator syntax: `>>` and `<<`

Ergonomic sugar using Rust's `Shr` and `Shl` operator overloading. Best for simple relationship patterns:

```rust
// >> means outgoing (arrow points right): (a)-[:ACTED_IN]->(m)
actor >> rel("ACTED_IN") >> movie

// << means incoming (arrow points left): (a)<-[:DIRECTED]-(m)
actor << rel("DIRECTED") << movie
```

The operators work with pre-configured `RelationshipDetail` values too:

```rust
let acted_in = rel("ACTED_IN")
    .named("r")
    .with_properties(props! { "role" => param("role") });

// Pre-built detail used cleanly with operators
actor >> acted_in >> movie
```

#### Mixed directions in a single chain

Cypher frequently mixes arrow directions. Both syntaxes handle this:

```rust
// Cypher: (actor)-[:ACTED_IN]->(movie)<-[:DIRECTED]-(director)-[:LIVES_IN]->(city)

// Method syntax — reads like English
actor.rel("ACTED_IN").to(movie.clone())
     .rel("DIRECTED").from(director.clone())
     .rel("LIVES_IN").to(city)

// Operator syntax — mirrors the Cypher arrows
actor >> rel("ACTED_IN") >> movie.clone()
      << rel("DIRECTED") << director.clone()
      >> rel("LIVES_IN") >> city
```

#### Implementation detail

The operators are implemented via Rust's `Shr` and `Shl` traits:

```rust
// Node >> RelationshipDetail produces a half-built relationship (knows left node + detail)
impl Shr<RelationshipDetail> for Node {
    type Output = OutgoingHalf;
    fn shr(self, detail: RelationshipDetail) -> OutgoingHalf { ... }
}

// OutgoingHalf >> Node completes the relationship
impl Shr<Node> for OutgoingHalf {
    type Output = Relationship;
    fn shr(self, right: Node) -> Relationship { ... }
}

// Similarly for << (Shl) producing incoming relationships
```

`OutgoingHalf` and `IncomingHalf` are intermediate types that exist only during the `>>` / `<<` expression evaluation. Users never need to name or store them.

#### When to use which

| Scenario | Recommended |
|----------|-------------|
| Simple typed relationships | `>>` / `<<` operators |
| Relationships with properties/length | Method syntax, or pre-build with `rel().named().with_properties()` then use operators |
| Mixed direction chains | Either works; operators mirror Cypher arrows more closely |
| Undirected relationships | Method syntax only (`.between()`) — no operator equivalent |

### 3.18 Functions Module

Functions are implemented as free functions in submodules, each returning `Expression::FunctionInvocation`. They mirror the Java `Cypher.*` static methods and `Functions.*` methods.

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::functions::aggregate::count;
use rust_cypher_dsl::functions::scalar::element_id;

let m = node("Movie").named("m");
let stmt = Cypher::match_node(m.clone())
    .returning((count(m.clone()), element_id(m)))
    .build()?;
```

**Categories and functions:**

| Module | Functions |
|--------|-----------|
| `aggregate` | `count`, `count_distinct`, `sum`, `sum_distinct`, `avg`, `avg_distinct`, `min`, `min_distinct`, `max`, `max_distinct`, `collect`, `collect_distinct`, `percentile_cont`, `percentile_disc`, `st_dev`, `st_dev_p` |
| `scalar` | `id`, `element_id`, `type_of`, `coalesce`, `timestamp`, `size`, `head`, `last`, `start_node`, `end_node`, `properties`, `random_uuid`, `null_if`, `value_type`, `char_length`, `length`, `path_length`, `to_integer`, `to_integer_or_null`, `to_float`, `to_float_or_null`, `to_string`, `to_string_or_null`, `to_boolean`, `to_boolean_or_null` |
| `string` | `to_lower`, `lower`, `to_upper`, `upper`, `trim`, `btrim`, `ltrim`, `rtrim`, `replace`, `substring`, `left`, `right`, `split`, `reverse_str`, `normalize` |
| `math_numeric` | `abs`, `ceil`, `ceiling`, `floor`, `round`, `sign`, `rand`, `is_nan` |
| `math_log` | `sqrt`, `log`, `ln`, `log10`, `exp`, `e` |
| `math_trig` | `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `cot`, `cosh`, `sinh`, `tanh`, `coth`, `degrees`, `radians`, `haversin`, `pi` |
| `list` | `range`, `keys`, `labels`, `nodes`, `relationships`, `tail`, `reverse_list`, `reduce`, `to_boolean_list`, `to_float_list`, `to_integer_list`, `to_string_list` |
| `coll` | `coll_distinct`, `coll_flatten`, `coll_index_of`, `coll_insert`, `coll_max`, `coll_min`, `coll_remove`, `coll_sort` |
| `temporal` | `datetime`, `localdatetime`, `date`, `localtime`, `time`, `duration`, `duration_between`, `duration_in_days`, `duration_in_months`, `duration_in_seconds`, `datetime_from_epoch`, `datetime_from_epoch_millis`, `format` and `.realtime()`, `.statement()`, `.transaction()`, `.truncate()` variants |
| `spatial` | `point`, `point_distance`, `point_within_bbox` |
| `predicate` | `exists`, `all`, `all_reduce`, `any`, `none`, `single`, `is_empty` |
| `database` | `db_name_from_element_id` |
| `graph` | `graph_by_element_id`, `graph_by_name`, `graph_names`, `graph_properties_by_name` |
| `vector` | `vector`, `vector_similarity_cosine`, `vector_similarity_euclidean` |
| `load_csv` | `file`, `linenumber` |

### 3.19 Renderer

```rust
/// Configuration for rendering.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Whether to always escape identifiers with backticks.
    pub escape_names: EscapeMode,
    /// Whether to pretty-print with indentation and newlines.
    pub pretty_print: bool,
    /// Indentation string (default: two spaces).
    pub indent: Cow<'static, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeMode {
    /// Always backtick-escape labels, types, and property names.
    Always,
    /// Only escape names that contain special characters or are reserved words.
    AsNeeded,
}

/// Renders a statement to a Cypher string.
pub trait Renderer {
    fn render(&self, statement: &Statement) -> String;
}

pub struct DefaultRenderer {
    config: RenderConfig,
}

pub struct PrettyRenderer {
    config: RenderConfig,
}
```

The renderers use a recursive visitor pattern: a `render_clause`, `render_expression`, `render_condition` set of functions that match on the enum variants and write to an internal `String` buffer.

`Statement::render()` uses `DefaultRenderer` with default config. `Statement::render_with(config)` allows customization.

### 3.20 StatementCatalog (Introspection)

```rust
pub struct StatementCatalog {
    pub labels: HashSet<String>,
    pub relationship_types: HashSet<String>,
    pub properties: Vec<CatalogProperty>,
    pub parameters: HashMap<String, Option<Expression>>,
}

pub struct CatalogProperty {
    pub name: String,
    pub owner_label: Option<String>,
    pub owner_type: Option<String>,
}
```

Built by walking the AST after `Statement` construction.

---

## Data Models

### Conversion Traits

To enable the fluent API, we define `Into` / `From` conversions:

| From | To | Purpose |
|------|----|---------|
| `Node` | `Expression` | Use nodes in return/where |
| `Relationship` | `Expression` | Use relationships in return/where |
| `Property` | `Expression` | Use properties in return/where/set |
| `Parameter` | `Expression` | Use parameters anywhere |
| `&str` | `Expression` | String literals shorthand |
| `i64` / `f64` / `bool` | `Expression` | Numeric/boolean literal shorthand |
| `Condition` | `Expression` | Conditions are expressions |
| `Node` | `PatternElement` | Nodes in MATCH patterns |
| `Relationship` | `PatternElement` | Relationships in MATCH patterns |
| `RelationshipChain` | `PatternElement` | Chains in MATCH patterns |
| `Vec<PatternElement>` | `Pattern` | Multiple elements form a pattern |

### Helper Traits

```rust
/// Anything that can be used as RETURN/WITH expressions.
pub trait IntoReturnExprs {
    fn into_return_exprs(self) -> Vec<Expression>;
}

/// Anything that can be used as a MATCH pattern.
pub trait IntoPattern {
    fn into_pattern(self) -> Pattern;
}

/// Anything that can be used as SET items.
pub trait IntoSetItems {
    fn into_set_items(self) -> Vec<SetItem>;
}
```

These traits are implemented for single items, tuples, `Vec`, and arrays, making the API ergonomic:

```rust
// Single expression
.returning(m)
// Multiple expressions
.returning((m.clone(), n.clone()))
// Vec
.returning(vec![m.clone(), n.clone()])
```

---

## Error Handling

The type system (via typestate builder) prevents most invalid queries at compile time. The few remaining semantic errors are captured in:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BuildError {
    /// A required clause is missing (e.g., RETURN without MATCH).
    MissingClause { clause: &'static str },
    /// A parameter name conflicts with another parameter.
    ConflictingParameters { name: String },
    /// An invalid combination of clauses was attempted.
    InvalidClauseSequence { message: String },
}

impl std::fmt::Display for BuildError { ... }
impl std::error::Error for BuildError {}
```

**Rendering is infallible.** A valid `Statement` (produced by `build()`) always renders to a valid Cypher string. There is no `Result` in the render path.

For builder methods that cannot fail (most of them due to typestate), we return the next state directly. Only `build()` returns `Result<Statement, BuildError>` for edge cases.

---

## Testing Strategy

### Test Organization

```
tests/
├── cypher_it.rs            # Ported from CypherIT.java — main integration tests
├── issue_related_it.rs     # Ported from IssueRelatedIT.java — regression tests
├── functions_it.rs         # Ported from FunctionsIT.java
├── expressions_it.rs       # Ported from ExpressionsIT.java
├── subqueries_it.rs        # Ported from SubqueriesIT.java
├── procedure_calls_it.rs   # Ported from ProcedureCallsIT.java
├── renderer_tests.rs       # Rendering configuration tests
├── catalog_tests.rs        # Statement catalog introspection tests
├── load_csv_it.rs          # LOAD CSV tests
├── patterns_it.rs          # QPP, quantified relationships, path selectors
├── query_hints_it.rs       # USING INDEX, USING SCAN, USING JOIN
└── cypher25_it.rs          # FINISH, FILTER, LET, WHEN, NEXT
```

### Test Pattern

Every test follows the same shape (mirroring the Java tests):

```rust
use rust_cypher_dsl::prelude::*;
use pretty_assertions::assert_eq;

#[test]
fn unrelated_nodes() {
    let b = node("Bike").named("b");
    let u = node("User").named("u");
    let o = node("U").named("o");

    let statement = Cypher::match_node((b.clone(), u.clone(), o))
        .returning((b, u))
        .build()
        .unwrap();

    assert_eq!(
        statement.render(),
        "MATCH (b:`Bike`), (u:`User`), (o:`U`) RETURN b, u"
    );
}
```

### Test Sources

- Expected Cypher strings are ported directly from the Java test files
- Each Java test method maps to a Rust `#[test]` function
- Test names use `snake_case` equivalents of the Java `camelCase` names
- Tests are organized in modules matching the Java nested class structure

### Test Coverage Tracking

Each test file includes a header comment listing the Java source and the count of ported tests, e.g.:

```rust
//! Ported from CypherIT.java (Java Cypher-DSL)
//! Total Java tests: 180+
//! Ported:           0 (updated as tests are added)
```

### Dev Dependencies

```toml
[dev-dependencies]
pretty_assertions = "1"
```

### Test Methodology

Per CLAUDE.md, we follow TDD (red-green-refactor):
1. Write the test with the expected Cypher output (red — does not compile or fails)
2. Implement the minimum code to make it pass (green)
3. Refactor for clarity and deduplication (refactor)

---

## Phase 15: Cypher Parser (Req 20)

### Overview

The parser converts Cypher query strings into the existing AST types (`Statement`, `Clause`, `Expression`, `Condition`, `Node`, `Relationship`, etc.). It is gated behind the `parser` cargo feature flag so the core library maintains zero runtime dependencies.

**Design goals:**

1. **Reuse existing AST** — Parse directly into the types defined in `src/types/`, `src/clauses/`, and `src/statement.rs`. No intermediate parse tree.
2. **Round-trip fidelity** — `parse(cypher).render()` should produce semantically equivalent Cypher (formatting may differ, but structure is preserved).
3. **Descriptive errors** — Parse errors include source position (line/column), expected token, and context (e.g., "in RETURN clause").
4. **Incremental scope** — Start with the clause subset our AST already supports; extend as needed.
5. **Feature-gated** — All parser code lives behind `#[cfg(feature = "parser")]` so the default build has zero added dependencies.

### Architecture

```
                    Cypher string
                         │
                         ▼
              ┌─────────────────────┐
              │     Lexer/Scanner    │  Tokenizes input into keywords,
              │  (parser/lexer.rs)   │  identifiers, literals, operators,
              │                     │  punctuation
              └──────────┬──────────┘
                         │ Token stream
                         ▼
              ┌─────────────────────┐
              │   Recursive-Descent  │  Parses token stream into AST
              │      Parser          │  using winnow combinators
              │  (parser/grammar.rs) │
              └──────────┬──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │     Statement        │  Existing AST types
              │  (statement.rs)      │  (no new types needed)
              └─────────────────────┘
```

### Module Layout

```
src/
├── parser/
│   ├── mod.rs              # Public API: parse(), ParseError
│   ├── error.rs            # ParseError type with span/context
│   ├── lexer.rs            # Tokenizer: keywords, identifiers, literals, operators
│   ├── tokens.rs           # Token enum and Keyword enum
│   ├── grammar.rs          # Top-level: statement, single_part_query, union
│   ├── clauses.rs          # Clause parsers: match_, return_, with_, where_, etc.
│   ├── expressions.rs      # Expression parsers: literals, names, properties, ops
│   ├── patterns.rs         # Pattern parsers: nodes, relationships, chains, paths
│   └── conditions.rs       # Condition parsers: comparisons, boolean combinators
```

### Library Choice: `winnow`

We use [`winnow`](https://docs.rs/winnow) (parser combinator, successor to `nom`) for the following reasons:

- **Direct AST output** — Combinators return our existing types directly (no untyped intermediate tree like `pest`).
- **Excellent error reporting** — `cut_err` + `context` provide positioned, contextual error messages out of the box.
- **Zero-copy parsing** — Parses `&str` input without allocating intermediate token structures.
- **Feature-gated dependency** — Only pulled in when `parser` feature is enabled.
- **Mature and maintained** — Active development, well-documented, strong community.

```toml
[features]
parser = ["dep:winnow"]

[dependencies]
winnow = { version = "0.6", optional = true }
```

### Public API

```rust
// src/parser/mod.rs

/// Parses a Cypher query string into a Statement.
///
/// # Errors
/// Returns `ParseError` with position and context on invalid input.
///
/// # Example
/// ```rust
/// use rust_cypher_dsl::parser::parse;
///
/// let stmt = parse("MATCH (n:Person) RETURN n").unwrap();
/// assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
/// ```
pub fn parse(input: &str) -> Result<Statement, ParseError> { ... }

/// Parse error with source position and context.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Byte offset in the input where the error occurred.
    pub offset: usize,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
    /// What the parser expected at this position.
    pub expected: Vec<String>,
    /// Parsing context stack (e.g., ["RETURN clause", "expression"]).
    pub context: Vec<String>,
    /// The portion of input near the error.
    pub snippet: String,
}

impl std::fmt::Display for ParseError { ... }
impl std::error::Error for ParseError { ... }
```

### Token Types

The lexer produces a stream of tokens. Keywords are case-insensitive.

```rust
// src/parser/tokens.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    // Keywords (case-insensitive)
    Keyword(Keyword),

    // Identifiers and names
    Identifier(&'a str),           // unquoted: myVar
    EscapedIdentifier(&'a str),    // backtick-quoted: `my var`

    // Literals
    IntegerLit(i64),
    FloatLit(f64),
    StringLit(String),             // single or double-quoted, unescaped
    BooleanLit(bool),
    NullLit,

    // Operators and punctuation
    Eq,             // =
    Ne,             // <>
    Lt,             // <
    Gt,             // >
    Lte,            // <=
    Gte,            // >=
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Caret,          // ^
    Dot,            // .
    DotDot,         // ..
    Colon,          // :
    Pipe,           // |
    Ampersand,      // &
    Bang,           // !
    Tilde,          // ~
    Dollar,         // $
    Arrow,          // ->
    LeftArrow,      // <-
    Comma,          // ,
    LParen,         // (
    RParen,         // )
    LBracket,       // [
    RBracket,       // ]
    LBrace,         // {
    RBrace,         // }

    // Special
    RegexMatch,     // =~
    PlusAssign,     // +=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Match, Optional, Where, Return, Distinct, With, As,
    Create, Merge, Set, Delete, Detach, Remove,
    Order, By, Asc, Ascending, Desc, Descending,
    Skip, Limit, Unwind,
    And, Or, Xor, Not, In, Is, Null,
    True, False,
    Starts, Ends, Contains,
    Case, When, Then, Else, End,
    Call, Yield,
    Foreach,
    Explain, Profile,
    Union, All,
    Load, Csv, From, Headers,
    Using, Index, Seek, Scan, Join, On,
    // Cypher 25
    Finish, Filter, Let, Next,
    // Path selectors
    Shortest, Any, Groups,
    // Existence/subquery
    Exists, Count, Collect,
    // Transactions
    Transactions, Of, Rows,
    // Normalization
    Normalized, Nfc, Nfd, Nfkc, Nfkd,
}
```

### Grammar Structure

The parser follows the openCypher grammar structure, adapted to produce our AST types directly. Each grammar rule maps to a winnow combinator function.

#### Statement level

```
Statement       = [ EXPLAIN | PROFILE ] QueryBody
QueryBody       = SinglePartQuery ( UNION [ALL] SinglePartQuery )*
SinglePartQuery = ReadingClause* UpdatingClause* Return?
                | ReadingClause* Return
```

#### Clause parsers

```
ReadingClause   = Match | Unwind | InQueryCall | LoadCsv
Match           = [OPTIONAL] MATCH Pattern [WHERE Expression]
Return          = RETURN [DISTINCT] ReturnItems [OrderBy] [Skip] [Limit]
With            = WITH [DISTINCT] ReturnItems [WHERE Expression]
Unwind          = UNWIND Expression AS Identifier
Create          = CREATE Pattern
Merge           = MERGE Pattern (ON CREATE SET ...)* (ON MATCH SET ...)*
Set             = SET SetItem (',' SetItem)*
Delete          = [DETACH] DELETE Expression (',' Expression)*
Remove          = REMOVE RemoveItem (',' RemoveItem)*
Foreach         = FOREACH '(' Identifier IN Expression '|' UpdatingClause+ ')'
OrderBy         = ORDER BY SortItem (',' SortItem)*
```

#### Expression parsing (precedence climbing)

Expressions use winnow's built-in `precedence()` combinator for operator precedence:

```
Expression = OrExpression
OrExpression   = XorExpression (OR XorExpression)*
XorExpression  = AndExpression (XOR AndExpression)*
AndExpression  = NotExpression (AND NotExpression)*
NotExpression  = NOT* ComparisonExpr
ComparisonExpr = AddSubExpr (CompOp AddSubExpr | IS [NOT] NULL | IN ListExpr | ...)*
AddSubExpr     = MulDivExpr (('+' | '-') MulDivExpr)*
MulDivExpr     = PowExpr (('*' | '/' | '%') PowExpr)*
PowExpr        = UnaryExpr ('^' UnaryExpr)*
UnaryExpr      = ['-' | '+'] PostfixExpr
PostfixExpr    = Atom ('.' PropertyName | '[' Expression ']')*
Atom           = Literal | Parameter | FunctionCall | Variable
               | '(' Expression ')' | CaseExpr | ListComprehension
               | PatternComprehension | ExistentialSubquery
               | CountSubquery | CollectSubquery
```

#### Pattern parsing

```
Pattern         = [PathSelector] PatternElement (',' PatternElement)*
PatternElement  = [Variable '='] AnonymousPattern
AnonymousPattern = NodePattern (RelPattern NodePattern)*
NodePattern     = '(' [Variable] [':' Labels] [Properties] [WHERE Expr] ')'
RelPattern      = LeftArrow? '-' '[' RelDetail ']' '-' RightArrow?
                | '--' [Quantifier] ['>']     // untyped shorthand
RelDetail       = [Variable] [':' Types] [Length] [Properties] [WHERE Expr]
Length          = '*' [IntegerLit ['..' IntegerLit]]
Quantifier     = '*' | '+' | '{' IntegerLit [',' IntegerLit] '}'
PathSelector   = SHORTEST IntegerLit | ALL SHORTEST | ANY
               | SHORTEST IntegerLit GROUPS
```

### Error Handling

The parser uses winnow's `ContextError` with `StrContext` labels for descriptive errors:

```rust
use winnow::error::{ContextError, StrContext, StrContextValue};

// Example: parsing a RETURN clause
fn parse_return<'a>(input: &mut &'a str) -> PResult<ReturnClause> {
    keyword("RETURN")
        .context(StrContext::Label("RETURN clause"))
        .parse_next(input)?;

    let distinct = opt(keyword("DISTINCT")).parse_next(input)?.is_some();

    let expressions = separated(1.., parse_expression, comma)
        .context(StrContext::Label("return expressions"))
        .parse_next(input)?;

    Ok(ReturnClause { distinct, expressions })
}
```

Error messages look like:

```
Parse error at line 1, column 25:
  MATCH (n:Person) RETURN
                         ^
Expected: expression
Context: RETURN clause
```

### Clause Ordering Validation

The typestate builder enforces valid clause ordering at compile time (e.g., `WHERE` can only follow `MATCH`, `ORDER BY` can only follow `RETURN`). Since the parser discovers clause sequences at runtime, it cannot use the typestate builder directly — each builder state is a different Rust type, and dynamic dispatch through them would require deeply nested match trees reimplementing the entire state machine.

Instead, the parser includes a **runtime validation layer** that enforces the same ordering rules:

```rust
// src/parser/validate.rs

/// Validates that a sequence of parsed clauses follows legal Cypher ordering.
///
/// Enforces the same rules that the typestate builder encodes at compile time:
/// - Reading clauses (MATCH, UNWIND, CALL) before writing clauses (CREATE, SET, DELETE)
/// - WHERE must follow MATCH or WITH
/// - RETURN/WITH position constraints
/// - ORDER BY, SKIP, LIMIT must follow RETURN
/// - No duplicate WHERE without AND/OR composition
pub fn validate_clause_ordering(clauses: &[Clause]) -> Result<(), ParseError> { ... }
```

This runs after syntactic parsing, before constructing the `Statement`. An erroneous input like `RETURN n MATCH (n)` or `WHERE x ORDER BY y` produces a clear semantic error with the offending clause position.

The validation rules are derived directly from the typestate transitions:

| Builder state | Valid next clauses |
|---|---|
| Start | MATCH, OPTIONAL MATCH, CREATE, MERGE, UNWIND, CALL, LOAD CSV, WITH, RETURN |
| After MATCH | WHERE, RETURN, WITH, MATCH, OPTIONAL MATCH, CREATE, MERGE, SET, DELETE, REMOVE |
| After WHERE | RETURN, WITH, CREATE, MERGE, SET, DELETE, AND/OR (extends WHERE) |
| After WITH | MATCH, WHERE, RETURN, UNWIND |
| After RETURN | ORDER BY, SKIP, LIMIT, (terminal) |
| After CREATE/SET/DELETE | RETURN, WITH, CREATE, MERGE, SET, DELETE, REMOVE |

### Testing Strategy

The parser uses three complementary testing approaches:

#### 1. Round-trip verification (parse → render → compare)

Parse a Cypher string, render the AST, compare output. Reuses all 250+ existing integration tests:

```rust
/// Round-trip test helper: parse → render → compare.
fn assert_roundtrip(cypher: &str) {
    let stmt = parse(cypher).unwrap_or_else(|e| panic!("Parse failed: {e}"));
    let rendered = stmt.render();
    assert_eq!(rendered, cypher, "Round-trip mismatch");
}

/// Round-trip with normalization (backtick escaping may differ).
fn assert_roundtrip_normalized(cypher: &str, expected: &str) {
    let stmt = parse(cypher).unwrap_or_else(|e| panic!("Parse failed: {e}"));
    let rendered = stmt.render();
    assert_eq!(rendered, expected);
}
```

Every integration test that constructs a statement via the DSL and asserts a rendered string can be flipped: parse the expected string and verify it round-trips.

#### 2. Builder-replay verification (parse → reconstruct via builder → compare)

This is the key strategy for **testing the DSL itself**. For each parsed statement, we extract its structure and reconstruct it through the fluent builder API. This proves that the builder can express everything the parser accepts:

```rust
// src/parser/replay.rs (behind #[cfg(test)])

/// Reconstructs a Statement by replaying parsed clauses through the
/// fluent builder API. This validates that the builder's typestate
/// machine can produce every valid Cypher query the parser accepts.
///
/// Panics if the builder cannot represent the parsed structure.
pub fn replay_through_builder(parsed: &Statement) -> Statement { ... }
```

The replay function pattern-matches on the parsed clause sequence and drives through the builder's typestate transitions:

```rust
fn replay_single_part(clauses: &[Clause]) -> Statement {
    // Identify the leading clause and dispatch into the correct builder state
    match &clauses[0] {
        Clause::Match(m) => {
            let ongoing = Cypher::match_(m.pattern.clone());
            replay_after_match(ongoing, &clauses[1..])
        }
        Clause::Create(c) => {
            let ongoing = Cypher::create(c.pattern.clone());
            replay_after_update(ongoing, &clauses[1..])
        }
        // ... each entry point dispatches to a state-specific continuation
    }
}

fn replay_after_match(state: OngoingMatch, remaining: &[Clause]) -> Statement {
    match remaining.first() {
        Some(Clause::Where(w)) => {
            let state = state.where_(w.condition.clone());
            replay_after_where(state, &remaining[1..])
        }
        Some(Clause::Return(r)) => {
            let state = state.returning(r.expressions.clone());
            replay_after_return(state, &remaining[1..])
        }
        Some(Clause::With(w)) => {
            let state = state.with(w.expressions.clone());
            replay_after_with(state, &remaining[1..])
        }
        None => state.returning(Expression::Asterisk).build(), // edge case
        _ => panic!("Builder cannot handle clause after MATCH: {:?}", remaining[0]),
    }
}
// ... one function per builder state
```

The test then compares the builder-produced statement with the parser-produced one:

```rust
#[test]
fn parsed_query_reconstructible_via_builder() {
    let cypher = "MATCH (n:`Person`) WHERE n.age > 21 RETURN n";
    let parsed = parse(cypher).unwrap();
    let rebuilt = replay_through_builder(&parsed);
    assert_eq!(parsed.render(), rebuilt.render());
}
```

This serves two purposes:
- **Validates the parser** — If the parser produces something the builder rejects, it exposes a parser bug or a missing builder capability.
- **Validates the builder** — If the builder cannot reconstruct a valid Cypher query, it exposes a gap in the fluent API.

#### 3. Error case testing (invalid input → descriptive errors)

Dedicated tests for malformed input, verifying error messages include position and context:

```rust
#[test]
fn error_on_missing_return() {
    let err = parse("MATCH (n) ORDER BY n.name").unwrap_err();
    assert!(err.to_string().contains("ORDER BY"));
    assert!(err.line == 1);
}

#[test]
fn error_on_unclosed_parenthesis() {
    let err = parse("MATCH (n:Person RETURN n").unwrap_err();
    assert!(err.to_string().contains("')'"));
}
```

### Scope (Incremental Phases)

**Phase 1 — Core (MVP):**
- Literals: integers, floats, strings, booleans, null
- Identifiers and parameters (`$name`)
- Nodes: `(n:Label {props})`
- Relationships: `(a)-[:R]->(b)`, `(a)<-[:R]-(b)`, `(a)-[:R]-(b)`, `(a)-->(b)`
- Patterns: single and multi-hop chains
- Clauses: MATCH, OPTIONAL MATCH, WHERE, RETURN, WITH, ORDER BY, SKIP, LIMIT
- Expressions: arithmetic, comparison, boolean (AND/OR/NOT/XOR), string predicates
- Function calls: `name(args...)`
- Aliases: `expr AS alias`
- Properties: `n.name`, `n.address.city`
- IS NULL / IS NOT NULL / IN

**Phase 2 — Write clauses:**
- CREATE, MERGE (ON CREATE SET / ON MATCH SET), SET, DELETE, DETACH DELETE, REMOVE
- FOREACH
- UNWIND ... AS

**Phase 3 — Advanced:**
- UNION / UNION ALL
- EXPLAIN / PROFILE
- CASE WHEN ... THEN ... ELSE ... END
- List comprehensions: `[x IN list WHERE cond | expr]`
- Pattern comprehensions: `[(a)-->(b) | b.name]`
- Existential subqueries: `EXISTS { MATCH ... }`
- COUNT / COLLECT subqueries
- CALL procedures, CALL subqueries, IN TRANSACTIONS
- Variable-length relationships: `*`, `*2..5`
- Quantified relationships: `-[:R]->{2}`, `--+`
- Quantified path patterns: `((a)-[:R]->(b)){1,3}`
- Path selectors: SHORTEST, ALL SHORTEST, ANY
- Named paths: `p = (a)-[:R]->(b)`
- LOAD CSV
- USING INDEX / SCAN / JOIN hints
- Label expressions: `:A&B`, `:A|B`, `:!A`
- Map projections: `n { .name, .age }`
- Cypher 25 clauses: FINISH, FILTER, LET

### Key Design Decisions

1. **Two-phase parsing (lexer + parser) vs single-pass:** We use a two-phase approach. The lexer handles whitespace stripping, keyword recognition (case-insensitive), string escaping, and number parsing. The parser works on a clean token stream. This simplifies the grammar combinators and makes error positions more accurate.

2. **Keyword case-insensitivity:** The lexer uppercases keyword candidates before matching. Identifiers that happen to be keywords are distinguished by context (e.g., `MATCH` as keyword vs `` `match` `` as escaped identifier).

3. **Owned vs borrowed strings:** Parsed string literals produce `Cow::Owned` (since they need unescaping). Identifiers produce `Cow::Owned` as well (the input `&str` lifetime doesn't extend to the returned `Statement`). This matches how the AST uses `Cow<'static, str>`.

4. **No semantic validation:** The parser produces a syntactically valid AST. It does not check semantic rules (e.g., "RETURN must follow MATCH"). The existing typestate builder enforces these at build time; the parser bypasses the builder and constructs `Statement` directly from `Vec<Clause>`.

5. **Whitespace and comments:** The lexer skips whitespace and line comments (`// ...`). Block comments (`/* ... */`) are also skipped. The parser never sees whitespace tokens.
