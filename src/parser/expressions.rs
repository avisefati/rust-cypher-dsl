//! Expression parsers: literals, names, properties, operators, function calls.

#![allow(dead_code, reason = "expression parsers will be used incrementally")]
#![allow(clippy::too_many_lines, reason = "expression precedence requires many small functions")]

use super::error::ParseError;
use super::grammar::TokenStream;
use super::tokens::{Keyword, Token};
use crate::types::expression::Expression;
use crate::types::parameter::Parameter;
use std::borrow::Cow;

/// Returns `true` for keywords that are admin-specific and can appear as
/// identifiers in expression contexts (e.g., YIELD field names, property names).
///
/// These "soft keywords" are recognized as keywords only in admin command
/// parsing. In regular expression contexts they are treated as identifiers.
const fn is_soft_keyword(kw: Keyword) -> bool {
    matches!(
        kw,
        Keyword::Text
            | Keyword::Point
            | Keyword::Fulltext
            | Keyword::Vector
            | Keyword::Lookup
            | Keyword::Executable
            | Keyword::Unique
            | Keyword::Key
            | Keyword::Require
            | Keyword::Each
            | Keyword::Type
            | Keyword::Options
            | Keyword::Built
            | Keyword::Defined
            | Keyword::User
            | Keyword::Current
            | Keyword::Relationship
            | Keyword::Node
            | Keyword::For
            | Keyword::Labels
            | Keyword::If
            | Keyword::Show
            | Keyword::Drop
            | Keyword::Constraint
            | Keyword::Constraints
            | Keyword::Indexes
            | Keyword::Functions
            | Keyword::Procedures
            | Keyword::Terminate
    )
}

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

    loop {
        if stream.at_token(&Token::Dot) {
            stream.advance();
            let prop_name = parse_identifier(stream)?;
            expr = Expression::from(expr.property(prop_name));
        } else if stream.at_token(&Token::LBrace) && is_map_projection_start(stream) {
            expr = parse_map_projection(stream, expr)?;
        } else {
            break;
        }
    }

    Ok(expr)
}

/// Returns true if the current `{` starts a map projection rather than a map literal.
///
/// Map projection entries start with `.` (property/all-properties) or `identifier:` (literal entry).
/// We check for `.` as the strongest signal — it can't appear in a map literal.
fn is_map_projection_start(stream: &TokenStream<'_, '_>) -> bool {
    matches!(stream.peek_nth(1), Some(Token::Dot | Token::Star | Token::RBrace))
        || matches!(
            (stream.peek_nth(1), stream.peek_nth(2)),
            (Some(Token::Identifier(_) | Token::EscapedIdentifier(_)), Some(Token::Colon))
        )
}

/// Parses map projection entries: `{.prop1, .prop2, key: expr, .*}`.
fn parse_map_projection(stream: &mut TokenStream<'_, '_>, variable: Expression) -> Result<Expression, ParseError> {
    use crate::types::expression::MapProjectionEntry;

    stream.expect_token(&Token::LBrace)?;
    let mut entries = Vec::new();

    while !stream.at_token(&Token::RBrace) && !stream.is_empty() {
        if !entries.is_empty() {
            stream.expect_token(&Token::Comma)?;
        }

        if stream.at_token(&Token::Dot) {
            stream.advance();
            if stream.at_token(&Token::Star) {
                stream.advance();
                entries.push(MapProjectionEntry::AllProperties);
            } else {
                let prop = parse_identifier(stream)?;
                entries.push(MapProjectionEntry::Property(prop));
            }
        } else {
            // Literal entry: key: expr
            let key = parse_identifier(stream)?;
            stream.expect_token(&Token::Colon)?;
            let value = parse_expression(stream)?;
            entries.push(MapProjectionEntry::Literal(key, value));
        }
    }

    stream.expect_token(&Token::RBrace)?;
    Ok(Expression::map_projection(variable, entries))
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
            // Try speculative pattern parse for pattern-in-WHERE support.
            // If the `(` starts a pattern (node with labels, or node followed
            // by a relationship arrow), parse as PatternPredicate condition.
            if let Some(pattern_expr) = try_parse_pattern_as_expression(stream) {
                return Ok(pattern_expr);
            }
            stream.advance();
            let expr = parse_expression(stream)?;
            stream.expect_token(&Token::RParen)?;
            Ok(expr)
        }
        Token::LBracket => {
            parse_bracket_expression(stream)
        }
        Token::LBrace => {
            parse_map_literal(stream)
        }
        Token::Keyword(Keyword::Case) => {
            parse_case_expression(stream)
        }
        Token::Keyword(Keyword::Exists) if matches!(stream.peek_nth(1), Some(Token::LBrace)) => {
            parse_exists_subquery(stream)
        }
        Token::Keyword(Keyword::Count) if matches!(stream.peek_nth(1), Some(Token::LBrace)) => {
            parse_count_subquery(stream)
        }
        Token::Keyword(Keyword::Collect) if matches!(stream.peek_nth(1), Some(Token::LBrace)) => {
            parse_collect_subquery(stream)
        }
        // Keywords used as function names: count(n), exists(n.prop), collect(n), etc.
        Token::Keyword(_) if matches!(stream.peek_nth(1), Some(Token::LParen)) => {
            let name = parse_identifier(stream)?;
            parse_function_call(stream, name)
        }
        // Soft keywords: admin-related keywords that can appear as identifiers
        // in expression contexts (e.g., YIELD field names, property names).
        Token::Keyword(kw) if is_soft_keyword(*kw) => {
            let name = parse_identifier(stream)?;
            Ok(Expression::symbolic_name(name))
        }
        _ => Err(stream.error(
            vec!["expression".to_owned()],
            vec![format!("unexpected token {tok}")],
        )),
    }
}

