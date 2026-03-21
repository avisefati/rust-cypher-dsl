//! Clause parsers: MATCH, RETURN, WITH, WHERE, etc.

#![allow(dead_code, reason = "clause parsers will be used incrementally")]
#![allow(clippy::too_many_lines, reason = "many clause types to parse")]

use super::conditions::parse_condition;
use super::error::ParseError;
use super::expressions::{parse_expression, parse_expression_with_alias};
use super::grammar::TokenStream;
use super::patterns::parse_pattern;
use super::tokens::{Keyword, Token};
use crate::clauses::{
    Clause, CreateClause, DeleteClause, FilterClause, ForeachClause, LetClause, LimitClause,
    MatchClause, MergeAction, MergeClause, OrderByClause, RemoveClause, RemoveItem, ReturnClause,
    SetClause, SetItem, SkipClause, UnwindClause, WhereClause, WithClause,
};
use crate::types::property::Property;
use std::borrow::Cow;

/// Parses a sequence of clauses until the stream is empty or reaches a statement separator.
pub fn parse_clauses(stream: &mut TokenStream<'_, '_>) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();

    while !stream.is_empty() {
        // Check for statement-level keywords that end the clause sequence
        if stream.at_keyword(Keyword::Union) {
            break;
        }

        let clause = parse_single_clause(stream)?;
        clauses.push(clause);
    }

    if clauses.is_empty() {
        return Err(stream.error(
            vec!["clause".to_owned()],
            vec!["expected at least one clause".to_owned()],
        ));
    }

    Ok(clauses)
}

/// Parses a single clause based on the current keyword.
fn parse_single_clause(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    let tok = stream.peek().ok_or_else(|| {
        stream.error(
            vec!["clause".to_owned()],
            vec!["unexpected end of input".to_owned()],
        )
    })?;

    match tok {
        Token::Keyword(Keyword::Match) => parse_match(stream),
        Token::Keyword(Keyword::Optional) => parse_optional_match(stream),
        Token::Keyword(Keyword::Where) => parse_where(stream),
        Token::Keyword(Keyword::Return) => parse_return(stream),
        Token::Keyword(Keyword::With) => parse_with(stream),
        Token::Keyword(Keyword::Order) => parse_order_by(stream),
        Token::Keyword(Keyword::Skip) => parse_skip(stream),
        Token::Keyword(Keyword::Limit) => parse_limit(stream),
        Token::Keyword(Keyword::Create) => parse_create(stream),
        Token::Keyword(Keyword::Merge) => parse_merge(stream),
        Token::Keyword(Keyword::Set) => parse_set(stream),
        Token::Keyword(Keyword::Delete) => parse_delete(stream, false),
        Token::Keyword(Keyword::Detach) => parse_detach_delete(stream),
        Token::Keyword(Keyword::Remove) => parse_remove(stream),
        Token::Keyword(Keyword::Unwind) => parse_unwind(stream),
        Token::Keyword(Keyword::Foreach) => parse_foreach(stream),
        Token::Keyword(Keyword::Filter) => parse_filter(stream),
        Token::Keyword(Keyword::Let) => parse_let(stream),
        Token::Keyword(Keyword::Finish) => {
            stream.advance();
            Ok(Clause::Finish)
        }
        _ => Err(stream.error(
            vec!["clause keyword".to_owned()],
            vec![format!("unexpected token {tok}")],
        )),
    }
}

/// Parses MATCH clause: `MATCH pattern`.
fn parse_match(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Match)?;
    let pattern = parse_pattern(stream)?;
    Ok(Clause::Match(MatchClause::new(pattern)))
}

/// Parses OPTIONAL MATCH clause.
fn parse_optional_match(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Optional)?;
    stream.expect_keyword(Keyword::Match)?;
    let pattern = parse_pattern(stream)?;
    Ok(Clause::Match(MatchClause::optional(pattern)))
}

/// Parses WHERE clause: `WHERE condition`.
fn parse_where(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Where)?;
    let condition = parse_condition(stream)?;
    Ok(Clause::Where(WhereClause::new(condition)))
}

