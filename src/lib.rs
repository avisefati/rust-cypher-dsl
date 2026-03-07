// Crate-level lint configuration
#![deny(unsafe_code)]
#![warn(rustdoc::missing_crate_level_docs)]

//! # Rust Cypher DSL
//!
//! A type-safe, idiomatic Rust library for programmatically constructing
//! Neo4j Cypher queries.

pub mod types;
pub mod clauses;
pub mod functions;
pub mod renderer;

pub mod builder;
pub mod cypher;
pub mod statement;
pub mod catalog;
pub mod prelude;

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
