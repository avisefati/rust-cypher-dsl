# Admin DSL Security Hardening -- Design Document

## Overview

The admin DSL (indexes, constraints, SHOW/TERMINATE commands) bypasses the
escaping infrastructure that protects the core query path.  Every admin string
-- index names, constraint names, variables, labels, relationship types,
property names, usernames, transaction IDs, and Cypher type names -- is
concatenated into output with raw `push_str()` calls.  This allows Cypher
injection, quote breakout, and semantic smuggling through the public builder
API.

This design closes all known injection vectors by applying a two-layer
defence: **validation at construction** (builders panic on invalid input) and
**escaping at render time** (renderer uses the same primitives as the core
DSL).

## Threat Model

| # | Severity | Vector | Root Cause |
|---|----------|--------|------------|
| 1 | High | Index/constraint names, variables, labels, rel types, property names in admin renderer | Raw `push_str()` -- no backtick escaping |
| 2 | Medium | Transaction IDs in SHOW/TERMINATE | Single-quoted but not escaped (`'` inside breaks out) |
| 3 | Medium | Usernames in `EXECUTABLE BY` | Raw `push_str()` -- no escaping or validation |
| 4 | Medium | Cypher type names in `IS :: TYPE` | Raw `push_str()` -- no validation |
| 5 | Low | Reserved keywords as identifiers in `AsNeeded` mode | `needs_escaping()` only checks character shape, not keywords |

## Architecture

### Layer 1: Renderer Escaping (Finding 1, 2, 3, 4)

Route every admin string through the existing safe rendering primitives.
No new escaping logic is introduced -- we reuse what already works for the
core DSL:

| Position | Rendering primitive | Rationale |
|----------|-------------------|-----------|
| Index/constraint **names** (schema object names) | `write_safe_identifier()` | Backtick-escaped if needed; these are Cypher identifiers |
| **Variables** in index/constraint patterns | `write_safe_identifier()` | Same as node/rel variables in the core DSL |
| **Labels** in index/constraint patterns | `write_escaped_name()` | Same as labels in MATCH patterns (config-controlled) |
| **Relationship types** in index/constraint patterns | `write_escaped_name()` | Same as rel types in MATCH patterns |
| **Property names** in index/constraint REQUIRE/ON | `write_safe_identifier()` | Same as property keys in the core DSL |
| **Transaction IDs** in SHOW/TERMINATE | `write_string_literal_static()` | These are string literals -- escape `'` and `\` |
| **Usernames** in `EXECUTABLE BY` | `write_safe_identifier()` | Neo4j usernames are identifiers |
| **Cypher type names** in `IS :: TYPE` | `write_safe_identifier()` | Type names are identifiers (e.g., `FLOAT`, `STRING`) |

#### Concrete renderer changes

**`write_create_index()`** (lines 1448-1477):
- Index name: `push_str(name)` -> `self.write_safe_identifier(buf, name)`

**`write_index_target()`** (lines 1479-1580):
- Variable: `push_str(variable)` -> `self.write_safe_identifier(buf, variable)`
- Labels: `push_str(label)` -> `self.write_escaped_name(buf, label)`
- Rel types: `push_str(t)` -> `self.write_escaped_name(buf, t)`
- Properties: `push_str(prop)` -> `self.write_safe_identifier(buf, prop)`

**`write_drop_index()`** (line 1591):
- Name: `push_str(di.name())` -> `self.write_safe_identifier(buf, di.name())`

**`write_create_constraint()`** (lines 1597-1658):
- Constraint name: `push_str(name)` -> `self.write_safe_identifier(buf, name)`
- Variable: `push_str(variable)` -> `self.write_safe_identifier(buf, variable)`
- Label: `push_str(label)` -> `self.write_escaped_name(buf, label)`
- Rel type: `push_str(rel_type)` -> `self.write_escaped_name(buf, rel_type)`
- Properties: `push_str(prop)` -> `self.write_safe_identifier(buf, prop)`
- Type name in `PropertyType`: `push_str(type_name)` -> `self.write_safe_identifier(buf, type_name)`

**`write_drop_constraint()`** (line 1669):
- Name: `push_str(dc.name())` -> `self.write_safe_identifier(buf, dc.name())`

**`write_show_command()`** (lines 1675-1733):
- Transaction IDs: manual `push('\''); push_str(id); push('\'')` -> `Self::write_string_literal_static(buf, id)`
- Username: `push_str(user)` -> `self.write_safe_identifier(buf, user)`

**`write_terminate_transactions()`** (lines 1735-1771):
- Transaction IDs: manual quoting -> `Self::write_string_literal_static(buf, id)`

**Pretty renderer** mirrors the same changes (it delegates to the same
rendering primitives for admin commands).

### Layer 2: Builder Validation (Finding 1, 3, 4)

Add `assert!` validation in admin builders, matching the pattern already used
by `Parameter::new()`, `Expression::function_invocation()`, and
`CallClause::new()`.  This provides fail-fast feedback at construction time.

#### Validation rules

| Input kind | Rule | Regex equivalent | Example rejects |
|-----------|------|-----------------|-----------------|
| Identifier (variable, property, username, schema name) | `is_valid_identifier()` | `[a-zA-Z_][a-zA-Z0-9_]*` | `"idx IF EXISTS"`, `"n) DELETE"` |
| Label / relationship type | `is_valid_label_name()` | Non-empty, no NUL bytes | `""` (but allows spaces, backticks, unicode -- renderer handles escaping) |
| Transaction ID | `is_valid_transaction_id()` | Non-empty, no single-quote `'`, no backslash `\`, no NUL | `"tx' YIELD *"` |
| Cypher type name | `is_valid_cypher_type()` | `[A-Z][A-Z ]*` (allows `NOT NULL`, `LIST<STRING>`, etc.) | We keep this permissive and rely on renderer escaping |

