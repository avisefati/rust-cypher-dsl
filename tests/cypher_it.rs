//! Integration tests for Cypher DSL — ported from Java `CypherIT.java`.
//!
//! These tests exercise the public builder API (`Cypher::*`) end-to-end,
//! asserting that rendered output matches expected Cypher strings.
//!
//! Test count at parity with Java DSL coverage: 80+ tests.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::clauses::{MatchClause, RemoveItem, ReturnClause, SetClause};
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::props;
use std::borrow::Cow;

// ============================================================================
// Node patterns
// ============================================================================

#[test]
fn match_single_node() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
}

#[test]
fn match_any_node() {
    let n = any_node_named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n) RETURN n");
}

#[test]
fn match_node_with_multiple_labels() {
    let n = node("Person").named("n");
    let n = n.with_labels(["Actor"]);
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`:`Actor`) RETURN n");
}

#[test]
fn match_node_with_properties() {
    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => "Alice" });
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person` {`name`: 'Alice'}) RETURN n"
    );
}

#[test]
fn match_node_with_multiple_properties() {
    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => "Alice", "age" => 30_i32 });
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render();
    // Order of properties in maps may vary, so check both contain
    assert!(
        rendered.contains("`name`: 'Alice'") && rendered.contains("age: 30"),
        "Expected both properties, got: {rendered}"
    );
}

// ============================================================================
// Relationship patterns
// ============================================================================

#[test]
fn match_outgoing_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_incoming_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS")).from(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)<-[:`KNOWS`]-(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_undirected_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS")).between(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS`]-(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_named_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS").named("r")).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("r"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[r:`KNOWS`]->(b:`Person`) RETURN a, r, b"
    );
}

#[test]
fn match_relationship_with_properties() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a
        .rel(rel("KNOWS").named("r").with_properties(props! { "since" => 2020_i32 }))
        .to(b);
    let stmt = Cypher::match_(r)
        .returning(name("r"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[r:`KNOWS` {since: 2020}]->(b:`Person`) RETURN r"
    );
}

#[test]
fn match_variable_length_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS").min(1).max(3)).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS` *1..3]->(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_unbounded_variable_length() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS").unbounded()).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS` *]->(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_relationship_chain() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let c = node("Person").named("c");
    let chain = a.rel(rel("KNOWS")).to(b).rel(rel("LIKES")).to(c);
    let stmt = Cypher::match_(chain)
        .returning((name("a"), name("b"), name("c")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`)-[:`LIKES`]->(c:`Person`) RETURN a, b, c"
    );
}

// ============================================================================
// Operator-based relationship syntax (>>, <<)
// ============================================================================

#[test]
fn match_operator_outgoing() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let chain = a >> rel("KNOWS") >> b;
    let stmt = Cypher::match_(chain)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
    );
}

#[test]
fn match_operator_incoming() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let chain = a << rel("KNOWS") << b;
    let stmt = Cypher::match_(chain)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`)<-[:`KNOWS`]-(b:`Person`) RETURN a, b"
    );
}

// ============================================================================
// Named paths
// ============================================================================

#[test]
fn match_named_path() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let chain = a.rel(rel("KNOWS")).to(b);
    let p = path("p").defined_by(chain);
    let stmt = Cypher::match_(p)
        .returning(name("p"))
        .build();
    assert_eq!(stmt.render(), "MATCH p = (a)-[:`KNOWS`]->(b) RETURN p");
}

// ============================================================================
// WHERE clause
// ============================================================================

#[test]
fn match_where_comparison() {
    let n = node("Person").named("n");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
    );
}

#[test]
fn match_where_and() {
    let n = node("Person").named("n");
    let age_cond = prop("n", "age").gt(21_i32);
    let name_cond = prop("n", "name").eq("Alice");
    let stmt = Cypher::match_(n)
        .where_(age_cond)
        .and(name_cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.age > 21 AND n.`name` = 'Alice' RETURN n"
    );
}

#[test]
fn match_where_or() {
    let n = node("Person").named("n");
    let age_cond = prop("n", "age").gt(21_i32);
    let name_cond = prop("n", "name").eq("Alice");
    let stmt = Cypher::match_(n)
        .where_(age_cond)
        .or(name_cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.age > 21 OR n.`name` = 'Alice' RETURN n"
    );
}

