// Crate-level lint configuration
#![deny(unsafe_code)]
#![warn(rustdoc::missing_crate_level_docs)]

//! # Rust Cypher DSL
//!
//! A type-safe, idiomatic Rust library for programmatically constructing
//! Neo4j Cypher queries. The DSL mirrors Cypher clause structure through
//! a typestate builder pattern, ensuring valid clause ordering at compile time.
//!
//! ## Quick Start
//!
//! Import the prelude and use the [`Cypher`](cypher::Cypher) entry point:
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! // MATCH (n:Person) WHERE n.age > 21 RETURN n
//! let n = node("Person").named("n");
//! let stmt = Cypher::match_(n)
//!     .where_(prop("n", "age").gt(21_i32))
//!     .returning(name("n"))
//!     .build();
//!
//! assert_eq!(stmt.render(), "MATCH (n:`Person`) WHERE n.age > 21 RETURN n");
//! ```
//!
//! ## Building Relationships
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! // MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a, b
//! let a = node("Person").named("a");
//! let b = node("Person").named("b");
//! let pattern = a.rel(rel("KNOWS")).to(b);
//!
//! let stmt = Cypher::match_(pattern)
//!     .returning((name("a"), name("b")))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
//! );
//! ```
//!
//! ## Operator Shorthand
//!
//! Use `>>` for outgoing and `<<` for incoming relationships:
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let a = node("Person").named("a");
//! let b = node("Person").named("b");
//! let pattern = a >> rel("KNOWS") >> b;
//!
//! let stmt = Cypher::match_(pattern)
//!     .returning(name("a"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a"
//! );
//! ```
//!
//! ## Pretty Printing
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! use rust_cypher_dsl::renderer::RenderConfig;
//!
//! let stmt = Cypher::match_(node("Person").named("n"))
//!     .returning(name("n"))
//!     .build();
//!
//! let config = RenderConfig { pretty_print: true, ..RenderConfig::default() };
//! let pretty = stmt.render_with(config);
//! assert!(pretty.contains('\n'));
//! ```
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`prelude`] | Re-exports for convenient `use prelude::*` imports |
//! | [`cypher`] | Entry point ([`Cypher`](cypher::Cypher)) for building statements |
//! | [`builder`] | Typestate builder structs enforcing valid clause ordering |
//! | [`statement`] | [`Statement`](statement::Statement) AST root and rendering |
//! | [`clauses`] | Individual clause types (MATCH, RETURN, CREATE, etc.) |
//! | [`types`] | Core AST types: expressions, conditions, nodes, relationships, patterns |
//! | [`renderer`] | Single-line and pretty-print renderers |
//! | [`functions`] | Built-in Cypher functions (aggregate, scalar, string, math, etc.) |
//! | [`catalog`] | [`StatementCatalog`](catalog::StatementCatalog) for AST introspection |

pub mod clauses;
pub mod functions;
pub mod renderer;
pub mod types;

pub mod builder;
pub mod catalog;
pub mod cypher;
pub mod prelude;
pub mod statement;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        // Smoke test: verify the crate root module tree is valid.
        // If this test runs, all module declarations compiled successfully.
    }
}
