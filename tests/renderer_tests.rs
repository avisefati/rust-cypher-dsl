//! Integration tests for rendering configuration variations.

use pretty_assertions::assert_eq;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::renderer::{EscapeMode, RenderConfig};

// ============================================================================
// Default rendering
// ============================================================================

#[test]
fn default_render_escapes_labels() {
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    // Default rendering always backtick-escapes labels
    assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
}

// ============================================================================
// RenderConfig with EscapeMode::AsNeeded
// ============================================================================

#[test]
fn escape_as_needed_simple_label() {
    let config = RenderConfig {
        escape_names: EscapeMode::AsNeeded,
        ..RenderConfig::default()
    };
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render_with(config);
    // Simple labels don't need escaping
    assert_eq!(rendered, "MATCH (n:Person) RETURN n");
}

#[test]
fn escape_as_needed_special_label() {
    let config = RenderConfig {
        escape_names: EscapeMode::AsNeeded,
        ..RenderConfig::default()
    };
    let n = node("My Label").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render_with(config);
    // Labels with spaces need escaping
    assert!(rendered.contains("`My Label`"));
}

// ============================================================================
// Pretty printing
// ============================================================================

#[test]
fn pretty_print_match_return() {
    let config = RenderConfig {
        pretty_print: true,
        ..RenderConfig::default()
    };
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render_with(config);
    // Pretty print separates clauses with newlines
    assert!(rendered.contains('\n'));
    assert!(rendered.contains("MATCH"));
    assert!(rendered.contains("RETURN"));
}

#[test]
fn pretty_print_complex_query() {
    let config = RenderConfig {
        pretty_print: true,
        ..RenderConfig::default()
    };
    let n = node("Person").named("n");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .order_by(Expression::from(prop("n", "name")).ascending())
        .skip(5_i32)
        .limit(10_i32)
        .build();
    let rendered = stmt.render_with(config);
    assert!(rendered.contains('\n'));
    assert!(rendered.contains("MATCH"));
    assert!(rendered.contains("WHERE"));
    assert!(rendered.contains("RETURN"));
    assert!(rendered.contains("ORDER BY"));
    assert!(rendered.contains("SKIP"));
    assert!(rendered.contains("LIMIT"));
}

#[test]
fn pretty_print_custom_indent() {
    let config = RenderConfig {
        pretty_print: true,
        indent: "    ".into(), // 4 spaces
        ..RenderConfig::default()
    };
    let n = node("Person").named("n");
    let stmt = Cypher::match_(n)
        .returning(name("n"))
        .build();
    let rendered = stmt.render_with(config);
    // Should use the custom indentation
    assert!(rendered.contains('\n'));
}

// ============================================================================
// Non-pretty print is single line
// ============================================================================

#[test]
fn non_pretty_is_single_line() {
    let n = node("Person").named("n");
    let cond = prop("n", "age").gt(21_i32);
    let stmt = Cypher::match_(n)
        .where_(cond)
        .returning(name("n"))
        .order_by(Expression::from(prop("n", "name")).ascending())
        .skip(5_i32)
        .limit(10_i32)
        .build();
    let rendered = stmt.render();
    // Non-pretty render should be a single line
    assert!(!rendered.contains('\n'));
}

// ============================================================================
// Display trait uses default rendering
// ============================================================================

#[test]
fn display_trait_uses_default() {
    let stmt = Cypher::match_(node("Person").named("n"))
        .returning(name("n"))
        .build();
    let display_output = format!("{stmt}");
    let render_output = stmt.render();
    assert_eq!(display_output, render_output);
}
