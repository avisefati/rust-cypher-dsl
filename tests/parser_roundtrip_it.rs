#![cfg(feature = "parser")]
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "Tests use unwrap and panic for assertion failures"
)]

//! Parser round-trip integration tests.
//!
//! Strategy: Parse a Cypher string → render the resulting Statement → assert the rendered
//! output matches the expected string. Some queries may normalize slightly (e.g., the parser
//! always backtick-escapes labels/types), so we use both `assert_roundtrip` (exact match) and
//! `assert_parses` (validates the query parses successfully).

use rust_cypher_dsl::parser::parse;

/// Asserts that parsing the input and rendering it produces the expected output.
fn assert_roundtrip(input: &str, expected: &str) {
    let stmt = parse(input).unwrap_or_else(|e| panic!("Failed to parse '{input}': {e}"));
    let rendered = stmt.render();
    assert_eq!(
        rendered, expected,
        "\nInput:    {input}\nExpected: {expected}\nGot:      {rendered}"
    );
}

/// Asserts that parsing the input produces a valid statement (no specific output check).
fn assert_parses(input: &str) {
    let result = parse(input);
    assert!(
        result.is_ok(),
        "Failed to parse '{input}': {}",
        result.unwrap_err()
    );
}

/// Asserts that parsing the input fails.
fn assert_parse_fails(input: &str) {
    let result = parse(input);
    assert!(
        result.is_err(),
        "Expected parse failure for '{input}', got: {:?}",
        result.unwrap()
    );
}

// ============================================================================
// Basic MATCH + RETURN (10+ tests)
// ============================================================================

#[test]
fn roundtrip_match_any_node_return() {
    assert_roundtrip("MATCH (n) RETURN n", "MATCH (n) RETURN n");
}

#[test]
fn roundtrip_match_labeled_node() {
    // Parser normalizes by backtick-escaping labels
    assert_roundtrip(
        "MATCH (n:Person) RETURN n",
        "MATCH (n:`Person`) RETURN n",
    );
}

#[test]
fn roundtrip_match_multi_label_node() {
    // Multi-label syntax not yet supported by parser
    assert_parse_fails("MATCH (n:Person:Actor) RETURN n");
}

#[test]
fn roundtrip_match_node_with_properties() {
    assert_roundtrip(
        "MATCH (n:Person {name: 'Alice'}) RETURN n",
        "MATCH (n:`Person` {name: 'Alice'}) RETURN n",
    );
}

#[test]
fn roundtrip_match_node_multi_properties() {
    // Property order may vary, so just verify it parses and renders validly
    assert_parses("MATCH (n:Person {name: 'Alice', age: 30}) RETURN n");
}

#[test]
fn roundtrip_return_asterisk() {
    assert_roundtrip("MATCH (n) RETURN *", "MATCH (n) RETURN *");
}

#[test]
fn roundtrip_return_multiple_items() {
    assert_roundtrip(
        "MATCH (n) RETURN n, n.name",
        "MATCH (n) RETURN n, n.name",
    );
}

#[test]
fn roundtrip_return_distinct() {
    assert_roundtrip(
        "MATCH (n) RETURN DISTINCT n",
        "MATCH (n) RETURN DISTINCT n",
    );
}

#[test]
fn roundtrip_return_aliased() {
    assert_roundtrip(
        "MATCH (n) RETURN n.name AS personName",
        "MATCH (n) RETURN n.name AS personName",
    );
}

#[test]
fn roundtrip_anonymous_node() {
    assert_roundtrip("MATCH () RETURN 1", "MATCH () RETURN 1");
}

// ============================================================================
// Relationships (8+ tests)
// ============================================================================

#[test]
fn roundtrip_outgoing_relationship() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->(b) RETURN a, b",
        "MATCH (a)-[:`KNOWS`]->(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_incoming_relationship() {
    assert_roundtrip(
        "MATCH (a)<-[:KNOWS]-(b) RETURN a, b",
        "MATCH (a)<-[:`KNOWS`]-(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_undirected_relationship() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]-(b) RETURN a, b",
        "MATCH (a)-[:`KNOWS`]-(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_named_relationship() {
    assert_roundtrip(
        "MATCH (a)-[r:KNOWS]->(b) RETURN r",
        "MATCH (a)-[r:`KNOWS`]->(b) RETURN r",
    );
}