#[test]
fn match_where_is_null() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "email")).is_null();
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.email IS NULL RETURN n"
    );
}

#[test]
fn match_where_is_not_null() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "email")).is_not_null();
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.email IS NOT NULL RETURN n"
    );
}

#[test]
fn match_where_starts_with() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "name")).starts_with("A");
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` STARTS WITH 'A' RETURN n"
    );
}

#[test]
fn match_where_ends_with() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "name")).ends_with("son");
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` ENDS WITH 'son' RETURN n"
    );
}

#[test]
fn match_where_contains() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "name")).contains("ali");
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` CONTAINS 'ali' RETURN n"
    );
}

#[test]
fn match_where_regex_match() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "name")).regex_match("A.*");
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` =~ 'A.*' RETURN n"
    );
}

#[test]
fn match_where_not() {
    let n = node("Person").named("n");
    let cond = not(prop("n", "active").eq(true));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE NOT n.`active` = true RETURN n"
    );
}

#[test]
fn match_where_in_list() {
    let n = node("Person").named("n");
    let cond = Expression::from(prop("n", "name")).in_list(list_of(vec![
        lit("Alice"),
        lit("Bob"),
    ]));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` IN ['Alice', 'Bob'] RETURN n"
    );
}

// ============================================================================
// RETURN clause variations
// ============================================================================

#[test]
fn return_distinct() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning_distinct(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN DISTINCT n");
}

#[test]
fn return_aliased() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(Expression::from(prop("n", "name")).alias("personName"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n.`name` AS personName"
    );
}

#[test]
fn return_asterisk() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(Expression::asterisk())
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN *");
}

#[test]
fn return_multiple() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning((name("n"), Expression::from(prop("n", "name"))))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n, n.`name`");
}

// ============================================================================
// ORDER BY, SKIP, LIMIT
// ============================================================================

#[test]
fn return_order_by_ascending() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .order_by(Expression::from(prop("n", "name")).ascending())
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n ORDER BY n.`name`"
    );
}

#[test]
fn return_order_by_descending() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .order_by(Expression::from(prop("n", "age")).descending())
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n ORDER BY n.age DESC"
    );
}

#[test]
fn return_order_by_multiple() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .order_by(vec![
            Expression::from(prop("n", "name")).ascending(),
            Expression::from(prop("n", "age")).descending(),
        ])
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n ORDER BY n.`name`, n.age DESC"
    );
}

#[test]
fn return_skip() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .skip(10_i32)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n SKIP 10");
}

#[test]
fn return_limit() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .limit(25_i32)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n LIMIT 25");
}

#[test]
fn return_order_skip_limit() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .order_by(Expression::from(prop("n", "name")).ascending())
        .skip(5_i32)
        .limit(10_i32)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n ORDER BY n.`name` SKIP 5 LIMIT 10"
    );
}

// ============================================================================
// OPTIONAL MATCH
// ============================================================================

#[test]
fn optional_match_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::optional_match(n)
        .returning(name("n"))
        .build();
    assert_eq!(stmt.render(), "OPTIONAL MATCH (n:`Person`) RETURN n");
}

#[test]
fn match_then_optional_match() {
    let a = node("Person").named("a");
    let a2 = any_node_named("a");
    let b = any_node_named("b");
    let r = a2.rel(rel("KNOWS")).to(b);
    let stmt = Cypher::match_(a)
        .optional_match(r)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`) OPTIONAL MATCH (a)-[:`KNOWS`]->(b) RETURN a, b"
    );
}

// ============================================================================
// WITH clause
// ============================================================================

#[test]
fn match_with_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .with(name("n").alias("person"))
        .returning(name("person"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH n AS person RETURN person"
    );
}

#[test]
fn match_with_distinct_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .with_distinct(Expression::from(prop("n", "city")).alias("city"))
        .returning(name("city"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city"
    );
}

