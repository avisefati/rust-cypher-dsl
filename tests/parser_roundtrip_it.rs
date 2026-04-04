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
    assert_roundtrip(
        "MATCH (n:Person:Actor) RETURN n",
        "MATCH (n:`Person`:`Actor`) RETURN n",
    );
}

#[test]
fn roundtrip_match_node_with_properties() {
    assert_roundtrip(
        "MATCH (n:Person {`name`: 'Alice'}) RETURN n",
        "MATCH (n:`Person` {`name`: 'Alice'}) RETURN n",
    );
}

#[test]
fn roundtrip_match_node_multi_properties() {
    // Property order may vary, so just verify it parses and renders validly
    assert_parses("MATCH (n:Person {`name`: 'Alice', age: 30}) RETURN n");
}

#[test]
fn roundtrip_return_asterisk() {
    assert_roundtrip("MATCH (n) RETURN *", "MATCH (n) RETURN *");
}

#[test]
fn roundtrip_return_multiple_items() {
    assert_roundtrip(
        "MATCH (n) RETURN n, n.`name`",
        "MATCH (n) RETURN n, n.`name`",
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
        "MATCH (n) RETURN n.`name` AS personName",
        "MATCH (n) RETURN n.`name` AS personName",
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
        "MATCH (n) WHERE n.`name` = 'Alice' RETURN n",
        "MATCH (n) WHERE n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_where_and() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 21 AND n.`name` = 'Alice' RETURN n",
        "MATCH (n) WHERE n.age > 21 AND n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_where_or() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 21 OR n.`name` = 'Alice' RETURN n",
        "MATCH (n) WHERE n.age > 21 OR n.`name` = 'Alice' RETURN n",
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
        "MATCH (n) WHERE n.`name` STARTS WITH 'A' RETURN n",
        "MATCH (n) WHERE n.`name` STARTS WITH 'A' RETURN n",
    );
}

#[test]
fn roundtrip_where_ends_with() {
    assert_roundtrip(
        "MATCH (n) WHERE n.`name` ENDS WITH 'son' RETURN n",
        "MATCH (n) WHERE n.`name` ENDS WITH 'son' RETURN n",
    );
}

#[test]
fn roundtrip_where_contains() {
    assert_roundtrip(
        "MATCH (n) WHERE n.`name` CONTAINS 'test' RETURN n",
        "MATCH (n) WHERE n.`name` CONTAINS 'test' RETURN n",
    );
}

#[test]
fn roundtrip_where_not() {
    assert_roundtrip(
        "MATCH (n) WHERE NOT n.`active` = true RETURN n",
        "MATCH (n) WHERE NOT n.`active` = true RETURN n",
    );
}

#[test]
fn roundtrip_where_in_list() {
    assert_roundtrip(
        "MATCH (n) WHERE n.`name` IN ['Alice', 'Bob'] RETURN n",
        "MATCH (n) WHERE n.`name` IN ['Alice', 'Bob'] RETURN n",
    );
}

#[test]
fn roundtrip_where_regex() {
    assert_roundtrip(
        "MATCH (n) WHERE n.`name` =~ '.*test.*' RETURN n",
        "MATCH (n) WHERE n.`name` =~ '.*test.*' RETURN n",
    );
}

#[test]
fn roundtrip_where_parameter() {
    assert_roundtrip(
        "MATCH (n) WHERE n.`name` = $name RETURN n",
        "MATCH (n) WHERE n.`name` = $name RETURN n",
    );
}

// ============================================================================
// ORDER BY, SKIP, LIMIT (5+ tests)
// ============================================================================

#[test]
fn roundtrip_order_by_ascending() {
    // ASC is default and may be omitted in rendering
    assert_parses("MATCH (n) RETURN n ORDER BY n.`name`");
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
        "MATCH (n) RETURN n ORDER BY n.`name`, n.age DESC",
        "MATCH (n) RETURN n ORDER BY n.`name`, n.age DESC",
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
        "MATCH (n) RETURN n ORDER BY n.`name` SKIP 5 LIMIT 10",
        "MATCH (n) RETURN n ORDER BY n.`name` SKIP 5 LIMIT 10",
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
    assert_roundtrip(
        "MATCH (n) WITH count(n) AS total RETURN total",
        "MATCH (n) WITH count(n) AS total RETURN total",
    );
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
    assert_roundtrip("MATCH (n) RETURN count(n)", "MATCH (n) RETURN count(n)");
}