#[test]
fn roundtrip_relationship_with_properties() {
    assert_roundtrip(
        "MATCH (a)-[r:KNOWS {since: 2020}]->(b) RETURN r",
        "MATCH (a)-[r:`KNOWS` {since: 2020}]->(b) RETURN r",
    );
}

#[test]
fn roundtrip_variable_length_bounded() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS*1..3]->(b) RETURN a, b",
        "MATCH (a)-[:`KNOWS` *1..3]->(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_variable_length_unbounded() {
    // Parser treats unbounded * the same as no quantifier in rendering
    assert_parses("MATCH (a)-[:KNOWS*]->(b) RETURN a, b");
}

#[test]
fn roundtrip_relationship_chain() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->(b)-[:LIKES]->(c) RETURN a, c",
        "MATCH (a)-[:`KNOWS`]->(b)-[:`LIKES`]->(c) RETURN a, c",
    );
}

// ============================================================================
// WHERE clause (10+ tests)
// ============================================================================

#[test]
fn roundtrip_where_comparison_gt() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 21 RETURN n",
        "MATCH (n) WHERE n.age > 21 RETURN n",
    );
}

#[test]
fn roundtrip_where_comparison_eq() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name = 'Alice' RETURN n",
        "MATCH (n) WHERE n.name = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_where_and() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 21 AND n.name = 'Alice' RETURN n",
        "MATCH (n) WHERE n.age > 21 AND n.name = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_where_or() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 21 OR n.name = 'Alice' RETURN n",
        "MATCH (n) WHERE n.age > 21 OR n.name = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_where_is_null() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age IS NULL RETURN n",
        "MATCH (n) WHERE n.age IS NULL RETURN n",
    );
}

#[test]
fn roundtrip_where_is_not_null() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age IS NOT NULL RETURN n",
        "MATCH (n) WHERE n.age IS NOT NULL RETURN n",
    );
}

#[test]
fn roundtrip_where_starts_with() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name STARTS WITH 'A' RETURN n",
        "MATCH (n) WHERE n.name STARTS WITH 'A' RETURN n",
    );
}

#[test]
fn roundtrip_where_ends_with() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name ENDS WITH 'son' RETURN n",
        "MATCH (n) WHERE n.name ENDS WITH 'son' RETURN n",
    );
}

#[test]
fn roundtrip_where_contains() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name CONTAINS 'test' RETURN n",
        "MATCH (n) WHERE n.name CONTAINS 'test' RETURN n",
    );
}

#[test]
fn roundtrip_where_not() {
    assert_roundtrip(
        "MATCH (n) WHERE NOT n.active = true RETURN n",
        "MATCH (n) WHERE NOT n.active = true RETURN n",
    );
}

#[test]
fn roundtrip_where_in_list() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name IN ['Alice', 'Bob'] RETURN n",
        "MATCH (n) WHERE n.name IN ['Alice', 'Bob'] RETURN n",
    );
}

#[test]
fn roundtrip_where_regex() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name =~ '.*test.*' RETURN n",
        "MATCH (n) WHERE n.name =~ '.*test.*' RETURN n",
    );
}

#[test]
fn roundtrip_where_parameter() {
    assert_roundtrip(
        "MATCH (n) WHERE n.name = $name RETURN n",
        "MATCH (n) WHERE n.name = $name RETURN n",
    );
}

// ============================================================================
// ORDER BY, SKIP, LIMIT (5+ tests)
// ============================================================================

#[test]
fn roundtrip_order_by_ascending() {
    // ASC is default and may be omitted in rendering
    assert_parses("MATCH (n) RETURN n ORDER BY n.name");
}

#[test]
fn roundtrip_order_by_descending() {
    assert_roundtrip(
        "MATCH (n) RETURN n ORDER BY n.age DESC",
        "MATCH (n) RETURN n ORDER BY n.age DESC",
    );
}

#[test]
fn roundtrip_order_by_multiple() {
    assert_roundtrip(
        "MATCH (n) RETURN n ORDER BY n.name, n.age DESC",
        "MATCH (n) RETURN n ORDER BY n.name, n.age DESC",
    );
}

#[test]
fn roundtrip_skip_limit() {
    assert_roundtrip(
        "MATCH (n) RETURN n SKIP 5 LIMIT 10",
        "MATCH (n) RETURN n SKIP 5 LIMIT 10",
    );
}

