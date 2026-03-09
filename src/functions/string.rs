//! String functions: toLower, toUpper, trim, replace, substring, etc.

use crate::types::expression::Expression;

/// `toLower(expr)` - converts string to lowercase.
pub fn to_lower(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toLower", vec![expr.into()])
}

/// `lower(expr)` - alias for `toLower`.
pub fn lower(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("lower", vec![expr.into()])
}

/// `toUpper(expr)` - converts string to uppercase.
pub fn to_upper(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toUpper", vec![expr.into()])
}

/// `upper(expr)` - alias for `toUpper`.
pub fn upper(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("upper", vec![expr.into()])
}

/// `trim(expr)` - trims whitespace from both ends.
pub fn trim(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("trim", vec![expr.into()])
}

/// `btrim(expr)` - trims whitespace from both ends.
pub fn btrim(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("btrim", vec![expr.into()])
}

/// `ltrim(expr)` - trims whitespace from the left.
pub fn ltrim(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("ltrim", vec![expr.into()])
}

/// `rtrim(expr)` - trims whitespace from the right.
pub fn rtrim(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("rtrim", vec![expr.into()])
}

/// `replace(original, search, replacement)` - replaces occurrences.
pub fn replace(
    original: impl Into<Expression>,
    search: impl Into<Expression>,
    replacement: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation(
        "replace",
        vec![original.into(), search.into(), replacement.into()],
    )
}

/// `substring(original, start, [length])` - extracts a substring.
pub fn substring(
    original: impl Into<Expression>,
    start: impl Into<Expression>,
    length: Option<Expression>,
) -> Expression {
    let mut args = vec![original.into(), start.into()];
    if let Some(len) = length {
        args.push(len);
    }
    Expression::function_invocation("substring", args)
}

/// `left(original, length)` - returns leftmost characters.
pub fn left(
    original: impl Into<Expression>,
    length: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("left", vec![original.into(), length.into()])
}

/// `right(original, length)` - returns rightmost characters.
pub fn right(
    original: impl Into<Expression>,
    length: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("right", vec![original.into(), length.into()])
}

/// `split(original, delimiter)` - splits a string into a list.
pub fn split(
    original: impl Into<Expression>,
    delimiter: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("split", vec![original.into(), delimiter.into()])
}

/// `reverse(expr)` - reverses a string.
pub fn reverse_str(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("reverse", vec![expr.into()])
}

/// `normalize(expr)` - normalizes a string to NFC form.
pub fn normalize(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("normalize", vec![expr.into()])
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
    fn render_to_lower() {
        assert_eq!(render(&to_lower(Expression::symbolic_name("s"))), "toLower(s)");
    }

    #[test]
    fn render_lower() {
        assert_eq!(render(&lower(Expression::symbolic_name("s"))), "lower(s)");
    }

    #[test]
    fn render_to_upper() {
        assert_eq!(render(&to_upper(Expression::symbolic_name("s"))), "toUpper(s)");
    }

    #[test]
    fn render_upper() {
        assert_eq!(render(&upper(Expression::symbolic_name("s"))), "upper(s)");
    }

    #[test]
    fn render_trim_variants() {
        assert_eq!(render(&trim(Expression::symbolic_name("s"))), "trim(s)");
        assert_eq!(render(&btrim(Expression::symbolic_name("s"))), "btrim(s)");
        assert_eq!(render(&ltrim(Expression::symbolic_name("s"))), "ltrim(s)");
        assert_eq!(render(&rtrim(Expression::symbolic_name("s"))), "rtrim(s)");
    }

    #[test]
    fn render_replace() {
        assert_eq!(
            render(&replace(Expression::symbolic_name("s"), Expression::from("a"), Expression::from("b"))),
            "replace(s, 'a', 'b')"
        );
    }

    #[test]
    fn render_substring_with_length() {
        assert_eq!(
            render(&substring(Expression::symbolic_name("s"), Expression::from(0_i32), Some(Expression::from(5_i32)))),
            "substring(s, 0, 5)"
        );
    }

    #[test]
    fn render_substring_without_length() {
        assert_eq!(
            render(&substring(Expression::symbolic_name("s"), Expression::from(3_i32), None)),
            "substring(s, 3)"
        );
    }

    #[test]
    fn render_left_right() {
        assert_eq!(render(&left(Expression::symbolic_name("s"), Expression::from(3_i32))), "left(s, 3)");
        assert_eq!(render(&right(Expression::symbolic_name("s"), Expression::from(3_i32))), "right(s, 3)");
    }

    #[test]
    fn render_split() {
        assert_eq!(
            render(&split(Expression::symbolic_name("s"), Expression::from(","))),
            "split(s, ',')"
        );
    }

    #[test]
    fn render_reverse_str() {
        assert_eq!(render(&reverse_str(Expression::symbolic_name("s"))), "reverse(s)");
    }

    #[test]
    fn render_normalize() {
        assert_eq!(render(&normalize(Expression::symbolic_name("s"))), "normalize(s)");
    }
}
