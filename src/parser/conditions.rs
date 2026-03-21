//! Condition parsers: comparisons, boolean combinators.

#![allow(dead_code, reason = "condition parsers will be used incrementally")]

use super::error::ParseError;
use super::expressions::parse_expression;
use super::grammar::TokenStream;
use crate::types::condition::Condition;
use crate::types::expression::Expression;

/// Parses a condition from an expression.
///
/// Conditions are just expressions that evaluate to boolean.
/// In the parser, we parse as `Expression` and wrap in `ExpressionCondition`.
pub fn parse_condition(stream: &mut TokenStream<'_, '_>) -> Result<Condition, ParseError> {
    let expr = parse_expression(stream)?;
    Ok(expression_to_condition(expr))
}

/// Converts an `Expression` to a `Condition`.
///
/// If the expression is already a condition (from comparisons etc.), extract it.
/// Otherwise, wrap it in `ExpressionCondition`.
pub const fn expression_to_condition(expr: Expression) -> Condition {
    // Since expressions created from conditions are wrapped with Expression::from(cond),
    // we can't easily extract them back. So we just wrap everything in ExpressionCondition.
    Condition::ExpressionCondition(expr)
}