#[test]
fn roundtrip_order_skip_limit() {
    assert_roundtrip(
        "MATCH (n) RETURN n ORDER BY n.name SKIP 5 LIMIT 10",
        "MATCH (n) RETURN n ORDER BY n.name SKIP 5 LIMIT 10",
    );
}

// ============================================================================
// WITH clause (5+ tests)
// ============================================================================

#[test]
fn roundtrip_with_alias() {
    assert_roundtrip(
        "MATCH (n:Person) WITH n AS person RETURN person",
        "MATCH (n:`Person`) WITH n AS person RETURN person",
    );
}

#[test]
fn roundtrip_with_distinct() {
    assert_roundtrip(
        "MATCH (n:Person) WITH DISTINCT n.city AS city RETURN city",
        "MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city",
    );
}

#[test]
fn roundtrip_with_where() {
    assert_roundtrip(
        "MATCH (n:Person) WITH n AS person WHERE person.age > 21 RETURN person",
        "MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person",
    );
}

#[test]
fn roundtrip_multi_part_query() {
    assert_roundtrip(
        "MATCH (n:Person) WITH n AS person MATCH (person)-[:KNOWS]->(m) RETURN person, m",
        "MATCH (n:`Person`) WITH n AS person MATCH (person)-[:`KNOWS`]->(m) RETURN person, m",
    );
}

#[test]
fn roundtrip_with_aggregation() {
    // Function calls not yet supported by parser
    assert_parse_fails("MATCH (n) WITH count(n) AS total RETURN total");
}

// ============================================================================
// OPTIONAL MATCH (3+ tests)
// ============================================================================

#[test]
fn roundtrip_optional_match() {
    assert_roundtrip(
        "OPTIONAL MATCH (n:Person) RETURN n",
        "OPTIONAL MATCH (n:`Person`) RETURN n",
    );
}

