//! Security integration tests — verify injection payloads are neutralized.
//!
//! Each test constructs a query with a crafted injection payload in a
//! specific input position and verifies the rendered output is safe
//! (escaped, backtick-quoted, or rejected at construction time).

#![expect(clippy::similar_names, reason = "renderer/rendered pattern is clear in tests")]

use rust_cypher_dsl::prelude::*;

// ---------------------------------------------------------------------------
// 1. String literals — single quotes are properly escaped
// ---------------------------------------------------------------------------

#[test]
fn string_literal_escapes_single_quotes() {
    let expr = lit("O'Brien");
    let stmt = Cypher::match_(node("Person").named("n"))
        .where_(prop("n", "name").eq(expr))
        .returning(name("n"))
        .build();
    let rendered = stmt.render();
    assert!(
        rendered.contains("'O''Brien'"),
        "single quote should be doubled: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 2. Symbolic names — injection payloads are backtick-escaped
// ---------------------------------------------------------------------------

#[test]
fn symbolic_name_with_spaces_is_escaped() {
    let expr = Expression::symbolic_name("evil name");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    assert_eq!(rendered, "`evil name`");
}

#[test]
fn symbolic_name_injection_is_escaped() {
    let expr = Expression::symbolic_name("n) RETURN n UNION MATCH (x");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    // The entire injection payload is wrapped in backticks
    assert_eq!(
        rendered,
        "`n) RETURN n UNION MATCH (x`",
        "injection payload should be contained in backticks"
    );
}

#[test]
fn safe_symbolic_name_is_not_escaped() {
    let expr = Expression::symbolic_name("validName");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    assert_eq!(rendered, "validName");
}

// ---------------------------------------------------------------------------
// 3. Parameter names — validation rejects invalid names
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "invalid parameter name")]
fn parameter_rejects_spaces() {
    let _p = param("bad name");
}

#[test]
#[should_panic(expected = "invalid parameter name")]
fn parameter_rejects_injection() {
    let _p = param("x} RETURN 1 //");
}

#[test]
fn parameter_accepts_valid_name() {
    let p = param("validName_123");
    assert_eq!(p.name(), "validName_123");
}

// ---------------------------------------------------------------------------
// 4. Labels — always backtick-escaped (default config)
// ---------------------------------------------------------------------------

