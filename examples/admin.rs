//! Administration examples: indexes, constraints, and monitoring.
//!
//! Run with: `cargo run --example admin`

use rust_cypher_dsl::prelude::*;

fn main() {
    println!("=== Administration — Cypher DSL Examples ===\n");

    // ── Index Management ──

    // 1. Create a range index on a single property
    let stmt = Cypher::create_index("movie_title")
        .for_node("m", "Movie", vec!["title"])
        .build();
    println!("1. Create index:\n   {}\n", stmt.render());

    // 2. Create a composite index
    let stmt = Cypher::create_index("person_name_born")
        .for_node("p", "Person", vec!["name", "born"])
        .build();
    println!("2. Composite index:\n   {}\n", stmt.render());

    // 3. Create index if not exists
    let stmt = Cypher::create_index_if_not_exists("movie_released")
        .for_node("m", "Movie", vec!["released"])
        .build();
    println!("3. Index if not exists:\n   {}\n", stmt.render());

    // 4. Create a text index
    let stmt = Cypher::create_index("movie_tagline_text")
        .text()
        .for_node("m", "Movie", vec!["tagline"])
        .build();
    println!("4. Text index:\n   {}\n", stmt.render());

    // 5. Create a fulltext index (multi-label, multi-property)
    let stmt = Cypher::create_index("search_index")
        .fulltext()
        .for_node("n", "Movie", vec!["title", "tagline"])
        .build();
    println!("5. Fulltext index:\n   {}\n", stmt.render());

    // 6. Create a relationship index
    let stmt = Cypher::create_index("acted_roles")
        .for_relationship("r", "ACTED_IN", vec!["roles"])
        .build();
    println!("6. Relationship index:\n   {}\n", stmt.render());

    // 7. Drop an index
    let stmt = Cypher::drop_index("movie_title");
    println!("7. Drop index:\n   {}\n", stmt.render());

    // 8. Drop index if exists
    let stmt = Cypher::drop_index_if_exists("movie_title");
    println!("8. Drop if exists:\n   {}\n", stmt.render());

    // 9. Show all indexes
    let stmt = Cypher::show_indexes().build();
    println!("9. Show indexes:\n   {}\n", stmt.render());

    // 10. Show indexes with type filter and YIELD
    let stmt = Cypher::show_indexes()
        .filter(IndexFilter::Range)
        .yield_all()
        .build();
    println!("10. Show range indexes:\n    {}\n", stmt.render());

    // ── Constraint Management ──

    // 11. Uniqueness constraint
    let stmt = Cypher::create_constraint("unique_person_name")
        .for_node("p", "Person")
        .is_unique(vec!["name"]);
    println!("11. Unique constraint:\n    {}\n", stmt.render());

    // 12. Node key constraint (composite)
    let stmt = Cypher::create_constraint("person_key")
        .for_node("p", "Person")
        .is_node_key(vec!["name", "born"]);
    println!("12. Node key:\n    {}\n", stmt.render());

    // 13. Not-null constraint
    let stmt = Cypher::create_constraint("movie_title_exists")
        .for_node("m", "Movie")
        .is_not_null("title");
    println!("13. Not null:\n    {}\n", stmt.render());

    // 14. Constraint if not exists
    let stmt = Cypher::create_constraint_if_not_exists("unique_movie_title")
        .for_node("m", "Movie")
        .is_unique(vec!["title"]);
    println!("14. Constraint if not exists:\n    {}\n", stmt.render());

    // 15. Relationship property type constraint
    let stmt = Cypher::create_constraint("reviewed_rating_type")
        .for_relationship("r", "REVIEWED")
        .is_typed("rating", "INTEGER");
    println!("15. Rel type constraint:\n    {}\n", stmt.render());

    // 16. Drop a constraint
    let stmt = Cypher::drop_constraint("unique_person_name");
    println!("16. Drop constraint:\n    {}\n", stmt.render());

    // 17. Drop constraint if exists
    let stmt = Cypher::drop_constraint_if_exists("unique_person_name");
    println!("17. Drop if exists:\n    {}\n", stmt.render());

    // 18. Show all constraints
    let stmt = Cypher::show_constraints().build();
    println!("18. Show constraints:\n    {}\n", stmt.render());

    // ── Monitoring ──

    // 19. Show functions
    let stmt = Cypher::show_functions().build();
    println!("19. Show functions:\n    {}\n", stmt.render());

    // 20. Show procedures with YIELD
    let stmt = Cypher::show_procedures().yield_all().build();
    println!("20. Show procedures:\n    {}\n", stmt.render());

    // 21. Show transactions
    let stmt = Cypher::show_transactions().build();
    println!("21. Show transactions:\n    {}\n", stmt.render());

    // 22. Terminate a transaction
    let stmt = Cypher::terminate_transactions(vec!["neo4j-transaction-123"]).build();
    println!("22. Terminate transaction:\n    {}\n", stmt.render());

    println!("=== All 22 admin examples rendered successfully ===");
}
