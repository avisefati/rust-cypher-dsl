//! Macros for ergonomic DSL usage (e.g., `props!{}`).

/// Creates a map literal expression for inline node/relationship properties.
///
/// Produces an `Expression::MapLiteral` from key-value pairs.
/// Values can be any type that implements `Into<Expression>`.
///
/// # Syntax
///
/// ```text
/// props! {}                          // empty map
/// props! { "key" => value }          // single entry
/// props! { "k1" => v1, "k2" => v2 } // multiple entries
/// ```
///
/// # Examples
///
/// ```rust
/// use rust_cypher_dsl::prelude::*;
///
/// let p = props! { "name" => "Alice", "age" => 30_i32 };
/// ```
#[macro_export]
macro_rules! props {
    // Empty map
    {} => {
        $crate::types::expression::Expression::map_literal(::std::vec::Vec::new())
    };
    // One or more key => value pairs (with optional trailing comma)
    { $( $key:expr => $val:expr ),+ $(,)? } => {
        $crate::types::expression::Expression::map_literal(
            ::std::vec![
                $(
                    (
                        ::std::borrow::Cow::from($key),
                        <$crate::types::expression::Expression as ::std::convert::From<_>>::from($val),
                    )
                ),+
            ]
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::types::expression::{Expression, ExpressionInner};
    use crate::types::parameter::Parameter;
    use std::borrow::Cow;

    #[test]
    fn props_empty() {
        let expr = props! {};
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert!(entries.is_empty());
    }

    #[test]
    fn props_single_entry() {
        let expr = props! { "name" => "Alice" };
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "name");
        assert_eq!(
            *entries[0].1.inner(),
            ExpressionInner::StringLiteral(Cow::Borrowed("Alice"))
        );
    }

    #[test]
    fn props_multiple_entries() {
        let expr = props! {
            "name" => "Bob",
            "age" => 42_i32,
            "active" => true,
        };
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0, "name");
        assert_eq!(entries[1].0, "age");
        assert_eq!(entries[2].0, "active");
    }

    #[test]
    fn props_mixed_literal_and_param() {
        let param = Expression::from(Parameter::new("limit"));
        let expr = props! {
            "name" => "Alice",
            "limit" => param,
        };
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 2);
        assert!(matches!(
            entries[1].1.inner(),
            ExpressionInner::Parameter(_)
        ));
    }

    #[test]
    fn props_with_integer_and_float_values() {
        let expr = props! {
            "count" => 10_i64,
            "ratio" => 2.72_f64,
        };
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(*entries[0].1.inner(), ExpressionInner::IntegerLiteral(10));
        assert_eq!(*entries[1].1.inner(), ExpressionInner::FloatLiteral(2.72));
    }

    #[test]
    fn props_trailing_comma() {
        // Trailing comma should compile fine
        let expr = props! { "a" => 1_i32, };
        let ExpressionInner::MapLiteral(entries) = expr.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 1);
    }
}
