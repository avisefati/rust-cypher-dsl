# Rust Cypher DSL

A type-safe, idiomatic Rust library for programmatically constructing
[Neo4j Cypher](https://neo4j.com/docs/cypher-manual/current/) queries.

The DSL mirrors Cypher clause structure through a **typestate builder pattern**,
ensuring valid clause ordering at compile time.

## Features

- **Compile-time safety** -- typestate builders prevent invalid clause ordering
- **Full Cypher coverage** -- MATCH, CREATE, MERGE, SET, DELETE, WITH, UNWIND,
  CALL, LOAD CSV, FOREACH, and Cypher 25 clauses (FINISH, FILTER, LET)
- **Operator syntax** -- `>>` / `<<` for outgoing / incoming relationships
- **Administration** -- indexes, constraints, SHOW, TERMINATE TRANSACTIONS
- **Parser** -- round-trip Cypher strings back into the AST (optional feature)
- **Pretty printing** -- single-line or indented multi-line output
- **AST introspection** -- extract labels, types, properties, and parameters
  from built statements via `StatementCatalog`
- **150+ built-in functions** -- aggregate, scalar, string, math, list,
  temporal, and spatial

## Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
rust-cypher-dsl = "0.1"
```

Import the prelude and use the `Cypher` entry point:

```rust
use rust_cypher_dsl::prelude::*;

let n = node("Person").named("n");
let stmt = Cypher::match_(n)
    .where_(prop("n", "age").gt(21_i32))
    .returning(name("n"))
    .build();

assert_eq!(
    stmt.render(),
    "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
);
```

## Building Relationships

### Operator syntax (`>>` / `<<`)

```rust
use rust_cypher_dsl::prelude::*;

let a = node("Person").named("a");
let b = node("Person").named("b");

let stmt = Cypher::match_(a >> rel("KNOWS") >> b)
    .returning((name("a"), name("b")))
    .build();

assert_eq!(
    stmt.render(),
    "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
);
```

### Shorthand methods

Nodes have `.to()`, `.from()`, and `.linked()` for quick one-liners:

```rust
use rust_cypher_dsl::prelude::*;

let a = node("Person").named("a");
let b = node("Person").named("b");

let stmt = Cypher::match_(a.to("KNOWS", b))
    .returning((name("a"), name("b")))
    .build();
```

### Full builder

For named variables, variable-length paths, or inline properties:

```rust
use rust_cypher_dsl::prelude::*;

let a = node("Person").named("a");
let b = node("Person").named("b");
let r = rel("KNOWS").named("r").min(1).max(3);
let pattern = a.rel(r).to(b);

let stmt = Cypher::match_(pattern)
    .returning(name("r"))
    .build();

assert_eq!(
    stmt.render(),
    "MATCH (a:`Person`)-[r:`KNOWS` *1..3]->(b:`Person`) RETURN r"
);
```

## Writing Data

```rust
use rust_cypher_dsl::prelude::*;

// CREATE
let stmt = Cypher::create(
    node("Movie").named("m").with_properties(
        props!("title" => "New Movie", "released" => 2024_i32)
    )
)
.returning(name("m"))
.build();

// MERGE with ON CREATE SET
let stmt = Cypher::merge(
    node("Person").named("p").with_properties(props!("name" => "Tom Hanks"))
)
.on_create(vec![SetItem::property(prop("p", "born"), 1956_i32)])
.returning(name("p"))
.build();
```

## Multi-Part Queries

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::functions::aggregate;

let p = node("Person").named("p");
let m = node("Movie").named("m");

let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
    .with((name("p"), aggregate::count(name("m")).alias("movieCount")))
    .where_(name("movieCount").gt(5_i32))
    .returning((Expression::from(prop("p", "name")), name("movieCount")))
    .order_by(name("movieCount").descending())
    .build();

assert_eq!(
    stmt.render(),
    "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`) \
     WITH p, count(m) AS movieCount \
     WHERE movieCount > 5 \
     RETURN p.name, movieCount \
     ORDER BY movieCount DESC"
);
```

## Administration

```rust
use rust_cypher_dsl::prelude::*;