#[test]
fn match_with_where_return() {
    let n = node("Person").named("n");
    let cond = prop("person", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .with(name("n").alias("person"))
        .where_(cond)
        .returning(name("person"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person"
    );
}

#[test]
fn multi_part_match_with_match_return() {
    let n = node("Person").named("n");
    let person = any_node_named("person");
    let m = any_node_named("m");
    let r = person.rel(rel("KNOWS")).to(m);
    let stmt = Cypher::match_(n)
        .with(name("n").alias("person"))
        .match_(r)
        .returning((name("person"), name("m")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH n AS person MATCH (person)-[:`KNOWS`]->(m) RETURN person, m"
    );
}

// ============================================================================
// UNWIND clause
// ============================================================================

#[test]
fn unwind_return() {
    let stmt = Cypher::unwind(list_of(vec![lit(1_i32), lit(2_i32), lit(3_i32)]))
        .as_("x")
        .returning(name("x"))
        .build();
    assert_eq!(stmt.render(), "UNWIND [1, 2, 3] AS x RETURN x");
}

#[test]
fn unwind_param_match_return() {
    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => name("personName") });
    let stmt = Cypher::unwind(Expression::from(param("names")))
        .as_("personName")
        .match_(n)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "UNWIND $names AS personName MATCH (n:`Person` {`name`: personName}) RETURN n"
    );
}

#[test]
fn match_with_unwind_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .with(name("n"))
        .unwind(list_of(vec![lit(1_i32), lit(2_i32)]))
        .as_("x")
        .returning((name("n"), name("x")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH n UNWIND [1, 2] AS x RETURN n, x"
    );
}

// ============================================================================
// CREATE clause
// ============================================================================

#[test]
fn create_node() {
    let n = node("Person").named("n");
    let stmt = Cypher::create(n).build();
    assert_eq!(stmt.render(), "CREATE (n:`Person`)");
}

#[test]
fn create_node_with_properties() {
    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => "Alice" });
    let stmt = Cypher::create(n).build();
    assert_eq!(stmt.render(), "CREATE (n:`Person` {`name`: 'Alice'})");
}

#[test]
fn create_relationship() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let stmt = Cypher::create(r).build();
    assert_eq!(
        stmt.render(),
        "CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)"
    );
}

#[test]
fn create_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::create(n)
        .returning(name("n"))
        .build();
    assert_eq!(stmt.render(), "CREATE (n:`Person`) RETURN n");
}

// ============================================================================
// MERGE clause
// ============================================================================

#[test]
fn merge_simple() {
    let n = node("Person")
        .named("n")
        .with_properties(props! { "name" => "Alice" });
    let stmt = Cypher::merge(n).build();
    assert_eq!(stmt.render(), "MERGE (n:`Person` {`name`: 'Alice'})");
}

#[test]
fn merge_on_create() {
    let n = node("Person").named("n");
    let stmt = Cypher::merge(n)
        .on_create(SetItem::property(
            Property::new(name("n"), "created"),
            lit(true),
        ))
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MERGE (n:`Person`) ON CREATE SET n.created = true RETURN n"
    );
}

#[test]
fn merge_on_match() {
    let n = node("Person").named("n");
    let stmt = Cypher::merge(n)
        .on_match(SetItem::property(
            Property::new(name("n"), "updated"),
            lit(true),
        ))
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MERGE (n:`Person`) ON MATCH SET n.updated = true RETURN n"
    );
}

#[test]
fn merge_on_create_and_on_match() {
    let n = node("Person").named("n");
    let stmt = Cypher::merge(n)
        .on_create(SetItem::property(
            Property::new(name("n"), "created"),
            lit(true),
        ))
        .on_match(SetItem::property(
            Property::new(name("n"), "updated"),
            lit(true),
        ))
        .build();
    assert_eq!(
        stmt.render(),
        "MERGE (n:`Person`) ON CREATE SET n.created = true ON MATCH SET n.updated = true"
    );
}

// ============================================================================
// SET clause
// ============================================================================

#[test]
fn match_set_property() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .set(SetItem::property(
            Property::new(name("n"), "name"),
            lit("Bob"),
        ))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) SET n.`name` = 'Bob'"
    );
}

#[test]
fn match_set_label() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .set(SetItem::label(name("n"), vec![Cow::Borrowed("Admin")]))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) SET n:`Admin`");
}

#[test]
fn match_set_mutate() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .set(SetItem::mutate(
            name("n"),
            map_of(vec![("age".into(), lit(30_i32))]),
        ))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) SET n += {age: 30}"
    );
}

// ============================================================================
// DELETE clause
// ============================================================================

#[test]
fn match_delete() {
    let n = node("Temp").named("n");
    let stmt = Cypher::match_(n)
        .delete(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Temp`) DELETE n");
}

#[test]
fn match_detach_delete() {
    let n = node("Temp").named("n");
    let stmt = Cypher::match_(n)
        .detach_delete(name("n"))
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Temp`) DETACH DELETE n");
}