#[test]
fn roundtrip_count_distinct() {
    assert_roundtrip(
        "MATCH (n) RETURN count(DISTINCT n)",
        "MATCH (n) RETURN count(DISTINCT n)",
    );
}

#[test]
fn roundtrip_count_asterisk() {
    assert_roundtrip("MATCH (n) RETURN count(*)", "MATCH (n) RETURN count(*)");
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
    assert_roundtrip("MATCH (n) RETURN n.`name`", "MATCH (n) RETURN n.`name`");
}

// ============================================================================
// Additional edge cases and coverage
// ============================================================================

#[test]
fn roundtrip_multiple_labels_and_properties() {
    assert_roundtrip(
        "MATCH (n:Person:Actor {`name`: 'Alice', age: 30}) RETURN n",
        "MATCH (n:`Person`:`Actor` {`name`: 'Alice', age: 30}) RETURN n",
    );
}

#[test]
fn roundtrip_complex_where_with_parentheses() {
    assert_parses("MATCH (n) WHERE (n.age > 18 AND n.age < 65) OR n.retired = true RETURN n");
}

#[test]
fn roundtrip_multiple_return_items_with_aliases() {
    assert_roundtrip(
        "MATCH (n) RETURN n.`name` AS `name`, n.age AS age",
        "MATCH (n) RETURN n.`name` AS `name`, n.age AS age",
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
    assert_parses("MATCH (n) RETURN {`name`: 'Alice', age: 30}");
}

#[test]
fn roundtrip_case_expression() {
    assert_roundtrip(
        "MATCH (n) RETURN CASE WHEN n.age > 18 THEN 'adult' ELSE 'minor' END",
        "MATCH (n) RETURN CASE WHEN n.age > 18 THEN 'adult' ELSE 'minor' END",
    );
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
        "CREATE (n:Person {`name`: 'Alice'}) RETURN n",
        "CREATE (n:`Person` {`name`: 'Alice'}) RETURN n",
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
        "MATCH (n) SET n.`name` = 'Bob' RETURN n",
        "MATCH (n) SET n.`name` = 'Bob' RETURN n",
    );
}

#[test]
fn roundtrip_match_set_multiple_properties() {
    assert_roundtrip(
        "MATCH (n) SET n.`name` = 'Bob', n.age = 30 RETURN n",
        "MATCH (n) SET n.`name` = 'Bob', n.age = 30 RETURN n",
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
        "MATCH (n) SET n += {`name`: 'Bob'} RETURN n",
        "MATCH (n) SET n += {`name`: 'Bob'} RETURN n",
    );
}

#[test]
fn roundtrip_match_set_replace_all() {
    assert_roundtrip(
        "MATCH (n) SET n = {`name`: 'Bob'} RETURN n",
        "MATCH (n) SET n = {`name`: 'Bob'} RETURN n",
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
        "MATCH (n) FOREACH (x IN [1, 2, 3] | SET n.`count` = x) RETURN n",
        "MATCH (n) FOREACH (x IN [1, 2, 3] | SET n.`count` = x) RETURN n",
    );
}