// Create an index
let stmt = Cypher::create_index("movie_title")
    .for_node("m", "Movie", vec!["title"])
    .build();

// Create a uniqueness constraint
let stmt = Cypher::create_constraint("unique_title")
    .for_node("m", "Movie")
    .is_unique(vec!["title"]);

// Show indexes
let stmt = Cypher::show_indexes()
    .filter(IndexFilter::Range)
    .yield_all()
    .build();
```

## Built-in Functions

Functions are organized by category under `rust_cypher_dsl::functions`:

| Module | Examples |
|--------|----------|
| `aggregate` | `count`, `sum`, `avg`, `min`, `max`, `collect`, `percentileCont` |
| `scalar` | `id`, `coalesce`, `size`, `head`, `last`, `properties`, `type_of` |
| `string` | `toLower`, `toUpper`, `trim`, `replace`, `substring`, `split` |
| `math` | `abs`, `ceil`, `floor`, `round`, `sqrt`, `log`, `sin`, `cos`, `pi` |
| `list` | `range`, `keys`, `labels`, `nodes`, `relationships`, `tail`, `reduce` |
| `temporal` | `datetime`, `date`, `time`, `duration`, `localtime`, `localdatetime` |
| `spatial` | `point`, `point.distance`, `point.withinBBox` |

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::functions::{aggregate, scalar};

let stmt = Cypher::match_(node("Person").named("p"))
    .returning((
        Expression::from(prop("p", "name")),
        scalar::coalesce(vec![
            Expression::from(prop("p", "nickname")),
            Expression::from(prop("p", "name")),
        ]),
    ))
    .build();
```

## Pretty Printing

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::renderer::RenderConfig;

let stmt = Cypher::match_(node("Person").named("n"))
    .returning(name("n"))
    .build();

let config = RenderConfig {
    pretty_print: true,
    ..RenderConfig::default()
};
let pretty = stmt.render_with(config);
// MATCH (n:`Person`)
// RETURN n
```

## Parser (optional)

The `parser` feature (enabled by default) can parse Cypher strings back into
the AST:

```rust
use rust_cypher_dsl::parser;

let stmt = parser::parse("MATCH (n:Person) WHERE n.age > 21 RETURN n").unwrap();
assert_eq!(
    stmt.render(),
    "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
);
```

Disable it if you only need query building:

```toml
[dependencies]
rust-cypher-dsl = { version = "0.1", default-features = false }
```

## AST Introspection

`StatementCatalog` extracts metadata from built statements:

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::catalog::StatementCatalog;

let stmt = Cypher::match_(
    node("Person").named("p") >> rel("ACTED_IN") >> node("Movie").named("m")
)
.where_(prop("p", "name").eq(param("name")))
.returning(name("m"))
.build();

let catalog = StatementCatalog::from_statement(&stmt);
// catalog.labels        -> {"Person", "Movie"}
// catalog.relationship_types -> {"ACTED_IN"}
// catalog.parameters    -> {"name": None}
```

## Examples

Runnable examples live in the `examples/` directory:

```sh
cargo run --example movies          # Neo4j Movies graph queries
cargo run --example social_network  # Friend-of-friend, recommendations
cargo run --example admin           # Indexes, constraints, monitoring
```

The `examples` module in the API docs contains the same queries with
inline assertions: `cargo doc --open` and navigate to the **examples** module.

## Module Overview

| Module | Description |
|--------|-------------|
| `prelude` | Re-exports for convenient `use prelude::*` imports |
| `cypher` | Entry point (`Cypher`) for building statements |
| `builder` | Typestate builder structs enforcing valid clause ordering |
| `statement` | `Statement` AST root and rendering |
| `clauses` | Individual clause types (MATCH, RETURN, CREATE, etc.) |
| `types` | Core AST types: expressions, conditions, nodes, relationships, patterns |
| `renderer` | Single-line and pretty-print renderers |
| `functions` | Built-in Cypher functions (aggregate, scalar, string, math, etc.) |
| `admin` | Index, constraint, and transaction management |
| `catalog` | `StatementCatalog` for AST introspection |
| `parser` | Cypher string parser (optional, enabled by default) |
| `examples` | Real-world query examples using the Neo4j Movies graph |

## License

MIT
