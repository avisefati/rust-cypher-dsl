//! Built-in Cypher functions: aggregation, scalar, string, math, list, temporal, spatial, etc.

pub mod aggregate;
pub mod list;
pub mod math;
pub mod scalar;
pub mod spatial;
pub mod string;
pub mod temporal;

/// `customFunction(name, args)` - invokes an arbitrary user-defined function.
pub fn custom_function(
    name: impl Into<std::borrow::Cow<'static, str>>,
    args: Vec<crate::types::expression::Expression>,
) -> crate::types::expression::Expression {
    crate::types::expression::Expression::function_invocation(name, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::expression::Expression;

    fn render(expr: &Expression) -> String {
        let r = crate::renderer::default::DefaultRenderer::with_defaults();
        let mut buf = String::new();
        r.write_expression(&mut buf, expr);
        buf
    }

    #[test]
    fn custom_function_no_args() {
        assert_eq!(render(&custom_function("myFunc", vec![])), "myFunc()");
    }

    #[test]
    fn custom_function_one_arg() {
        assert_eq!(
            render(&custom_function("apoc.text.capitalize", vec![Expression::from("hello")])),
            "apoc.text.capitalize('hello')"
        );
    }

    #[test]
    fn custom_function_multiple_args() {
        assert_eq!(
            render(&custom_function(
                "my.ns.add",
                vec![Expression::from(1_i32), Expression::from(2_i32)],
            )),
            "my.ns.add(1, 2)"
        );
    }
}
