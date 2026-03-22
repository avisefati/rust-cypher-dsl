//! Scalar and type conversion functions.

use crate::types::expression::Expression;

/// `id(expr)` - returns the internal id (deprecated in favor of `elementId`).
pub fn id(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("id", vec![expr.into()])
}

/// `elementId(expr)` - returns the element id as a string.
pub fn element_id(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("elementId", vec![expr.into()])
}

/// `type(expr)` - returns the relationship type as a string.
pub fn type_of(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("type", vec![expr.into()])
}

/// `coalesce(expr1, expr2, ...)` - returns first non-null value.
pub fn coalesce(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("coalesce", args)
}

/// `timestamp()` - returns the current timestamp in milliseconds.
pub fn timestamp() -> Expression {
    Expression::function_invocation("timestamp", vec![])
}

/// `size(expr)` - returns the size of a string or list.
pub fn size(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("size", vec![expr.into()])
}

/// `head(expr)` - returns the first element of a list.
pub fn head(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("head", vec![expr.into()])
}

/// `last(expr)` - returns the last element of a list.
pub fn last(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("last", vec![expr.into()])
}

/// `startNode(expr)` - returns the start node of a relationship.
pub fn start_node(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("startNode", vec![expr.into()])
}

/// `endNode(expr)` - returns the end node of a relationship.
pub fn end_node(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("endNode", vec![expr.into()])
}

/// `properties(expr)` - returns properties of a node/relationship as a map.
pub fn properties(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("properties", vec![expr.into()])
}

/// `randomUUID()` - returns a random UUID string.
pub fn random_uuid() -> Expression {
    Expression::function_invocation("randomUUID", vec![])
}

/// `nullIf(expr1, expr2)` - returns null if values are equal.
pub fn null_if(
    expr1: impl Into<Expression>,
    expr2: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("nullIf", vec![expr1.into(), expr2.into()])
}

/// `valueType(expr)` - returns the type of a value as a string.
pub fn value_type(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("valueType", vec![expr.into()])
}

/// `char_length(expr)` - returns the character length of a string.
pub fn char_length(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("char_length", vec![expr.into()])
}

/// `length(path)` - returns the length of a path.
pub fn length(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("length", vec![expr.into()])
}

/// `length(path)` - alias for `length` specific to paths.
pub fn path_length(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("length", vec![expr.into()])
}

/// `toInteger(expr)` - converts to integer.
pub fn to_integer(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toInteger", vec![expr.into()])
}

/// `toIntegerOrNull(expr)` - converts to integer or returns null.
pub fn to_integer_or_null(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toIntegerOrNull", vec![expr.into()])
}

/// `toFloat(expr)` - converts to float.
pub fn to_float(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toFloat", vec![expr.into()])
}

/// `toFloatOrNull(expr)` - converts to float or returns null.
pub fn to_float_or_null(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toFloatOrNull", vec![expr.into()])
}

/// `toBoolean(expr)` - converts to boolean.
pub fn to_boolean(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toBoolean", vec![expr.into()])
}

/// `toBooleanOrNull(expr)` - converts to boolean or returns null.
pub fn to_boolean_or_null(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toBooleanOrNull", vec![expr.into()])
}

/// `toString(expr)` - converts to string.
pub fn to_string_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toString", vec![expr.into()])
}

/// `toStringOrNull(expr)` - converts to string or returns null.
pub fn to_string_or_null(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toStringOrNull", vec![expr.into()])
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

    #[test]
    fn render_id() {
        assert_eq!(render(&id(Expression::symbolic_name("n"))), "id(n)");
    }

    #[test]
    fn render_element_id() {
        assert_eq!(render(&element_id(Expression::symbolic_name("n"))), "elementId(n)");
    }

    #[test]
    fn render_type_of() {
        assert_eq!(render(&type_of(Expression::symbolic_name("r"))), "type(r)");
    }

    #[test]
    fn render_coalesce() {
        assert_eq!(
            render(&coalesce(vec![
                Expression::symbolic_name("a"),
                Expression::symbolic_name("b"),
            ])),
            "coalesce(a, b)"
        );
    }

    #[test]
    fn render_timestamp() {
        assert_eq!(render(&timestamp()), "timestamp()");
    }

    #[test]
    fn render_size() {
        assert_eq!(render(&size(Expression::symbolic_name("list"))), "size(`list`)");
    }

    #[test]
    fn render_head_last() {
        assert_eq!(render(&head(Expression::symbolic_name("l"))), "head(l)");
        assert_eq!(render(&last(Expression::symbolic_name("l"))), "last(l)");
    }

    #[test]
    fn render_start_end_node() {
        assert_eq!(render(&start_node(Expression::symbolic_name("r"))), "startNode(r)");
        assert_eq!(render(&end_node(Expression::symbolic_name("r"))), "endNode(r)");
    }

    #[test]
    fn render_properties() {
        assert_eq!(render(&properties(Expression::symbolic_name("n"))), "properties(n)");
    }

    #[test]
    fn render_random_uuid() {
        assert_eq!(render(&random_uuid()), "randomUUID()");
    }

    #[test]
    fn render_null_if() {
        assert_eq!(
            render(&null_if(Expression::symbolic_name("a"), Expression::symbolic_name("b"))),
            "nullIf(a, b)"
        );
    }

    #[test]
    fn render_value_type() {
        assert_eq!(render(&value_type(Expression::symbolic_name("x"))), "valueType(x)");
    }

    #[test]
    fn render_to_integer() {
        assert_eq!(render(&to_integer(Expression::from("42"))), "toInteger('42')");
    }

    #[test]
    fn render_to_float() {
        assert_eq!(render(&to_float(Expression::from("3.14"))), "toFloat('3.14')");
    }

    #[test]
    fn render_to_boolean() {
        assert_eq!(render(&to_boolean(Expression::from("true"))), "toBoolean('true')");
    }

    #[test]
    fn render_to_string_fn() {
        assert_eq!(render(&to_string_fn(Expression::from(42_i32))), "toString(42)");
    }

    #[test]
    fn render_or_null_variants() {
        assert_eq!(render(&to_integer_or_null(Expression::from("x"))), "toIntegerOrNull('x')");
        assert_eq!(render(&to_float_or_null(Expression::from("x"))), "toFloatOrNull('x')");
        assert_eq!(render(&to_boolean_or_null(Expression::from("x"))), "toBooleanOrNull('x')");
        assert_eq!(render(&to_string_or_null(Expression::from(1_i32))), "toStringOrNull(1)");
    }

    #[test]
    fn render_length() {
        assert_eq!(render(&length(Expression::symbolic_name("p"))), "length(p)");
        assert_eq!(render(&path_length(Expression::symbolic_name("p"))), "length(p)");
    }

    #[test]
    fn render_char_length() {
        assert_eq!(render(&char_length(Expression::symbolic_name("s"))), "char_length(s)");
    }
}