/// Parses an identifier (unescaped, escaped, or keyword used as identifier).
///
/// In Cypher, keywords can be used as identifiers in many contexts
/// (e.g., property names like `n.count`, variable names like `SET node = ...`).
/// This function accepts keywords as identifiers to handle such cases.
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
        Token::Keyword(kw) => Ok(Cow::Owned(kw.to_string().to_lowercase())),
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
/// Parses bracket expression: list literal, list comprehension, or pattern comprehension.
///
/// Disambiguates:
/// - `[ident IN ...]` → list comprehension
/// - `[(pattern) ... | expr]` → pattern comprehension
/// - `[expr, expr, ...]` → list literal
fn parse_bracket_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    // Check for list comprehension: `[ident IN ...]`
    if matches!(stream.peek_nth(1), Some(Token::Identifier(_) | Token::EscapedIdentifier(_)))
        && matches!(stream.peek_nth(2), Some(Token::Keyword(Keyword::In)))
    {
        return parse_list_comprehension(stream);
    }

    // Check for pattern comprehension: `[(pattern) ... | expr]`
    if matches!(stream.peek_nth(1), Some(Token::LParen)) {
        return parse_pattern_comprehension(stream);
    }

    parse_list_literal(stream)
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

/// Parses a list comprehension: `[var IN list WHERE cond | expr]`.
fn parse_list_comprehension(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LBracket)?;

    let variable = parse_identifier(stream)?;
    stream.expect_keyword(Keyword::In)?;
    let list = parse_expression(stream)?;

    // Optional WHERE filter
    let where_clause = if stream.at_keyword(Keyword::Where) {
        stream.advance();
        Some(parse_expression(stream)?)
    } else {
        None
    };

    // Optional projection: `| expr`
    let projection = if stream.at_token(&Token::Pipe) {
        stream.advance();
        Some(parse_expression(stream)?)
    } else {
        None
    };

    stream.expect_token(&Token::RBracket)?;
    Ok(Expression::list_comprehension(variable, list, where_clause, projection))
}

/// Parses a pattern comprehension: `[(pattern) WHERE cond | expr]`.
fn parse_pattern_comprehension(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LBracket)?;

    // Parse pattern element and render to raw expression
    let pattern_element = super::patterns::parse_pattern_element(stream)?;
    let fmt = crate::renderer::default::DefaultRenderer::with_defaults();
    let pattern_str = fmt.render_pattern_element(&pattern_element);
    let pattern = Expression::raw_unchecked(pattern_str);

    // Optional WHERE filter
    let where_clause = if stream.at_keyword(Keyword::Where) {
        stream.advance();
        Some(parse_expression(stream)?)
    } else {
        None
    };

    // Projection: `| expr`
    stream.expect_token(&Token::Pipe)?;
    let projection = parse_expression(stream)?;

    stream.expect_token(&Token::RBracket)?;
    Ok(Expression::pattern_comprehension(pattern, where_clause, projection))
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
///
/// Supports both simple (`CASE expr WHEN val THEN result`) and
/// generic (`CASE WHEN cond THEN result`) forms.
fn parse_case_expression(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_keyword(Keyword::Case)?;

    // Determine if simple or generic CASE
    let operand = if stream.at_keyword(Keyword::When) {
        None
    } else {
        // Simple CASE: `CASE expr WHEN ...`
        Some(parse_expression(stream)?)
    };

    let mut when_clauses = Vec::new();
    while stream.at_keyword(Keyword::When) {
        stream.advance();
        let condition = parse_expression(stream)?;
        stream.expect_keyword(Keyword::Then)?;
        let result = parse_expression(stream)?;
        when_clauses.push((condition, result));
    }

    let else_clause = if stream.at_keyword(Keyword::Else) {
        stream.advance();
        Some(parse_expression(stream)?)
    } else {
        None
    };

    stream.expect_keyword(Keyword::End)?;

    if let Some(op) = operand {
        Ok(Expression::simple_case(op, when_clauses, else_clause))
    } else {
        Ok(Expression::generic_case(when_clauses, else_clause))
    }
}

/// Parses `EXISTS { subquery }` expression.
fn parse_exists_subquery(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_keyword(Keyword::Exists)?;
    let inner = parse_subquery_body(stream)?;
    Ok(Expression::existential_subquery(inner))
}