/// Parses RETURN clause: `RETURN [DISTINCT] expr1, expr2, ...`.
fn parse_return(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Return)?;

    let distinct = if stream.at_keyword(Keyword::Distinct) {
        stream.advance();
        true
    } else {
        false
    };

    let mut expressions = Vec::new();
    loop {
        expressions.push(parse_expression_with_alias(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }

    let clause = if distinct {
        ReturnClause::distinct(expressions)
    } else {
        ReturnClause::new(expressions)
    };

    Ok(Clause::Return(clause))
}

/// Parses WITH clause: `WITH [DISTINCT] expr1, expr2, ...`.
fn parse_with(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::With)?;

    let distinct = if stream.at_keyword(Keyword::Distinct) {
        stream.advance();
        true
    } else {
        false
    };

    let mut expressions = Vec::new();
    loop {
        expressions.push(parse_expression_with_alias(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }

    let clause = if distinct {
        WithClause::distinct(expressions)
    } else {
        WithClause::new(expressions)
    };

    Ok(Clause::With(clause))
}

/// Parses ORDER BY clause: `ORDER BY expr1 [ASC|DESC], expr2 [ASC|DESC], ...`.
fn parse_order_by(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Order)?;
    stream.expect_keyword(Keyword::By)?;

    let mut items = Vec::new();
    loop {
        let expr = parse_expression(stream)?;

        // Check for ASC/DESC/ASCENDING/DESCENDING
        let sort_expr = if stream.at_keyword(Keyword::Desc) || stream.at_keyword(Keyword::Descending) {
            stream.advance();
            expr.descending()
        } else {
            // ASC is optional and default
            if stream.at_keyword(Keyword::Asc) || stream.at_keyword(Keyword::Ascending) {
                stream.advance();
            }
            expr.ascending()
        };

        items.push(sort_expr);

        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }

    Ok(Clause::OrderBy(OrderByClause::new(items)))
}

/// Parses SKIP clause: `SKIP n`.
fn parse_skip(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Skip)?;
    let expr = parse_expression(stream)?;
    Ok(Clause::Skip(SkipClause::new(expr)))
}

/// Parses LIMIT clause: `LIMIT n`.
fn parse_limit(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Limit)?;
    let expr = parse_expression(stream)?;
    Ok(Clause::Limit(LimitClause::new(expr)))
}

/// Parses FILTER clause: `FILTER condition`.
fn parse_filter(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Filter)?;
    let condition = parse_condition(stream)?;
    Ok(Clause::Filter(FilterClause::new(condition)))
}

/// Parses LET clause: `LET var = expr`.
fn parse_let(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Let)?;
    let var = super::expressions::parse_identifier(stream)?;
    stream.expect_token(&Token::Eq)?;
    let expr = parse_expression(stream)?;
    Ok(Clause::Let(LetClause::new(var, expr)))
}

// ── Phase 2: Write clauses ──

/// Parses CREATE clause: `CREATE pattern`.
fn parse_create(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Create)?;
    let pattern = parse_pattern(stream)?;
    Ok(Clause::Create(CreateClause::new(pattern)))
}

/// Parses MERGE clause: `MERGE pattern [ON CREATE SET ...] [ON MATCH SET ...]`.
fn parse_merge(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Merge)?;
    let pattern = parse_pattern(stream)?;

    let mut actions = Vec::new();

    while stream.at_keyword(Keyword::On) {
        stream.advance();
        if stream.at_keyword(Keyword::Create) {
            stream.advance();
            stream.expect_keyword(Keyword::Set)?;
            let items = parse_set_items(stream)?;
            actions.push(MergeAction::OnCreate(items));
        } else if stream.at_keyword(Keyword::Match) {
            stream.advance();
            stream.expect_keyword(Keyword::Set)?;
            let items = parse_set_items(stream)?;
            actions.push(MergeAction::OnMatch(items));
        } else {
            return Err(stream.error(
                vec!["CREATE".to_owned(), "MATCH".to_owned()],
                vec!["ON CREATE SET or ON MATCH SET".to_owned()],
            ));
        }
    }

    if actions.is_empty() {
        Ok(Clause::Merge(MergeClause::new(pattern)))
    } else {
        Ok(Clause::Merge(MergeClause::with_actions(pattern, actions)))
    }
}

/// Parses SET clause: `SET item1, item2, ...`.
fn parse_set(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Set)?;
    let items = parse_set_items(stream)?;
    Ok(Clause::Set(SetClause::new(items)))
}

