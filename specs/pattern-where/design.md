# Pattern-in-WHERE Design

## 1. Overview

Add support for graph patterns as boolean existence predicates in WHERE
clauses. In Cypher, `WHERE (a)-[:KNOWS]->(b)` checks whether the pattern
exists and evaluates to a boolean. This feature bridges the `Pattern`
type system with the `Condition` type system.

## 2. Architecture

### Current type flow

```
Builder .where_(impl Into<Condition>)
  -> WhereClause { condition: Condition }
  -> Renderer: write_condition() dispatches on Condition variant
```

Patterns and conditions are isolated type hierarchies today:

```
Pattern / PatternElement   <-- used by MATCH / CREATE / MERGE
Condition                  <-- used by WHERE
Expression                 <-- used everywhere, bridges Condition via ExpressionInner::Condition
```

There is no `Pattern -> Condition` or `Pattern -> Expression` conversion.

### Target type flow

```
Builder .where_(pattern)       // pattern: impl Into<Condition>
  -> From<Pattern> for Condition  // creates Condition::PatternPredicate(Pattern)
  -> WhereClause { condition: Condition::PatternPredicate(pattern) }
  -> Renderer: write_condition matches PatternPredicate, delegates to write_pattern()
```

## 3. Components and Interfaces

### 3.1 New Condition variant

Add to `Condition` enum in `src/types/condition.rs`:

```rust
/// Pattern existence check: `(a)-[:KNOWS]->(b)` in WHERE context.
PatternPredicate(Pattern),
```

**Rationale:** Adding to `Condition` (not `Expression`) because:
- WHERE stores a `Condition` directly
- Pattern predicates are inherently boolean
- Avoids adding a `Pattern` dependency to the `Expression` enum
- Composition with `.and()` / `.or()` / `.not()` works automatically
  via existing `Condition` methods

### 3.2 Type conversions

In `src/types/condition.rs`:

```rust
impl From<Pattern> for Condition {
    fn from(pattern: Pattern) -> Self {
        Self::PatternPredicate(pattern)
    }
}
```

This is sufficient because `where_()` accepts `impl Into<Condition>`,
and `Pattern` already has `From` impls from all `IntoPattern` types
(`Node`, `Relationship`, `RelationshipChain`, etc.). However, those
go `Node -> Pattern`, not `Node -> Condition`. To allow
`.where_(some_relationship)` directly, we add convenience impls:

```rust
impl<T: IntoPattern> From<T> for Condition { ... }
```

**Problem:** This conflicts with the existing blanket impls. Instead,
we add a dedicated builder method:

```rust
// On all builder states that have where_():
pub fn where_pattern(self, pattern: impl IntoPattern) -> ... {
    self.where_(Condition::PatternPredicate(pattern.into_pattern()))
}
```

**Decision:** Use `From<Pattern> for Condition` only (no blanket `IntoPattern`).
Users pass a `Pattern` directly, or construct one inline. The existing
`>>` / `<<` operators produce `Relationship` / `RelationshipChain` which
implement `Into<Pattern>`, so a two-step is needed:

```rust
// Option A: explicit Pattern::new()
.where_(Pattern::new(a >> rel("KNOWS") >> b))

// Option B: .into_pattern() call
.where_((a >> rel("KNOWS") >> b).into_pattern())
```

Both work because `Pattern: Into<Condition>` via the new `From` impl.

To make Option A unnecessary, we also add:

```rust
impl From<Relationship> for Condition { ... }
impl From<RelationshipChain> for Condition { ... }
impl From<Node> for Condition { ... }
```

These wrap the value in `PatternPredicate(Pattern::new(value))`. This
lets users write `.where_(a >> rel("KNOWS") >> b)` directly.

### 3.3 Renderer changes

In `src/renderer/default.rs`, add a match arm in `write_condition()`:

```rust
Condition::PatternPredicate(pattern) => {
    self.write_pattern(buf, pattern);
}
```

This reuses the existing `write_pattern()` which already handles all
`PatternElement` variants (Node, Relationship, Chain, NamedPath,
QuantifiedPath, SelectedPath).

### 3.4 Parser changes

In `src/parser/conditions.rs` (or `expressions.rs`), the parser must
detect when a `(` in WHERE context starts a pattern rather than a
parenthesized expression.

**Disambiguation strategy:**

A pattern node starts with `(` followed by:
- `)` (empty node)
- `:Label` (labeled node)
- `identifier :` (named node with label)
- `identifier )` (named node, bare)
- `identifier {` (named node with properties)
- `identifier -` or `identifier <` (would be parsed as relationship after)