/// Parses `COUNT { subquery }` expression.
fn parse_count_subquery(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_keyword(Keyword::Count)?;
    let inner = parse_subquery_body(stream)?;
    Ok(Expression::count_subquery(inner))
}

/// Parses `COLLECT { subquery }` expression.
fn parse_collect_subquery(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_keyword(Keyword::Collect)?;
    let inner = parse_subquery_body(stream)?;
    Ok(Expression::collect_subquery(inner))
}

/// Parses the body of a subquery expression: `{ clauses }`.
///
/// The inner clauses are parsed as a statement, then rendered to a raw expression
/// to match the existing AST representation.
fn parse_subquery_body(stream: &mut TokenStream<'_, '_>) -> Result<Expression, ParseError> {
    stream.expect_token(&Token::LBrace)?;

    let mut clauses = Vec::new();
    while !stream.at_token(&Token::RBrace) && !stream.is_empty() {
        clauses.push(super::clauses::parse_single_clause_public(stream)?);
    }

    stream.expect_token(&Token::RBrace)?;

    // Build a statement from the inner clauses and render it
    let stmt = crate::statement::Statement::SinglePart(
        crate::statement::SinglePartQuery::new(clauses),
    );
    Ok(Expression::raw_unchecked(stmt.render()))
}

// ---------------------------------------------------------------------------
// Pattern-in-WHERE support: speculative pattern parsing
// ---------------------------------------------------------------------------

/// Attempts to parse a graph pattern starting at `(` when it could be either
/// a parenthesized expression or a pattern existence predicate.
///
/// Uses lookahead to detect pattern-like token sequences, then performs a
/// speculative parse with checkpoint/restore. Returns `Some(Expression)` if
/// a genuine pattern (relationship, chain, or labeled node) is found.
///
/// Returns `None` if the tokens look like a parenthesized expression, letting
/// the caller fall through to normal expression parsing.
fn try_parse_pattern_as_expression(
    stream: &mut TokenStream<'_, '_>,
) -> Option<Expression> {
    // Quick lookahead: `(` must be current token
    if !stream.at_token(&Token::LParen) {
        return None;
    }

    // Lookahead heuristics: detect pattern-like sequences after `(`
    // Pattern indicators inside `(...)`:
    //   - `( :` — anonymous labeled node
    //   - `( identifier :` — named labeled node
    //   - `( )` followed by `-` or `<` — empty node in relationship
    //   - `( identifier )` followed by `-` or `<` — named node in relationship
    //   - `( identifier {` — node with properties
    let looks_like_pattern = match stream.peek_nth(1) {
        // `(:Label` — definitely a pattern
        Some(Token::Colon) => true,
        // `()` — check if followed by relationship arrow
        Some(Token::RParen) => matches!(
            stream.peek_nth(2),
            Some(Token::Minus | Token::LeftArrow)
        ),
        // `(identifier ...` — check what follows the identifier
        Some(Token::Identifier(_) | Token::EscapedIdentifier(_)) => {
            match stream.peek_nth(2) {
                // `(identifier:` — labeled node, or `(identifier {` — node with properties
                Some(Token::Colon | Token::LBrace) => true,
                // `(identifier)` — check if followed by relationship arrow
                Some(Token::RParen) => matches!(
                    stream.peek_nth(3),
                    Some(Token::Minus | Token::LeftArrow)
                ),
                _ => false,
            }
        }
        _ => false,
    };

    if !looks_like_pattern {
        return None;
    }

    // Speculative parse: save position, try pattern parsing
    let checkpoint = stream.checkpoint();

    if let Ok(pattern) = super::patterns::parse_pattern(stream) {
        // Verify this is a genuine pattern (not just a bare identifier in parens)
        if is_genuine_pattern(&pattern) {
            let cond = crate::types::condition::Condition::PatternPredicate(pattern);
            return Some(Expression::from(cond));
        }
        // Bare node with no labels/properties — ambiguous, treat as expression
        stream.restore(checkpoint);
        return None;
    }
    // Pattern parse failed — restore and let expression parsing handle it
    stream.restore(checkpoint);
    None
}

/// Returns `true` if a parsed pattern is genuinely a graph pattern
/// (not just a bare identifier in parentheses like `(x)`).
///
/// A pattern is "genuine" if it contains:
/// - A relationship or chain (multi-node pattern)
/// - A node with labels
/// - A node with properties
/// - A named path
/// - A quantified path
/// - A selected path
fn is_genuine_pattern(pattern: &crate::types::pattern::Pattern) -> bool {
    pattern.elements().iter().any(|elem| {
        use crate::types::pattern::PatternElement;
        match elem {
            PatternElement::Node(node) => {
                // Genuine if it has labels or properties
                !node.labels().is_empty() || node.properties().is_some()
            }
            // Relationships, chains, named/quantified/selected paths are always genuine
            PatternElement::Relationship(_)
            | PatternElement::Chain(_)
            | PatternElement::NamedPath(_)
            | PatternElement::QuantifiedPath(_)
            | PatternElement::SelectedPath(_, _)
            | PatternElement::PathConcatenation(_) => true,
        }
    })
}
