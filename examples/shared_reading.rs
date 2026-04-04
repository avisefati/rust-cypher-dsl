//! Shared-reading analytics query with incoming relationships, COALESCE,
//! WHERE after WITH, ORDER BY DESC, and LIMIT.
//!
//! Demonstrates: `<<` operator for incoming relationships, count(DISTINCT),
//! COALESCE with multiple fallbacks, WHERE filtering after WITH, and
//! dynamic labels.
//!
//! Run with: `cargo run --example shared_reading`

use rust_cypher_dsl::functions::aggregate::{count_distinct, sum};
use rust_cypher_dsl::functions::scalar::coalesce;
use rust_cypher_dsl::prelude::*;

/// Builds a shared-reading analytics query with dynamic labels.
///
/// Finds pairs of readers who borrowed the same books (incoming
/// relationship pattern), aggregates the overlap, and returns the top
/// pairs by shared count.
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (r1:Reader)-[:BORROWED]->(b:Book)<-[:BORROWED]-(r2:Reader)
/// WHERE r1.id <> r2.id
/// WITH r1, r2, count(DISTINCT b) AS sharedCount
/// WHERE sharedCount > 0
/// WITH COALESCE(r1.name, r1.id, '') AS readerName, sum(sharedCount) AS totalShared
/// RETURN readerName, totalShared AS sharedCount
/// ORDER BY totalShared DESC
/// LIMIT $limit
/// ```
fn build_shared_reading_query(reader_label: &str, book_label: &str) -> Statement {
    let r1 = node(reader_label.to_owned()).named("r1");
    let b = node(book_label.to_owned()).named("b");
    let r2 = node(reader_label.to_owned()).named("r2");

    // (r1)-[:BORROWED]->(b)<-[:BORROWED]-(r2)
    let pattern = r1 >> rel("BORROWED") >> b << rel("BORROWED") << r2;

    Cypher::match_(pattern)
        // WHERE r1.id <> r2.id
        .where_(prop("r1", "id").ne(prop("r2", "id")))
        // WITH r1, r2, count(DISTINCT b) AS sharedCount
        .with((
            name("r1"),
            name("r2"),
            count_distinct(name("b")).alias("sharedCount"),
        ))
        // WHERE sharedCount > 0
        .where_(name("sharedCount").gt(lit(0_i64)))
        // WITH COALESCE(r1.name, r1.id, '') AS readerName, sum(sharedCount) AS totalShared
        .with((
            coalesce(vec![
                Expression::from(prop("r1", "name")),
                Expression::from(prop("r1", "id")),
                lit(""),
            ])
            .alias("readerName"),
            sum(name("sharedCount")).alias("totalShared"),
        ))
        // RETURN readerName, totalShared AS sharedCount
        .returning((
            name("readerName"),
            name("totalShared").alias("sharedCount"),
        ))
        // ORDER BY totalShared DESC
        .order_by(name("totalShared").descending())
        // LIMIT $limit
        .limit(param("limit"))
        .build()
}

fn main() {
    let stmt = build_shared_reading_query("Reader", "Book");
    let cypher = stmt.render();
    println!("Generated Cypher:\n{cypher}\n");

    match rust_cypher_dsl::parser::parse(&cypher) {
        Ok(parsed) => {
            let round_tripped = parsed.render();
            assert_eq!(cypher, round_tripped, "round-trip must be identical");
            println!("Round-trip OK");
        }
        Err(e) => {
            eprintln!("Parser failed: {e}");
            std::process::exit(1);
        }
    }
}