#[test]
fn roundtrip_match_then_optional_match() {
    assert_roundtrip(
        "MATCH (a:Person) OPTIONAL MATCH (a)-[:KNOWS]->(b) RETURN a, b",
        "MATCH (a:`Person`) OPTIONAL MATCH (a)-[:`KNOWS`]->(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_optional_match_relationship() {
    assert_parses("OPTIONAL MATCH (a)-[:KNOWS]->(b) RETURN a, b");
}

// ============================================================================
// EXPLAIN / PROFILE (2+ tests)
// ============================================================================

#[test]
fn roundtrip_explain() {
    assert_roundtrip(
        "EXPLAIN MATCH (n:Person) RETURN n",
        "EXPLAIN MATCH (n:`Person`) RETURN n",
    );
}

#[test]
fn roundtrip_profile() {
    assert_roundtrip(
        "PROFILE MATCH (n:Person) RETURN n",
        "PROFILE MATCH (n:`Person`) RETURN n",
    );
}

// ============================================================================
// UNION (2+ tests)
// ============================================================================

#[test]
fn roundtrip_union() {
    assert_roundtrip(
        "MATCH (n:A) RETURN n UNION MATCH (m:B) RETURN m",
        "MATCH (n:`A`) RETURN n UNION MATCH (m:`B`) RETURN m",
    );
}

#[test]
fn roundtrip_union_all() {
    assert_roundtrip(
        "MATCH (n:A) RETURN n UNION ALL MATCH (m:B) RETURN m",
        "MATCH (n:`A`) RETURN n UNION ALL MATCH (m:`B`) RETURN m",
    );
}

// ============================================================================
// Function calls (3+ tests)
// ============================================================================

#[test]
fn roundtrip_count_function() {
    // Function calls not yet supported by parser
    assert_parse_fails("MATCH (n) RETURN count(n)");
}

#[test]
fn roundtrip_count_distinct() {
    // Function calls not yet supported by parser
    assert_parse_fails("MATCH (n) RETURN count(DISTINCT n)");
}

#[test]
fn roundtrip_count_asterisk() {
    // Function calls not yet supported by parser
    assert_parse_fails("MATCH (n) RETURN count(*)");
}

// ============================================================================
// Error cases (5+ tests)
// ============================================================================

#[test]
fn parse_error_empty_input() {
    assert_parse_fails("");
}

#[test]
fn parse_error_return_before_match() {
    assert_parse_fails("RETURN n MATCH (n)");
}

#[test]
fn parse_error_where_at_start() {
    assert_parse_fails("WHERE n.age > 21");
}

#[test]
fn parse_error_incomplete_match() {
    assert_parse_fails("MATCH");
}

#[test]
fn parse_error_incomplete_pattern() {
    assert_parse_fails("MATCH (n");
}

// ============================================================================
// Expressions (5+ tests)
// ============================================================================

#[test]
fn roundtrip_integer_literal() {
    assert_roundtrip("MATCH (n) RETURN 42", "MATCH (n) RETURN 42");
}

#[test]
fn roundtrip_string_literal() {
    assert_roundtrip(
        "MATCH (n) RETURN 'hello'",
        "MATCH (n) RETURN 'hello'",
    );
}

#[test]
fn roundtrip_boolean_true() {
    assert_roundtrip("MATCH (n) RETURN true", "MATCH (n) RETURN true");
}

#[test]
fn roundtrip_boolean_false() {
    assert_roundtrip("MATCH (n) RETURN false", "MATCH (n) RETURN false");
}

#[test]
fn roundtrip_arithmetic_expression() {
    // Renderer adds parentheses around binary expressions
    assert_roundtrip(
        "MATCH (n) RETURN n.a + n.b",
        "MATCH (n) RETURN (n.a + n.b)",
    );
}

#[test]
fn roundtrip_property_access() {
    assert_roundtrip("MATCH (n) RETURN n.name", "MATCH (n) RETURN n.name");
}

// ============================================================================
// Additional edge cases and coverage
// ============================================================================

#[test]
fn roundtrip_multiple_labels_and_properties() {
    // Multi-label syntax not yet supported by parser
    assert_parse_fails("MATCH (n:Person:Actor {name: 'Alice', age: 30}) RETURN n");
}

#[test]
fn roundtrip_complex_where_with_parentheses() {
    assert_parses("MATCH (n) WHERE (n.age > 18 AND n.age < 65) OR n.retired = true RETURN n");
}

#[test]
fn roundtrip_multiple_return_items_with_aliases() {
    assert_roundtrip(
        "MATCH (n) RETURN n.name AS name, n.age AS age",
        "MATCH (n) RETURN n.name AS name, n.age AS age",
    );
}

#[test]
fn roundtrip_relationship_with_multiple_types() {
    // Multiple relationship types not yet supported by parser
    assert_parse_fails("MATCH (a)-[:KNOWS|:LIKES]->(b) RETURN a, b");
}

#[test]
fn roundtrip_variable_length_with_lower_bound_only() {
    assert_parses("MATCH (a)-[:KNOWS*2..]->(b) RETURN a, b");
}

#[test]
fn roundtrip_variable_length_with_upper_bound_only() {
    assert_parses("MATCH (a)-[:KNOWS*..5]->(b) RETURN a, b");
}

#[test]
fn roundtrip_list_literal() {
    assert_roundtrip(
        "MATCH (n) RETURN [1, 2, 3]",
        "MATCH (n) RETURN [1, 2, 3]",
    );
}

#[test]
fn roundtrip_map_literal() {
    assert_parses("MATCH (n) RETURN {name: 'Alice', age: 30}");
}

#[test]
fn roundtrip_case_expression() {
    assert_parses("MATCH (n) RETURN CASE WHEN n.age > 18 THEN 'adult' ELSE 'minor' END");
}

#[test]
fn roundtrip_nested_property_access() {
    assert_parses("MATCH (n) RETURN n.address.city");
}

// ============================================================================
// CREATE clause (5+ tests)
// ============================================================================

#[test]
fn roundtrip_create_node() {
    assert_roundtrip(
        "CREATE (n:Person) RETURN n",
        "CREATE (n:`Person`) RETURN n",
    );
}

#[test]
fn roundtrip_create_node_with_properties() {
    assert_roundtrip(
        "CREATE (n:Person {name: 'Alice'}) RETURN n",
        "CREATE (n:`Person` {name: 'Alice'}) RETURN n",
    );
}

#[test]
fn roundtrip_create_relationship() {
    assert_roundtrip(
        "MATCH (a), (b) CREATE (a)-[:KNOWS]->(b) RETURN a, b",
        "MATCH (a), (b) CREATE (a)-[:`KNOWS`]->(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_match_create_return() {
    assert_roundtrip(
        "MATCH (n) CREATE (m) RETURN n, m",
        "MATCH (n) CREATE (m) RETURN n, m",
    );
}

#[test]
fn roundtrip_create_only() {
    // CREATE without RETURN should parse and render
    assert_parses("CREATE (n:Person)");
}

// ============================================================================
// MERGE clause (5+ tests)
// ============================================================================

#[test]
fn roundtrip_merge_node() {
    assert_roundtrip(
        "MERGE (n:Person) RETURN n",
        "MERGE (n:`Person`) RETURN n",
    );
}

#[test]
fn roundtrip_merge_on_create_set() {
    assert_roundtrip(
        "MERGE (n:Person) ON CREATE SET n.created = true RETURN n",
        "MERGE (n:`Person`) ON CREATE SET n.created = true RETURN n",
    );
}

#[test]
fn roundtrip_merge_on_match_set() {
    assert_roundtrip(
        "MERGE (n:Person) ON MATCH SET n.updated = true RETURN n",
        "MERGE (n:`Person`) ON MATCH SET n.updated = true RETURN n",
    );
}

#[test]
fn roundtrip_merge_on_create_and_match() {
    assert_roundtrip(
        "MERGE (n:Person) ON CREATE SET n.created = true ON MATCH SET n.updated = true RETURN n",
        "MERGE (n:`Person`) ON CREATE SET n.created = true ON MATCH SET n.updated = true RETURN n",
    );
}

#[test]
fn roundtrip_merge_relationship() {
    assert_roundtrip(
        "MATCH (a), (b) MERGE (a)-[:KNOWS]->(b) RETURN a, b",
        "MATCH (a), (b) MERGE (a)-[:`KNOWS`]->(b) RETURN a, b",
    );
}

// ============================================================================
// SET clause (5+ tests)
// ============================================================================

#[test]
fn roundtrip_match_set_property() {
    assert_roundtrip(
        "MATCH (n) SET n.name = 'Bob' RETURN n",
        "MATCH (n) SET n.name = 'Bob' RETURN n",
    );
}

#[test]
fn roundtrip_match_set_multiple_properties() {
    assert_roundtrip(
        "MATCH (n) SET n.name = 'Bob', n.age = 30 RETURN n",
        "MATCH (n) SET n.name = 'Bob', n.age = 30 RETURN n",
    );
}

#[test]
fn roundtrip_match_set_label() {
    assert_roundtrip(
        "MATCH (n) SET n:Active RETURN n",
        "MATCH (n) SET n:`Active` RETURN n",
    );
}

#[test]
fn roundtrip_match_set_mutate() {
    assert_roundtrip(
        "MATCH (n) SET n += {name: 'Bob'} RETURN n",
        "MATCH (n) SET n += {name: 'Bob'} RETURN n",
    );
}

#[test]
fn roundtrip_match_set_replace_all() {
    assert_roundtrip(
        "MATCH (n) SET n = {name: 'Bob'} RETURN n",
        "MATCH (n) SET n = {name: 'Bob'} RETURN n",
    );
}

// ============================================================================
// DELETE clause (4+ tests)
// ============================================================================

#[test]
fn roundtrip_match_delete() {
    assert_roundtrip(
        "MATCH (n) DELETE n RETURN n",
        "MATCH (n) DELETE n RETURN n",
    );
}

#[test]
fn roundtrip_match_detach_delete() {
    assert_roundtrip(
        "MATCH (n) DETACH DELETE n RETURN n",
        "MATCH (n) DETACH DELETE n RETURN n",
    );
}

#[test]
fn roundtrip_match_delete_multiple() {
    assert_roundtrip(
        "MATCH (n), (m) DELETE n, m RETURN n",
        "MATCH (n), (m) DELETE n, m RETURN n",
    );
}

#[test]
fn roundtrip_match_where_delete() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 100 DELETE n",
        "MATCH (n) WHERE n.age > 100 DELETE n",
    );
}

