//! Expression parsers: literals, names, properties, operators, function calls.

#![allow(dead_code, reason = "expression parsers will be used incrementally")]
#![allow(clippy::too_many_lines, reason = "expression precedence requires many small functions")]

use super::error::ParseError;
use super::grammar::TokenStream;
use super::tokens::{Keyword, Token};
use crate::types::expression::Expression;
use crate::types::parameter::Parameter;
use std::borrow::Cow;

/// Parses an expression (entry point).
///
/// Handles OR precedence (lowest).
pub fn parse_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    parse_or_expression(stream)
}

/// Parses an expression with optional AS alias.
pub fn parse_expression_with_alias(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let expr = parse_expression(stream)?;
    if stream.at_keyword(Keyword::As) {
        stream.advance();
        let alias = parse_identifier(stream)?;
        Ok(expr.alias(alias))
    } else {
        Ok(expr)
    }
}

/// Parses OR expressions.
fn parse_or_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_xor_expression(stream)?;

    while stream.at_keyword(Keyword::Or) {
        stream.advance();
        let right = parse_xor_expression(stream)?;
        // Convert to Condition and wrap back
        let cond = crate::types::condition::Condition::ExpressionCondition(left)
            .or(crate::types::condition::Condition::ExpressionCondition(right));
        left = Expression::from(cond);
    }

    Ok(left)
}

/// Parses XOR expressions.
fn parse_xor_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_and_expression(stream)?;

    while stream.at_keyword(Keyword::Xor) {
        stream.advance();
        let right = parse_and_expression(stream)?;
        let cond = crate::types::condition::Condition::ExpressionCondition(left)
            .xor(crate::types::condition::Condition::ExpressionCondition(right));
        left = Expression::from(cond);
    }

    Ok(left)
}

/// Parses AND expressions.
fn parse_and_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_not_expression(stream)?;

    while stream.at_keyword(Keyword::And) {
        stream.advance();
        let right = parse_not_expression(stream)?;
        let cond = crate::types::condition::Condition::ExpressionCondition(left)
            .and(crate::types::condition::Condition::ExpressionCondition(right));
        left = Expression::from(cond);
    }

    Ok(left)
}

/// Parses NOT prefix.
fn parse_not_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    if stream.at_keyword(Keyword::Not) {
        stream.advance();
        let expr = parse_not_expression(stream)?;
        let cond = crate::types::condition::Condition::ExpressionCondition(expr).not();
        Ok(Expression::from(cond))
    } else {
        parse_comparison(stream)
    }
}

/// Parses comparison operators and predicates.
fn parse_comparison(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let left = parse_addition(stream)?;

    // Check for comparison operators
    if stream.at_token(&Token::Eq) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.eq(right)));
    }
    if stream.at_token(&Token::Ne) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.ne(right)));
    }
    if stream.at_token(&Token::Lt) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.lt(right)));
    }
    if stream.at_token(&Token::Lte) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.lte(right)));
    }
    if stream.at_token(&Token::Gt) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.gt(right)));
    }
    if stream.at_token(&Token::Gte) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.gte(right)));
    }

    // IS NULL / IS NOT NULL
    if stream.at_keyword(Keyword::Is) {
        stream.advance();
        if stream.at_keyword(Keyword::Not) {
            stream.advance();
            stream.expect_keyword(Keyword::Null)?;
            return Ok(Expression::from(crate::types::condition::Condition::IsNotNull(left)));
        } else if stream.at_keyword(Keyword::Null) {
            stream.advance();
            return Ok(Expression::from(crate::types::condition::Condition::IsNull(left)));
        }
        return Err(stream.error(
            vec!["NULL".to_owned(), "NOT NULL".to_owned()],
            vec!["IS predicate".to_owned()],
        ));
    }

    // IN
    if stream.at_keyword(Keyword::In) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(crate::types::condition::Condition::In { left, right }));
    }

    // STARTS WITH / ENDS WITH / CONTAINS
    if stream.at_keyword(Keyword::Starts) {
        stream.advance();
        stream.expect_keyword(Keyword::With)?;
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.starts_with(right)));
    }
    if stream.at_keyword(Keyword::Ends) {
        stream.advance();
        stream.expect_keyword(Keyword::With)?;
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.ends_with(right)));
    }
    if stream.at_keyword(Keyword::Contains) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(left.contains(right)));
    }

    // Regex match =~
    if stream.at_token(&Token::RegexMatch) {
        stream.advance();
        let right = parse_addition(stream)?;
        return Ok(Expression::from(crate::types::condition::Condition::RegexMatch { left, pattern: right }));
    }

    Ok(left)
}