#[test]
fn label_with_injection_payload_is_escaped() {
    let n = node("Person`) RETURN n UNION MATCH (x:`Foo").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render();
    // Backticks inside labels get doubled: ` becomes ``
    assert!(
        rendered.contains("``"),
        "backticks in label should be doubled: {rendered}"
    );
    // The label wrapping means the injection payload is inside backticks
    // and cannot break out of the label context
    assert!(
        rendered.starts_with("MATCH (n:`"),
        "label should be backtick-wrapped: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 5. Relationship types — always backtick-escaped (default config)
// ---------------------------------------------------------------------------

#[test]
fn relationship_type_injection_is_escaped() {
    let a = node("A").named("a");
    let b = node("B").named("b");
    let pattern = a.rel(rel("KNOWS`]->(x) DETACH DELETE x //")).to(b);
    let stmt = Cypher::match_(pattern)
        .returning(name("a"))
        .build();
    let rendered = stmt.render();
    // Backtick in the type name gets doubled, whole type wrapped in backticks
    assert!(
        rendered.contains("``"),
        "backtick in type should be doubled: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 6. Property names — injection payloads are escaped
// ---------------------------------------------------------------------------

#[test]
fn property_name_with_spaces_is_escaped() {
    let expr = prop("n", "bad name");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&Expression::from(expr));
    assert!(
        rendered.contains("`bad name`"),
        "property name should be escaped: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 7. Alias names — injection payloads are escaped
// ---------------------------------------------------------------------------

#[test]
fn alias_injection_is_escaped() {
    let expr = name("n").alias("x RETURN 1 UNION MATCH (y");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    // Injection payload is contained inside backticks after AS
    assert_eq!(
        rendered,
        "n AS `x RETURN 1 UNION MATCH (y`",
        "alias injection should be backtick-escaped"
    );
}

// ---------------------------------------------------------------------------
// 8. Map literal keys — injection payloads are escaped
// ---------------------------------------------------------------------------

#[test]
fn map_key_injection_is_escaped() {
    let expr = map_of(vec![("}: 1} RETURN 1 //".into(), lit(1_i32))]);
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    assert!(
        rendered.contains('`'),
        "map key with injection should be backtick-escaped: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 9. Procedure names — validation rejects injection payloads
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "invalid procedure name")]
fn procedure_rejects_injection() {
    use rust_cypher_dsl::clauses::CallClause;
    let _c = CallClause::new("db.labels() YIELD label RETURN label //", vec![]);
}

#[test]
fn procedure_accepts_dotted_name() {
    use rust_cypher_dsl::clauses::CallClause;
    let c = CallClause::new("db.index.fulltext.queryNodes", vec![]);
    assert_eq!(c.procedure(), "db.index.fulltext.queryNodes");
}

// ---------------------------------------------------------------------------
// 10. Function names — validation rejects injection payloads
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "invalid function name")]
fn function_name_rejects_injection() {
    let _e = Expression::function_invocation("count() RETURN 1 //", vec![]);
}

#[test]
fn function_name_accepts_dotted_name() {
    let e = Expression::function_invocation("db.custom.fn", vec![lit(1_i32)]);
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&e);
    assert_eq!(rendered, "db.custom.fn(1)");
}

// ---------------------------------------------------------------------------
// 11. LOAD CSV field terminator — single quotes are escaped
// ---------------------------------------------------------------------------

#[test]
fn load_csv_field_terminator_quote_escaped() {
    use rust_cypher_dsl::clauses::{Clause, LoadCsvClause, ReturnClause};
    use rust_cypher_dsl::statement::{SinglePartQuery, Statement};

    let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
        Clause::LoadCsv(
            LoadCsvClause::new(Expression::from(Parameter::new("url")), "row")
                .field_terminator("'"),
        ),
        Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("row")])),
    ]));
    let rendered = stmt.render();
    assert!(
        rendered.contains("FIELDTERMINATOR ''''"),
        "single quote in terminator should be doubled: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 12. raw_unchecked — renders verbatim (documented risk)
// ---------------------------------------------------------------------------

#[test]
fn raw_unchecked_renders_verbatim() {
    let expr = raw_unchecked("anything goes here");
    let renderer = rust_cypher_dsl::renderer::default::DefaultRenderer::with_defaults();
    let rendered = renderer.render_expression(&expr);
    assert_eq!(rendered, "anything goes here");
}

// ---------------------------------------------------------------------------
// 13. Node variable name — injection escaped
// ---------------------------------------------------------------------------

#[test]
fn node_variable_injection_escaped() {
    let n = node("Person").named("n) DETACH DELETE n //");
    let stmt = Cypher::match_(n)
        .returning(name("x"))
        .build();
    let rendered = stmt.render();
    // The injection payload in the node variable is backtick-escaped
    assert!(
        rendered.contains("`n) DETACH DELETE n //`"),
        "node variable injection should be backtick-escaped: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 14. Relationship variable name — injection escaped
// ---------------------------------------------------------------------------

#[test]
fn relationship_variable_injection_escaped() {
    let a = node("A").named("a");
    let b = node("B").named("b");
    let pattern = a.rel(rel("KNOWS").named("r] RETURN r //")).to(b);
    let stmt = Cypher::match_(pattern)
        .returning(name("a"))
        .build();
    let rendered = stmt.render();
    // The injection payload in the rel variable is backtick-escaped
    assert!(
        rendered.contains("`r] RETURN r //`"),
        "relationship variable injection should be backtick-escaped: {rendered}"
    );
}

// ===========================================================================
// 15. Admin index — name injection rejected at construction
// ===========================================================================

#[test]
#[should_panic(expected = "invalid index name")]
fn admin_index_name_injection_rejected() {
    let _stmt = Cypher::create_index("idx IF EXISTS")
        .for_node("n", "Person", vec!["name"])
        .build();
}

#[test]
#[should_panic(expected = "invalid index name")]
fn admin_index_name_smuggle_rejected() {
    let _stmt = Cypher::drop_index("idx IF EXISTS");
}

// ===========================================================================
// 16. Admin index — variable injection rejected at construction
// ===========================================================================

#[test]
#[should_panic(expected = "invalid index variable")]
fn admin_index_variable_injection_rejected() {
    let _stmt = Cypher::create_index("idx")
        .for_node("n) DELETE n //", "Person", vec!["name"])
        .build();
}

// ===========================================================================
// 17. Admin index — property injection rejected at construction
// ===========================================================================

#[test]
#[should_panic(expected = "invalid index property")]
fn admin_index_property_injection_rejected() {
    let _stmt = Cypher::create_index("idx")
        .for_node("n", "Person", vec!["name) //"])
        .build();
}

// ===========================================================================
// 18. Admin index — label injection escaped by renderer
// ===========================================================================

#[test]
fn admin_index_label_injection_escaped() {
    // Labels are permissive at construction but escaped at render time.
    let stmt = Cypher::create_index("idx")
        .for_node("n", "Person) ON (n.x) //", vec!["name"])
        .build();
    let rendered = stmt.render();
    assert!(
        rendered.contains('`'),
        "label injection should be backtick-escaped: {rendered}"
    );
    // The injected label is safely contained inside backticks.
    assert!(
        rendered.contains("`Person) ON (n.x) //`"),
        "injection payload should be wrapped: {rendered}"
    );
}

// ===========================================================================
// 19. Admin constraint — name injection rejected
// ===========================================================================

#[test]
#[should_panic(expected = "invalid constraint name")]
fn admin_constraint_name_injection_rejected() {
    let _stmt = Cypher::create_constraint("c IF EXISTS")
        .for_node("n", "Person")
        .is_unique(vec!["email"]);
}

#[test]
#[should_panic(expected = "invalid constraint name")]
fn admin_constraint_name_smuggle_rejected() {
    let _stmt = Cypher::drop_constraint("c IF EXISTS");
}

// ===========================================================================
// 20. Admin constraint — variable injection rejected
// ===========================================================================

#[test]
#[should_panic(expected = "invalid constraint variable")]
fn admin_constraint_variable_injection_rejected() {
    let _stmt = Cypher::create_constraint("c")
        .for_node("n) DELETE n //", "Person")
        .is_unique(vec!["email"]);
}

// ===========================================================================
// 21. Admin constraint — property injection rejected
// ===========================================================================

#[test]
#[should_panic(expected = "invalid constraint property")]
fn admin_constraint_property_injection_rejected() {
    let _stmt = Cypher::create_constraint("c")
        .for_node("n", "Person")
        .is_unique(vec!["email) //"]);
}

// ===========================================================================
// 22. Admin constraint — label injection escaped by renderer
// ===========================================================================

#[test]
fn admin_constraint_label_injection_escaped() {
    let stmt = Cypher::create_constraint("c")
        .for_node("n", "Person) DELETE n //")
        .is_unique(vec!["email"]);
    let rendered = stmt.render();
    assert!(
        rendered.contains("`Person) DELETE n //`"),
        "label injection should be backtick-escaped: {rendered}"
    );
}

// ===========================================================================
// 23. Drop index/constraint — name smuggling rejected
// ===========================================================================

#[test]
#[should_panic(expected = "invalid index name")]
fn drop_index_name_smuggle_rejected() {
    let _stmt = Cypher::drop_index_if_exists("idx IF EXISTS");
}

#[test]
#[should_panic(expected = "invalid constraint name")]
fn drop_constraint_name_smuggle_rejected() {
    let _stmt = Cypher::drop_constraint_if_exists("c IF EXISTS");
}

// ===========================================================================
// 24. Transaction ID — quote breakout rejected at construction
// ===========================================================================

#[test]
#[should_panic(expected = "invalid transaction ID")]
fn show_transaction_id_quote_breakout_rejected() {
    let _stmt = Cypher::show_transactions()
        .ids(vec!["neo4j-tx-1' YIELD * WHERE username = 'alice"])
        .build();
}

#[test]
#[should_panic(expected = "invalid transaction ID")]
fn terminate_transaction_id_quote_breakout_rejected() {
    let _stmt = Cypher::terminate_transactions(vec![
        "neo4j-tx-1' YIELD * WHERE username = 'alice",
    ])
    .build();
}

// ===========================================================================
// 25. Transaction ID — backslash rejected
// ===========================================================================

#[test]
#[should_panic(expected = "invalid transaction ID")]
fn transaction_id_backslash_rejected() {
    let _stmt = Cypher::terminate_transactions(vec!["tx\\injection"]).build();
}

// ===========================================================================
// 26. Username — injection rejected at construction
// ===========================================================================

#[test]
#[should_panic(expected = "invalid username")]
fn executable_by_username_injection_rejected() {
    let _stmt = Cypher::show_functions()
        .executable_by("alice YIELD * WHERE true")
        .build();
}

#[test]
#[should_panic(expected = "invalid username")]
fn executable_by_procedures_username_injection_rejected() {
    let _stmt = Cypher::show_procedures()
        .executable_by("alice YIELD *")
        .build();
}

// ===========================================================================
// 27. Property type name — injection escaped by renderer
// ===========================================================================

#[test]
fn property_type_injection_escaped() {
    let stmt = Cypher::create_constraint("c")
        .for_relationship("r", "REVIEWED")
        .is_typed("score", "FLOAT) //");
    let rendered = stmt.render();
    assert!(
        rendered.contains("`FLOAT) //`"),
        "type name injection should be backtick-escaped: {rendered}"
    );
}

// ===========================================================================
// 28. Transaction ID rendering — special chars properly escaped
// ===========================================================================

#[test]
fn transaction_id_with_special_chars_safe() {
    // Transaction IDs with dashes and dots are valid and safe.
    let stmt = Cypher::show_transactions()
        .ids(vec!["neo4j-tx-123.abc"])
        .build();
    let rendered = stmt.render();
    assert_eq!(rendered, "SHOW TRANSACTIONS 'neo4j-tx-123.abc'");
}

// ===========================================================================
// 29. Admin index — reserved keyword as variable is escaped by renderer
// ===========================================================================

#[test]
fn admin_reserved_keyword_as_index_name_escaped() {
    // Using a reserved keyword as an index name should be backtick-escaped
    // by write_admin_identifier() at render time.
    // Note: reserved keywords pass is_valid_identifier (character shape is OK),
    // but the renderer escapes them in admin positions.
    let stmt = Cypher::create_index("MATCH")
        .for_node("n", "Person", vec!["name"])
        .build();
    let rendered = stmt.render();
    assert!(
        rendered.contains("`MATCH`"),
        "reserved keyword as index name should be backtick-escaped: {rendered}"
    );
}