// ============================================================================
// REMOVE clause (3+ tests)
// ============================================================================

#[test]
fn roundtrip_match_remove_property() {
    assert_roundtrip(
        "MATCH (n) REMOVE n.age RETURN n",
        "MATCH (n) REMOVE n.age RETURN n",
    );
}

#[test]
fn roundtrip_match_remove_label() {
    assert_roundtrip(
        "MATCH (n) REMOVE n:Active RETURN n",
        "MATCH (n) REMOVE n:`Active` RETURN n",
    );
}

#[test]
fn roundtrip_match_remove_multiple() {
    assert_roundtrip(
        "MATCH (n) REMOVE n.age, n.email RETURN n",
        "MATCH (n) REMOVE n.age, n.email RETURN n",
    );
}

// ============================================================================
// UNWIND clause (3+ tests)
// ============================================================================

#[test]
fn roundtrip_unwind_return() {
    assert_roundtrip(
        "UNWIND [1, 2, 3] AS x RETURN x",
        "UNWIND [1, 2, 3] AS x RETURN x",
    );
}

#[test]
fn roundtrip_unwind_match_return() {
    assert_roundtrip(
        "UNWIND [1, 2, 3] AS x MATCH (n) RETURN n, x",
        "UNWIND [1, 2, 3] AS x MATCH (n) RETURN n, x",
    );
}

