//! Integration tests for the statement catalog introspection.

use rust_cypher_dsl::prelude::*;

// ============================================================================
// Label collection
// ============================================================================

#[test]
fn catalog_collects_single_label() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.labels.contains("Person"));
    assert_eq!(catalog.labels.len(), 1);
}

#[test]
fn catalog_collects_multiple_labels() {
    let a = node("Person").named("a");
    let b = node("Movie").named("b");
    let stmt = Cypher::match_(a)
        .match_(b)
        .returning((name("a"), name("b")))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.labels.contains("Person"));
    assert!(catalog.labels.contains("Movie"));
}

#[test]
fn catalog_deduplicates_labels() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let stmt = Cypher::match_(a)
        .match_(b)
        .returning((name("a"), name("b")))
        .build();
    let catalog = stmt.catalog();
    assert_eq!(catalog.labels.len(), 1);
    assert!(catalog.labels.contains("Person"));
}

// ============================================================================
// Relationship type collection
// ============================================================================

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
fn catalog_collects_multiple_relationship_types() {
    let a = node("Person").named("a");
    let b = node("Person").named("b");
    let c = node("Movie").named("c");
    let chain = a.rel(rel("KNOWS")).to(b).rel(rel("ACTED_IN")).to(c);
    let stmt = Cypher::match_(chain)
        .returning(name("a"))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.relationship_types.contains("KNOWS"));
    assert!(catalog.relationship_types.contains("ACTED_IN"));
}

// ============================================================================
// Property collection
// ============================================================================

#[test]
fn catalog_collects_property_from_where() {
    let n = node("Person").named("n");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.properties.iter().any(|p| p.name == "age"));
}

#[test]
fn catalog_collects_property_from_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(Expression::from(prop("n", "name")))
        .build();
    let catalog = stmt.catalog();
    assert!(catalog.properties.iter().any(|p| p.name == "name"));
}

// ============================================================================
// Parameter collection
// ============================================================================

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

#[test]
fn catalog_collects_bound_parameters() {
    let n = node("Person").named("n");
    let cond = prop("n", "name").eq(param_with_value("name", "Alice"));
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .build();
    let params = stmt.get_parameters();
    // Bound parameters appear in get_parameters() (only bound ones)
    assert!(params.contains_key("name"));
}

#[test]
fn catalog_collects_multiple_parameters() {
    let n = node("Person").named("n");
    let cond1 = prop("n", "name").eq(param("name"));
    let cond2 = prop("n", "age").gt(param("minAge"));
    let stmt = Cypher::match_(n)
        .where_(cond1)
        .and(cond2)
        .returning(name("n"))
        .build();
    let names = stmt.get_parameter_names();
    assert!(names.contains("name"));
    assert!(names.contains("minAge"));
}

// ============================================================================
// Union catalog
// ============================================================================

#[test]
fn catalog_collects_from_union() {
    let left = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let right = Cypher::match_(node("Movie").named("m"))
        .returning(name("m"))
        .build();
    let stmt = left.union(right);
    let catalog = stmt.catalog();
    assert!(catalog.labels.contains("Person"));
    assert!(catalog.labels.contains("Movie"));
}