A parenthesized expression starts with `(` followed by:
- a literal, `$param`, a function call, a sub-expression, etc.

The key insight: after parsing what's inside `(...)`, if a relationship
arrow (`-[`, `-`, `<-`) follows the `)`, it's a pattern. If a comparison
operator, boolean keyword, or clause keyword follows, it's an expression.

**Implementation:** In `parse_not_expression` (or `parse_primary`), when
we see `(` in a WHERE context:
1. Save the stream position
2. Try `parse_pattern(stream)`
3. If it succeeds and the result is a relationship/chain (not just a bare
   name that could be an expression), wrap in `PatternPredicate`
4. If ambiguous or fails, restore position and parse as expression

A simpler approach: in `parse_primary`, when encountering `(`, use
lookahead to detect pattern syntax:
- `(` + identifier/escaped + `:` -> pattern (labeled node)
- `(` + `:` -> pattern (anonymous labeled node)
- `(` + `)` followed by `-` or `<` -> pattern (empty node in relationship)
- Otherwise -> parenthesized expression

**Chosen approach:** Speculative parse with backtrack. This is the most
reliable approach since it handles all edge cases:

```rust
// In parse_primary or a new try_parse_pattern_predicate:
fn try_parse_pattern_predicate(stream: &mut TokenStream) -> Option<Condition> {
    let checkpoint = stream.checkpoint();
    match parse_pattern(stream) {
        Ok(pattern) => {
            // Verify this is actually a pattern (has relationships/chains,
            // or is a labeled node - not just a bare identifier in parens)
            if is_genuine_pattern(&pattern) {
                Some(Condition::PatternPredicate(pattern))
            } else {
                stream.restore(checkpoint);
                None
            }
        }
        Err(_) => {
            stream.restore(checkpoint);
            None
        }
    }
}
```

**Does `TokenStream` support checkpointing?** Need to verify. If not,
we use lookahead-based detection instead.

### 3.5 TokenStream checkpoint support

If `TokenStream` doesn't have checkpoint/restore, we add it:

```rust
impl TokenStream {
    pub fn checkpoint(&self) -> usize { self.pos }
    pub fn restore(&mut self, checkpoint: usize) { self.pos = checkpoint; }
}
```

## 4. Data Model Changes

### Condition enum (src/types/condition.rs)

```diff
 pub enum Condition {
     // ... existing variants ...
+    /// Pattern existence check: `(a)-[:KNOWS]->(b)`.
+    PatternPredicate(Pattern),
     NoCondition,
 }
```

### New From impls

```rust
// condition.rs
impl From<Pattern> for Condition { ... }
impl From<Relationship> for Condition { ... }
impl From<RelationshipChain> for Condition { ... }
```

## 5. Error Handling

- **Builder:** No new error paths. `From` conversions are infallible.
- **Parser:** If speculative pattern parse fails, falls back to expression
  parsing. No new error variants needed.
- **Renderer:** No new error paths. `write_pattern()` already handles
  all pattern elements.

## 6. Testing Strategy

### Unit tests

1. **Condition creation:** `Condition::PatternPredicate` from Pattern, Relationship, RelationshipChain
2. **Composition:** PatternPredicate `.and()` / `.or()` / `.not()` with other conditions
3. **Renderer:** `write_condition` for PatternPredicate renders correct Cypher
4. **Renderer:** NOT PatternPredicate renders `NOT (a)-[:R]->(b)`
5. **Renderer:** Mixed conditions render correctly (e.g. `a.x > 1 AND (a)-[:R]->(b)`)

### Parser tests

6. **Simple pattern-in-WHERE:** `WHERE (a)-[:KNOWS]->(b)` parses to PatternPredicate
7. **NOT pattern:** `WHERE NOT (a)-[:KNOWS]->(b)` parses to Not(PatternPredicate)
8. **Labeled node pattern:** `WHERE (a:Person)-[:KNOWS]->(b:Person)` parses correctly
9. **Chain pattern:** `WHERE (a)-[:R1]->(b)-[:R2]->(c)` parses correctly
10. **Mixed:** `WHERE a.x > 1 AND (a)-[:KNOWS]->(b)` parses correctly

### Integration / round-trip tests

11. **Builder -> render -> parse -> render** for each pattern type
12. **Update program_listing example** (or add new example) exercising pattern-in-WHERE

### Regression

13. All existing tests pass unchanged
