//! Pattern parsers: nodes, relationships, chains, paths.

#![allow(dead_code, reason = "pattern parsers will be used incrementally")]
#![allow(clippy::too_many_lines, reason = "pattern parsing is inherently complex")]

use super::error::ParseError;
use super::expressions::{parse_expression, parse_identifier};
use super::grammar::TokenStream;
use super::tokens::Token;
use crate::types::node::Node;
use crate::types::pattern::{
    NamedPath, PathSelector, Pattern, PatternElement, QuantifiedPath, Quantifier,
};
use crate::types::relationship::{Direction, Relationship, RelationshipDetail};
use std::borrow::Cow;

/// Parses a pattern: comma-separated pattern elements.
pub fn parse_pattern(stream: &mut TokenStream<'_, '_>) -> Result<Pattern, ParseError> {
    let mut elements = Vec::new();

    loop {
        elements.push(parse_pattern_element(stream)?);
        if stream.at_token(&Token::Comma) {
            stream.advance();
        } else {
            break;
        }
    }

    if elements.len() == 1 {
        elements.into_iter().next().map_or_else(
            || Ok(Pattern::from_elements(Vec::new())),
            |element| Ok(Pattern::new(element))
        )
    } else {
        Ok(Pattern::from_elements(elements))
    }
}

/// Parses a single pattern element, optionally with `name = ` prefix for named paths,
/// and path selectors (`SHORTEST k`, `ALL SHORTEST`, `ANY`, `SHORTEST k GROUPS`).
///
/// Named path syntax: `p = (a)-[:KNOWS]->(b)`.
/// Uses two-token lookahead: if the current token is an identifier and the next is `=`,
/// this is a named path; otherwise fall through to anonymous pattern parsing.
pub fn parse_pattern_element(stream: &mut TokenStream<'_, '_>) -> Result<PatternElement, ParseError> {
    // Check for path selectors: SHORTEST k, ALL SHORTEST, ANY, SHORTEST k GROUPS
    if let Some(selector) = try_parse_path_selector(stream)? {
        let inner = parse_anonymous_pattern(stream)?;
        return Ok(PatternElement::SelectedPath(selector, Box::new(inner)));
    }

    // Check for named path: `identifier = pattern`
    // Lookahead: identifier followed by `=` (not inside a node)
    if matches!(stream.peek(), Some(Token::Identifier(_) | Token::EscapedIdentifier(_)))
        && matches!(stream.peek_nth(1), Some(Token::Eq))
    {
        let name = parse_identifier(stream)?;
        stream.expect_token(&Token::Eq)?;
        let inner = parse_anonymous_pattern(stream)?;
        return Ok(PatternElement::NamedPath(NamedPath::new(name, inner)));
    }

    parse_anonymous_pattern(stream)
}

/// Parses an anonymous pattern (node, relationship chain, or quantified path).
///
/// Quantified path patterns are `((inner pattern) WHERE expr){quantifier}`.
/// They are disambiguated by `((` — the outer `(` is the QPP wrapper.
fn parse_anonymous_pattern(stream: &mut TokenStream<'_, '_>) -> Result<PatternElement, ParseError> {
    // Check for quantified path pattern: `(` followed by `(` indicates QPP
    if stream.at_token(&Token::LParen) && matches!(stream.peek_nth(1), Some(Token::LParen)) {
        return parse_quantified_path(stream);
    }

    let first_node = parse_node(stream)?;

    // Check if there's a relationship following
    if is_relationship_start(stream) {
        let rel_or_chain = parse_relationship_chain(stream, first_node)?;
        Ok(rel_or_chain)
    } else {
        Ok(PatternElement::Node(first_node))
    }
}

/// Returns true if the next tokens indicate a relationship pattern.
fn is_relationship_start(stream: &TokenStream<'_, '_>) -> bool {
    matches!(
        stream.peek(),
        Some(Token::Minus | Token::LeftArrow)
    )
}