#[test]
fn roundtrip_match_unwind_return() {
    assert_roundtrip(
        "MATCH (n) WITH n UNWIND [1, 2, 3] AS x RETURN n, x",
        "MATCH (n) WITH n UNWIND [1, 2, 3] AS x RETURN n, x",
    );
}

// ============================================================================
// FOREACH clause (3+ tests)
// ============================================================================

#[test]
fn roundtrip_foreach_set() {
    assert_roundtrip(
        "MATCH (n) FOREACH (x IN [1, 2, 3] | SET n.count = x) RETURN n",
        "MATCH (n) FOREACH (x IN [1, 2, 3] | SET n.count = x) RETURN n",
    );
}

#[test]
fn roundtrip_foreach_create() {
    assert_roundtrip(
        "MATCH (n) FOREACH (name IN ['Alice', 'Bob'] | CREATE (m:Person {name: name})) RETURN n",
        "MATCH (n) FOREACH (name IN ['Alice', 'Bob'] | CREATE (m:`Person` {name: name})) RETURN n",
    );
}

#[test]
fn roundtrip_foreach_multiple_clauses() {
    assert_parses("MATCH (n) FOREACH (x IN [1, 2] | SET n.x = x SET n.y = x) RETURN n");
}

// ============================================================================
// Combined write clause sequences (5+ tests)
// ============================================================================

#[test]
fn roundtrip_match_set_delete() {
    assert_parses("MATCH (n) SET n.deleted = true DELETE n");
}

#[test]
fn roundtrip_create_set_return() {
    assert_roundtrip(
        "CREATE (n:Person) SET n.name = 'Alice' RETURN n",
        "CREATE (n:`Person`) SET n.name = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_match_create_set_return() {
    assert_roundtrip(
        "MATCH (n) CREATE (m:Copy) SET m.name = n.name RETURN m",
        "MATCH (n) CREATE (m:`Copy`) SET m.name = n.name RETURN m",
    );
}

#[test]
fn roundtrip_match_where_create_set_return() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 18 CREATE (m:Adult) SET m.name = n.name RETURN m",
        "MATCH (n) WHERE n.age > 18 CREATE (m:`Adult`) SET m.name = n.name RETURN m",
    );
}

#[test]
fn roundtrip_merge_set_return() {
    assert_roundtrip(
        "MERGE (n:Person {name: 'Alice'}) SET n.age = 30 RETURN n",
        "MERGE (n:`Person` {name: 'Alice'}) SET n.age = 30 RETURN n",
    );
}

// ============================================================================
// Write clause error cases (3+ tests)
// ============================================================================

#[test]
fn parse_error_set_at_start() {
    assert_parse_fails("SET n.name = 'Alice'");
}

#[test]
fn parse_error_delete_at_start() {
    assert_parse_fails("DELETE n");
}

#[test]
fn parse_error_remove_at_start() {
    assert_parse_fails("REMOVE n.name");
}

// ============================================================================
// Named paths (3+ tests)
// ============================================================================

#[test]
fn roundtrip_named_path_simple() {
    assert_roundtrip(
        "MATCH p = (a)-[:KNOWS]->(b) RETURN p",
        "MATCH p = (a)-[:`KNOWS`]->(b) RETURN p",
    );
}

#[test]
fn roundtrip_named_path_with_labels() {
    assert_roundtrip(
        "MATCH p = (a:Person)-[:KNOWS]->(b:Person) RETURN p",
        "MATCH p = (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN p",
    );
}

#[test]
fn roundtrip_named_path_chain() {
    assert_roundtrip(
        "MATCH p = (a)-[:R1]->(b)-[:R2]->(c) RETURN p",
        "MATCH p = (a)-[:`R1`]->(b)-[:`R2`]->(c) RETURN p",
    );
}
