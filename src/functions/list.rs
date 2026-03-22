//! List and coll namespace functions.

use crate::types::expression::Expression;

// ── List functions ───────────────────────────────────────────────────────────

/// `range(start, end)` - generates a list of integers from start to end.
pub fn range(
    start: impl Into<Expression>,
    end: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("range", vec![start.into(), end.into()])
}

/// `range(start, end, step)` - generates a list with a given step.
pub fn range_with_step(
    start: impl Into<Expression>,
    end: impl Into<Expression>,
    step: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("range", vec![start.into(), end.into(), step.into()])
}

/// `keys(expr)` - returns the keys of a map or the property names of a node/relationship.
pub fn keys(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("keys", vec![expr.into()])
}

/// `labels(node)` - returns the labels of a node.
pub fn labels_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("labels", vec![expr.into()])
}

/// `nodes(path)` - returns the nodes in a path.
pub fn nodes_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("nodes", vec![expr.into()])
}

/// `relationships(path)` - returns the relationships in a path.
pub fn relationships_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("relationships", vec![expr.into()])
}

/// `tail(list)` - returns all elements except the first.
pub fn tail(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("tail", vec![expr.into()])
}

/// `reverse(list)` - reverses a list.
pub fn reverse_list(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("reverse", vec![expr.into()])
}

/// `reduce(accumulator = initial, variable IN list | expression)` - reduces a list.
///
/// This produces a raw expression because `reduce` has special syntax.
pub fn reduce_fn(
    accumulator: &str,
    initial: impl Into<Expression>,
    variable: &str,
    list: impl Into<Expression>,
    expression: impl Into<Expression>,
) -> Expression {
    let r = crate::renderer::default::DefaultRenderer::with_defaults();
    let mut buf = String::new();
    buf.push_str("reduce(");
    buf.push_str(accumulator);
    buf.push_str(" = ");
    r.write_expression(&mut buf, &initial.into());
    buf.push_str(", ");
    buf.push_str(variable);
    buf.push_str(" IN ");
    r.write_expression(&mut buf, &list.into());
    buf.push_str(" | ");
    r.write_expression(&mut buf, &expression.into());
    buf.push(')');
    Expression::raw_unchecked(buf)
}

/// `toBooleanList(list)` - converts each element to boolean.
pub fn to_boolean_list(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toBooleanList", vec![expr.into()])
}

/// `toFloatList(list)` - converts each element to float.
pub fn to_float_list(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toFloatList", vec![expr.into()])
}

/// `toIntegerList(list)` - converts each element to integer.
pub fn to_integer_list(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toIntegerList", vec![expr.into()])
}

/// `toStringList(list)` - converts each element to string.
pub fn to_string_list(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toStringList", vec![expr.into()])
}

// ── coll namespace ───────────────────────────────────────────────────────────

/// `coll.distinct(list)` - returns distinct elements.
pub fn coll_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coll.distinct", vec![expr.into()])
}

/// `coll.flatten(list)` - flattens nested lists.
pub fn coll_flatten(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coll.flatten", vec![expr.into()])
}

/// `coll.indexOf(list, value)` - returns the index of a value.
pub fn coll_index_of(
    list: impl Into<Expression>,
    value: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("coll.indexOf", vec![list.into(), value.into()])
}

/// `coll.insert(list, index, value)` - inserts a value at an index.
pub fn coll_insert(
    list: impl Into<Expression>,
    index: impl Into<Expression>,
    value: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("coll.insert", vec![list.into(), index.into(), value.into()])
}

/// `coll.max(list)` - returns the maximum element.
pub fn coll_max(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coll.max", vec![expr.into()])
}

/// `coll.min(list)` - returns the minimum element.
pub fn coll_min(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coll.min", vec![expr.into()])
}

/// `coll.remove(list, index)` - removes the element at an index.
pub fn coll_remove(
    list: impl Into<Expression>,
    index: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("coll.remove", vec![list.into(), index.into()])
}