/// Parses a node pattern: `(` [name] [`:Label`] [{properties}] `)`.
fn parse_node(stream: &mut TokenStream<'_, '_>) -> Result<Node, ParseError> {
    stream.expect_token(&Token::LParen)?;

    let mut name: Option<Cow<'static, str>> = None;
    let mut labels: Vec<Cow<'static, str>> = Vec::new();
    let mut properties = None;

    // Parse optional variable name
    if matches!(stream.peek(), Some(Token::Identifier(_) | Token::EscapedIdentifier(_))) {
        name = Some(parse_identifier(stream)?);
    }

    // Parse optional labels (one or more `:Label` tokens)
    while stream.at_token(&Token::Colon) {
        stream.advance();
        labels.push(parse_identifier(stream)?);
    }

    // Parse optional properties
    if stream.at_token(&Token::LBrace) {
        properties = Some(parse_expression(stream)?);
    }

    stream.expect_token(&Token::RParen)?;

    // Build the node from collected labels
    let mut node = if labels.is_empty() {
        Node::any()
    } else {
        let first = labels.remove(0);
        let n = Node::new(first);
        if labels.is_empty() { n } else { n.with_labels(labels) }
    };

    if let Some(n) = name {
        node = node.named(n);
    }

    if let Some(props) = properties {
        node = node.with_properties(props);
    }

    Ok(node)
}

/// Parses relationship patterns after the first node, building a Relationship or `RelationshipChain`.
fn parse_relationship_chain(stream: &mut TokenStream<'_, '_>, first_node: Node) -> Result<PatternElement, ParseError> {
    let mut links = Vec::new();

    while is_relationship_start(stream) {
        let (detail, direction, next_node) = parse_single_relationship(stream)?;
        links.push((detail, direction, next_node));
    }

    if links.is_empty() {
        return Ok(PatternElement::Node(first_node));
    }

    // Build the first relationship from the first link
    let (first_detail, first_dir, first_target) = links.remove(0);
    let first_rel = build_relationship(first_node, first_detail, first_dir, first_target);

    if links.is_empty() {
        return Ok(PatternElement::Relationship(first_rel));
    }

    // Build the chain
    let mut chain = first_rel.rel(links[0].0.clone()).to(links[0].2.clone());
    for (detail, _dir, target) in links.iter().skip(1) {
        chain = chain.rel(detail.clone()).to(target.clone());
    }

    Ok(PatternElement::Chain(chain))
}

/// Parses a single relationship segment: `-[...]->`, `<-[...]-`, or `-[...]-`.
///
/// Also handles quantified relationships:
/// - With brackets: `-[:R]->+`, `-[:R]->{1,3}`
/// - Without brackets (bare): `--+>`, `<--+`
fn parse_single_relationship(stream: &mut TokenStream<'_, '_>) -> Result<(RelationshipDetail, Direction, Node), ParseError> {
    // Determine direction (left side)
    let left_arrow = if stream.at_token(&Token::LeftArrow) {
        stream.advance();
        true
    } else if stream.at_token(&Token::Minus) {
        stream.advance();
        false
    } else {
        return Err(stream.error(
            vec!["- or <-".to_owned()],
            vec!["relationship pattern".to_owned()],
        ));
    };

    // Parse relationship detail inside brackets (if any)
    let mut detail = if stream.at_token(&Token::LBracket) {
        parse_relationship_detail(stream)?
    } else {
        RelationshipDetail::untyped()
    };

    // Determine right side: `->`, `-`, or error
    // For bare quantified relationships: after the second `-`, parse quantifier then `>`
    let right_arrow = if stream.at_token(&Token::Arrow) {
        stream.advance();
        true
    } else if stream.at_token(&Token::Minus) {
        stream.advance();
        // Check for bare quantifier after second `-`: `--+>`, `--*>`, `--{n}>`
        if let Some(q) = try_parse_quantifier(stream)? {
            detail = detail.quantified(q);
        }
        // Check for `>` after quantifier (indicating outgoing direction)
        if stream.at_token(&Token::Gt) {
            stream.advance();
            true
        } else {
            false
        }
    } else {
        return Err(stream.error(
            vec!["-> or -".to_owned()],
            vec!["relationship pattern".to_owned()],
        ));
    };

    // After the arrow, check for quantifier: `-[:R]->+`, `-[:R]->{1,3}`
    // This handles the case where `->` is a single Arrow token followed by quantifier
    if let Some(q) = try_parse_quantifier(stream)? {
        detail = detail.quantified(q);
    }

    // Determine direction
    let direction = match (left_arrow, right_arrow) {
        (false, true) => Direction::Outgoing,
        (true, false) => Direction::Incoming,
        (false, false) => Direction::Undirected,
        (true, true) => {
            return Err(stream.error(
                vec!["valid relationship direction".to_owned()],
                vec!["cannot have arrows on both sides".to_owned()],
            ));
        }
    };

    // Parse the target node
    let target = parse_node(stream)?;

    Ok((detail, direction, target))
}

