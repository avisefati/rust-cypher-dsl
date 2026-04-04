# Pattern-in-WHERE Implementation Tasks

1. [x] **Add `PatternPredicate` variant to `Condition` enum** (AC-1)
   1. [x] Add `PatternPredicate(Pattern)` variant to `Condition` in `src/types/condition.rs`
   2. [x] Add `From<Pattern> for Condition` impl
   3. [x] Add `From<Relationship> for Condition` and `From<RelationshipChain> for Condition` impls
   4. [x] Add unit tests: create `PatternPredicate` from Pattern, Relationship, RelationshipChain
   5. [x] Add unit tests: `.and()`, `.or()`, `.not()` composition with `PatternPredicate`

2. [x] **Add renderer support for `PatternPredicate`** (AC-1, AC-2, AC-3)
   1. [x] Add `Condition::PatternPredicate` match arm in `write_condition()` in `src/renderer/default.rs`
   2. [x] Add renderer test: simple pattern renders `(a)-[:KNOWS]->(b)`
   3. [x] Add renderer test: NOT pattern renders `NOT (a)-[:KNOWS]->(b)`
   4. [x] Add renderer test: mixed condition renders `a.x > 1 AND (a)-[:R]->(b)`

3. [x] **Add builder round-trip test** (AC-1, AC-6)
   1. [x] Add builder test in `src/builder.rs`: `.where_(pattern)` builds and renders correctly
   2. [x] Add builder test: `.where_(pattern).and(other_condition)` renders correctly

4. [x] **Add `checkpoint()` / `restore()` to `TokenStream`** (AC-4)
   1. [x] Add `checkpoint(&self) -> usize` and `restore(&mut self, usize)` to `TokenStream` in `src/parser/grammar.rs`

5. [x] **Add parser support for pattern-in-WHERE** (AC-4, AC-5, AC-6)
   1. [x] Add `try_parse_pattern_as_expression()` function in `src/parser/expressions.rs`
   2. [x] Integrate into `parse_atom` to try pattern parse before falling through to parenthesized expression
   3. [x] Add parser test: `WHERE (a)-[:KNOWS]->(b)` round-trips
   4. [x] Add parser test: `WHERE NOT (a)-[:KNOWS]->(b)` round-trips
   5. [x] Add parser test: `WHERE (a:Person)-[:KNOWS]->(b:Actor)` round-trips
   6. [x] Add parser test: `WHERE (a)-[:R1]->(b)-[:R2]->(c)` chain round-trips
   7. [x] Add parser test: `WHERE a.x > 1 AND (a)-[:KNOWS]->(b)` mixed condition round-trips
   8. [x] Add parser test: parenthesized expression `WHERE (n.age + 1) > 2` still works (no regression)

6. [x] **Full round-trip tests** (AC-6, AC-7)
   1. [x] Verify all existing tests pass: `cargo test` — 1,764 tests pass
   2. [x] Verify clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