#[test]
fn roundtrip_foreach_create() {
    assert_roundtrip(
        "MATCH (n) FOREACH (name IN ['Alice', 'Bob'] | CREATE (m:Person {`name`: name})) RETURN n",
        "MATCH (n) FOREACH (`name` IN ['Alice', 'Bob'] | CREATE (m:`Person` {`name`: `name`})) RETURN n",
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
        "CREATE (n:Person) SET n.`name` = 'Alice' RETURN n",
        "CREATE (n:`Person`) SET n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_match_create_set_return() {
    assert_roundtrip(
        "MATCH (n) CREATE (m:Copy) SET m.`name` = n.`name` RETURN m",
        "MATCH (n) CREATE (m:`Copy`) SET m.`name` = n.`name` RETURN m",
    );
}

#[test]
fn roundtrip_match_where_create_set_return() {
    assert_roundtrip(
        "MATCH (n) WHERE n.age > 18 CREATE (m:Adult) SET m.`name` = n.`name` RETURN m",
        "MATCH (n) WHERE n.age > 18 CREATE (m:`Adult`) SET m.`name` = n.`name` RETURN m",
    );
}

#[test]
fn roundtrip_merge_set_return() {
    assert_roundtrip(
        "MERGE (n:Person {`name`: 'Alice'}) SET n.age = 30 RETURN n",
        "MERGE (n:`Person` {`name`: 'Alice'}) SET n.age = 30 RETURN n",
    );
}

// ============================================================================
// Write clause error cases (3+ tests)
// ============================================================================

#[test]
fn parse_error_set_at_start() {
    assert_parse_fails("SET n.`name` = 'Alice'");
}

#[test]
fn parse_error_delete_at_start() {
    assert_parse_fails("DELETE n");
}

#[test]
fn parse_error_remove_at_start() {
    assert_parse_fails("REMOVE n.`name`");
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

// ============================================================================
// CALL procedure
// ============================================================================

#[test]
fn roundtrip_call_procedure_no_args() {
    assert_roundtrip("CALL db.labels()", "CALL db.labels()");
}

#[test]
fn roundtrip_call_procedure_with_args() {
    assert_roundtrip(
        "CALL db.index.fulltext.queryNodes('titleIndex', 'hello')",
        "CALL db.index.fulltext.queryNodes('titleIndex', 'hello')",
    );
}

#[test]
fn roundtrip_call_procedure_yield() {
    assert_roundtrip(
        "CALL db.labels() YIELD `label`",
        "CALL db.labels() YIELD `label`",
    );
}

#[test]
fn roundtrip_call_procedure_yield_multiple() {
    assert_roundtrip(
        "CALL db.propertyKeys() YIELD propertyKey, type",
        "CALL db.propertyKeys() YIELD propertyKey, `type`",
    );
}

#[test]
fn roundtrip_call_procedure_yield_aliased() {
    assert_roundtrip(
        "CALL db.labels() YIELD label AS myLabel",
        "CALL db.labels() YIELD `label` AS myLabel",
    );
}

#[test]
fn roundtrip_call_procedure_yield_where() {
    assert_roundtrip(
        "CALL db.labels() YIELD label WHERE label STARTS WITH 'A'",
        "CALL db.labels() YIELD `label` WHERE `label` STARTS WITH 'A'",
    );
}

#[test]
fn roundtrip_call_procedure_yield_return() {
    assert_roundtrip(
        "CALL db.labels() YIELD label RETURN label",
        "CALL db.labels() YIELD `label` RETURN `label`",
    );
}

#[test]
fn roundtrip_call_procedure_yield_where_return() {
    assert_roundtrip(
        "CALL db.stats.retrieve('GRAPH COUNTS') YIELD section, nodeCount WHERE nodeCount > 0 RETURN section, nodeCount",
        "CALL db.stats.retrieve('GRAPH COUNTS') YIELD section, nodeCount WHERE nodeCount > 0 RETURN section, nodeCount",
    );
}

#[test]
fn roundtrip_call_security_procedure() {
    assert_roundtrip(
        "CALL dbms.security.createUser('bob', 'secret123', false)",
        "CALL dbms.security.createUser('bob', 'secret123', false)",
    );
}

#[test]
fn roundtrip_explain_call() {
    assert_roundtrip(
        "EXPLAIN CALL db.labels()",
        "EXPLAIN CALL db.labels()",
    );
}

#[test]
fn roundtrip_profile_call() {
    assert_roundtrip(
        "PROFILE CALL db.labels()",
        "PROFILE CALL db.labels()",
    );
}

// ============================================================================
// CALL subquery (in-query)
// ============================================================================

#[test]
fn roundtrip_call_return() {
    assert_roundtrip(
        "CALL { MATCH (m:Movie) RETURN m } RETURN m",
        "CALL { MATCH (m:`Movie`) RETURN m } RETURN m",
    );
}

#[test]
fn roundtrip_call_then_match() {
    assert_roundtrip(
        "CALL { MATCH (m:Movie) RETURN m } MATCH (m)-[:ACTED_IN]->(a) RETURN m, a",
        "CALL { MATCH (m:`Movie`) RETURN m } MATCH (m)-[:`ACTED_IN`]->(a) RETURN m, a",
    );
}

#[test]
fn roundtrip_call_in_transactions() {
    assert_roundtrip(
        "CALL { MATCH (m:Movie) RETURN m } IN TRANSACTIONS",
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS",
    );
}

#[test]
fn roundtrip_call_in_transactions_of_rows() {
    assert_roundtrip(
        "CALL { MATCH (m:Movie) RETURN m } IN TRANSACTIONS OF 500 ROWS",
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS OF 500 ROWS",
    );
}

#[test]
fn roundtrip_call_in_transactions_return() {
    assert_roundtrip(
        "CALL { MATCH (m:Movie) RETURN m } IN TRANSACTIONS OF 100 ROWS RETURN m",
        "CALL { MATCH (m:`Movie`) RETURN m } IN TRANSACTIONS OF 100 ROWS RETURN m",
    );
}

// ============================================================================
// LOAD CSV
// ============================================================================

#[test]
fn roundtrip_load_csv_basic() {
    assert_roundtrip(
        "LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
        "LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
    );
}

#[test]
fn roundtrip_load_csv_with_headers() {
    assert_roundtrip(
        "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row",
        "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row RETURN row",
    );
}

#[test]
fn roundtrip_load_csv_field_terminator() {
    assert_roundtrip(
        "LOAD CSV FROM 'file:///data.csv' AS row FIELDTERMINATOR ';' RETURN row",
        "LOAD CSV FROM 'file:///data.csv' AS row FIELDTERMINATOR ';' RETURN row",
    );
}

#[test]
fn roundtrip_load_csv_with_param() {
    assert_roundtrip(
        "LOAD CSV FROM $url AS row RETURN row",
        "LOAD CSV FROM $url AS row RETURN row",
    );
}

#[test]
fn roundtrip_load_csv_create() {
    assert_roundtrip(
        "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row CREATE (n:Person {`name`: row.`name`})",
        "LOAD CSV WITH HEADERS FROM 'file:///data.csv' AS row CREATE (n:`Person` {`name`: row.`name`})",
    );
}

#[test]
fn roundtrip_periodic_commit_load_csv() {
    assert_roundtrip(
        "USING PERIODIC COMMIT 500 LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
        "USING PERIODIC COMMIT 500 LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
    );
}

#[test]
fn roundtrip_periodic_commit_no_size() {
    assert_roundtrip(
        "USING PERIODIC COMMIT LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
        "USING PERIODIC COMMIT LOAD CSV FROM 'file:///data.csv' AS row RETURN row",
    );
}

// ============================================================================
// USING hints
// ============================================================================

#[test]
fn roundtrip_using_index() {
    assert_roundtrip(
        "MATCH (n:Person) USING INDEX n:Person(`name`) WHERE n.`name` = 'Alice' RETURN n",
        "MATCH (n:`Person`) USING INDEX n:`Person`(`name`) WHERE n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_using_index_seek() {
    assert_roundtrip(
        "MATCH (n:Person) USING INDEX SEEK n:Person(`name`) WHERE n.`name` = 'Alice' RETURN n",
        "MATCH (n:`Person`) USING INDEX SEEK n:`Person`(`name`) WHERE n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_using_scan() {
    assert_roundtrip(
        "MATCH (n:Person) USING SCAN n:Person WHERE n.`name` = 'Alice' RETURN n",
        "MATCH (n:`Person`) USING SCAN n:`Person` WHERE n.`name` = 'Alice' RETURN n",
    );
}

#[test]
fn roundtrip_using_join() {
    assert_roundtrip(
        "MATCH (a:Person)-[:KNOWS]->(b:Person) USING JOIN ON b RETURN a, b",
        "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) USING JOIN ON b RETURN a, b",
    );
}

// ============================================================================
// List comprehension
// ============================================================================

#[test]
fn roundtrip_list_comprehension_basic() {
    assert_parses("MATCH (n) RETURN [x IN n.list | x * 2]");
}

#[test]
fn roundtrip_list_comprehension_with_where() {
    assert_parses("MATCH (n) RETURN [x IN n.list WHERE x > 0 | x]");
}

#[test]
fn roundtrip_list_comprehension_filter_only() {
    assert_parses("MATCH (n) RETURN [x IN n.list WHERE x > 0]");
}

// ============================================================================
// Pattern comprehension
// ============================================================================

#[test]
fn roundtrip_pattern_comprehension() {
    assert_parses("MATCH (n:Person) RETURN [(n)-[:KNOWS]->(m) | m.`name`]");
}

#[test]
fn roundtrip_pattern_comprehension_with_where() {
    assert_parses("MATCH (n:Person) RETURN [(n)-[:KNOWS]->(m) WHERE m.age > 21 | m.`name`]");
}

// ============================================================================
// Map projection
// ============================================================================

#[test]
fn roundtrip_map_projection_properties() {
    assert_roundtrip(
        "MATCH (n) RETURN n {.name, .age}",
        "MATCH (n) RETURN n { .`name`, .age }",
    );
}

#[test]
fn roundtrip_map_projection_all_properties() {
    assert_roundtrip(
        "MATCH (n) RETURN n {.*}",
        "MATCH (n) RETURN n { .* }",
    );
}

#[test]
fn roundtrip_map_projection_literal_entry() {
    assert_parses("MATCH (n) RETURN n {.name, active: true}");
}

// ============================================================================
// Quantified path patterns
// ============================================================================

#[test]
fn roundtrip_quantified_path_plus() {
    assert_roundtrip(
        "MATCH ((a:Person)-[:KNOWS]->(b:Person))+ RETURN a, b",
        "MATCH ((a:`Person`)-[:`KNOWS`]->(b:`Person`))+ RETURN a, b",
    );
}

#[test]
fn roundtrip_quantified_path_star() {
    assert_roundtrip(
        "MATCH ((a)-[:R]->(b))* RETURN a, b",
        "MATCH ((a)-[:`R`]->(b))* RETURN a, b",
    );
}

#[test]
fn roundtrip_quantified_path_exact() {
    assert_roundtrip(
        "MATCH ((a)-[:R]->(b)){3} RETURN a",
        "MATCH ((a)-[:`R`]->(b)){3} RETURN a",
    );
}

#[test]
fn roundtrip_quantified_path_range() {
    assert_roundtrip(
        "MATCH ((a)-[:R]->(b)){1,5} RETURN a",
        "MATCH ((a)-[:`R`]->(b)){1,5} RETURN a",
    );
}

#[test]
fn roundtrip_quantified_path_range_open_upper() {
    assert_roundtrip(
        "MATCH ((a)-[:R]->(b)){2,} RETURN a",
        "MATCH ((a)-[:`R`]->(b)){2,} RETURN a",
    );
}

#[test]
fn roundtrip_quantified_path_with_where() {
    assert_parses("MATCH ((a)-[:R]->(b) WHERE a.x > 0)+ RETURN a");
}

#[test]
fn roundtrip_path_concatenation_chain_qpp_chain() {
    assert_parses(
        "MATCH p = (a)-[:R]->(b) ((x)-[:S]->(y)){2,5} (c)-[:T]->(d) RETURN p",
    );
}

#[test]
fn roundtrip_path_concatenation_in_fraud_ring() {
    assert_parses(
        "MATCH (a:`Account`)-[f:`SENT`]->(first_tx:`Transaction`) \
         MATCH `path` = (a)-[f:`SENT`]->(first_tx) \
         ((tx_i:`Transaction`)-[:`RECEIVED`]->(a_i:`Account`)-[:`SENT`]->(tx_j:`Transaction`) \
         WHERE tx_i.`date` < tx_j.`date` AND tx_i.amount >= tx_j.amount \
         AND tx_j.amount >= (0.8 * tx_i.amount)){2,15} \
         (last_tx:`Transaction`)-[:`RECEIVED`]->(a) \
         WHERE COUNT { WITH a, a_i UNWIND ([a] + a_i) AS b RETURN DISTINCT b } = size(([a] + a_i)) \
         RETURN COUNT { WITH a, a_i UNWIND ([a] + a_i) AS b RETURN DISTINCT b } AS ringSize, \
         a.accountNumber AS EntryAccount, `path` AS ring",
    );
}

// ============================================================================
// Quantified relationships
// ============================================================================

#[test]
fn roundtrip_quantified_rel_plus() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->+(b) RETURN a, b",
        "MATCH (a)-[:`KNOWS`]->+(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_quantified_rel_star() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->*(b) RETURN a, b",
        "MATCH (a)-[:`KNOWS`]->*(b) RETURN a, b",
    );
}

#[test]
fn roundtrip_quantified_rel_exact() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->{3}(b) RETURN a",
        "MATCH (a)-[:`KNOWS`]->{3}(b) RETURN a",
    );
}

