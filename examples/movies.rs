//! End-to-end examples using the Neo4j Movies graph.
//!
//! Run with: `cargo run --example movies`

use rust_cypher_dsl::functions::aggregate;
use rust_cypher_dsl::prelude::*;

#[allow(clippy::too_many_lines, reason = "example showcasing many query patterns")]
fn main() {
    println!("=== Neo4j Movies Graph — Cypher DSL Examples ===\n");

    // ── Reading Data ──

    // 1. Find all movies
    let stmt = Cypher::match_(node("Movie").named("m"))
        .returning(name("m"))
        .build();
    println!("1. Find all movies:\n   {}\n", stmt.render());

    // 2. Find a movie by title
    let stmt = Cypher::match_(
        node("Movie")
            .named("m")
            .with_properties(props!("title" => "The Matrix")),
    )
    .returning(name("m"))
    .build();
    println!("2. Find movie by title:\n   {}\n", stmt.render());

    // 3. Find actors in a movie
    let p = node("Person").named("p");
    let m = node("Movie")
        .named("m")
        .with_properties(props!("title" => "The Matrix"));
    let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
        .returning(Expression::from(prop("p", "name")))
        .build();
    println!("3. Find actors in The Matrix:\n   {}\n", stmt.render());

    // 4. Find director (shorthand syntax)
    let p = node("Person").named("p");
    let m = node("Movie")
        .named("m")
        .with_properties(props!("title" => "The Matrix"));
    let stmt = Cypher::match_(p.to("DIRECTED", m))
        .returning(Expression::from(prop("p", "name")))
        .build();
    println!("4. Find director:\n   {}\n", stmt.render());

    // 5. Movies released after 2000
    let stmt = Cypher::match_(node("Movie").named("m"))
        .where_(prop("m", "released").gt(2000_i32))
        .returning((
            Expression::from(prop("m", "title")),
            Expression::from(prop("m", "released")),
        ))
        .build();
    println!("5. Movies after 2000:\n   {}\n", stmt.render());

    // 6. Co-actors (multi-hop)
    let p = node("Person").named("p");
    let m = node("Movie").named("m");
    let co = node("Person").named("coActor");
    let pattern = (p >> rel("ACTED_IN") >> m) << rel("ACTED_IN") << co;
    let stmt = Cypher::match_(pattern)
        .where_(prop("p", "name").eq("Tom Hanks"))
        .returning((
            Expression::from(prop("coActor", "name")),
            Expression::from(prop("m", "title")),
        ))
        .build();
    println!("6. Co-actors of Tom Hanks:\n   {}\n", stmt.render());

    // 7. Count movies per actor
    let p = node("Person").named("p");
    let m = node("Movie").named("m");
    let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
        .returning((
            Expression::from(prop("p", "name")),
            aggregate::count(name("m")).alias("movieCount"),
        ))
        .order_by(name("movieCount").descending())
        .build();
    println!("7. Movies per actor:\n   {}\n", stmt.render());

    // 8. Collect actors per movie
    let p = node("Person").named("p");
    let m = node("Movie").named("m");
    let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
        .returning((
            Expression::from(prop("m", "title")),
            aggregate::collect(prop("p", "name")).alias("cast"),
        ))
        .build();
    println!("8. Cast per movie:\n   {}\n", stmt.render());

    // 9. Parameterized query
    let p = node("Person")
        .named("p")
        .with_properties(props!("name" => param("name")));
    let m = node("Movie").named("m");
    let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
        .returning(Expression::from(prop("m", "title")))
        .build();
    println!("9. Parameterized:\n   {}\n", stmt.render());

    // ── Writing Data ──

    // 10. Create a movie
    let stmt = Cypher::create(
        node("Movie")
            .named("m")
            .with_properties(props!("title" => "New Movie", "released" => 2024_i32)),
    )
    .returning(name("m"))
    .build();
    println!("10. Create movie:\n    {}\n", stmt.render());

    // 11. Create relationship between existing nodes
    let p = node("Person")
        .named("p")
        .with_properties(props!("name" => "Tom Hanks"));
    let m = node("Movie")
        .named("m")
        .with_properties(props!("title" => "New Movie"));
    let acted = rel("ACTED_IN").with_properties(props!("roles" => list_of(vec![lit("Hero")])));
    let p_ref = any_node_named("p");
    let m_ref = any_node_named("m");
    let stmt = Cypher::match_((p, m))
        .create(p_ref.rel(acted).to(m_ref))
        .returning((name("p"), name("m")))
        .build();
    println!("11. Create relationship:\n    {}\n", stmt.render());

    // 12. Merge with ON CREATE SET
    let p = node("Person")
        .named("p")
        .with_properties(props!("name" => "Tom Hanks"));
    let stmt = Cypher::merge(p)
        .on_create(vec![SetItem::property(prop("p", "born"), 1956_i32)])
        .returning(name("p"))
        .build();
    println!("12. Merge:\n    {}\n", stmt.render());

    // 13. Update with SET
    let m = node("Movie")
        .named("m")
        .with_properties(props!("title" => "The Matrix"));
    let stmt = Cypher::match_(m)
        .set(vec![SetItem::property(
            prop("m", "tagline"),
            "Welcome to the Real World",
        )])
        .returning(name("m"))
        .build();
    println!("13. Update SET:\n    {}\n", stmt.render());

    // 14. Delete node
    let m = node("Movie")
        .named("m")
        .with_properties(props!("title" => "New Movie"));
    let stmt = Cypher::match_(m).detach_delete(name("m")).build();
    println!("14. Detach delete:\n    {}\n", stmt.render());

    // ── Multi-Part Queries ──

    // 15. WITH pipeline: prolific actors
    let p = node("Person").named("p");
    let m = node("Movie").named("m");
    let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
        .with((
            name("p"),
            aggregate::count(name("m")).alias("movieCount"),
        ))
        .where_(name("movieCount").gt(5_i32))
        .returning((Expression::from(prop("p", "name")), name("movieCount")))
        .order_by(name("movieCount").descending())
        .build();
    println!("15. Prolific actors:\n    {}\n", stmt.render());

    println!("=== All 15 examples rendered successfully ===");
}
