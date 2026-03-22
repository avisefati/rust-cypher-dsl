//! Integration tests for Cypher functions — ported from Java `FunctionsIT.java`.
//!
//! Tests exercise built-in function invocations via the fluent API,
//! verifying rendered Cypher output.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::functions::{aggregate, list, math, scalar, spatial, string, temporal};
use rust_cypher_dsl::prelude::*;

/// Helper: render an expression in a simple `RETURN expr` statement.
fn render_expr(expr: Expression) -> String {
    Cypher::match_(node("N").named("n"))
        .returning(expr)
        .build()
        .render()
}

// ============================================================================
// Aggregation functions
// ============================================================================

#[test]
fn count_expression() {
    let expr = aggregate::count(name("n"));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN count(n)");
}

#[test]
fn count_distinct_expression() {
    let expr = aggregate::count_distinct(name("n"));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN count(DISTINCT n)"
    );
}

#[test]
fn count_asterisk() {
    let expr = aggregate::count(Expression::asterisk());
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN count(*)");
}

#[test]
fn sum_expression() {
    let expr = aggregate::sum(Expression::from(prop("n", "age")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN sum(n.age)");
}

#[test]
fn avg_expression() {
    let expr = aggregate::avg(Expression::from(prop("n", "age")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN avg(n.age)");
}

#[test]
fn min_expression() {
    let expr = aggregate::min(Expression::from(prop("n", "age")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN min(n.age)");
}

#[test]
fn max_expression() {
    let expr = aggregate::max(Expression::from(prop("n", "age")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN max(n.age)");
}

#[test]
fn collect_expression() {
    let expr = aggregate::collect(Expression::from(prop("n", "name")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN collect(n.`name`)"
    );
}

#[test]
fn collect_distinct_expression() {
    let expr = aggregate::collect_distinct(Expression::from(prop("n", "city")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN collect(DISTINCT n.city)"
    );
}

#[test]
fn percentile_cont_expression() {
    let expr = aggregate::percentile_cont(Expression::from(prop("n", "age")), lit(0.5_f64));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN percentileCont(n.age, 0.5)"
    );
}

#[test]
fn st_dev_expression() {
    let expr = aggregate::st_dev(Expression::from(prop("n", "score")));
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(expr)
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN stDev(n.`score`)"
    );
}

// ============================================================================
// Scalar functions
// ============================================================================

#[test]
fn id_function() {
    let expr = scalar::id(name("n"));
    assert!(render_expr(expr).contains("id(n)"));
}

#[test]
fn element_id_function() {
    let expr = scalar::element_id(name("n"));
    assert!(render_expr(expr).contains("elementId(n)"));
}

#[test]
fn type_of_function() {
    let expr = scalar::type_of(name("r"));
    assert!(render_expr(expr).contains("type(r)"));
}

#[test]
fn coalesce_function() {
    let expr = scalar::coalesce(vec![
        Expression::from(prop("n", "nickname")),
        Expression::from(prop("n", "name")),
    ]);
    assert!(render_expr(expr).contains("coalesce(n.nickname, n.`name`)"));
}

#[test]
fn timestamp_function() {
    let expr = scalar::timestamp();
    assert!(render_expr(expr).contains("timestamp()"));
}

#[test]
fn size_function() {
    let expr = scalar::size(name("list"));
    assert!(render_expr(expr).contains("size(`list`)"));
}

#[test]
fn head_function() {
    let expr = scalar::head(name("list"));
    assert!(render_expr(expr).contains("head(`list`)"));
}

#[test]
fn last_function() {
    let expr = scalar::last(name("list"));
    assert!(render_expr(expr).contains("last(`list`)"));
}

#[test]
fn properties_function() {
    let expr = scalar::properties(name("n"));
    assert!(render_expr(expr).contains("properties(n)"));
}

#[test]
fn to_integer_function() {
    let expr = scalar::to_integer(lit("42"));
    assert!(render_expr(expr).contains("toInteger('42')"));
}

#[test]
fn to_float_function() {
    let expr = scalar::to_float(lit("3.14"));
    assert!(render_expr(expr).contains("toFloat('3.14')"));
}

#[test]
fn to_boolean_function() {
    let expr = scalar::to_boolean(lit("true"));
    assert!(render_expr(expr).contains("toBoolean('true')"));
}

#[test]
fn to_string_function() {
    let expr = scalar::to_string_fn(lit(42_i32));
    assert!(render_expr(expr).contains("toString(42)"));
}

// ============================================================================
// String functions
// ============================================================================

#[test]
fn to_lower_function() {
    let expr = string::to_lower(Expression::from(prop("n", "name")));
    assert!(render_expr(expr).contains("toLower(n.`name`)"));
}

#[test]
fn to_upper_function() {
    let expr = string::to_upper(Expression::from(prop("n", "name")));
    assert!(render_expr(expr).contains("toUpper(n.`name`)"));
}

#[test]
fn trim_function() {
    let expr = string::trim(lit("  hello  "));
    assert!(render_expr(expr).contains("trim('  hello  ')"));
}

#[test]
fn replace_function() {
    let expr = string::replace(Expression::from(prop("n", "name")), lit("a"), lit("b"));
    assert!(render_expr(expr).contains("replace(n.`name`, 'a', 'b')"));
}

#[test]
fn substring_function() {
    let expr = string::substring(Expression::from(prop("n", "name")), lit(0_i32), Some(lit(3_i32)));
    assert!(render_expr(expr).contains("substring(n.`name`, 0, 3)"));
}

#[test]
fn split_function() {
    let expr = string::split(Expression::from(prop("n", "name")), lit(","));
    assert!(render_expr(expr).contains("split(n.`name`, ',')"));
}

#[test]
fn reverse_str_function() {
    let expr = string::reverse_str(lit("hello"));
    assert!(render_expr(expr).contains("reverse('hello')"));
}

#[test]
fn left_function() {
    let expr = string::left(Expression::from(prop("n", "name")), lit(3_i32));
    assert!(render_expr(expr).contains("left(n.`name`, 3)"));
}

#[test]
fn right_function() {
    let expr = string::right(Expression::from(prop("n", "name")), lit(3_i32));
    assert!(render_expr(expr).contains("right(n.`name`, 3)"));
}

// ============================================================================
// Math functions
// ============================================================================

#[test]
fn abs_function() {
    let expr = math::abs(lit(-42_i32));
    assert!(render_expr(expr).contains("abs(-42)"));
}

#[test]
fn ceil_function() {
    let expr = math::ceil(lit(2.3_f64));
    assert!(render_expr(expr).contains("ceil(2.3)"));
}

#[test]
fn floor_function() {
    let expr = math::floor(lit(2.7_f64));
    assert!(render_expr(expr).contains("floor(2.7)"));
}

#[test]
fn round_function() {
    let expr = math::round(lit(2.5_f64));
    assert!(render_expr(expr).contains("round(2.5)"));
}

#[test]
fn sqrt_function() {
    let expr = math::sqrt(lit(16_i32));
    assert!(render_expr(expr).contains("sqrt(16)"));
}

#[test]
fn rand_function() {
    let expr = math::rand();
    assert!(render_expr(expr).contains("rand()"));
}

#[test]
fn sign_function() {
    let expr = math::sign(lit(-5_i32));
    assert!(render_expr(expr).contains("sign(-5)"));
}

#[test]
fn log10_function() {
    let expr = math::log10(lit(100_i32));
    assert!(render_expr(expr).contains("log10(100)"));
}

#[test]
fn sin_function() {
    let expr = math::sin(lit(1_i32));
    assert!(render_expr(expr).contains("sin(1)"));
}

#[test]
fn pi_function() {
    let expr = math::pi();
    assert!(render_expr(expr).contains("pi()"));
}

// ============================================================================
// List functions
// ============================================================================

#[test]
fn range_function() {
    let expr = list::range(lit(0_i32), lit(10_i32));
    assert!(render_expr(expr).contains("range(0, 10)"));
}

#[test]
fn range_with_step_function() {
    let expr = list::range_with_step(lit(0_i32), lit(10_i32), lit(2_i32));
    assert!(render_expr(expr).contains("range(0, 10, 2)"));
}

#[test]
fn keys_function() {
    let expr = list::keys(name("n"));
    assert!(render_expr(expr).contains("keys(n)"));
}

#[test]
fn labels_function() {
    let expr = list::labels_fn(name("n"));
    assert!(render_expr(expr).contains("labels(n)"));
}

#[test]
fn nodes_function() {
    let expr = list::nodes_fn(name("p"));
    assert!(render_expr(expr).contains("nodes(p)"));
}

#[test]
fn relationships_function() {
    let expr = list::relationships_fn(name("p"));
    assert!(render_expr(expr).contains("relationships(p)"));
}

#[test]
fn tail_function() {
    let expr = list::tail(name("list"));
    assert!(render_expr(expr).contains("tail(`list`)"));
}

// ============================================================================
// Temporal functions
// ============================================================================

#[test]
fn datetime_function() {
    let expr = temporal::datetime_fn(vec![]);
    assert!(render_expr(expr).contains("datetime()"));
}

#[test]
fn date_function() {
    let expr = temporal::date_fn(vec![]);
    assert!(render_expr(expr).contains("date()"));
}

#[test]
fn duration_function() {
    let expr = temporal::duration_fn(vec![lit("P14DT16H12M")]);
    assert!(render_expr(expr).contains("duration('P14DT16H12M')"));
}

#[test]
fn datetime_realtime_function() {
    let expr = temporal::datetime_realtime();
    assert!(render_expr(expr).contains("datetime.realtime()"));
}

// ============================================================================
// Spatial functions
// ============================================================================

#[test]
fn point_function() {
    let expr = spatial::point(map_of(vec![
        ("x".into(), lit(1.0_f64)),
        ("y".into(), lit(2.0_f64)),
    ]));
    assert!(render_expr(expr).contains("point("));
}

#[test]
fn point_distance_function() {
    let p1 = spatial::point(map_of(vec![("x".into(), lit(0_i32)), ("y".into(), lit(0_i32))]));
    let p2 = spatial::point(map_of(vec![("x".into(), lit(3_i32)), ("y".into(), lit(4_i32))]));
    let expr = spatial::point_distance(p1, p2);
    assert!(render_expr(expr).contains("point.distance("));
}

// ============================================================================
// Predicate functions
// ============================================================================

#[test]
fn exists_function() {
    let expr = spatial::exists(Expression::from(prop("n", "name")));
    assert!(render_expr(expr).contains("exists(n.`name`)"));
}

#[test]
fn is_empty_function() {
    let expr = spatial::is_empty(name("list"));
    assert!(render_expr(expr).contains("isEmpty(`list`)"));
}

// ============================================================================
// Custom functions
// ============================================================================

#[test]
fn custom_function_invocation() {
    let expr = rust_cypher_dsl::functions::custom_function(
        "apoc.text.join",
        vec![name("list"), lit(", ")],
    );
    assert!(render_expr(expr).contains("apoc.text.join(`list`, ', ')"));
}

// ============================================================================
// Function in full query context
// ============================================================================

#[test]
fn count_in_with_clause() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .with(aggregate::count(name("n")).alias("total"))
        .returning(name("total"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) WITH count(n) AS total RETURN total"
    );
}

#[test]
fn function_aliased_in_return() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(aggregate::collect(Expression::from(prop("n", "name"))).alias("names"))
        .build();
    assert_eq!(
        stmt.render(),
        "MATCH (n:`Person`) RETURN collect(n.`name`) AS names"
    );
}