#### Where validation is added

**`IndexBuilder`**:
- `new()`: validate `name` as identifier
- `for_node()`, `for_node_multi_label()`: validate `variable` as identifier; labels are left permissive (renderer escapes)
- `for_relationship()`, `for_relationship_multi_type()`: validate `variable` as identifier; types are left permissive
- `for_node_lookup()`, `for_relationship_lookup()`: validate `variable` as identifier
- All: validate each property name as identifier

**`ConstraintBuilder`**:
- `new()`: validate `name` as identifier
- `for_node()`: validate `variable` as identifier; label is left permissive
- `for_relationship()`: validate `variable` as identifier; rel_type is left permissive
- `is_typed()`: validate `property` as identifier

**`ConstraintRequire`**:
- `is_unique()`, `is_not_null()`, `is_node_key()`, `is_relationship_key()`: validate properties as identifiers

**`ShowFunctionsBuilder` / `ShowProceduresBuilder`**:
- `executable_by()`: validate `user` as identifier

**`ShowTransactionsBuilder`**:
- `ids()`: validate each transaction ID (no `'`, no `\`, no NUL, non-empty)

**`TerminateBuilder`** (via `Cypher::terminate_transactions()`):
- Validate each transaction ID

**`Cypher` entry points**:
- `drop_index()`, `drop_index_if_exists()`: validate name as identifier
- `drop_constraint()`, `drop_constraint_if_exists()`: validate name as identifier

#### Validation functions

Placed in a new `src/admin/validate.rs` module:

```rust
/// Checks if a string is a valid Cypher identifier.
///
/// Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`
/// Used for: variables, property names, schema object names, usernames.
pub(crate) fn is_valid_identifier(name: &str) -> bool {
    // Same logic as types::parameter::is_valid_identifier
}

/// Checks if a string is a valid transaction ID for SHOW/TERMINATE.
///
/// Rules: non-empty, no single-quote, no backslash, no NUL.
pub(crate) fn is_valid_transaction_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('\'') && !id.contains('\\') && !id.contains('\0')
}