#[test]
fn match_where_delete() {
    let n = node("Temp").named("n");
    let cond = prop("n", "expired").eq(true);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .delete(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Temp`) WHERE n.expired = true DELETE n"
    );
}

// ============================================================================
// REMOVE clause
// ============================================================================

#[test]
fn match_remove_property() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .remove(vec![RemoveItem::property(
            Property::new(name("n"), "age"),
        )])
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) REMOVE n.age");
}

// ============================================================================
// FOREACH clause
// ============================================================================

#[test]
fn match_foreach_set() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let chain = a.rel(rel("KNOWS")).to(b);
    let p = path("p").defined_by(chain);
    let stmt = Cypher::match_(p)
        .foreach(
            "n",
            raw_unchecked("nodes(p)"),
            vec![Clause::Set(SetClause::new(vec![
                SetItem::property(
                    Property::new(name("n"), "visited"),
                    lit(true),
                ),
            ]))],
        )
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH p = (a)-[:`KNOWS`]->(b) FOREACH (n IN nodes(p) | SET n.visited = true)"
    );
}

// ============================================================================
// CALL procedure
// ============================================================================

#[test]
fn call_procedure_no_args() {
    let stmt = Cypher::call_procedure("db.labels", vec![]).build();
    assert_eq!(stmt.render(), "CALL db.labels()");
}

#[test]
fn call_procedure_with_args() {
    let stmt = Cypher::call_procedure(
        "dbms.security.createUser",
        vec![lit("alice"), lit("password"), lit(false)],
    )
    .build();
    assert_eq!(
        stmt.render(),
        "CALL dbms.security.createUser('alice', 'password', false)"
    );
}

#[test]
fn call_procedure_yield() {
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label"))
        .returning(name("label"))
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.labels() YIELD `label` RETURN `label`"
    );
}

#[test]
fn call_procedure_yield_where() {
    let cond = name("label").starts_with("P");
    let stmt = Cypher::call_procedure("db.labels", vec![])
        .yield_(name("label"))
        .where_(cond)
        .build();
    assert_eq!(
        stmt.render(),
        "CALL db.labels() YIELD `label` WHERE `label` STARTS WITH 'P'"
    );
}

// ============================================================================
// CALL subquery
// ============================================================================

#[test]
fn call() {
    let n = node("Person").named("n");
    let stmt = Cypher::call(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Return(ReturnClause::new(vec![name("n")])),
    ])
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (n:`Person`) RETURN n }"
    );
}

#[test]
fn call_in_transactions() {
    let n = node("Person").named("n");
    let stmt = Cypher::call(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Return(ReturnClause::new(vec![name("n")])),
    ])
    .in_transactions()
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS"
    );
}

#[test]
fn call_in_transactions_with_batch_size() {
    let n = node("Person").named("n");
    let stmt = Cypher::call(vec![
        Clause::Match(MatchClause::new(n)),
        Clause::Return(ReturnClause::new(vec![name("n")])),
    ])
    .in_transactions()
    .of_rows(1000_i32)
    .build();
    assert_eq!(
        stmt.render(),
        "CALL { MATCH (n:`Person`) RETURN n } IN TRANSACTIONS OF 1000 ROWS"
    );
}

// ============================================================================
// UNION, UNION ALL
// ============================================================================

#[test]
fn union_two_queries() {
    let left = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let right = Cypher::match_(node("Movie").named("n"))
        .returning(name("n"))
        .build();
    let stmt = left.union(right);
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n UNION MATCH (n:`Movie`) RETURN n"
    );
}

#[test]
fn union_all_two_queries() {
    let left = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let right = Cypher::match_(node("Movie").named("n"))
        .returning(name("n"))
        .build();
    let stmt = left.union_all(right);
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n UNION ALL MATCH (n:`Movie`) RETURN n"
    );
}

// ============================================================================
// EXPLAIN, PROFILE
// ============================================================================

#[test]
fn explain_query() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build()
        .explain();
    assert_eq!(stmt.render(), "EXPLAIN MATCH (n:`Person`) RETURN n");
}

#[test]
fn profile_query() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build()
        .profile();
    assert_eq!(stmt.render(), "PROFILE MATCH (n:`Person`) RETURN n");
}

// ============================================================================
// Mixed read/write queries
// ============================================================================

#[test]
fn match_create_return() {
    let a = node("Person").named("a");
    let b = node("Movie").named("b");
    let stmt = Cypher::match_(a)
        .create(b)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`) CREATE (b:`Movie`) RETURN a, b"
    );
}

