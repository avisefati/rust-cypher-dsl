//! Integration tests for admin commands (indexes, constraints, SHOW, TERMINATE).
//!
//! These tests exercise the public builder API (`Cypher::*`) end-to-end,
//! asserting that rendered output matches expected Cypher strings.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::prelude::*;

// ============================================================================
// CREATE INDEX
// ============================================================================

#[test]
fn create_range_index_for_node() {
    let stmt = Cypher::create_index("person_name_idx")
        .for_node("n", "Person", vec!["name"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE INDEX person_name_idx FOR (n:Person) ON (n.name)"
    );
}

#[test]
fn create_range_index_composite() {
    let stmt = Cypher::create_index("person_composite")
        .for_node("n", "Person", vec!["firstName", "lastName"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE INDEX person_composite FOR (n:Person) ON (n.firstName, n.lastName)"
    );
}

#[test]
fn create_text_index_if_not_exists() {
    let stmt = Cypher::create_index_if_not_exists("bio_idx")
        .text()
        .for_node("n", "Person", vec!["bio"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)"
    );
}

#[test]
fn create_point_index() {
    let stmt = Cypher::create_index("loc_idx")
        .point()
        .for_node("n", "Place", vec!["location"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE POINT INDEX loc_idx FOR (n:Place) ON (n.location)"
    );
}

#[test]
fn create_fulltext_index_single_label() {
    let stmt = Cypher::create_index("ft_movie")
        .fulltext()
        .for_node("n", "Movie", vec!["title", "description"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE FULLTEXT INDEX ft_movie FOR (n:Movie) ON EACH [n.title, n.description]"
    );
}

#[test]
fn create_fulltext_index_multi_label() {
    let stmt = Cypher::create_index("ft_content")
        .fulltext()
        .for_node_multi_label("n", vec!["Movie", "Book"], vec!["title", "summary"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE FULLTEXT INDEX ft_content FOR (n:Movie|Book) ON EACH [n.title, n.summary]"
    );
}

#[test]
fn create_vector_index_with_options() {
    let opts = Expression::raw_unchecked(
        "{`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}",
    );
    let stmt = Cypher::create_index("vec_embedding")
        .vector()
        .for_node("n", "Document", vec!["embedding"])
        .options(opts)
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE VECTOR INDEX vec_embedding FOR (n:Document) ON (n.embedding) OPTIONS {`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}"
    );
}

#[test]
fn create_lookup_index_node() {
    let stmt = Cypher::create_index("node_label_lookup")
        .lookup()
        .for_node_lookup("n")
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE LOOKUP INDEX node_label_lookup FOR (n) ON EACH labels(n)"
    );
}

#[test]
fn create_lookup_index_relationship() {
    let stmt = Cypher::create_index("rel_type_lookup")
        .lookup()
        .for_relationship_lookup("r")
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE LOOKUP INDEX rel_type_lookup FOR ()-[r]-() ON EACH type(r)"
    );
}

#[test]
fn create_index_for_relationship() {
    let stmt = Cypher::create_index("knows_since")
        .for_relationship("r", "KNOWS", vec!["since"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE INDEX knows_since FOR ()-[r:KNOWS]-() ON (r.since)"
    );
}

#[test]
fn create_fulltext_index_relationship_multi_type() {
    let stmt = Cypher::create_index("ft_rels")
        .fulltext()
        .for_relationship_multi_type("r", vec!["KNOWS", "WORKS_WITH"], vec!["note"])
        .build();
    assert_eq!(
        stmt.render(),
        "CREATE FULLTEXT INDEX ft_rels FOR ()-[r:KNOWS|WORKS_WITH]-() ON EACH [r.note]"
    );
}

// ============================================================================
// DROP INDEX
// ============================================================================

#[test]
fn drop_index() {
    let stmt = Cypher::drop_index("my_index");
    assert_eq!(stmt.render(), "DROP INDEX my_index");
}

#[test]
fn drop_index_if_exists() {
    let stmt = Cypher::drop_index_if_exists("my_index");
    assert_eq!(stmt.render(), "DROP INDEX my_index IF EXISTS");
}

// ============================================================================
// SHOW INDEXES
// ============================================================================

#[test]
fn show_indexes() {
    let stmt = Cypher::show_indexes().build();
    assert_eq!(stmt.render(), "SHOW INDEXES");
}

#[test]
fn show_range_indexes_yield_all() {
    let stmt = Cypher::show_indexes()
        .filter(IndexFilter::Range)
        .yield_all()
        .build();
    assert_eq!(stmt.render(), "SHOW RANGE INDEXES YIELD *");
}

#[test]
fn show_indexes_yield_fields_where() {
    let stmt = Cypher::show_indexes()
        .yield_fields(vec![name("name"), name("type"), name("state")])
        .where_(name("state").eq(Expression::string_literal("ONLINE")))
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW INDEXES YIELD name, type, state WHERE state = 'ONLINE'"
    );
}

// ============================================================================
// CREATE CONSTRAINT
// ============================================================================

#[test]
fn create_unique_constraint() {
    let stmt = Cypher::create_constraint("unique_email")
        .for_node("n", "Person")
        .is_unique(vec!["email"]);
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE"
    );
}

#[test]
fn create_unique_constraint_composite() {
    let stmt = Cypher::create_constraint("unique_name")
        .for_node("n", "Person")
        .is_unique(vec!["firstName", "lastName"]);
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT unique_name FOR (n:Person) REQUIRE (n.firstName, n.lastName) IS UNIQUE"
    );
}

#[test]
fn create_existence_constraint_if_not_exists() {
    let stmt = Cypher::create_constraint_if_not_exists("exists_name")
        .for_node("n", "Person")
        .is_not_null("name");
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.name IS NOT NULL"
    );
}

#[test]
fn create_node_key_constraint() {
    let stmt = Cypher::create_constraint("person_key")
        .for_node("n", "Person")
        .is_node_key(vec!["id", "name"]);
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT person_key FOR (n:Person) REQUIRE (n.id, n.name) IS NODE KEY"
    );
}

#[test]
fn create_relationship_key_constraint() {
    let stmt = Cypher::create_constraint("rel_key")
        .for_relationship("r", "REVIEWED")
        .is_relationship_key(vec!["id"]);
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT rel_key FOR ()-[r:REVIEWED]-() REQUIRE r.id IS RELATIONSHIP KEY"
    );
}

#[test]
fn create_property_type_constraint() {
    let stmt = Cypher::create_constraint("score_type")
        .for_relationship("r", "REVIEWED")
        .is_typed("score", "FLOAT");
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT score_type FOR ()-[r:REVIEWED]-() REQUIRE r.score IS :: FLOAT"
    );
}

#[test]
fn create_existence_constraint_on_relationship() {
    let stmt = Cypher::create_constraint("rel_date")
        .for_relationship("r", "WORKS_AT")
        .is_not_null("startDate");
    assert_eq!(
        stmt.render(),
        "CREATE CONSTRAINT rel_date FOR ()-[r:WORKS_AT]-() REQUIRE r.startDate IS NOT NULL"
    );
}

// ============================================================================
// DROP CONSTRAINT
// ============================================================================

#[test]
fn drop_constraint() {
    let stmt = Cypher::drop_constraint("my_constraint");
    assert_eq!(stmt.render(), "DROP CONSTRAINT my_constraint");
}

#[test]
fn drop_constraint_if_exists() {
    let stmt = Cypher::drop_constraint_if_exists("my_constraint");
    assert_eq!(stmt.render(), "DROP CONSTRAINT my_constraint IF EXISTS");
}

// ============================================================================
// SHOW CONSTRAINTS
// ============================================================================

#[test]
fn show_constraints() {
    let stmt = Cypher::show_constraints().build();
    assert_eq!(stmt.render(), "SHOW CONSTRAINTS");
}

#[test]
fn show_unique_constraints() {
    let stmt = Cypher::show_constraints()
        .filter(ConstraintFilter::Unique)
        .build();
    assert_eq!(stmt.render(), "SHOW UNIQUE CONSTRAINTS");
}

#[test]
fn show_constraints_yield_all() {
    let stmt = Cypher::show_constraints()
        .yield_all()
        .build();
    assert_eq!(stmt.render(), "SHOW CONSTRAINTS YIELD *");
}

// ============================================================================
// SHOW FUNCTIONS
// ============================================================================

#[test]
fn show_functions() {
    let stmt = Cypher::show_functions().build();
    assert_eq!(stmt.render(), "SHOW FUNCTIONS");
}

#[test]
fn show_built_in_functions() {
    let stmt = Cypher::show_functions()
        .filter(CallableFilter::BuiltIn)
        .build();
    assert_eq!(stmt.render(), "SHOW BUILT IN FUNCTIONS");
}

#[test]
fn show_user_defined_functions() {
    let stmt = Cypher::show_functions()
        .filter(CallableFilter::UserDefined)
        .build();
    assert_eq!(stmt.render(), "SHOW USER DEFINED FUNCTIONS");
}

#[test]
fn show_functions_executable_by_current_user() {
    let stmt = Cypher::show_functions()
        .executable_by_current_user()
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW FUNCTIONS EXECUTABLE BY CURRENT USER"
    );
}

#[test]
fn show_functions_executable_by_user() {
    let stmt = Cypher::show_functions()
        .executable_by("alice")
        .build();
    assert_eq!(stmt.render(), "SHOW FUNCTIONS EXECUTABLE BY alice");
}

#[test]
fn show_built_in_functions_executable() {
    let stmt = Cypher::show_functions()
        .filter(CallableFilter::BuiltIn)
        .executable_by_current_user()
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW BUILT IN FUNCTIONS EXECUTABLE BY CURRENT USER"
    );
}

// ============================================================================
// SHOW PROCEDURES
// ============================================================================

#[test]
fn show_procedures() {
    let stmt = Cypher::show_procedures().build();
    assert_eq!(stmt.render(), "SHOW PROCEDURES");
}

#[test]
fn show_procedures_yield_fields_where() {
    let stmt = Cypher::show_procedures()
        .yield_fields(vec![name("name"), name("signature")])
        .where_(name("name").starts_with("db."))
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW PROCEDURES YIELD name, signature WHERE name STARTS WITH 'db.'"
    );
}

#[test]
fn show_procedures_executable_by_current_user() {
    let stmt = Cypher::show_procedures()
        .executable_by_current_user()
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW PROCEDURES EXECUTABLE BY CURRENT USER"
    );
}

// ============================================================================
// SHOW TRANSACTIONS
// ============================================================================

#[test]
fn show_transactions() {
    let stmt = Cypher::show_transactions().build();
    assert_eq!(stmt.render(), "SHOW TRANSACTIONS");
}

#[test]
fn show_transactions_with_ids() {
    let stmt = Cypher::show_transactions()
        .ids(vec!["neo4j-tx-123"])
        .build();
    assert_eq!(stmt.render(), "SHOW TRANSACTIONS 'neo4j-tx-123'");
}

#[test]
fn show_transactions_multiple_ids() {
    let stmt = Cypher::show_transactions()
        .ids(vec!["neo4j-tx-1", "neo4j-tx-2"])
        .build();
    assert_eq!(
        stmt.render(),
        "SHOW TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'"
    );
}

#[test]
fn show_transactions_yield_all() {
    let stmt = Cypher::show_transactions()
        .yield_all()
        .build();
    assert_eq!(stmt.render(), "SHOW TRANSACTIONS YIELD *");
}

// ============================================================================
// TERMINATE TRANSACTIONS
// ============================================================================

#[test]
fn terminate_single_transaction() {
    let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"]).build();
    assert_eq!(stmt.render(), "TERMINATE TRANSACTIONS 'neo4j-tx-123'");
}

#[test]
fn terminate_multiple_transactions() {
    let stmt =
        Cypher::terminate_transactions(vec!["neo4j-tx-1", "neo4j-tx-2"]).build();
    assert_eq!(
        stmt.render(),
        "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'"
    );
}

#[test]
fn terminate_transactions_with_yield() {
    let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"])
        .yield_all()
        .build();
    assert_eq!(
        stmt.render(),
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD *"
    );
}

#[test]
fn terminate_transactions_with_yield_and_where() {
    let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"])
        .yield_fields(vec![name("transactionId"), name("username")])
        .where_(name("username").eq(Expression::string_literal("bob")))
        .build();
    assert_eq!(
        stmt.render(),
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD transactionId, username WHERE username = 'bob'"
    );
}

// ============================================================================
// Pretty rendering for admin commands
// ============================================================================

#[test]
fn pretty_render_create_index() {
    let stmt = Cypher::create_index("idx")
        .for_node("n", "Person", vec!["name"])
        .build();
    let config = rust_cypher_dsl::renderer::RenderConfig {
        pretty_print: true,
        ..Default::default()
    };
    // Admin commands are single-line, so pretty-print is identical to default.
    assert_eq!(
        stmt.render_with(config),
        "CREATE INDEX idx FOR (n:Person) ON (n.name)"
    );
}

// ============================================================================
// Catalog integration with admin commands
// ============================================================================

#[test]
fn catalog_from_create_index() {
    let stmt = Cypher::create_index("idx")
        .fulltext()
        .for_node_multi_label("n", vec!["Movie", "Book"], vec!["title", "desc"])
        .build();
    let catalog = rust_cypher_dsl::catalog::StatementCatalog::from_statement(&stmt);
    assert!(catalog.labels.contains("Movie"));
    assert!(catalog.labels.contains("Book"));
    let prop_names: std::collections::HashSet<_> =
        catalog.properties.iter().map(|p| p.name.as_str()).collect();
    assert!(prop_names.contains("title"));
    assert!(prop_names.contains("desc"));
}

#[test]
fn catalog_from_create_constraint() {
    let stmt = Cypher::create_constraint("c")
        .for_node("n", "Person")
        .is_node_key(vec!["id", "email"]);
    let catalog = rust_cypher_dsl::catalog::StatementCatalog::from_statement(&stmt);
    assert!(catalog.labels.contains("Person"));
    let prop_names: std::collections::HashSet<_> =
        catalog.properties.iter().map(|p| p.name.as_str()).collect();
    assert!(prop_names.contains("id"));
    assert!(prop_names.contains("email"));
}
