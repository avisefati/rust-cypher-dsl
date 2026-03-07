// Crate-level lint configuration
#![deny(unsafe_code)]
#![warn(rustdoc::missing_crate_level_docs)]

//! # Rust Cypher DSL
//!
//! A type-safe, idiomatic Rust library for programmatically constructing
//! Neo4j Cypher queries.

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