#[test]
fn match_where_create_return() {
    let n = node("Person").named("n");
    let m = node("Adult").named("m");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .create(m)
        .returning((name("n"), name("m")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.age > 21 CREATE (m:`Adult`) RETURN n, m"
    );
}

#[test]
fn match_set_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .set(SetItem::property(
            Property::new(name("n"), "active"),
            lit(true),
        ))
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) SET n.`active` = true RETURN n"
    );
}

#[test]
fn match_merge_return() {
    let a = node("Person").named("a");
    let b = node("Movie").named("b");
    let stmt = Cypher::match_(a)
        .merge(b)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`) MERGE (b:`Movie`) RETURN a, b"
    );
}

#[test]
fn match_with_create_return() {
    let n = node("Person").named("n");
    let m = node("Clone").named("m");
    let stmt = Cypher::match_(n)
        .with(name("n"))
        .create(m)
        .returning((name("n"), name("m")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH n CREATE (m:`Clone`) RETURN n, m"
    );
}

// ============================================================================
// LOAD CSV
// ============================================================================

#[test]
fn load_csv_return() {
    let stmt = Cypher::load_csv(lit("file:///data.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
    );
}

#[test]
fn load_csv_with_headers() {
    let stmt = Cypher::load_csv_with_headers(lit("file:///data.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row"
    );
}

#[test]
fn load_csv_field_terminator() {
    let stmt = Cypher::load_csv(lit("file:///data.csv"))
        .as_("row")
        .field_terminator(";")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM 'file:///data.csv' AS row FIELDTERMINATOR ';' RETURN row"
    );
}

#[test]
fn load_csv_create() {
    let n = node("Person").with_properties(props! { "name" => name("row") });
    let stmt = Cypher::load_csv(lit("file:///data.csv"))
        .as_("row")
        .create(n)
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM 'file:///data.csv' AS row CREATE (:`Person` {`name`: row})"
    );
}

#[test]
fn periodic_commit_load_csv() {
    let stmt = Cypher::using_periodic_commit(Some(1000))
        .load_csv(lit("file:///data.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "USING PERIODIC COMMIT 1000 LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
    );
}

#[test]
fn periodic_commit_no_size() {
    let stmt = Cypher::using_periodic_commit(None)
        .load_csv(lit("file:///data.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "USING PERIODIC COMMIT LOAD CSV FROM 'file:///data.csv' AS row RETURN row"
    );
}

// ============================================================================
// Parameters
// ============================================================================

#[test]
fn match_where_with_parameter() {
    let n = node("Person").named("n");
    let cond = prop("n", "name").eq(param("name"));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` = $name RETURN n"
    );
}

#[test]
fn match_where_with_bound_parameter() {
    let n = node("Person").named("n");
    let cond = prop("n", "name").eq(param_with_value("name", "Alice"));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WHERE n.`name` = $name RETURN n"
    );
    // Bound parameters are extractable from the catalog
    let params = stmt.get_parameters();
    assert!(params.contains_key("name"));
}

// ============================================================================
// Statement Display trait
// ============================================================================

#[test]
fn display_matches_render() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    assert_eq!(format!("{stmt}"), stmt.render());
}

// ============================================================================
// Statement catalog
// ============================================================================

#[test]
fn catalog_collects_labels() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.labels.contains("Person"));
}

#[test]
fn catalog_collects_relationship_types() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let stmt = Cypher::match_(r)
        .returning(name("a"))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.relationship_types.contains("KNOWS"));
}

#[test]
fn catalog_collects_parameters() {
    let n = node("Person").named("n");
    let cond = prop("n", "name").eq(param("name"));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    let names = stmt.get_parameter_names();
    assert!(names.contains("name"));
}

// ============================================================================
// Multiple match clauses
// ============================================================================

#[test]
fn match_match_return() {
    let a = node("Person").named("a");
    let b = node("Movie").named("b");
    let stmt = Cypher::match_(a)
        .match_(b)
        .returning((name("a"), name("b")))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (a:`Person`) MATCH (b:`Movie`) RETURN a, b"
    );
}