/// `coll.sort(list)` - sorts a list.
pub fn coll_sort(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coll.sort", vec![expr.into()])
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

    // ── List functions ──

    #[test]
    fn render_range() {
        assert_eq!(
            render(&range(Expression::from(0_i32), Expression::from(10_i32))),
            "range(0, 10)"
        );
    }

    #[test]
    fn render_range_with_step() {
        assert_eq!(
            render(&range_with_step(
                Expression::from(0_i32),
                Expression::from(10_i32),
                Expression::from(2_i32),
            )),
            "range(0, 10, 2)"
        );
    }

    #[test]
    fn render_keys() {
        assert_eq!(render(&keys(Expression::symbolic_name("n"))), "keys(n)");
    }

    #[test]
    fn render_labels_fn() {
        assert_eq!(render(&labels_fn(Expression::symbolic_name("n"))), "labels(n)");
    }

    #[test]
    fn render_nodes_fn() {
        assert_eq!(render(&nodes_fn(Expression::symbolic_name("p"))), "nodes(p)");
    }

    #[test]
    fn render_relationships_fn() {
        assert_eq!(
            render(&relationships_fn(Expression::symbolic_name("p"))),
            "relationships(p)"
        );
    }

    #[test]
    fn render_tail() {
        assert_eq!(render(&tail(Expression::symbolic_name("l"))), "tail(l)");
    }

    #[test]
    fn render_reverse_list() {
        assert_eq!(
            render(&reverse_list(Expression::symbolic_name("l"))),
            "reverse(l)"
        );
    }

    #[test]
    fn render_reduce_fn() {
        let expr = reduce_fn(
            "total",
            Expression::from(0_i32),
            "x",
            Expression::symbolic_name("list"),
            Expression::raw_unchecked("total + x"),
        );
        assert_eq!(render(&expr), "reduce(total = 0, x IN `list` | total + x)");
    }

    #[test]
    fn render_conversion_lists() {
        assert_eq!(
            render(&to_boolean_list(Expression::symbolic_name("l"))),
            "toBooleanList(l)"
        );
        assert_eq!(
            render(&to_float_list(Expression::symbolic_name("l"))),
            "toFloatList(l)"
        );
        assert_eq!(
            render(&to_integer_list(Expression::symbolic_name("l"))),
            "toIntegerList(l)"
        );
        assert_eq!(
            render(&to_string_list(Expression::symbolic_name("l"))),
            "toStringList(l)"
        );
    }

    // ── coll namespace ──

    #[test]
    fn render_coll_distinct() {
        assert_eq!(
            render(&coll_distinct(Expression::symbolic_name("l"))),
            "coll.distinct(l)"
        );
    }

    #[test]
    fn render_coll_flatten() {
        assert_eq!(
            render(&coll_flatten(Expression::symbolic_name("l"))),
            "coll.flatten(l)"
        );
    }

    #[test]
    fn render_coll_index_of() {
        assert_eq!(
            render(&coll_index_of(Expression::symbolic_name("l"), Expression::from(42_i32))),
            "coll.indexOf(l, 42)"
        );
    }

    #[test]
    fn render_coll_insert() {
        assert_eq!(
            render(&coll_insert(
                Expression::symbolic_name("l"),
                Expression::from(0_i32),
                Expression::from("val"),
            )),
            "coll.insert(l, 0, 'val')"
        );
    }

    #[test]
    fn render_coll_max_min() {
        assert_eq!(render(&coll_max(Expression::symbolic_name("l"))), "coll.max(l)");
        assert_eq!(render(&coll_min(Expression::symbolic_name("l"))), "coll.min(l)");
    }

    #[test]
    fn render_coll_remove() {
        assert_eq!(
            render(&coll_remove(Expression::symbolic_name("l"), Expression::from(1_i32))),
            "coll.remove(l, 1)"
        );
    }

    #[test]
    fn render_coll_sort() {
        assert_eq!(
            render(&coll_sort(Expression::symbolic_name("l"))),
            "coll.sort(l)"
        );
    }
}
