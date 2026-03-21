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
    Clause, FilterClause, LetClause, LimitClause, MatchClause, OrderByClause, ReturnClause,
    SkipClause, WhereClause, WithClause,
};

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
