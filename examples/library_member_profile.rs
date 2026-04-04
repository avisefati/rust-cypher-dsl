//! Library-member profile query with CALL {} subqueries, EXISTS {} predicates,
//! COALESCE, collect(DISTINCT), map literals, and list concatenation.
//!
//! Demonstrates: `exists()` shorthand, `coalesce()`, `collect_distinct()`,
//! map literals inside aggregation, list `+` concatenation, multiple
//! CALL {} subqueries with nested EXISTS {} filters.
//!
//! Run with: `cargo run --example library_member_profile`

use rust_cypher_dsl::functions::aggregate::collect_distinct;
use rust_cypher_dsl::functions::scalar::coalesce;
use rust_cypher_dsl::prelude::*;

/// Builds a library-member profile query with four CALL {} subqueries.
///
/// Given a member, retrieves their reading clubs, curator assignments,
/// borrowing privileges (direct + via curator role), all scoped to a
/// specific library branch and optionally filtered by a collection.
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (m:Member)
/// WHERE m.id = $member_id
/// CALL {
///   WITH m
///   OPTIONAL MATCH (m)-[:BELONGS_TO]->(c:ReadingClub)
///   WHERE c.branch_id = $branch_id
///     AND ($collection_id = '' OR EXISTS {
///       MATCH (c)-[:COVERS_COLLECTION]->(:Collection {branch_id: $branch_id, id: $collection_id})
///     })
///   RETURN collect(DISTINCT c.name) AS clubs
/// }
/// CALL {
///   WITH m
///   OPTIONAL MATCH (m)-[:CURATES]->(cur:Curator)
///   WHERE cur.branch_id = $branch_id
///     AND ($collection_id = '' OR EXISTS {
///       MATCH (app:Collection {branch_id: $branch_id, id: $collection_id})
///       WHERE COALESCE(cur.source_id, '') = COALESCE(app.source_id, '')
///     })
///   RETURN collect(DISTINCT {id: cur.id, name: cur.name}) AS curators
/// }
/// CALL {
///   WITH m
///   OPTIONAL MATCH (m)-[:HOLDS_PRIVILEGE]->(priv:BorrowPrivilege)
///   WHERE priv.branch_id = $branch_id
///     AND ($collection_id = '' OR EXISTS {
///       MATCH (app:Collection {branch_id: $branch_id, id: $collection_id})
///       WHERE COALESCE(priv.source_id, '') = COALESCE(app.source_id, '')
///     })
///   RETURN collect(DISTINCT {id: priv.id, name: priv.name}) AS privileges
/// }
/// CALL {
///   WITH m
///   OPTIONAL MATCH (m)-[:HOLDS_RESERVATION]->(directRes:Reservation)
///   WHERE directRes.branch_id = $branch_id
///     AND ($collection_id = '' OR EXISTS {
///       MATCH (app:Collection {branch_id: $branch_id, id: $collection_id})
///       WHERE COALESCE(directRes.source_id, '') = COALESCE(app.source_id, '')
///     })
///   WITH m, collect(DISTINCT directRes.name) AS directReservationNames
///   OPTIONAL MATCH (m)-[:CURATES]->(curForRes:Curator)-[:GRANTS_RESERVATION]->(curRes:Reservation)
///   WHERE curForRes.branch_id = $branch_id
///     AND curRes.branch_id = $branch_id
///     AND ($collection_id = '' OR EXISTS {
///       MATCH (app:Collection {branch_id: $branch_id, id: $collection_id})
///       WHERE COALESCE(curForRes.source_id, '') = COALESCE(app.source_id, '')
///         AND COALESCE(curRes.source_id, '') = COALESCE(app.source_id, '')
///     })
///   WITH directReservationNames,
///        collect(DISTINCT curRes.name) AS curatorReservationNames
///   WITH directReservationNames + curatorReservationNames AS reservationNames
///   RETURN reservationNames AS reservations
/// }
/// RETURN m.id AS id, m.name AS memberName,
///        clubs, curators, privileges, reservations
/// ```
#[expect(clippy::too_many_lines, reason = "example intentionally shows a complex real-world query")]
fn build_member_profile_query() -> Statement {
    let m = node("Member").named("m");

    // --- Helper: builds EXISTS { MATCH (c)-[:COVERS]->(:Collection {branch_id, id}) } ---
    // The EXISTS body is a bare MATCH (no RETURN needed in EXISTS subqueries).
    let simple_exists = |source: &str, rel_type: &str| -> Expression {
        let inner = Cypher::match_(
            any_node_named(source.to_owned())
                >> rel(rel_type.to_owned())
                >> node("Collection").with_properties(props! {
                    "branch_id" => param("branch_id"),
                    "id" => param("collection_id")
                }),
        )
        .returning(lit_true())
        .build();
        exists(&inner)
    };

    // --- Helper: builds EXISTS { MATCH (app:Collection {...}) WHERE COALESCE(...) } ---
    let coalesce_exists = |sources: Vec<&str>| -> Expression {
        let app = node("Collection")
            .named("app")
            .with_properties(props! {
                "branch_id" => param("branch_id"),
                "id" => param("collection_id")
            });
        let src0 = sources[0].to_owned();
        let mut q = Cypher::match_(app).where_(
            coalesce(vec![
                Expression::from(prop(src0, "source_id")),
                lit(""),
            ])
            .eq(coalesce(vec![
                Expression::from(prop("app", "source_id")),
                lit(""),
            ])),
        );
        for &src in &sources[1..] {
            q = q.and(
                coalesce(vec![
                    Expression::from(prop(src.to_owned(), "source_id")),
                    lit(""),
                ])
                .eq(coalesce(vec![
                    Expression::from(prop("app", "source_id")),
                    lit(""),
                ])),
            );
        }
        exists(&q.returning(lit_true()).build())
    };

    // --- Helper: $collection_id = '' OR EXISTS { ... } ---
    let collection_filter = |exists_expr: Expression| -> Condition {
        Expression::from(param("collection_id"))
            .eq(lit(""))
            .or(Condition::ExpressionCondition(exists_expr))
    };

    // --- CALL subquery 1: reading clubs ---
    let sub_clubs = Cypher::with(name("m"))
        .optional_match(
            any_node_named("m") >> rel("BELONGS_TO") >> node("ReadingClub").named("c"),
        )
        .where_(prop("c", "branch_id").eq(param("branch_id")))
        .and(collection_filter(simple_exists("c", "COVERS_COLLECTION")))
        .returning(collect_distinct(prop("c", "name")).alias("clubs"))
        .build();

    // --- CALL subquery 2: curator assignments ---
    let sub_curators = Cypher::with(name("m"))
        .optional_match(
            any_node_named("m") >> rel("CURATES") >> node("Curator").named("cur"),
        )
        .where_(prop("cur", "branch_id").eq(param("branch_id")))
        .and(collection_filter(coalesce_exists(vec!["cur"])))
        .returning(
            collect_distinct(map_of(vec![
                ("id".into(), Expression::from(prop("cur", "id"))),
                ("name".into(), Expression::from(prop("cur", "name"))),
            ]))
            .alias("curators"),
        )
        .build();

    // --- CALL subquery 3: borrowing privileges ---
    let sub_privileges = Cypher::with(name("m"))
        .optional_match(
            any_node_named("m")
                >> rel("HOLDS_PRIVILEGE")
                >> node("BorrowPrivilege").named("priv"),
        )
        .where_(prop("priv", "branch_id").eq(param("branch_id")))
        .and(collection_filter(coalesce_exists(vec!["priv"])))
        .returning(
            collect_distinct(map_of(vec![
                ("id".into(), Expression::from(prop("priv", "id"))),
                ("name".into(), Expression::from(prop("priv", "name"))),
            ]))
            .alias("privileges"),
        )
        .build();

    // --- CALL subquery 4: reservations (direct + via curator, then concatenated) ---
    let sub_reservations = Cypher::with(name("m"))
        .optional_match(
            any_node_named("m")
                >> rel("HOLDS_RESERVATION")
                >> node("Reservation").named("directRes"),
        )
        .where_(prop("directRes", "branch_id").eq(param("branch_id")))
        .and(collection_filter(coalesce_exists(vec!["directRes"])))
        .with((
            name("m"),
            collect_distinct(prop("directRes", "name")).alias("directReservationNames"),
        ))
        .optional_match(
            any_node_named("m")
                >> rel("CURATES")
                >> node("Curator").named("curForRes")
                >> rel("GRANTS_RESERVATION")
                >> node("Reservation").named("curRes"),
        )
        .where_(prop("curForRes", "branch_id").eq(param("branch_id")))
        .and(prop("curRes", "branch_id").eq(param("branch_id")))
        .and(collection_filter(coalesce_exists(vec![
            "curForRes", "curRes",
        ])))
        .with((
            name("directReservationNames"),
            collect_distinct(prop("curRes", "name")).alias("curatorReservationNames"),
        ))
        .with(
            name("directReservationNames")
                .add(name("curatorReservationNames"))
                .alias("reservationNames"),
        )
        .returning(name("reservationNames").alias("reservations"))
        .build();

    // --- Assemble the full query ---
    Cypher::match_(m)
        .where_(prop("m", "id").eq(param("member_id")))
        .call_subquery(sub_clubs)
        .call_subquery(sub_curators)
        .call_subquery(sub_privileges)
        .call_subquery(sub_reservations)
        .returning((
            prop("m", "id").alias("id"),
            prop("m", "name").alias("memberName"),
            name("clubs"),
            name("curators"),
            name("privileges"),
            name("reservations"),
        ))
        .build()
}

fn main() {
    let stmt = build_member_profile_query();
    let cypher = stmt.render();
    println!("Generated Cypher:\n{cypher}\n");

    // Verify the parser can at least parse the generated Cypher.
    // Note: full round-trip fidelity for parenthesized OR-inside-AND
    // (`AND (x OR y)`) is a known parser limitation — the parser
    // currently drops the grouping parentheses.
    match rust_cypher_dsl::parser::parse(&cypher) {
        Ok(_) => println!("Parser accepted the generated Cypher."),
        Err(e) => {
            eprintln!("Parser failed: {e}");
            std::process::exit(1);
        }
    }
}