#[test]
fn roundtrip_quantified_rel_range() {
    assert_roundtrip(
        "MATCH (a)-[:KNOWS]->{1,5}(b) RETURN a",
        "MATCH (a)-[:`KNOWS`]->{1,5}(b) RETURN a",
    );
}

// ============================================================================
// Path selectors
// ============================================================================

#[test]
fn roundtrip_shortest_k() {
    assert_roundtrip(
        "MATCH SHORTEST 1 (a)-[:R]->(b) RETURN a",
        "MATCH SHORTEST 1 (a)-[:`R`]->(b) RETURN a",
    );
}

#[test]
fn roundtrip_all_shortest() {
    assert_roundtrip(
        "MATCH ALL SHORTEST (a)-[:R]->(b) RETURN a",
        "MATCH ALL SHORTEST (a)-[:`R`]->(b) RETURN a",
    );
}

#[test]
fn roundtrip_any_path() {
    assert_roundtrip(
        "MATCH ANY (a)-[:R]->(b) RETURN a",
        "MATCH ANY (a)-[:`R`]->(b) RETURN a",
    );
}

#[test]
fn roundtrip_shortest_groups() {
    assert_roundtrip(
        "MATCH SHORTEST 2 GROUPS (a)-[:R]->(b) RETURN a",
        "MATCH SHORTEST 2 GROUPS (a)-[:`R`]->(b) RETURN a",
    );
}

