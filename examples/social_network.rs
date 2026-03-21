//! Social network examples: friend-of-friend, recommendations, shortest paths.
//!
//! Run with: `cargo run --example social_network`

use rust_cypher_dsl::functions::aggregate;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::renderer::RenderConfig;

#[allow(clippy::too_many_lines, reason = "example showcasing many query patterns")]
fn main() {
    let pretty = RenderConfig {
        pretty_print: true,
        ..RenderConfig::default()
    };

    println!("=== Social Network — Cypher DSL Examples ===\n");

    // 1. Find direct friends
    // MATCH (a:Person {name: $name})-[:FRIENDS_WITH]->(b:Person)
    // RETURN b.name
    let a = node("Person")
        .named("a")
        .with_properties(props!("name" => param("name")));
    let b = node("Person").named("b");
    let stmt = Cypher::match_(a >> rel("FRIENDS_WITH") >> b)
        .returning(Expression::from(prop("b", "name")))
        .build();
    println!("1. Direct friends:\n{}\n", stmt.render_with(pretty.clone()));

    // 2. Friend-of-friend (2 hops)
    // MATCH (a:Person {name: $name})-[:FRIENDS_WITH*2]->(fof:Person)
    // WHERE fof <> a
    // RETURN DISTINCT fof.name
    let a = node("Person")
        .named("a")
        .with_properties(props!("name" => param("name")));
    let fof = node("Person").named("fof");
    let r = rel("FRIENDS_WITH").min(2).max(2);
    let stmt = Cypher::match_(a.rel(r).to(fof))
        .where_(name("fof").ne(name("a")))
        .returning_distinct(Expression::from(prop("fof", "name")))
        .build();
    println!(
        "2. Friend-of-friend:\n{}\n",
        stmt.render_with(pretty.clone())
    );

    // 3. Mutual friends
    // MATCH (a:Person {name: $name1})-[:FRIENDS_WITH]->(mutual:Person)<-[:FRIENDS_WITH]-(b:Person {name: $name2})
    // RETURN mutual.name
    let a = node("Person")
        .named("a")
        .with_properties(props!("name" => param("name1")));
    let mutual = node("Person").named("mutual");
    let b = node("Person")
        .named("b")
        .with_properties(props!("name" => param("name2")));
    let pattern = (a >> rel("FRIENDS_WITH") >> mutual) << rel("FRIENDS_WITH") << b;
    let stmt = Cypher::match_(pattern)
        .returning(Expression::from(prop("mutual", "name")))
        .build();
    println!(
        "3. Mutual friends:\n{}\n",
        stmt.render_with(pretty.clone())
    );

    // 4. Most connected people
    // MATCH (p:Person)-[:FRIENDS_WITH]-(other:Person)
    // RETURN p.name, count(other) AS connections
    // ORDER BY connections DESC
    // LIMIT 10
    let p = node("Person").named("p");
    let other = node("Person").named("other");
    let stmt = Cypher::match_(p.linked("FRIENDS_WITH", other))
        .returning((
            Expression::from(prop("p", "name")),
            aggregate::count(name("other")).alias("connections"),
        ))
        .order_by(name("connections").descending())
        .limit(10)
        .build();
    println!(
        "4. Most connected:\n{}\n",
        stmt.render_with(pretty.clone())
    );

    // 5. Recommend friends (friends-of-friends not yet connected)
    // MATCH (a:Person {name: $name})-[:FRIENDS_WITH]->(friend)-[:FRIENDS_WITH]->(suggestion)
    // WHERE suggestion <> a
    // RETURN suggestion.name, count(friend) AS mutualFriends
    // ORDER BY mutualFriends DESC
    let a = node("Person")
        .named("a")
        .with_properties(props!("name" => param("name")));
    let friend = node("Person").named("friend");
    let suggestion = node("Person").named("suggestion");
    let stmt = Cypher::match_(a >> rel("FRIENDS_WITH") >> friend >> rel("FRIENDS_WITH") >> suggestion)
        .where_(name("suggestion").ne(name("a")))
        .returning((
            Expression::from(prop("suggestion", "name")),
            aggregate::count(name("friend")).alias("mutualFriends"),
        ))
        .order_by(name("mutualFriends").descending())
        .build();
    println!(
        "5. Friend recommendations:\n{}\n",
        stmt.render_with(pretty.clone())
    );

    // 6. Create friendship
    // MATCH (a:Person {name: $name1}), (b:Person {name: $name2})
    // CREATE (a)-[:FRIENDS_WITH {since: $since}]->(b)
    // RETURN a.name, b.name
    let a = node("Person")
        .named("a")
        .with_properties(props!("name" => param("name1")));
    let b = node("Person")
        .named("b")
        .with_properties(props!("name" => param("name2")));
    let friendship = rel("FRIENDS_WITH").with_properties(props!("since" => param("since")));
    let a_ref = any_node_named("a");
    let b_ref = any_node_named("b");
    let stmt = Cypher::match_((a, b))
        .create(a_ref.rel(friendship).to(b_ref))
        .returning((
            Expression::from(prop("a", "name")),
            Expression::from(prop("b", "name")),
        ))
        .build();
    println!(
        "6. Create friendship:\n{}\n",
        stmt.render_with(pretty.clone())
    );

    // 7. Community detection via shared interests
    // MATCH (a:Person)-[:INTERESTED_IN]->(topic:Topic)<-[:INTERESTED_IN]-(b:Person)
    // WHERE a <> b
    // RETURN a.name, b.name, collect(topic.name) AS sharedInterests
    // ORDER BY size(sharedInterests) DESC
    let a = node("Person").named("a");
    let topic = node("Topic").named("topic");
    let b = node("Person").named("b");
    let pattern = (a >> rel("INTERESTED_IN") >> topic) << rel("INTERESTED_IN") << b;
    let stmt = Cypher::match_(pattern)
        .where_(name("a").ne(name("b")))
        .returning((
            Expression::from(prop("a", "name")),
            Expression::from(prop("b", "name")),
            aggregate::collect(prop("topic", "name")).alias("sharedInterests"),
        ))
        .build();
    println!(
        "7. Shared interests:\n{}\n",
        stmt.render_with(pretty)
    );

    println!("=== All 7 social network examples rendered successfully ===");
}