/// Parses relationship detail: `[` [name] [`:Type`] [*range] [{properties}] `]`.
fn parse_relationship_detail(stream: &mut TokenStream<'_, '_>) -> Result<RelationshipDetail, ParseError> {
    stream.expect_token(&Token::LBracket)?;

    let mut name: Option<Cow<'static, str>> = None;
    let mut rel_type: Option<Cow<'static, str>> = None;
    let mut properties = None;
    let mut min: Option<u32> = None;
    let mut max: Option<u32> = None;

    // Parse optional variable name
    if matches!(stream.peek(), Some(Token::Identifier(_) | Token::EscapedIdentifier(_))) {
        name = Some(parse_identifier(stream)?);
    }

    // Parse optional type
    if stream.at_token(&Token::Colon) {
        stream.advance();
        rel_type = Some(parse_identifier(stream)?);
    }

    // Parse optional variable length: `*`, `*n`, `*n..m`, `*..m`, `*n..`
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, reason = "relationship length must be u32, input is i64 from lexer")]
    if stream.at_token(&Token::Star) {
        stream.advance();

        if let Some(Token::IntegerLit(n)) = stream.peek() {
            let n_val = *n as u32;
            stream.advance();

            if stream.at_token(&Token::DotDot) {
                stream.advance();
                if let Some(Token::IntegerLit(m)) = stream.peek() {
                    let m_val = *m as u32;
                    stream.advance();
                    min = Some(n_val);
                    max = Some(m_val);
                } else {
                    // *n..
                    min = Some(n_val);
                    max = None;
                }
            } else {
                // *n (exact)
                min = Some(n_val);
                max = Some(n_val);
            }
        } else if stream.at_token(&Token::DotDot) {
            // *..m
            stream.advance();
            if let Some(Token::IntegerLit(m)) = stream.peek() {
                let m_val = *m as u32;
                stream.advance();
                min = None;
                max = Some(m_val);
            } else {
                // *.. (unbounded)
                min = None;
                max = None;
            }
        } else {
            // * (unbounded)
            min = None;
            max = None;
        }
    }

    // Parse optional properties
    if stream.at_token(&Token::LBrace) {
        properties = Some(parse_expression(stream)?);
    }

    stream.expect_token(&Token::RBracket)?;

    // Parse optional quantifier after `]`: `+`, `*`, `{n}`, `{n,m}`
    let quantifier = try_parse_quantifier(stream)?;

    // Build the relationship detail
    let mut detail = rel_type.map_or_else(RelationshipDetail::untyped, RelationshipDetail::new);

    if let Some(n) = name {
        detail = detail.named(n);
    }

    if let Some(props) = properties {
        detail = detail.with_properties(props);
    }

    // Apply range (variable-length: *n..m)
    if let Some(mn) = min {
        detail = detail.min(mn);
    }
    if let Some(mx) = max {
        detail = detail.max(mx);
    } else if min.is_some() {
        detail = detail.unbounded();
    }

    // Apply quantifier (+, *, {n}, {n,m})
    if let Some(q) = quantifier {
        detail = detail.quantified(q);
    }

    Ok(detail)
}

/// Builds a Relationship from components using the builder API.
#[allow(clippy::needless_pass_by_value, reason = "builder API consumes self")]
fn build_relationship(source: Node, detail: RelationshipDetail, direction: Direction, target: Node) -> Relationship {
    match direction {
        Direction::Outgoing => source.rel(detail).to(target),
        Direction::Incoming => source.rel(detail).from(target),
        Direction::Undirected => source.rel(detail).between(target),
    }
}