// ============================================================================
// Admin commands — index management
// ============================================================================

#[test]
fn roundtrip_create_range_index() {
    assert_roundtrip(
        "CREATE INDEX person_name FOR (n:Person) ON (n.`name`)",
        "CREATE INDEX person_name FOR (n:Person) ON (n.`name`)",
    );
}

#[test]
fn roundtrip_create_range_index_composite() {
    // "composite" is a Cypher reserved keyword, so it gets backtick-escaped
    // in admin schema-name positions.
    assert_roundtrip(
        "CREATE INDEX composite FOR (n:Person) ON (n.firstName, n.lastName)",
        "CREATE INDEX `composite` FOR (n:Person) ON (n.firstName, n.lastName)",
    );
}

#[test]
fn roundtrip_create_text_index_if_not_exists() {
    assert_roundtrip(
        "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)",
        "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)",
    );
}

#[test]
fn roundtrip_create_point_index() {
    assert_roundtrip(
        "CREATE POINT INDEX loc FOR (n:Place) ON (n.location)",
        "CREATE POINT INDEX loc FOR (n:Place) ON (n.location)",
    );
}

#[test]
fn roundtrip_create_fulltext_index() {
    assert_roundtrip(
        "CREATE FULLTEXT INDEX ft FOR (n:Movie) ON EACH [n.title, n.description]",
        "CREATE FULLTEXT INDEX ft FOR (n:Movie) ON EACH [n.title, n.description]",
    );
}

