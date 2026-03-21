//! Integration tests for Expression types — ported from Java `ExpressionsIT.java`.
//!
//! Tests exercise expression construction and rendering: literals, CASE,
//! list comprehensions, pattern comprehensions, map projections, and more.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::types::expression::MapProjectionEntry;

/// Render a MATCH + RETURN with a single expression.
fn render_return(expr: Expression) -> String {
    Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build()
        .render()
}

// ============================================================================
// Literal expressions
// ============================================================================

#[test]
fn string_literal() {
    let expr = lit("hello");
    assert!(render_return(expr).contains("'hello'"));
}

#[test]
fn integer_literal() {
    let expr = lit(42_i32);
    assert!(render_return(expr).contains("42"));
}

#[test]
fn float_literal() {
    let expr = lit(9.81_f64);
    assert!(render_return(expr).contains("9.81"));
}

#[test]
fn boolean_literal() {
    let expr = lit(true);
    assert!(render_return(expr).contains("true"));
}

#[test]
fn null_literal() {
    let expr = Expression::null_literal();
    assert!(render_return(expr).contains("NULL"));
}

#[test]
fn list_literal() {
    let expr = list_of(vec![lit(1_i32), lit(2_i32), lit(3_i32)]);
    assert!(render_return(expr).contains("[1, 2, 3]"));
}

#[test]
fn map_literal() {
    let expr = map_of(vec![("name".into(), lit("Alice"))]);
    assert!(render_return(expr).contains("{name: 'Alice'}"));
}

// ============================================================================
// Arithmetic operations
// ============================================================================

#[test]
fn addition() {
    let expr = lit(1_i32).add(lit(2_i32));
    assert!(render_return(expr).contains("(1 + 2)"));
}

#[test]
fn subtraction() {
    let expr = lit(10_i32).subtract(lit(3_i32));
    assert!(render_return(expr).contains("(10 - 3)"));
}

#[test]
fn multiplication() {
    let expr = lit(4_i32).multiply(lit(5_i32));
    assert!(render_return(expr).contains("(4 * 5)"));
}

#[test]
fn division() {
    let expr = lit(10_i32).divide(lit(2_i32));
    assert!(render_return(expr).contains("(10 / 2)"));
}

#[test]
fn remainder_operation() {
    let expr = lit(10_i32).remainder(lit(3_i32));
    assert!(render_return(expr).contains("(10 % 3)"));
}

#[test]
fn power() {
    let expr = lit(2_i32).pow(lit(3_i32));
    assert!(render_return(expr).contains("(2 ^ 3)"));
}

// ============================================================================
// Comparison operations
// ============================================================================

#[test]
fn equality() {
    let expr = Expression::from(name("n").eq(lit(42_i32)));
    assert!(render_return(expr).contains("n = 42"));
}

#[test]
fn inequality() {
    let expr = Expression::from(name("n").ne(lit(42_i32)));
    assert!(render_return(expr).contains("n <> 42"));
}

#[test]
fn less_than() {
    let expr = Expression::from(name("n").lt(lit(42_i32)));
    assert!(render_return(expr).contains("n < 42"));
}

#[test]
fn greater_than() {
    let expr = Expression::from(name("n").gt(lit(42_i32)));
    assert!(render_return(expr).contains("n > 42"));
}

#[test]
fn less_than_or_equal() {
    let expr = Expression::from(name("n").lte(lit(42_i32)));
    assert!(render_return(expr).contains("n <= 42"));
}

#[test]
fn greater_than_or_equal() {
    let expr = Expression::from(name("n").gte(lit(42_i32)));
    assert!(render_return(expr).contains("n >= 42"));
}

// ============================================================================
// Aliasing
// ============================================================================

#[test]
fn expression_aliased() {
    let expr = Expression::from(prop("n", "name")).alias("personName");
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN n.name AS personName"
    );
}

// ============================================================================
// CASE expressions
// ============================================================================

#[test]
fn simple_case_expression() {
    let expr = Expression::simple_case(
        Expression::from(prop("n", "eyes")),
        vec![
            (lit("blue"), lit("b")),
            (lit("brown"), lit("br")),
        ],
        Some(lit("other")),
    );
    let rendered = render_return(expr);
    assert!(rendered.contains("CASE n.eyes"));
    assert!(rendered.contains("WHEN 'blue' THEN 'b'"));
    assert!(rendered.contains("WHEN 'brown' THEN 'br'"));
    assert!(rendered.contains("ELSE 'other'"));
    assert!(rendered.contains("END"));
}

#[test]
fn generic_case_expression() {
    let expr = Expression::generic_case(
        vec![(
            Expression::from(prop("n", "age").lt(18_i32)),
            lit("minor"),
        )],
        Some(lit("adult")),
    );
    let rendered = render_return(expr);
    assert!(rendered.contains("CASE"));
    assert!(rendered.contains("WHEN n.age < 18 THEN 'minor'"));
    assert!(rendered.contains("ELSE 'adult'"));
    assert!(rendered.contains("END"));
}

// ============================================================================
// List comprehension
// ============================================================================

#[test]
fn list_comprehension() {
    let expr = Expression::list_comprehension("x", name("list"), None, Some(name("x").multiply(lit(2_i32))));
    let rendered = render_return(expr);
    assert!(rendered.contains("[x IN list | (x * 2)]"));
}

#[test]
fn list_comprehension_with_where() {
    let where_cond = Expression::from(name("x").gt(0_i32));
    let expr = Expression::list_comprehension("x", name("list"), Some(where_cond), Some(name("x")));
    let rendered = render_return(expr);
    assert!(rendered.contains("[x IN list WHERE x > 0 | x]"));
}

// ============================================================================
// Map projection
// ============================================================================

#[test]
fn map_projection_simple() {
    let expr = Expression::map_projection(
        name("n"),
        vec![
            MapProjectionEntry::Property("name".into()),
            MapProjectionEntry::Property("age".into()),
        ],
    );
    let rendered = render_return(expr);
    assert!(rendered.contains("n { .name, .age }"));
}

// ============================================================================
// Raw expression
// ============================================================================

#[test]
fn raw_expression() {
    let expr = raw_unchecked("n.name + ' ' + n.surname");
    assert!(render_return(expr).contains("n.name + ' ' + n.surname"));
}

// ============================================================================
// Property access chaining
// ============================================================================

#[test]
fn property_access() {
    let expr = Expression::from(prop("n", "address")).property("city");
    let rendered = render_return(Expression::from(expr));
    assert!(rendered.contains("n.address.city"));
}
