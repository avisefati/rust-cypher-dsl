//! Spatial, predicate, database, graph, vector, and load-csv functions.

use crate::types::expression::Expression;

// ── Spatial ──────────────────────────────────────────────────────────────────

/// `point(map)` - creates a point from a map of coordinates.
pub fn point(map: impl Into<Expression>) -> Expression {
    Expression::function_invocation("point", vec![map.into()])
}

/// `point.distance(p1, p2)` - returns the geodesic distance between two points.
pub fn point_distance(
    p1: impl Into<Expression>,
    p2: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("point.distance", vec![p1.into(), p2.into()])
}

/// `point.withinBBox(point, lowerLeft, upperRight)` - checks if point is within bounding box.
pub fn point_within_bbox(
    point_expr: impl Into<Expression>,
    lower_left: impl Into<Expression>,
    upper_right: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation(
        "point.withinBBox",
        vec![point_expr.into(), lower_left.into(), upper_right.into()],
    )
}

// ── Predicate ────────────────────────────────────────────────────────────────

/// `exists(expr)` - returns true if the value exists (is not null).
pub fn exists(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("exists", vec![expr.into()])
}

/// `all(variable IN list WHERE predicate)` - returns true if all elements satisfy the predicate.
///
/// Produces a raw expression due to special syntax.
pub fn all_fn(variable: &str, list: impl Into<Expression>, predicate: impl Into<Expression>) -> Expression {
    let r = crate::renderer::default::DefaultRenderer::with_defaults();
    let mut buf = String::new();
    buf.push_str("all(");
    buf.push_str(variable);
    buf.push_str(" IN ");
    r.write_expression(&mut buf, &list.into());
    buf.push_str(" WHERE ");
    r.write_expression(&mut buf, &predicate.into());
    buf.push(')');
    Expression::raw(buf)
}

/// `any(variable IN list WHERE predicate)` - returns true if any element satisfies the predicate.
pub fn any_fn(variable: &str, list: impl Into<Expression>, predicate: impl Into<Expression>) -> Expression {
    let r = crate::renderer::default::DefaultRenderer::with_defaults();
    let mut buf = String::new();
    buf.push_str("any(");
    buf.push_str(variable);
    buf.push_str(" IN ");
    r.write_expression(&mut buf, &list.into());
    buf.push_str(" WHERE ");
    r.write_expression(&mut buf, &predicate.into());
    buf.push(')');
    Expression::raw(buf)
}

/// `none(variable IN list WHERE predicate)` - returns true if no element satisfies the predicate.
pub fn none_fn(variable: &str, list: impl Into<Expression>, predicate: impl Into<Expression>) -> Expression {
    let r = crate::renderer::default::DefaultRenderer::with_defaults();
    let mut buf = String::new();
    buf.push_str("none(");
    buf.push_str(variable);
    buf.push_str(" IN ");
    r.write_expression(&mut buf, &list.into());
    buf.push_str(" WHERE ");
    r.write_expression(&mut buf, &predicate.into());
    buf.push(')');
    Expression::raw(buf)
}

/// `single(variable IN list WHERE predicate)` - returns true if exactly one element satisfies.
pub fn single(variable: &str, list: impl Into<Expression>, predicate: impl Into<Expression>) -> Expression {
    let r = crate::renderer::default::DefaultRenderer::with_defaults();
    let mut buf = String::new();
    buf.push_str("single(");
    buf.push_str(variable);
    buf.push_str(" IN ");
    r.write_expression(&mut buf, &list.into());
    buf.push_str(" WHERE ");
    r.write_expression(&mut buf, &predicate.into());
    buf.push(')');
    Expression::raw(buf)
}

/// `isEmpty(expr)` - returns true if a list, map, or string is empty.
pub fn is_empty(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("isEmpty", vec![expr.into()])
}

// ── Database ─────────────────────────────────────────────────────────────────

/// `db.nameFromElementId(elementId)` - extracts the database name from an element id.
pub fn db_name_from_element_id(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("db.nameFromElementId", vec![expr.into()])
}

// ── Graph ────────────────────────────────────────────────────────────────────

/// `graph.byElementId(elementId)` - returns a graph by element id.
pub fn graph_by_element_id(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("graph.byElementId", vec![expr.into()])
}

/// `graph.byName(name)` - returns a graph by name.
pub fn graph_by_name(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("graph.byName", vec![expr.into()])
}

/// `graph.names()` - returns all graph names.
pub fn graph_names() -> Expression {
    Expression::function_invocation("graph.names", vec![])
}

/// `graph.propertiesByName(name)` - returns the properties of a graph by name.
pub fn graph_properties_by_name(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("graph.propertiesByName", vec![expr.into()])
}

// ── Vector ───────────────────────────────────────────────────────────────────

/// `vector(list)` - creates a vector from a list.
pub fn vector_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("vector", vec![expr.into()])
}

