# Pattern-in-WHERE Requirements

## Overview

Support using graph patterns as boolean predicates inside WHERE clauses.
In Cypher, a pattern used in a WHERE clause acts as an existence check:
`WHERE (a)-[:KNOWS]->(b)` evaluates to true when the pattern exists.

## User Stories

1. **US-1 – Pattern existence check (builder)**
   As a DSL user, I want to pass a graph pattern to `.where_()` so that
   the rendered Cypher contains `WHERE <pattern>` as a boolean existence test.

2. **US-2 – Negated pattern check (builder)**
   As a DSL user, I want to negate a pattern predicate so that the rendered
   Cypher contains `WHERE NOT <pattern>`.

3. **US-3 – Pattern predicate with AND/OR composition (builder)**
   As a DSL user, I want to combine a pattern predicate with other conditions
   using `.and()` / `.or()` so that complex WHERE clauses work.

4. **US-4 – Parser round-trip**
   As a DSL user, I want the parser to recognize a pattern inside a WHERE
   clause and produce the same AST as the builder, enabling render-parse-render
   round-trip fidelity.

## Acceptance Criteria (EARS format)

| ID   | Criterion |
|------|-----------|
| AC-1 | **When** a `Pattern` (or any `IntoPattern` type) is passed to `.where_()`, **the system shall** render `WHERE <pattern>` using the standard pattern renderer. |
| AC-2 | **When** a pattern predicate is negated via `.not()` or the `not()` free function, **the system shall** render `WHERE NOT <pattern>`. |
| AC-3 | **When** a pattern predicate is composed with another condition via `.and()` / `.or()`, **the system shall** render the combined expression correctly (e.g. `WHERE a.x > 1 AND (a)-[:R]->(b)`). |
| AC-4 | **When** the parser encounters a graph pattern (starting with `(`) inside a WHERE clause, **the system shall** parse it into a `PatternPredicate` condition variant. |
| AC-5 | **When** the parser encounters `NOT` followed by a graph pattern inside a WHERE clause, **the system shall** parse it into `Not(PatternPredicate(...))`. |
| AC-6 | **When** a builder-rendered query containing pattern-in-WHERE is parsed and re-rendered, **the system shall** produce byte-identical output (round-trip). |
| AC-7 | All existing tests shall continue to pass with no regressions. |

## Scope

- Single-element and multi-hop chain patterns in WHERE.
- Negation (`NOT`) of pattern predicates.
- Composition with other conditions via AND/OR/XOR.
- Parser support for recognizing patterns in WHERE context.

## Out of Scope

- Named paths in WHERE (e.g. `WHERE p = (a)-[:R]->(b)`).
- Quantified path patterns in WHERE.
- Pattern comprehension in WHERE (already supported via `ExpressionInner::PatternComprehension`).