/// Asserts the name is a valid identifier, panicking with a clear message.
pub(crate) fn assert_valid_identifier(name: &str, context: &str) {
    assert!(
        is_valid_identifier(name),
        "invalid {context} `{name}`: must match [a-zA-Z_][a-zA-Z0-9_]*"
    );
}

/// Asserts the transaction ID is valid, panicking with a clear message.
pub(crate) fn assert_valid_transaction_id(id: &str) {
    assert!(
        is_valid_transaction_id(id),
        "invalid transaction ID `{id}`: must not contain single quotes, \
         backslashes, or NUL bytes, and must not be empty"
    );
}
```

### Layer 3: Reserved Keyword Escaping (Finding 5)

Expand `needs_escaping()` to also return `true` for Cypher reserved keywords.
This closes the `AsNeeded` gap where names like `MATCH`, `RETURN`, or `SET`
pass the character-shape check but break parsing.

```rust
fn needs_escaping(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    let Some(first) = name.chars().next() else {
        unreachable!("guarded by is_empty check above");
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return true;
    }
    if name.chars().skip(1).any(|ch| !ch.is_ascii_alphanumeric() && ch != '_') {
        return true;
    }
    // Also escape Cypher reserved keywords (case-insensitive).
    is_reserved_keyword(name)
}
```

The `is_reserved_keyword()` function performs a case-insensitive lookup against
a compile-time set of Cypher reserved words.  Implementation options:

- **`phf::Set`** (compile-time perfect hash) -- zero runtime allocation, O(1)
  lookup.  Adds a build dependency.
- **Sorted `&[&str]` + binary search** -- zero dependencies, O(log n),
  ~200 entries.
- **`HashSet` lazy-initialized** -- simplest, but runtime allocation.

Recommendation: **sorted slice + binary search** for zero dependencies and
simplicity.  The list has ~200 entries; binary search is ~8 comparisons.

```rust
/// Cypher reserved keywords (sorted, uppercase).
const RESERVED_KEYWORDS: &[&str] = &[
    "ACCESS", "ACTIVE", "ADMIN", "ADMINISTRATOR", "ALIAS", "ALL",
    "AND", "ANY", "ARRAY", "AS", "ASC", "ASCENDING", ...
];

fn is_reserved_keyword(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    RESERVED_KEYWORDS.binary_search(&upper.as_str()).is_ok()
}
```

### Impact on Existing Tests

Renderer escaping changes will alter the output of admin commands that use
names containing reserved keywords or special characters.  For typical
well-formed names like `person_name_idx`, `Person`, `name`, etc., the output
is unchanged because:

- `write_safe_identifier("person_name_idx")` -> `person_name_idx` (no escaping needed)
- `write_escaped_name("Person")` -> `` `Person` `` in `Always` mode (which is
  the current default), matching existing test expectations

**Wait** -- the default `EscapeMode` is `Always`, which means labels are
already backtick-wrapped in the core DSL.  But admin tests currently expect
**unescaped** labels: `"FOR (n:Person) ON (n.name)"`.  Switching to
`write_escaped_name()` with `Always` mode would produce
`` FOR (n:`Person`) ON (n.name) ``.

**Resolution**: Admin labels and relationship types in schema commands follow
a different convention from query patterns.  In Neo4j's own output and docs,
schema command labels are unescaped unless they contain special characters.
We should use `write_safe_identifier()` for labels/types in admin commands
(AsNeeded escaping, config-independent), not `write_escaped_name()`.  This
matches the admin context where labels are schema identifiers, not pattern
matchers.

**Updated table**:

| Position | Rendering primitive |
|----------|-------------------|
| Labels in admin index/constraint | `write_safe_identifier()` |
| Relationship types in admin index/constraint | `write_safe_identifier()` |

This ensures:
- `"Person"` -> `Person` (no change to existing tests)
- `"Person) ON (n.name) //"` -> `` `Person) ON (n.name) //` `` (injection neutralized)