#[test]
fn roundtrip_create_fulltext_multi_label() {
    assert_roundtrip(
        "CREATE FULLTEXT INDEX ft FOR (n:Movie|Book) ON EACH [n.title, n.summary]",
        "CREATE FULLTEXT INDEX ft FOR (n:Movie|Book) ON EACH [n.title, n.summary]",
    );
}

#[test]
fn roundtrip_create_lookup_index_node() {
    assert_roundtrip(
        "CREATE LOOKUP INDEX lk FOR (n) ON EACH labels(n)",
        "CREATE LOOKUP INDEX lk FOR (n) ON EACH labels(n)",
    );
}

#[test]
fn roundtrip_create_lookup_index_relationship() {
    assert_roundtrip(
        "CREATE LOOKUP INDEX lk FOR ()-[r]-() ON EACH type(r)",
        "CREATE LOOKUP INDEX lk FOR ()-[r]-() ON EACH type(r)",
    );
}

#[test]
fn roundtrip_create_index_for_relationship() {
    assert_roundtrip(
        "CREATE INDEX knows FOR ()-[r:KNOWS]-() ON (r.since)",
        "CREATE INDEX knows FOR ()-[r:KNOWS]-() ON (r.since)",
    );
}

#[test]
fn roundtrip_create_fulltext_relationship_multi_type() {
    assert_roundtrip(
        "CREATE FULLTEXT INDEX ft FOR ()-[r:KNOWS|WORKS_WITH]-() ON EACH [r.note]",
        "CREATE FULLTEXT INDEX ft FOR ()-[r:KNOWS|WORKS_WITH]-() ON EACH [r.note]",
    );
}

