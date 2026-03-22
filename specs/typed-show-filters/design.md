# Typed Show Filters — Design Document

## Overview

Replace the stringly-typed `type_filter(&str)` on `ShowBuilder` with
per-command enum types, providing compile-time safety and IDE
discoverability. Rename the method from `type_filter` to `filter`.

## Current State

A single `ShowBuilder` serves all five SHOW commands. The filter is an
`Option<Cow<'static, str>>` stored on `ShowCommand`:

```rust
Cypher::show_indexes().type_filter("RANGE").build();
Cypher::show_constraints().type_filter("UNIQUE").build();
Cypher::show_functions().type_filter("BUILT IN").build();
```

Problems:
- No compile-time validation — typos like `"RNAGE"` silently produce invalid Cypher.
- No IDE autocomplete for valid filter values.
- Nothing prevents cross-command misuse (e.g., `"RANGE"` on `show_constraints()`).

## Design

### New Enums

Three new enums, one per command family that supports filtering:

```rust
/// Filter for `SHOW ... INDEXES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexFilter {
    Range,
    Text,
    Point,
    Fulltext,
    Vector,
    Lookup,
}

/// Filter for `SHOW ... CONSTRAINTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintFilter {
    Unique,
    Uniqueness,
    Exists,
    NotNull,
    NodeKey,
    RelationshipKey,
    PropertyType,
}

/// Filter for `SHOW ... FUNCTIONS` and `SHOW ... PROCEDURES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableFilter {
    BuiltIn,
    UserDefined,
    All,
}
```

Each enum implements `Display` to produce the Cypher keyword(s):

| Variant | Rendered |
|---------|----------|
| `IndexFilter::Range` | `RANGE` |
| `IndexFilter::Fulltext` | `FULLTEXT` |
| `ConstraintFilter::NotNull` | `NOT NULL` |
| `ConstraintFilter::NodeKey` | `NODE KEY` |
| `ConstraintFilter::PropertyType` | `PROPERTY TYPE` |
| `CallableFilter::BuiltIn` | `BUILT IN` |
| `CallableFilter::UserDefined` | `USER DEFINED` |

### Split `ShowBuilder` Into Per-Command Builders

Replace the single `ShowBuilder` with typed builders so `.filter()`
accepts only the correct enum:

```rust
pub struct ShowIndexesBuilder    { inner: ShowCommand }
pub struct ShowConstraintsBuilder { inner: ShowCommand }
pub struct ShowFunctionsBuilder  { inner: ShowCommand }
pub struct ShowProceduresBuilder { inner: ShowCommand }
pub struct ShowTransactionsBuilder { inner: ShowCommand }
```

Each builder exposes only the methods relevant to that command:

| Builder | `.filter()` type | `.ids()` | `.executable_by*()` |
|---------|------------------|----------|---------------------|
| `ShowIndexesBuilder` | `IndexFilter` | -- | -- |
| `ShowConstraintsBuilder` | `ConstraintFilter` | -- | -- |
| `ShowFunctionsBuilder` | `CallableFilter` | -- | yes |
| `ShowProceduresBuilder` | `CallableFilter` | -- | yes |
| `ShowTransactionsBuilder` | -- | yes | -- |

Common methods (`yield_all`, `yield_fields`, `where_`, `build`) are
shared via a macro or trait to avoid duplication.

### `ShowCommand` Internal Change — Wrapper Enum

A wrapper enum preserves type information end-to-end (builder through
storage through rendering):

```rust
/// Wrapper that holds the typed filter for any SHOW command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowTypeFilter {
    Index(IndexFilter),
    Constraint(ConstraintFilter),
    Callable(CallableFilter),
}
```

`ShowCommand` stores the wrapper instead of a raw string:

```rust
pub struct ShowCommand {
    // Before:  type_filter: Option<Cow<'static, str>>,
    // After:
    pub(crate) type_filter: Option<ShowTypeFilter>,
    // ... rest unchanged
}
```

Each specific enum implements `From` into the wrapper:

```rust
impl From<IndexFilter> for ShowTypeFilter {
    fn from(f: IndexFilter) -> Self { Self::Index(f) }
}
// ... same for ConstraintFilter, CallableFilter
```

The `type_filter()` accessor returns `Option<&ShowTypeFilter>`.

The renderer delegates to `Display`:

```rust
impl Display for ShowTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(v) => v.fmt(f),
            Self::Constraint(v) => v.fmt(f),
            Self::Callable(v) => v.fmt(f),
        }
    }
}

