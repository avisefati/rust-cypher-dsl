//! Fraud-ring detection query using Quantified Path Patterns (QPP).
//!
//! Adapted from the Neo4j fraud-detection demo:
//! <https://neo4j.com/developer/demos/fraud-demo/#_fraud_rings>
//!
//! Demonstrates: quantified path patterns with `{2,15}` range quantifier,
//! inline WHERE predicates inside QPP, path concatenation (`concat_path`),
//! named paths (`path = ...`), `COUNT {}` subqueries, `size()`,
//! list concatenation (`+`), and `RETURN DISTINCT`.
//!
//! Run with: `cargo run --example fraud_ring_detection`

use rust_cypher_dsl::functions::scalar::size;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::types::pattern::{concat_path, quantified_path};

/// Builds a fraud-ring detection query.
///
/// The query identifies cyclic money-transfer patterns where:
/// 1. The ring starts and ends at the same account.
/// 2. Transactions occur sequentially in time.
/// 3. All intermediate accounts are unique.
/// 4. Each account retains at most 20 % of the transferred amount.
/// 5. The ring contains between 3 and 16 accounts (2–15 intermediate hops).
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (a:Account)-[f:SENT]->(first_tx:Transaction)
/// MATCH path = (a)-[f]->(first_tx)
///   (
///     (tx_i:Transaction)-[:RECEIVED]->(a_i:Account)-[:SENT]->(tx_j:Transaction)
///     WHERE tx_i.date < tx_j.date
///       AND tx_i.amount >= tx_j.amount
///       AND tx_j.amount >= 0.80 * tx_i.amount
///   ){2,15}
///   (last_tx:Transaction)-[:RECEIVED]->(a)
/// WHERE COUNT { WITH a, a_i UNWIND [a] + a_i AS b RETURN DISTINCT b }
///     = size([a] + a_i)
/// RETURN
///   COUNT { WITH a, a_i UNWIND [a] + a_i AS b RETURN DISTINCT b } AS ringSize,
///   a.accountNumber AS EntryAccount,
///   path AS ring
/// ```
fn build_fraud_ring_query() -> Statement {
    // ── MATCH 1: anchor the starting account and its first transaction ──
    let a = node("Account").named("a");
    let first_tx = node("Transaction").named("first_tx");
    let f = rel("SENT").named("f");

    // ── QPP inner pattern ──────────────────────────────────────────────
    //
    //   (tx_i:Transaction)-[:RECEIVED]->(a_i:Account)-[:SENT]->(tx_j:Transaction)
    //   WHERE tx_i.date < tx_j.date
    //     AND tx_i.amount >= tx_j.amount
    //     AND tx_j.amount >= 0.80 * tx_i.amount
    //
    // Cypher supports chained comparisons like `a >= b >= c`, but the DSL
    // represents them as explicit AND-connected conditions.
    let inner_chain = node("Transaction").named("tx_i")
        >> rel("RECEIVED")
        >> node("Account").named("a_i")
        >> rel("SENT")
        >> node("Transaction").named("tx_j");

    let qpp_where = prop("tx_i", "date")
        .lt(prop("tx_j", "date"))
        .and(prop("tx_i", "amount").gte(prop("tx_j", "amount")))
        .and(
            prop("tx_j", "amount")
                .gte(lit(0.80_f64).multiply(prop("tx_i", "amount"))),
        );

    let qpp = quantified_path(inner_chain)
        .where_(Expression::from(qpp_where))
        .range(Some(2), Some(15));

    // ── MATCH 2: full ring path with QPP ───────────────────────────────
    //
    // path = (a)-[f]->(first_tx) ((qpp)){2,15} (last_tx)-[:RECEIVED]->(a)
    //
    // Uses `concat_path` to juxtapose the leading chain, QPP, and
    // trailing chain into a single path expression.
    let leading = any_node_named("a") >> rel("SENT").named("f") >> any_node_named("first_tx");

    let trailing = node("Transaction").named("last_tx")
        >> rel("RECEIVED")
        >> any_node_named("a");

    let full_path = path("path").defined_by(concat_path(vec![
        leading.into(),
        qpp.into(),
        trailing.into(),
    ]));

    // ── COUNT subquery (used in both WHERE and RETURN) ─────────────────
    //
    // COUNT { WITH a, a_i UNWIND [a] + a_i AS b RETURN DISTINCT b }
    //
    // Counts the distinct accounts participating in the ring.
    let count_sub = Cypher::with((name("a"), name("a_i")))
        .unwind(list_of(vec![name("a")]).add(name("a_i")))
        .as_("b")
        .returning_distinct(name("b"))
        .build();
    let count_expr =
        Expression::count_subquery(Expression::raw_unchecked(count_sub.render()));

    // ── Uniqueness predicate ───────────────────────────────────────────
    //
    // WHERE COUNT { ... } = size([a] + a_i)
    let unique_check =
        count_expr.clone().eq(size(list_of(vec![name("a")]).add(name("a_i"))));

    // ── Assemble the full query ────────────────────────────────────────
    Cypher::match_(a >> f >> first_tx)
        .match_(full_path)
        .where_(unique_check)
        .returning((
            count_expr.alias("ringSize"),
            prop("a", "accountNumber").alias("EntryAccount"),
            name("path").alias("ring"),
        ))
        .build()
}

fn main() {
    let stmt = build_fraud_ring_query();
    let cypher = stmt.render();
    println!("Generated Cypher:\n{cypher}\n");

    // Verify the parser can handle the generated Cypher.
    match rust_cypher_dsl::parser::parse(&cypher) {
        Ok(_) => println!("Parser accepted the generated Cypher."),
        Err(e) => {
            eprintln!("Parser rejected the output: {e}");
            std::process::exit(1);
        }
    }
}