/// Tries to parse a quantifier suffix: `+`, `*`, `{n}`, `{n,m}`, `{n,}`, `{,m}`.
///
/// Returns `Ok(None)` if no quantifier is present (does not consume tokens).
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, reason = "quantifier values are small non-negative integers")]
fn try_parse_quantifier(stream: &mut TokenStream<'_, '_>) -> Result<Option<Quantifier>, ParseError> {
    if stream.at_token(&Token::Plus) {
        stream.advance();
        return Ok(Some(Quantifier::Plus));
    }
    if stream.at_token(&Token::Star) {
        stream.advance();
        return Ok(Some(Quantifier::Star));
    }
    if stream.at_token(&Token::LBrace) {
        stream.advance();
        // Parse `{n}`, `{n,m}`, `{n,}`, `{,m}`
        if let Some(Token::IntegerLit(n)) = stream.peek() {
            let n_val = *n as u32;
            stream.advance();
            if stream.at_token(&Token::Comma) {
                stream.advance();
                // `{n,m}` or `{n,}`
                if let Some(Token::IntegerLit(m)) = stream.peek() {
                    let m_val = *m as u32;
                    stream.advance();
                    stream.expect_token(&Token::RBrace)?;
                    Ok(Some(Quantifier::Range {
                        min: Some(n_val),
                        max: Some(m_val),
                    }))
                } else {
                    stream.expect_token(&Token::RBrace)?;
                    Ok(Some(Quantifier::Range {
                        min: Some(n_val),
                        max: None,
                    }))
                }
            } else {
                // `{n}` exact
                stream.expect_token(&Token::RBrace)?;
                Ok(Some(Quantifier::Exact(n_val)))
            }
        } else if stream.at_token(&Token::Comma) {
            // `{,m}`
            stream.advance();
            if let Some(Token::IntegerLit(m)) = stream.peek() {
                let m_val = *m as u32;
                stream.advance();
                stream.expect_token(&Token::RBrace)?;
                Ok(Some(Quantifier::Range {
                    min: None,
                    max: Some(m_val),
                }))
            } else {
                stream.expect_token(&Token::RBrace)?;
                Ok(Some(Quantifier::Range {
                    min: None,
                    max: None,
                }))
            }
        } else {
            Err(stream.error(
                vec!["integer or comma".to_owned()],
                vec!["quantifier".to_owned()],
            ))
        }
    } else {
        Ok(None)
    }
}

/// Tries to parse a path selector: `SHORTEST k`, `ALL SHORTEST`, `ANY`, `SHORTEST k GROUPS`.
///
/// Returns `Ok(None)` if no path selector is present (does not consume tokens).
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, reason = "path selector k values are small non-negative integers")]
fn try_parse_path_selector(stream: &mut TokenStream<'_, '_>) -> Result<Option<PathSelector>, ParseError> {
    use super::tokens::Keyword;

    if stream.at_keyword(Keyword::Shortest) {
        stream.advance();
        // Must have an integer k after SHORTEST
        if let Some(Token::IntegerLit(k)) = stream.peek() {
            let k_val = *k as u32;
            stream.advance();
            // Check for GROUPS
            if stream.at_keyword(Keyword::Groups) {
                stream.advance();
                Ok(Some(PathSelector::ShortestGroups(k_val)))
            } else {
                Ok(Some(PathSelector::Shortest(k_val)))
            }
        } else {
            Err(stream.error(
                vec!["integer".to_owned()],
                vec!["SHORTEST k".to_owned()],
            ))
        }
    } else if stream.at_keyword(Keyword::All) && matches!(stream.peek_nth(1), Some(Token::Keyword(Keyword::Shortest))) {
        stream.advance(); // ALL
        stream.advance(); // SHORTEST
        Ok(Some(PathSelector::AllShortest))
    } else if stream.at_keyword(Keyword::Any) && matches!(stream.peek_nth(1), Some(Token::LParen)) {
        // ANY followed by `(` — this is a path selector, not the ANY() function
        stream.advance();
        Ok(Some(PathSelector::Any))
    } else {
        Ok(None)
    }
}

/// Parses a quantified path pattern: `(inner_pattern WHERE expr){quantifier}`.
///
/// The leading `(` has already been confirmed by lookahead (we saw `((` ).
fn parse_quantified_path(stream: &mut TokenStream<'_, '_>) -> Result<PatternElement, ParseError> {
    stream.expect_token(&Token::LParen)?; // outer `(`

    // Parse the inner pattern element
    let inner = parse_anonymous_pattern(stream)?;

    // Parse optional WHERE clause inside the QPP
    let where_clause = if stream.at_keyword(super::tokens::Keyword::Where) {
        stream.advance();
        Some(parse_expression(stream)?)
    } else {
        None
    };

    stream.expect_token(&Token::RParen)?; // outer `)`

    // Parse the quantifier suffix (required for a QPP)
    let quantifier = try_parse_quantifier(stream)?;
    let quantifier = quantifier.ok_or_else(|| {
        stream.error(
            vec!["+, *, or {quantifier}".to_owned()],
            vec!["quantified path pattern requires a quantifier".to_owned()],
        )
    })?;

    let mut qp = QuantifiedPath::new(inner, quantifier);
    if let Some(w) = where_clause {
        qp = qp.where_(w);
    }

    Ok(PatternElement::QuantifiedPath(qp))
}