#[test]
fn roundtrip_drop_index() {
    assert_roundtrip("DROP INDEX my_index", "DROP INDEX my_index");
}

#[test]
fn roundtrip_drop_index_if_exists() {
    assert_roundtrip(
        "DROP INDEX my_index IF EXISTS",
        "DROP INDEX my_index IF EXISTS",
    );
}

// ============================================================================
// Admin commands — constraint management
// ============================================================================

#[test]
fn roundtrip_create_unique_constraint() {
    assert_roundtrip(
        "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE",
        "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE",
    );
}

#[test]
fn roundtrip_create_unique_constraint_composite() {
    assert_roundtrip(
        "CREATE CONSTRAINT uq FOR (n:Person) REQUIRE (n.firstName, n.lastName) IS UNIQUE",
        "CREATE CONSTRAINT uq FOR (n:Person) REQUIRE (n.firstName, n.lastName) IS UNIQUE",
    );
}

#[test]
fn roundtrip_create_existence_constraint() {
    assert_roundtrip(
        "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.`name` IS NOT NULL",
        "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.`name` IS NOT NULL",
    );
}

#[test]
fn roundtrip_create_node_key_constraint() {
    assert_roundtrip(
        "CREATE CONSTRAINT pk FOR (n:Person) REQUIRE (n.`id`, n.`name`) IS NODE KEY",
        "CREATE CONSTRAINT pk FOR (n:Person) REQUIRE (n.`id`, n.`name`) IS NODE KEY",
    );
}

#[test]
fn roundtrip_create_relationship_key_constraint() {
    assert_roundtrip(
        "CREATE CONSTRAINT rk FOR ()-[r:REVIEWED]-() REQUIRE r.`id` IS RELATIONSHIP KEY",
        "CREATE CONSTRAINT rk FOR ()-[r:REVIEWED]-() REQUIRE r.`id` IS RELATIONSHIP KEY",
    );
}

#[test]
fn roundtrip_create_property_type_constraint() {
    assert_roundtrip(
        "CREATE CONSTRAINT st FOR ()-[r:REVIEWED]-() REQUIRE r.`score` IS :: `FLOAT`",
        "CREATE CONSTRAINT st FOR ()-[r:REVIEWED]-() REQUIRE r.`score` IS :: `FLOAT`",
    );
}

#[test]
fn roundtrip_drop_constraint() {
    assert_roundtrip(
        "DROP CONSTRAINT my_constraint",
        "DROP CONSTRAINT my_constraint",
    );
}

#[test]
fn roundtrip_drop_constraint_if_exists() {
    assert_roundtrip(
        "DROP CONSTRAINT my_constraint IF EXISTS",
        "DROP CONSTRAINT my_constraint IF EXISTS",
    );
}

// ============================================================================
// Admin commands — SHOW commands
// ============================================================================

#[test]
fn roundtrip_show_indexes() {
    assert_roundtrip("SHOW INDEXES", "SHOW INDEXES");
}

#[test]
fn roundtrip_show_range_indexes_yield_all() {
    assert_roundtrip(
        "SHOW RANGE INDEXES YIELD *",
        "SHOW RANGE INDEXES YIELD *",
    );
}

#[test]
fn roundtrip_show_indexes_yield_where() {
    assert_roundtrip(
        "SHOW INDEXES YIELD `name`, `type`, state WHERE state = 'ONLINE'",
        "SHOW INDEXES YIELD `name`, `type`, state WHERE state = 'ONLINE'",
    );
}

#[test]
fn roundtrip_show_constraints() {
    assert_roundtrip("SHOW CONSTRAINTS", "SHOW CONSTRAINTS");
}

#[test]
fn roundtrip_show_unique_constraints() {
    assert_roundtrip("SHOW UNIQUE CONSTRAINTS", "SHOW UNIQUE CONSTRAINTS");
}

#[test]
fn roundtrip_show_constraints_yield_all() {
    assert_roundtrip("SHOW CONSTRAINTS YIELD *", "SHOW CONSTRAINTS YIELD *");
}