## Components Modified

| File | Changes |
|------|---------|
| `src/admin/validate.rs` | **New file.** Validation functions. |
| `src/admin/mod.rs` | Add `mod validate;` |
| `src/admin/builder.rs` | Add validation calls in all builder methods |
| `src/cypher.rs` | Add validation in `drop_index()`, `drop_constraint()` entry points |
| `src/renderer/default.rs` | Replace raw `push_str()` with safe primitives in all admin write methods; add reserved keyword list to `needs_escaping()` |
| `src/renderer/pretty.rs` | Mirror admin escaping changes (if admin rendering is duplicated there) |
| `tests/security_it.rs` | Add admin-focused injection tests (see Testing Strategy) |
| `tests/admin_commands_it.rs` | Verify existing tests still pass unchanged |

## Testing Strategy

### New security tests (`tests/security_it.rs`)

Each test constructs an admin command with an injection payload and verifies
the output is safe:

1. **Index name injection** -- `create_index("idx IF EXISTS")` panics
2. **Index name injection via label** -- `for_node("n", "P) ON (n.x) //", ...)` -> backtick-escaped
3. **Index variable injection** -- `for_node("n) DELETE n //", ...)` panics
4. **Index property injection** -- `for_node("n", "P", vec!["x) //"])` panics
5. **Constraint name injection** -- `create_constraint("c IF EXISTS")` panics
6. **Constraint variable injection** -- `for_node("n) DELETE", "P")` panics
7. **Constraint label injection** -- `for_node("n", "P) DELETE n")` -> backtick-escaped
8. **Drop index name smuggling** -- `drop_index("idx IF EXISTS")` panics
9. **Drop constraint name smuggling** -- `drop_constraint("c IF EXISTS")` panics
10. **Transaction ID quote breakout** -- `ids(vec!["tx' YIELD *"])` panics
11. **Transaction ID in terminate** -- `terminate_transactions(vec!["tx' YIELD *"])` panics
12. **Username injection** -- `executable_by("alice YIELD *")` panics
13. **Property type injection** -- `is_typed("score", "FLOAT) //")` -> backtick-escaped
14. **Reserved keyword as variable** -- `for_node("MATCH", "Person", vec!["name"])` -> `` `MATCH` `` (backtick-escaped, not panicking -- it's a valid identifier shape but a reserved word)

### Validation unit tests (`src/admin/validate.rs`)

- `is_valid_identifier` accepts `"foo"`, `"_bar"`, `"x123"`, `"a"`
- `is_valid_identifier` rejects `""`, `"123"`, `"a b"`, `"a)b"`, `"a\0b"`
- `is_valid_transaction_id` accepts `"neo4j-tx-123"`, `"abc"`
- `is_valid_transaction_id` rejects `""`, `"tx'"`, `"tx\\"`, `"tx\0"`

### Existing tests

All ~1121 existing tests must continue to pass unchanged.  The renderer
changes are backwards-compatible for well-formed inputs because
`write_safe_identifier()` is a no-op for names matching
`[a-zA-Z_][a-zA-Z0-9_]*`.

## Error Handling

Validation failures panic with `assert!`, consistent with the existing pattern
in `Parameter::new()`, `Expression::function_invocation()`, and
`CallClause::new()`.  This is appropriate for a query-builder DSL where
invalid input is a programming error, not a runtime condition.

## Non-Goals

- **`raw_unchecked()` / `raw()`** -- These are intentionally unsafe escape
  hatches.  They are documented as such and are out of scope.
- **Parameterised admin commands** -- Neo4j does not support `$param` in DDL
  positions (index names, labels in schema commands, etc.), so parameterisation
  is not an option here.
- **Fallible constructors (`Result`)** -- The existing crate convention is
  `assert!` for programmer errors.  Switching to `Result` would be a breaking
  API change and is out of scope for this hardening pass.