/// Parses a comma-separated list of SET items.
///
/// SET items can be:
/// - `n.prop = value` → `SetItem::Property`
/// - `n:Label` → `SetItem::Label`
/// - `n += {map}` → `SetItem::Mutate`
/// - `n = {map}` → `SetItem::ReplaceAll` (when target is not a property)
fn parse_set_items(stream: &mut TokenStream<'_, '_>) -> Result<Vec<SetItem>, ParseError> {
    let mut items = Vec::new();
    loop {
        items.push(parse_single_set_item(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }
    Ok(items)
}

/// Parses a single SET item.
fn parse_single_set_item(stream: &mut TokenStream<'_, '_>) -> Result<SetItem, ParseError> {
    let name = super::expressions::parse_identifier(stream)?;

    // Check for label assignment: `n:Label` or `n:Label1:Label2`
    if stream.at_token(&Token::Colon) {
        let node_expr = crate::types::expression::Expression::symbolic_name(name);
        let mut labels: Vec<Cow<'static, str>> = Vec::new();
        while stream.at_token(&Token::Colon) {
            stream.advance();
            labels.push(super::expressions::parse_identifier(stream)?);
        }
        return Ok(SetItem::label(node_expr, labels));
    }

    // Check for property assignment: `n.prop = value`
    if stream.at_token(&Token::Dot) {
        stream.advance();
        let prop_name = super::expressions::parse_identifier(stream)?;
        let property = Property::new(
            crate::types::expression::Expression::symbolic_name(name),
            prop_name,
        );

        stream.expect_token(&Token::Eq)?;
        let value = parse_expression(stream)?;
        return Ok(SetItem::property(property, value));
    }

    // Check for mutate: `n += {map}`
    if stream.at_token(&Token::PlusAssign) {
        stream.advance();
        let value = parse_expression(stream)?;
        let target = crate::types::expression::Expression::symbolic_name(name);
        return Ok(SetItem::mutate(target, value));
    }

    // Check for replace all: `n = {map}`
    if stream.at_token(&Token::Eq) {
        stream.advance();
        let value = parse_expression(stream)?;
        let target = crate::types::expression::Expression::symbolic_name(name);
        return Ok(SetItem::replace_all(target, value));
    }

    Err(stream.error(
        vec!["= or += or :Label".to_owned()],
        vec!["SET item".to_owned()],
    ))
}

/// Parses DELETE clause: `DELETE expr1, expr2, ...`.
fn parse_delete(stream: &mut TokenStream<'_, '_>, detach: bool) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Delete)?;
    let mut expressions = Vec::new();
    loop {
        expressions.push(parse_expression(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }
    let clause = if detach {
        DeleteClause::detach(expressions)
    } else {
        DeleteClause::new(expressions)
    };
    Ok(Clause::Delete(clause))
}

/// Parses DETACH DELETE clause: `DETACH DELETE expr1, expr2, ...`.
fn parse_detach_delete(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Detach)?;
    parse_delete(stream, true)
}

/// Parses REMOVE clause: `REMOVE item1, item2, ...`.
fn parse_remove(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Remove)?;
    let mut items = Vec::new();
    loop {
        items.push(parse_single_remove_item(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }
    Ok(Clause::Remove(RemoveClause::new(items)))
}

/// Parses a single REMOVE item: `n.prop` or `n:Label`.
fn parse_single_remove_item(stream: &mut TokenStream<'_, '_>) -> Result<RemoveItem, ParseError> {
    let name = super::expressions::parse_identifier(stream)?;

    // Check for label removal: `n:Label`
    if stream.at_token(&Token::Colon) {
        let node_expr = crate::types::expression::Expression::symbolic_name(name);
        let mut labels: Vec<Cow<'static, str>> = Vec::new();
        while stream.at_token(&Token::Colon) {
            stream.advance();
            labels.push(super::expressions::parse_identifier(stream)?);
        }
        return Ok(RemoveItem::label(node_expr, labels));
    }

    // Check for property removal: `n.prop`
    if stream.at_token(&Token::Dot) {
        stream.advance();
        let prop_name = super::expressions::parse_identifier(stream)?;
        let property = Property::new(
            crate::types::expression::Expression::symbolic_name(name),
            prop_name,
        );
        return Ok(RemoveItem::property(property));
    }

    Err(stream.error(
        vec![".property or :Label".to_owned()],
        vec!["REMOVE item".to_owned()],
    ))
}

/// Parses UNWIND clause: `UNWIND expr AS var`.
fn parse_unwind(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Unwind)?;
    let expr = parse_expression(stream)?;
    stream.expect_keyword(Keyword::As)?;
    let alias = super::expressions::parse_identifier(stream)?;
    let aliased = expr.alias(alias);
    Ok(Clause::Unwind(UnwindClause::new(aliased)))
}

/// Parses FOREACH clause: `FOREACH (var IN expr | clauses)`.
fn parse_foreach(stream: &mut TokenStream<'_, '_>) -> Result<Clause, ParseError> {
    stream.expect_keyword(Keyword::Foreach)?;
    stream.expect_token(&Token::LParen)?;

    let variable = super::expressions::parse_identifier(stream)?;
    stream.expect_keyword(Keyword::In)?;
    let list = parse_expression(stream)?;
    stream.expect_token(&Token::Pipe)?;

    // Parse update clauses inside FOREACH
    let mut clauses = Vec::new();
    while !stream.at_token(&Token::RParen) && !stream.is_empty() {
        let clause = parse_single_clause(stream)?;
        clauses.push(clause);
    }

    stream.expect_token(&Token::RParen)?;

    Ok(Clause::Foreach(ForeachClause::new(variable, list, clauses)))
}