#[test]
fn roundtrip_show_functions() {
    assert_roundtrip("SHOW FUNCTIONS", "SHOW FUNCTIONS");
}

#[test]
fn roundtrip_show_built_in_functions() {
    assert_roundtrip("SHOW BUILT IN FUNCTIONS", "SHOW BUILT IN FUNCTIONS");
}

#[test]
fn roundtrip_show_user_defined_functions() {
    assert_roundtrip(
        "SHOW USER DEFINED FUNCTIONS",
        "SHOW USER DEFINED FUNCTIONS",
    );
}

#[test]
fn roundtrip_show_functions_executable_current_user() {
    assert_roundtrip(
        "SHOW FUNCTIONS EXECUTABLE BY CURRENT USER",
        "SHOW FUNCTIONS EXECUTABLE BY CURRENT USER",
    );
}

#[test]
fn roundtrip_show_functions_executable_by_user() {
    assert_roundtrip(
        "SHOW FUNCTIONS EXECUTABLE BY alice",
        "SHOW FUNCTIONS EXECUTABLE BY alice",
    );
}

#[test]
fn roundtrip_show_procedures() {
    assert_roundtrip("SHOW PROCEDURES", "SHOW PROCEDURES");
}

#[test]
fn roundtrip_show_transactions() {
    assert_roundtrip("SHOW TRANSACTIONS", "SHOW TRANSACTIONS");
}

#[test]
fn roundtrip_show_transactions_with_ids() {
    assert_roundtrip(
        "SHOW TRANSACTIONS 'neo4j-tx-123'",
        "SHOW TRANSACTIONS 'neo4j-tx-123'",
    );
}

#[test]
fn roundtrip_show_transactions_multiple_ids() {
    assert_roundtrip(
        "SHOW TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
        "SHOW TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
    );
}

// ============================================================================
// Admin commands — TERMINATE TRANSACTIONS
// ============================================================================

#[test]
fn roundtrip_terminate_transactions() {
    assert_roundtrip(
        "TERMINATE TRANSACTIONS 'neo4j-tx-123'",
        "TERMINATE TRANSACTIONS 'neo4j-tx-123'",
    );
}

#[test]
fn roundtrip_terminate_transactions_multiple() {
    assert_roundtrip(
        "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
        "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
    );
}

#[test]
fn roundtrip_terminate_with_yield() {
    assert_roundtrip(
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD *",
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD *",
    );
}

#[test]
fn roundtrip_terminate_with_yield_and_where() {
    assert_roundtrip(
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD transactionId, username WHERE username = 'bob'",
        "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD transactionId, username WHERE username = 'bob'",
    );
}

// ── Pattern-in-WHERE ──

#[test]
fn roundtrip_where_pattern_relationship() {
    assert_roundtrip(
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
    );
}

#[test]
fn roundtrip_where_not_pattern() {
    assert_roundtrip(
        "MATCH (a:`Person`) WHERE NOT (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
        "MATCH (a:`Person`) WHERE NOT (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
    );
}

#[test]
fn roundtrip_where_pattern_chain() {
    assert_roundtrip(
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`R1`]->(b:`Person`)-[:`R2`]->(c:`Person`) RETURN a",
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`R1`]->(b:`Person`)-[:`R2`]->(c:`Person`) RETURN a",
    );
}

#[test]
fn roundtrip_where_pattern_and_comparison() {
    assert_roundtrip(
        "MATCH (a:`Person`) WHERE a.age > 21 AND (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
        "MATCH (a:`Person`) WHERE a.age > 21 AND (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a",
    );
}

#[test]
fn roundtrip_where_parenthesized_expression_still_works() {
    // Ensure parenthesized expressions in WHERE are not broken
    assert_roundtrip(
        "MATCH (n:`Person`) WHERE (n.age + 1) > 2 RETURN n",
        "MATCH (n:`Person`) WHERE (n.age + 1) > 2 RETURN n",
    );
}

#[test]
fn roundtrip_where_labeled_node_pattern() {
    assert_roundtrip(
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`KNOWS`]->(b:`Actor`) RETURN a",
        "MATCH (a:`Person`) WHERE (a:`Person`)-[:`KNOWS`]->(b:`Actor`) RETURN a",
    );
}
