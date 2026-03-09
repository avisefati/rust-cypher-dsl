//! Integration tests for LOAD CSV patterns.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::props;

#[test]
fn load_csv_basic() {
    let stmt = Cypher::load_csv(lit("file:///people.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM 'file:///people.csv' AS row RETURN row"
    );
}

#[test]
fn load_csv_with_headers_basic() {
    let stmt = Cypher::load_csv_with_headers(lit("file:///people.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV WITH HEADERS FROM 'file:///people.csv' AS row RETURN row"
    );
}

#[test]
fn load_csv_with_field_terminator() {
    let stmt = Cypher::load_csv(lit("file:///data.tsv"))
        .as_("row")
        .field_terminator("\\t")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM 'file:///data.tsv' AS row FIELDTERMINATOR '\\t' RETURN row"
    );
}

#[test]
fn load_csv_create_nodes() {
    let n = node("Person").with_properties(props! {
        "name" => name("row")
    });
    let stmt = Cypher::load_csv_with_headers(lit("file:///people.csv"))
        .as_("row")
        .create(n)
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV WITH HEADERS FROM 'file:///people.csv' AS row CREATE (:`Person` {name: row})"
    );
}

#[test]
fn load_csv_match_merge() {
    let n = node("Person").named("n");
    let stmt = Cypher::load_csv_with_headers(lit("file:///people.csv"))
        .as_("row")
        .merge(n)
        .build();
    let rendered = stmt.render();
    assert!(rendered.contains("LOAD CSV WITH HEADERS"));
    assert!(rendered.contains("MERGE (n:`Person`)"));
}

#[test]
fn periodic_commit_with_load_csv() {
    let stmt = Cypher::using_periodic_commit(Some(500))
        .load_csv_with_headers(lit("file:///big_data.csv"))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "USING PERIODIC COMMIT 500 LOAD CSV WITH HEADERS FROM 'file:///big_data.csv' AS row RETURN row"
    );
}

#[test]
fn load_csv_with_param_url() {
    let stmt = Cypher::load_csv(Expression::from(param("csvUrl")))
        .as_("row")
        .returning(name("row"))
        .build();
    assert_eq!(
        stmt.render(),
        "LOAD CSV FROM $csvUrl AS row RETURN row"
    );
}