/// Parses addition and subtraction.
fn parse_addition(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_multiplication(stream)?;

    loop {
        if stream.at_token(&Token::Plus) {
            stream.advance();
            let right = parse_multiplication(stream)?;
            left = left.add(right);
        } else if stream.at_token(&Token::Minus) {
            stream.advance();
            let right = parse_multiplication(stream)?;
            left = left.subtract(right);
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parses multiplication, division, and modulo.
fn parse_multiplication(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_power(stream)?;

    loop {
        if stream.at_token(&Token::Star) {
            stream.advance();
            let right = parse_power(stream)?;
            left = left.multiply(right);
        } else if stream.at_token(&Token::Slash) {
            stream.advance();
            let right = parse_power(stream)?;
            left = left.divide(right);
        } else if stream.at_token(&Token::Percent) {
            stream.advance();
            let right = parse_power(stream)?;
            left = left.remainder(right);
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parses exponentiation (^).
fn parse_power(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut left = parse_unary(stream)?;

    if stream.at_token(&Token::Caret) {
        stream.advance();
        let right = parse_power(stream)?; // Right-associative
        left = left.pow(right);
    }

    Ok(left)
}

/// Parses unary plus/minus.
fn parse_unary(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    if stream.at_token(&Token::Minus) {
        stream.advance();
        let expr = parse_unary(stream)?;
        // Unary minus: 0 - expr
        Ok(Expression::from(0_i64).subtract(expr))
    } else if stream.at_token(&Token::Plus) {
        stream.advance();
        parse_unary(stream)
    } else {
        parse_postfix(stream)
    }
}

/// Parses postfix operations (property access).
fn parse_postfix(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let mut expr = parse_atom(stream)?;

    while stream.at_token(&Token::Dot) {
        stream.advance();
        let prop_name = parse_identifier(stream)?;
        expr = Expression::from(expr.property(prop_name));
    }

    Ok(expr)
}

/// Parses atomic expressions: literals, identifiers, parameters, function calls, parenthesized.
fn parse_atom(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    let tok = stream.peek().ok_or_else(|| {
        stream.error(
            vec!["expression".to_owned()],
            vec!["unexpected end of input".to_owned()],
        )
    })?;

    match tok {
        Token::IntegerLit(n) => {
            let val = *n;
            stream.advance();
            Ok(Expression::from(val))
        }
        Token::FloatLit(f) => {
            let val = *f;
            stream.advance();
            Ok(Expression::from(val))
        }
        Token::StringLit(s) => {
            let val = s.clone();
            stream.advance();
            Ok(Expression::from(val))
        }
        Token::Keyword(Keyword::True) => {
            stream.advance();
            Ok(Expression::from(true))
        }
        Token::Keyword(Keyword::False) => {
            stream.advance();
            Ok(Expression::from(false))
        }
        Token::Keyword(Keyword::Null) => {
            stream.advance();
            Ok(Expression::null_literal())
        }
        Token::Star => {
            stream.advance();
            Ok(Expression::asterisk())
        }
        Token::Dollar => {
            stream.advance();
            let name = parse_identifier(stream)?;
            Ok(Expression::from(Parameter::new(name)))
        }
        Token::Identifier(_) | Token::EscapedIdentifier(_) => {
            let name = parse_identifier(stream)?;
            // Check if it's a function call
            if stream.at_token(&Token::LParen) {
                parse_function_call(stream, name)
            } else {
                Ok(Expression::symbolic_name(name))
            }
        }
        Token::LParen => {
            stream.advance();
            let expr = parse_expression(stream)?;
            stream.expect_token(&Token::RParen)?;
            Ok(expr)
        }
        Token::LBracket => {
            parse_list_literal(stream)
        }
        Token::LBrace => {
            parse_map_literal(stream)
        }
        Token::Keyword(Keyword::Case) => {
            parse_case_expression(stream)
        }
        _ => Err(stream.error(
            vec!["expression".to_owned()],
            vec![format!("unexpected token {tok}")],
        )),
    }
}

/// Parses an identifier (unescaped or escaped).
pub fn parse_identifier(stream: &mut TokenStream<'_, '_>) -> Result<Cow<'static, str>, ParseError> {
    let tok = stream.advance().ok_or_else(|| {
        stream.error(
            vec!["identifier".to_owned()],
            vec!["unexpected end of input".to_owned()],
        )
    })?;

    match &tok.token {
        Token::Identifier(id) => Ok(Cow::Owned((*id).to_owned())),
        Token::EscapedIdentifier(id) => Ok(Cow::Owned(id.clone())),
        _ => Err(stream.error(
            vec!["identifier".to_owned()],
            vec![format!("expected identifier, got {}", tok.token)],
        )),
    }
}

/// Parses a function call: `name(args)` or `name(DISTINCT args)`.
fn parse_function_call(stream: &mut TokenStream<'_, '_>, name: Cow<'static, str>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LParen)?;

    let distinct = if stream.at_keyword(Keyword::Distinct) {
        stream.advance();
        true
    } else {
        false
    };

    let mut args = Vec::new();
    if !stream.at_token(&Token::RParen) {
        loop {
            args.push(parse_expression(stream)?);
            if stream.at_token(&Token::Comma) {
                stream.advance();
            } else {
                break;
            }
        }
    }

    stream.expect_token(&Token::RParen)?;

    if distinct {
        Ok(Expression::function_invocation_distinct(name, args))
    } else {
        Ok(Expression::function_invocation(name, args))
    }
}

/// Parses a list literal: `[expr1, expr2, ...]`.
fn parse_list_literal(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LBracket)?;

    let mut elements = Vec::new();
    if !stream.at_token(&Token::RBracket) {
        loop {
            elements.push(parse_expression(stream)?);
            if stream.at_token(&Token::Comma) {
                stream.advance();
            } else {
                break;
            }
        }
    }

    stream.expect_token(&Token::RBracket)?;
    Ok(Expression::list_literal(elements))
}

/// Parses a map literal: `{key1: val1, key2: val2, ...}`.
fn parse_map_literal(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LBrace)?;

    let mut entries = Vec::new();
    if !stream.at_token(&Token::RBrace) {
        loop {
            let key = parse_identifier(stream)?;
            stream.expect_token(&Token::Colon)?;
            let value = parse_expression(stream)?;
            entries.push((key, value));

            if stream.at_token(&Token::Comma) {
                stream.advance();
            } else {
                break;
            }
        }
    }

    stream.expect_token(&Token::RBrace)?;
    Ok(Expression::map_literal(entries))
}

/// Parses a CASE expression: `CASE [expr] WHEN ... THEN ... [ELSE ...] END`.
fn parse_case_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_keyword(Keyword::Case)?;

    // For now, we don't have CASE in the Expression API, so we return a placeholder.
    // This would need to be added to the Expression type if CASE expressions are supported.
    // For now, skip to END and return a placeholder.
    let mut depth = 1;
    while !stream.is_empty() && depth > 0 {
        if stream.at_keyword(Keyword::Case) {
            depth += 1;
        } else if stream.at_keyword(Keyword::End) {
            depth -= 1;
        }
        stream.advance();
    }

    // Return a raw expression placeholder
    Ok(Expression::raw_unchecked("CASE ... END"))
}