// In the renderer:
if let Some(filter) = sc.type_filter() {
    write!(buf, "{filter} ");
}
```

This means:
- Type information is preserved, not discarded at storage time.
- Pattern matching on the stored filter is possible downstream.
- The renderer change is minimal — `Display` on the wrapper delegates
  to the inner enum.

### Entry Points (on `Cypher`)

No signature changes — each method already exists, just returns a
different type now:

```rust
impl Cypher {
    pub fn show_indexes() -> ShowIndexesBuilder { ... }
    pub fn show_constraints() -> ShowConstraintsBuilder { ... }
    pub fn show_functions() -> ShowFunctionsBuilder { ... }
    pub fn show_procedures() -> ShowProceduresBuilder { ... }
    pub fn show_transactions() -> ShowTransactionsBuilder { ... }
}
```

### API Examples

```rust
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::admin::IndexFilter;

// Typed filter — compiler checks validity
let stmt = Cypher::show_indexes()
    .filter(IndexFilter::Range)
    .yield_all()
    .build();

// Short form with variant import
use rust_cypher_dsl::admin::ConstraintFilter::*;
let stmt = Cypher::show_constraints()
    .filter(Unique)
    .build();

// No filter — still works
let stmt = Cypher::show_indexes().build();

// Compile error — IndexFilter has no `Unique` variant
// Cypher::show_indexes().filter(ConstraintFilter::Unique)
```

### Parser Changes

The parser (`src/parser/admin.rs`) currently collects filter words into
a `String`. It will instead map the collected string to the appropriate
enum variant and store the `ShowTypeFilter` wrapper:

```rust
let filter = match (show_kind, type_filter_str.as_str()) {
    (Indexes, "RANGE") => ShowTypeFilter::Index(IndexFilter::Range),
    (Indexes, "TEXT") => ShowTypeFilter::Index(IndexFilter::Text),
    (Constraints, "UNIQUE") => ShowTypeFilter::Constraint(ConstraintFilter::Unique),
    (Functions, "BUILT IN") => ShowTypeFilter::Callable(CallableFilter::BuiltIn),
    // ...
};
sc = sc.with_type_filter(filter);
```

The `with_type_filter` method on `ShowCommand` changes to accept
`ShowTypeFilter` instead of `impl Into<Cow<'static, str>>`.

### Prelude Exports

Export the enum **types** (not variants) from the prelude:

```rust
// In prelude.rs
pub use crate::admin::{IndexFilter, ConstraintFilter, CallableFilter};
```

Users who want short variants can opt in: `use admin::IndexFilter::*;`

### Reducing Boilerplate — Macro

The five builders share `yield_all`, `yield_fields`, `where_`, and
`build`. A declarative macro generates each builder:

```rust
macro_rules! show_builder {
    ($name:ident, $filter_type:ty, $kind:expr $(, $extra:tt)*) => {
        pub struct $name { inner: ShowCommand }
        impl $name {
            pub fn filter(mut self, f: $filter_type) -> Self { ... }
            pub fn yield_all(mut self) -> Self { ... }
            pub fn yield_fields(mut self, fields: Vec<Expression>) -> Self { ... }
            pub fn where_(mut self, condition: impl Into<Condition>) -> Self { ... }
            pub fn build(self) -> Statement { ... }
        }
    };
}
```

## Files Changed

| File | Change |
|------|--------|
| `src/admin/mod.rs` | Add `IndexFilter`, `ConstraintFilter`, `CallableFilter` enums |
| `src/admin/builder.rs` | Replace `ShowBuilder` + `ShowKind` with 5 typed builders; rename `type_filter` to `filter` |
| `src/admin/show.rs` | Change `type_filter` from `Option<Cow<str>>` to `Option<ShowTypeFilter>` |
| `src/cypher.rs` | Update return types of `show_*()` methods |
| `src/parser/admin.rs` | Map parsed filter string to enum before storing |
| `src/renderer/default.rs` | Use `Display` on `ShowTypeFilter` instead of raw `&str` |
| `src/renderer/pretty.rs` | Same — use `Display` on `ShowTypeFilter` |
| `src/prelude.rs` | Re-export `IndexFilter`, `ConstraintFilter`, `CallableFilter` |
| Tests | Update all `type_filter("...")` calls to `filter(Enum::Variant)` |
| `examples/admin.rs` | Update filter usage |

## Testing Strategy

1. **Unit tests per enum**: `Display` output matches expected Cypher keyword.
2. **Builder tests**: Each builder accepts only its own filter enum (existing tests migrated).
3. **Compile-fail assertion**: Document that cross-command filters don't compile (comment-based, not automated).
4. **Round-trip**: Parser tests parse `SHOW RANGE INDEXES` and verify the filter value.
5. **Renderer**: Existing renderer tests pass unchanged (output is the same).