/// `vector.similarity.cosine(v1, v2)` - cosine similarity between two vectors.
pub fn vector_similarity_cosine(
    v1: impl Into<Expression>,
    v2: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("vector.similarity.cosine", vec![v1.into(), v2.into()])
}

/// `vector.similarity.euclidean(v1, v2)` - euclidean similarity between two vectors.
pub fn vector_similarity_euclidean(
    v1: impl Into<Expression>,
    v2: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("vector.similarity.euclidean", vec![v1.into(), v2.into()])
}

// ── Load CSV ─────────────────────────────────────────────────────────────────

/// `file()` - returns the absolute path of the file being loaded.
pub fn file_fn() -> Expression {
    Expression::function_invocation("file", vec![])
}

/// `linenumber()` - returns the line number being loaded.
pub fn linenumber() -> Expression {
    Expression::function_invocation("linenumber", vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(expr: &Expression) -> String {
        let r = crate::renderer::default::DefaultRenderer::with_defaults();
        let mut buf = String::new();
        r.write_expression(&mut buf, expr);
        buf
    }

    // ── Spatial ──

    #[test]
    fn render_point() {
        assert_eq!(
            render(&point(Expression::symbolic_name("m"))),
            "point(m)"
        );
    }

    #[test]
    fn render_point_distance() {
        assert_eq!(
            render(&point_distance(
                Expression::symbolic_name("p1"),
                Expression::symbolic_name("p2"),
            )),
            "point.distance(p1, p2)"
        );
    }

    #[test]
    fn render_point_within_bbox() {
        assert_eq!(
            render(&point_within_bbox(
                Expression::symbolic_name("p"),
                Expression::symbolic_name("ll"),
                Expression::symbolic_name("ur"),
            )),
            "point.withinBBox(p, ll, ur)"
        );
    }

    // ── Predicate ──

    #[test]
    fn render_exists() {
        assert_eq!(
            render(&exists(Expression::symbolic_name("n").property("name"))),
            "exists(n.name)"
        );
    }

    #[test]
    fn render_all_fn() {
        assert_eq!(
            render(&all_fn("x", Expression::symbolic_name("list"), Expression::raw("x > 0"))),
            "all(x IN list WHERE x > 0)"
        );
    }

    #[test]
    fn render_any_fn() {
        assert_eq!(
            render(&any_fn("x", Expression::symbolic_name("list"), Expression::raw("x > 0"))),
            "any(x IN list WHERE x > 0)"
        );
    }

    #[test]
    fn render_none_fn() {
        assert_eq!(
            render(&none_fn("x", Expression::symbolic_name("list"), Expression::raw("x > 0"))),
            "none(x IN list WHERE x > 0)"
        );
    }

    #[test]
    fn render_single() {
        assert_eq!(
            render(&single("x", Expression::symbolic_name("list"), Expression::raw("x = 1"))),
            "single(x IN list WHERE x = 1)"
        );
    }

    #[test]
    fn render_is_empty() {
        assert_eq!(
            render(&is_empty(Expression::symbolic_name("l"))),
            "isEmpty(l)"
        );
    }

    // ── Database ──

    #[test]
    fn render_db_name_from_element_id() {
        assert_eq!(
            render(&db_name_from_element_id(Expression::from("4:abc:123"))),
            "db.nameFromElementId('4:abc:123')"
        );
    }

    // ── Graph ──

    #[test]
    fn render_graph_by_element_id() {
        assert_eq!(
            render(&graph_by_element_id(Expression::symbolic_name("eid"))),
            "graph.byElementId(eid)"
        );
    }

    #[test]
    fn render_graph_by_name() {
        assert_eq!(
            render(&graph_by_name(Expression::from("myGraph"))),
            "graph.byName('myGraph')"
        );
    }

    #[test]
    fn render_graph_names() {
        assert_eq!(render(&graph_names()), "graph.names()");
    }

    #[test]
    fn render_graph_properties_by_name() {
        assert_eq!(
            render(&graph_properties_by_name(Expression::from("myGraph"))),
            "graph.propertiesByName('myGraph')"
        );
    }

    // ── Vector ──

    #[test]
    fn render_vector_fn() {
        assert_eq!(
            render(&vector_fn(Expression::symbolic_name("l"))),
            "vector(l)"
        );
    }

    #[test]
    fn render_vector_similarity_cosine() {
        assert_eq!(
            render(&vector_similarity_cosine(
                Expression::symbolic_name("v1"),
                Expression::symbolic_name("v2"),
            )),
            "vector.similarity.cosine(v1, v2)"
        );
    }

    #[test]
    fn render_vector_similarity_euclidean() {
        assert_eq!(
            render(&vector_similarity_euclidean(
                Expression::symbolic_name("v1"),
                Expression::symbolic_name("v2"),
            )),
            "vector.similarity.euclidean(v1, v2)"
        );
    }

    // ── Load CSV ──

    #[test]
    fn render_file_fn() {
        assert_eq!(render(&file_fn()), "file()");
    }

    #[test]
    fn render_linenumber() {
        assert_eq!(render(&linenumber()), "linenumber()");
    }
}
