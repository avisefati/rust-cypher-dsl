//! Integration tests for QPP (Quantified Path Patterns), quantified
//! relationships, and path selectors.

use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::types::pattern::{
    all_shortest, any_path, quantified_path, shortest, shortest_groups, Quantifier,
};

// ============================================================================
// Quantified Path Patterns
// ============================================================================

#[test]
fn quantified_path_star() {
    // MATCH (a)-[:`KNOWS`]->(b) (()-[:`FOLLOWS`]->())* RETURN a
    let a = any_node_named("a");
    let b = any_node_named("b");
    let _r = a.rel(rel("KNOWS")).to(b);
    let inner_rel = any_node().rel(rel("FOLLOWS")).to(any_node());
    let qp = quantified_path(inner_rel).star();
    let stmt = Cypher::match_(qp)
        .returning(Expression::asterisk())
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("FOLLOWS"));
    assert!(rendered.contains('*'));
}

#[test]
fn quantified_path_plus() {
    let inner_rel = any_node().rel(rel("FOLLOWS")).to(any_node());
    let qp = quantified_path(inner_rel).plus();
    let stmt = Cypher::match_(qp)
        .returning(Expression::asterisk())
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains('+'));
}

#[test]
fn quantified_path_exact() {
    let inner_rel = any_node().rel(rel("KNOWS")).to(any_node());
    let qp = quantified_path(inner_rel).exact(3);
    let stmt = Cypher::match_(qp)
        .returning(Expression::asterisk())
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("{3}"));
}

#[test]
fn quantified_path_range() {
    let inner_rel = any_node().rel(rel("KNOWS")).to(any_node());
    let qp = quantified_path(inner_rel).range(Some(1), Some(5));
    let stmt = Cypher::match_(qp)
        .returning(Expression::asterisk())
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("{1, 5}") || rendered.contains("{1,5}"));
}

#[test]
fn quantified_path_with_where() {
    let inner_rel = any_node_named("x").rel(rel("KNOWS")).to(any_node());
    let qp = quantified_path(inner_rel)
        .where_(Expression::from(prop("x", "active").eq(true)))
        .plus();
    let stmt = Cypher::match_(qp)
        .returning(Expression::asterisk())
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("WHERE"));
    assert!(rendered.contains("active"));
}

// ============================================================================
// Quantified Relationships
// ============================================================================

#[test]
fn quantified_relationship_star() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS").quantified(Quantifier::Star)).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("KNOWS"));
    // Quantifier should render as * on the relationship
}

#[test]
fn quantified_relationship_plus() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS").quantified(Quantifier::Plus)).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("KNOWS"));
}

#[test]
fn quantified_relationship_exact() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS").quantified(Quantifier::Exact(2))).to(b);
    let stmt = Cypher::match_(r)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("{2}"));
}

// ============================================================================
// Path selectors
// ============================================================================

#[test]
fn shortest_path_selector() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let selected = shortest(1, r);
    let stmt = Cypher::match_(selected)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("SHORTEST 1"));
}

#[test]
fn all_shortest_path_selector() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let selected = all_shortest(r);
    let stmt = Cypher::match_(selected)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("ALL SHORTEST"));
}

#[test]
fn any_path_selector() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let selected = any_path(r);
    let stmt = Cypher::match_(selected)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("ANY"));
}

#[test]
fn shortest_groups_path_selector() {
    let a = any_node_named("a");
    let b = any_node_named("b");
    let r = a.rel(rel("KNOWS")).to(b);
    let selected = shortest_groups(3, r);
    let stmt = Cypher::match_(selected)
        .returning((name("a"), name("b")))
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("SHORTEST 3 GROUPS"));
}
