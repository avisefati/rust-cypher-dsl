//! Parser for admin commands (CREATE/DROP INDEX, CREATE/DROP CONSTRAINT,
//! SHOW commands, TERMINATE TRANSACTIONS).

use std::borrow::Cow;

use super::error::ParseError;
use super::grammar::TokenStream;
use super::tokens::{Keyword, Token};
use crate::admin::{
    AdminCommand, ConstraintTarget, ConstraintType, CreateConstraint, CreateIndex, DropConstraint,
    DropIndex, ExecutableFilter, IndexTarget, IndexType, ShowCommand, TerminateTransactions,
};
use crate::statement::Statement;
use crate::types::expression::Expression;

/// Tries to parse an admin command from the token stream.
///
/// Returns `Some(Statement::Admin(...))` if the stream starts with an admin keyword
/// (CREATE INDEX, CREATE CONSTRAINT, DROP INDEX, DROP CONSTRAINT, SHOW, TERMINATE).
/// Returns `None` if the stream does not start with an admin command (so the
/// caller can fall back to regular clause parsing).
///
/// # Errors
///
/// Returns `ParseError` if the stream starts with an admin keyword but the
/// rest of the command is malformed.
pub fn try_parse_admin(stream: &mut TokenStream<'_, '_>) -> Result<Option<Statement>, ParseError> {
    match stream.peek() {
        Some(Token::Keyword(Keyword::Show)) => parse_show_command(stream).map(Some),
        Some(Token::Keyword(Keyword::Terminate)) => parse_terminate(stream).map(Some),
        Some(Token::Keyword(Keyword::Drop)) => try_parse_drop(stream),
        Some(Token::Keyword(Keyword::Create)) => try_parse_create_admin(stream),
        _ => Ok(None),
    }
}

// ── CREATE INDEX / CREATE CONSTRAINT ──

/// Tries to parse a CREATE INDEX or CREATE CONSTRAINT admin command.
///
/// Returns `None` if the CREATE is followed by something else (e.g., CREATE (n:Person)).
fn try_parse_create_admin(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<Statement>, ParseError> {
    // Save position to backtrack if this is not an admin CREATE.
    let save_pos = stream.pos();

    stream.advance(); // consume CREATE

    // Check for optional index type keyword before INDEX
    match stream.peek() {
        Some(Token::Keyword(Keyword::Index)) => {
            let stmt = parse_create_index(stream, IndexType::Range)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Text)) => {
            stream.advance();
            stream.expect_keyword(Keyword::Index)?;
            let stmt = parse_create_index(stream, IndexType::Text)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Point)) => {
            stream.advance();
            stream.expect_keyword(Keyword::Index)?;
            let stmt = parse_create_index(stream, IndexType::Point)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Fulltext)) => {
            stream.advance();
            stream.expect_keyword(Keyword::Index)?;
            let stmt = parse_create_index(stream, IndexType::Fulltext)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Vector)) => {
            stream.advance();
            stream.expect_keyword(Keyword::Index)?;
            let stmt = parse_create_index(stream, IndexType::Vector)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Lookup)) => {
            stream.advance();
            stream.expect_keyword(Keyword::Index)?;
            let stmt = parse_create_index(stream, IndexType::Lookup)?;
            Ok(Some(stmt))
        }
        Some(Token::Keyword(Keyword::Constraint)) => {
            let stmt = parse_create_constraint(stream)?;
            Ok(Some(stmt))
        }
        _ => {
            // Not an admin CREATE — backtrack.
            stream.set_pos(save_pos);
            Ok(None)
        }
    }
}

/// Parses the rest of a CREATE INDEX statement after `CREATE [TYPE] INDEX`.
///
/// Grammar: `INDEX [name] [IF NOT EXISTS] FOR target ON properties [OPTIONS expr]`
fn parse_create_index(
    stream: &mut TokenStream<'_, '_>,
    index_type: IndexType,
) -> Result<Statement, ParseError> {
    // Consume INDEX keyword (for Range type, it hasn't been consumed yet)
    if stream.at_keyword(Keyword::Index) {
        stream.advance();
    }

    // Parse optional name
    let name = parse_optional_identifier(stream);

    // Parse optional IF NOT EXISTS
    let if_not_exists = parse_if_not_exists(stream)?;

    // Parse FOR target ON properties
    stream.expect_keyword(Keyword::For)?;
    let target = parse_index_target(stream, &index_type)?;

    // Parse optional OPTIONS
    let options = if stream.at_keyword(Keyword::Options) {
        stream.advance();
        Some(parse_raw_expression_until_end(stream))
    } else {
        None
    };

    let mut ci = CreateIndex::new(index_type, name, if_not_exists, target);
    if let Some(opts) = options {
        ci = ci.with_options(opts);
    }

    Ok(Statement::Admin(AdminCommand::CreateIndex(ci)))
}

/// Parses the index target: `(var:Label) ON (var.prop)` or `()-[var:TYPE]-() ON (var.prop)`.
fn parse_index_target(
    stream: &mut TokenStream<'_, '_>,
    index_type: &IndexType,
) -> Result<IndexTarget, ParseError> {
    // Peek to see if this is a node target or relationship target.
    // Node target starts with `(var:Label)` or `(var)` for lookup.
    // Relationship target starts with `()-[...`.
    // We distinguish by: after `(`, is it `)` (meaning empty node for relationship pattern)?

    stream.expect_token(&Token::LParen)?;

    if matches!(stream.peek(), Some(Token::RParen)) {
        // ()-[ ... ]-() — relationship target
        stream.advance(); // consume )
        stream.expect_token(&Token::Minus)?;
        stream.expect_token(&Token::LBracket)?;

        let var = expect_identifier(stream)?;

        if matches!(stream.peek(), Some(Token::Colon)) {
            // ()-[var:TYPE]-() — normal relationship index
            stream.advance(); // consume :
            let types = parse_pipe_separated_identifiers(stream)?;
            stream.expect_token(&Token::RBracket)?;
            stream.expect_token(&Token::Minus)?;
            stream.expect_token(&Token::LParen)?;
            stream.expect_token(&Token::RParen)?;
            stream.expect_keyword(Keyword::On)?;

            let properties = if matches!(index_type, IndexType::Fulltext) {
                stream.expect_keyword(Keyword::Each)?;
                parse_bracketed_properties(stream)?
            } else {
                parse_parenthesized_properties(stream)?
            };

            Ok(IndexTarget::Relationship {
                variable: Cow::Owned(var),
                types: types.into_iter().map(Cow::Owned).collect(),
                properties: properties.into_iter().map(Cow::Owned).collect(),
            })
        } else {
            // ()-[var]-() — relationship lookup
            stream.expect_token(&Token::RBracket)?;
            stream.expect_token(&Token::Minus)?;
            stream.expect_token(&Token::LParen)?;
            stream.expect_token(&Token::RParen)?;
            stream.expect_keyword(Keyword::On)?;
            stream.expect_keyword(Keyword::Each)?;
            // "type(var)"
            expect_identifier(stream)?; // "type"
            stream.expect_token(&Token::LParen)?;
            expect_identifier(stream)?; // var
            stream.expect_token(&Token::RParen)?;

            Ok(IndexTarget::RelationshipLookup {
                variable: Cow::Owned(var),
            })
        }
    } else {
        // Node target: (var:Label) or (var) for lookup
        let var = expect_identifier(stream)?;

        if matches!(stream.peek(), Some(Token::Colon)) {
            // (var:Label) — normal node index
            stream.advance(); // consume :
            let labels = parse_pipe_separated_identifiers(stream)?;
            stream.expect_token(&Token::RParen)?;
            stream.expect_keyword(Keyword::On)?;

            let properties = if matches!(index_type, IndexType::Fulltext) {
                stream.expect_keyword(Keyword::Each)?;
                parse_bracketed_properties(stream)?
            } else {
                parse_parenthesized_properties(stream)?
            };

            Ok(IndexTarget::Node {
                variable: Cow::Owned(var),
                labels: labels.into_iter().map(Cow::Owned).collect(),
                properties: properties.into_iter().map(Cow::Owned).collect(),
            })
        } else {
            // (var) — lookup
            stream.expect_token(&Token::RParen)?;
            stream.expect_keyword(Keyword::On)?;
            stream.expect_keyword(Keyword::Each)?;
            // "labels(var)" or "type(var)"
            let func_name = expect_identifier(stream)?;
            stream.expect_token(&Token::LParen)?;
            expect_identifier(stream)?; // re-consume var inside function
            stream.expect_token(&Token::RParen)?;

            if func_name.eq_ignore_ascii_case("labels") {
                Ok(IndexTarget::NodeLookup {
                    variable: Cow::Owned(var),
                })
            } else {
                Ok(IndexTarget::RelationshipLookup {
                    variable: Cow::Owned(var),
                })
            }
        }
    }
}

/// Parses `(var.prop1, var.prop2)` — parenthesized properties.
fn parse_parenthesized_properties(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Vec<String>, ParseError> {
    stream.expect_token(&Token::LParen)?;
    let mut props = Vec::new();
    loop {
        expect_identifier(stream)?; // var
        stream.expect_token(&Token::Dot)?;
        props.push(expect_identifier(stream)?);
        if !matches!(stream.peek(), Some(Token::Comma)) {
            break;
        }
        stream.advance(); // consume ,
    }
    stream.expect_token(&Token::RParen)?;
    Ok(props)
}

/// Parses `[var.prop1, var.prop2]` — bracketed properties for FULLTEXT.
fn parse_bracketed_properties(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Vec<String>, ParseError> {
    stream.expect_token(&Token::LBracket)?;
    let mut props = Vec::new();
    loop {
        expect_identifier(stream)?; // var
        stream.expect_token(&Token::Dot)?;
        props.push(expect_identifier(stream)?);
        if !matches!(stream.peek(), Some(Token::Comma)) {
            break;
        }
        stream.advance(); // consume ,
    }
    stream.expect_token(&Token::RBracket)?;
    Ok(props)
}

// ── CREATE CONSTRAINT ──

/// Parses `CREATE CONSTRAINT [name] [IF NOT EXISTS] FOR target REQUIRE spec`.
fn parse_create_constraint(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    stream.advance(); // consume CONSTRAINT

    let name = parse_optional_identifier(stream);
    let if_not_exists = parse_if_not_exists(stream)?;

    stream.expect_keyword(Keyword::For)?;
    let target = parse_constraint_target(stream)?;

    stream.expect_keyword(Keyword::Require)?;
    let (properties, constraint_type) = parse_constraint_specification(stream)?;

    let cc = CreateConstraint::new(name, if_not_exists, target, properties, constraint_type);
    Ok(Statement::Admin(AdminCommand::CreateConstraint(cc)))
}

/// Parses the constraint target: `(var:Label)` or `()-[var:TYPE]-()`.
fn parse_constraint_target(
    stream: &mut TokenStream<'_, '_>,
) -> Result<ConstraintTarget, ParseError> {
    stream.expect_token(&Token::LParen)?;

    // Check if next token is ), meaning empty node → relationship target: ()-[...]
    if matches!(stream.peek(), Some(Token::RParen)) {
        // ()-[var:TYPE]-()
        stream.advance(); // )
        stream.expect_token(&Token::Minus)?;
        stream.expect_token(&Token::LBracket)?;
        let var = expect_identifier(stream)?;
        stream.expect_token(&Token::Colon)?;
        let rel_type = expect_identifier(stream)?;
        stream.expect_token(&Token::RBracket)?;
        stream.expect_token(&Token::Minus)?;
        stream.expect_token(&Token::LParen)?;
        stream.expect_token(&Token::RParen)?;
        Ok(ConstraintTarget::Relationship {
            variable: Cow::Owned(var),
            rel_type: Cow::Owned(rel_type),
        })
    } else {
        // (var:Label)
        let var = expect_identifier(stream)?;
        stream.expect_token(&Token::Colon)?;
        let label = expect_identifier(stream)?;
        stream.expect_token(&Token::RParen)?;
        Ok(ConstraintTarget::Node {
            variable: Cow::Owned(var),
            label: Cow::Owned(label),
        })
    }
}

/// Parses the REQUIRE specification: `var.prop IS UNIQUE`, `(var.prop1, var.prop2) IS NODE KEY`, etc.
fn parse_constraint_specification(
    stream: &mut TokenStream<'_, '_>,
) -> Result<(Vec<Cow<'static, str>>, ConstraintType), ParseError> {
    let properties = if matches!(stream.peek(), Some(Token::LParen)) {
        // Composite: (var.prop1, var.prop2)
        stream.advance(); // (
        let mut props = Vec::new();
        loop {
            expect_identifier(stream)?; // var
            stream.expect_token(&Token::Dot)?;
            props.push(Cow::Owned(expect_identifier(stream)?));
            if !matches!(stream.peek(), Some(Token::Comma)) {
                break;
            }
            stream.advance(); // ,
        }
        stream.expect_token(&Token::RParen)?;
        props
    } else {
        // Single: var.prop
        expect_identifier(stream)?; // var
        stream.expect_token(&Token::Dot)?;
        vec![Cow::Owned(expect_identifier(stream)?)]
    };

    stream.expect_keyword(Keyword::Is)?;

    // Determine constraint type
    let constraint_type = if stream.at_keyword(Keyword::Unique) {
        stream.advance();
        ConstraintType::Unique
    } else if stream.at_keyword(Keyword::Not) {
        stream.advance(); // NOT
        stream.expect_keyword(Keyword::Null)?;
        ConstraintType::Exists
    } else if stream.at_keyword(Keyword::Node) {
        stream.advance(); // NODE
        stream.expect_keyword(Keyword::Key)?;
        ConstraintType::NodeKey
    } else if stream.at_keyword(Keyword::Relationship) {
        stream.advance(); // RELATIONSHIP
        stream.expect_keyword(Keyword::Key)?;
        ConstraintType::RelationshipKey
    } else if matches!(stream.peek(), Some(Token::Colon)) {
        // IS :: TYPE — the lexer produces two Colon tokens for `::`
        stream.advance(); // first :
        stream.expect_token(&Token::Colon)?; // second :
        let type_name = expect_identifier(stream)?;
        ConstraintType::PropertyType(Cow::Owned(type_name))
    } else {
        return Err(stream.error(
            vec![
                "UNIQUE".to_string(),
                "NOT NULL".to_string(),
                "NODE KEY".to_string(),
                "RELATIONSHIP KEY".to_string(),
                ":: TYPE".to_string(),
            ],
            vec!["expected constraint type".to_string()],
        ));
    };

    Ok((properties, constraint_type))
}

// ── DROP INDEX / DROP CONSTRAINT ──

fn try_parse_drop(stream: &mut TokenStream<'_, '_>) -> Result<Option<Statement>, ParseError> {
    let save_pos = stream.pos();
    stream.advance(); // consume DROP

    if stream.at_keyword(Keyword::Index) {
        stream.advance(); // consume INDEX
        let name = expect_identifier(stream)?;
        let if_exists = parse_if_exists(stream)?;
        Ok(Some(Statement::Admin(AdminCommand::DropIndex(
            DropIndex::new(name, if_exists),
        ))))
    } else if stream.at_keyword(Keyword::Constraint) {
        stream.advance(); // consume CONSTRAINT
        let name = expect_identifier(stream)?;
        let if_exists = parse_if_exists(stream)?;
        Ok(Some(Statement::Admin(AdminCommand::DropConstraint(
            DropConstraint::new(name, if_exists),
        ))))
    } else {
        stream.set_pos(save_pos);
        Ok(None)
    }
}

// ── SHOW commands ──

#[allow(clippy::too_many_lines, reason = "SHOW command parsing has many sequential steps")]
fn parse_show_command(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    stream.advance(); // consume SHOW

    // Collect type filter words: RANGE, TEXT, POINT, FULLTEXT, UNIQUE, ALL, BUILT IN, USER DEFINED
    let mut type_filter_words: Vec<String> = Vec::new();

    // Check for multi-word type filters before the main keyword
    loop {
        match stream.peek() {
            Some(Token::Keyword(
                Keyword::Indexes
                | Keyword::Constraints
                | Keyword::Functions
                | Keyword::Procedures
                | Keyword::Transactions,
            )) => break,
            Some(Token::Keyword(kw)) => {
                let word = kw.to_string();
                stream.advance();
                type_filter_words.push(word);
            }
            Some(Token::Identifier(ident)) => {
                let word = (*ident).to_string();
                stream.advance();
                type_filter_words.push(word.to_ascii_uppercase());
            }
            _ => break,
        }
    }

    let type_filter: Option<Cow<'static, str>> = if type_filter_words.is_empty() {
        None
    } else {
        Some(Cow::Owned(type_filter_words.join(" ")))
    };

    // Parse the main keyword
    let (kind, mut sc) = if stream.at_keyword(Keyword::Indexes) {
        stream.advance();
        ("indexes", ShowCommand::new())
    } else if stream.at_keyword(Keyword::Constraints) {
        stream.advance();
        ("constraints", ShowCommand::new())
    } else if stream.at_keyword(Keyword::Functions) {
        stream.advance();
        ("functions", ShowCommand::new())
    } else if stream.at_keyword(Keyword::Procedures) {
        stream.advance();
        ("procedures", ShowCommand::new())
    } else if stream.at_keyword(Keyword::Transactions) {
        stream.advance();
        ("transactions", ShowCommand::new())
    } else {
        return Err(stream.error(
            vec![
                "INDEXES".into(),
                "CONSTRAINTS".into(),
                "FUNCTIONS".into(),
                "PROCEDURES".into(),
                "TRANSACTIONS".into(),
            ],
            vec!["expected SHOW target".into()],
        ));
    };

    if let Some(filter) = type_filter {
        sc = sc.with_type_filter(filter);
    }

    // Parse optional transaction IDs (for SHOW TRANSACTIONS only)
    if kind == "transactions"
        && matches!(stream.peek(), Some(Token::StringLit(_)))
    {
        let mut ids = Vec::new();
        while let Some(Token::StringLit(s)) = stream.peek() {
            let id = s.clone();
            stream.advance();
            ids.push(Cow::Owned(id));
            if matches!(stream.peek(), Some(Token::Comma)) {
                stream.advance();
            } else {
                break;
            }
        }
        if !ids.is_empty() {
            sc = sc.with_transaction_ids(ids);
        }
    }

    // Parse optional EXECUTABLE BY
    if stream.at_keyword(Keyword::Executable) {
        stream.advance(); // EXECUTABLE
        stream.expect_keyword(Keyword::By)?;
        if stream.at_keyword(Keyword::Current) {
            stream.advance(); // CURRENT
            stream.expect_keyword(Keyword::User)?;
            sc = sc.with_executable(ExecutableFilter::CurrentUser);
        } else {
            let user = expect_identifier(stream)?;
            sc = sc.with_executable(ExecutableFilter::User(Cow::Owned(user)));
        }
    }

    // Parse optional YIELD
    if stream.at_keyword(Keyword::Yield) {
        stream.advance();
        if matches!(stream.peek(), Some(Token::Star)) {
            stream.advance();
            sc = sc.with_yield_all();
        } else {
            let fields = parse_yield_fields(stream)?;
            sc = sc.with_yield_fields(fields);
        }
    }

    // Parse optional WHERE (only after YIELD)
    if stream.at_keyword(Keyword::Where) {
        stream.advance();
        let cond = super::conditions::parse_condition(stream)?;
        sc = sc.with_where(cond);
    }

    let stmt = match kind {
        "indexes" => Statement::Admin(AdminCommand::ShowIndexes(sc)),
        "constraints" => Statement::Admin(AdminCommand::ShowConstraints(sc)),
        "functions" => Statement::Admin(AdminCommand::ShowFunctions(sc)),
        "procedures" => Statement::Admin(AdminCommand::ShowProcedures(sc)),
        "transactions" => Statement::Admin(AdminCommand::ShowTransactions(sc)),
        _ => unreachable!(),
    };

    Ok(stmt)
}

// ── TERMINATE TRANSACTIONS ──

fn parse_terminate(stream: &mut TokenStream<'_, '_>) -> Result<Statement, ParseError> {
    stream.advance(); // consume TERMINATE
    stream.expect_keyword(Keyword::Transactions)?;

    // Parse transaction IDs (string literals)
    let mut ids: Vec<Cow<'static, str>> = Vec::new();
    while let Some(Token::StringLit(s)) = stream.peek() {
        let id = s.clone();
        stream.advance();
        ids.push(Cow::Owned(id));
        if matches!(stream.peek(), Some(Token::Comma)) {
            stream.advance();
        } else {
            break;
        }
    }

    let mut tt = TerminateTransactions::new(ids);

    // Parse optional YIELD
    if stream.at_keyword(Keyword::Yield) {
        stream.advance();
        if matches!(stream.peek(), Some(Token::Star)) {
            stream.advance();
            tt = tt.with_yield_all();
        } else {
            let fields = parse_yield_fields(stream)?;
            tt = tt.with_yield_fields(fields);
        }
    }

    // Parse optional WHERE
    if stream.at_keyword(Keyword::Where) {
        stream.advance();
        let cond = super::conditions::parse_condition(stream)?;
        tt = tt.with_where(cond);
    }

    Ok(Statement::Admin(AdminCommand::TerminateTransactions(tt)))
}

// ── Helpers ──

/// Parses a comma-separated list of YIELD field identifiers.
fn parse_yield_fields(stream: &mut TokenStream<'_, '_>) -> Result<Vec<Expression>, ParseError> {
    let mut fields = Vec::new();
    loop {
        let field_name = expect_identifier(stream)?;
        fields.push(Expression::symbolic_name(field_name));
        if matches!(stream.peek(), Some(Token::Comma)) {
            stream.advance();
        } else {
            break;
        }
    }
    Ok(fields)
}

/// Tries to parse `IF NOT EXISTS`.
fn parse_if_not_exists(stream: &mut TokenStream<'_, '_>) -> Result<bool, ParseError> {
    if stream.at_keyword(Keyword::If) {
        stream.advance(); // IF
        stream.expect_keyword(Keyword::Not)?;
        stream.expect_keyword(Keyword::Exists)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Tries to parse `IF EXISTS`.
fn parse_if_exists(stream: &mut TokenStream<'_, '_>) -> Result<bool, ParseError> {
    if stream.at_keyword(Keyword::If) {
        stream.advance(); // IF
        stream.expect_keyword(Keyword::Exists)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Tries to parse an identifier at the current position; returns None if not found.
fn parse_optional_identifier(stream: &mut TokenStream<'_, '_>) -> Option<Cow<'static, str>> {
    match stream.peek() {
        Some(Token::Identifier(name)) => {
            let s = (*name).to_string();
            stream.advance();
            Some(Cow::Owned(s))
        }
        _ => None,
    }
}

/// Expects an identifier at the current position.
///
/// In admin command contexts, many keywords can also be used as identifiers
/// (e.g., TYPE, USER, NODE, etc.). This function accepts both identifiers
/// and keywords.
fn expect_identifier(stream: &mut TokenStream<'_, '_>) -> Result<String, ParseError> {
    match stream.peek() {
        Some(Token::Identifier(name)) => {
            let s = (*name).to_string();
            stream.advance();
            Ok(s)
        }
        Some(Token::EscapedIdentifier(name)) => {
            let s = name.clone();
            stream.advance();
            Ok(s)
        }
        // Some keywords can also be used as identifiers in admin contexts.
        // We lowercase them so that e.g. YIELD `type` renders as `type` not `TYPE`.
        Some(Token::Keyword(kw)) => {
            let s = kw.to_string().to_ascii_lowercase();
            stream.advance();
            Ok(s)
        }
        _ => Err(stream.error(
            vec!["identifier".to_string()],
            vec!["expected identifier".to_string()],
        )),
    }
}

/// Parses pipe-separated identifiers: `Label1|Label2|Label3`.
fn parse_pipe_separated_identifiers(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Vec<String>, ParseError> {
    let mut names = vec![expect_identifier(stream)?];
    while matches!(stream.peek(), Some(Token::Pipe)) {
        stream.advance();
        names.push(expect_identifier(stream)?);
    }
    Ok(names)
}

/// Reads all remaining tokens as a raw expression string until end of stream.
fn parse_raw_expression_until_end(stream: &mut TokenStream<'_, '_>) -> Expression {
    // Collect remaining text as a raw expression
    let start_offset = stream.offset();
    while !stream.is_empty() {
        stream.advance();
    }
    let end_offset = stream.offset();
    let raw = stream.input()[start_offset..end_offset].trim();
    // Convert to owned string since Expression::raw_unchecked requires 'static
    Expression::raw_unchecked(raw.to_owned())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "tests use unwrap/panic for assertions"
)]
mod tests {
    use crate::parser::parse;

    /// Helper: parse and render, asserting the output.
    fn assert_parses_to(input: &str, expected: &str) {
        let stmt = parse(input).unwrap_or_else(|e| panic!("Failed to parse '{input}': {e}"));
        assert_eq!(stmt.render(), expected, "\nInput: {input}");
    }

    // ── CREATE INDEX ──

    #[test]
    fn parse_create_range_index() {
        assert_parses_to(
            "CREATE INDEX person_name FOR (n:Person) ON (n.name)",
            "CREATE INDEX person_name FOR (n:Person) ON (n.name)",
        );
    }

    #[test]
    fn parse_create_range_index_composite() {
        assert_parses_to(
            "CREATE INDEX person_composite FOR (n:Person) ON (n.firstName, n.lastName)",
            "CREATE INDEX person_composite FOR (n:Person) ON (n.firstName, n.lastName)",
        );
    }

    #[test]
    fn parse_create_text_index_if_not_exists() {
        assert_parses_to(
            "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)",
            "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)",
        );
    }

    #[test]
    fn parse_create_point_index() {
        assert_parses_to(
            "CREATE POINT INDEX loc_idx FOR (n:Place) ON (n.location)",
            "CREATE POINT INDEX loc_idx FOR (n:Place) ON (n.location)",
        );
    }

    #[test]
    fn parse_create_fulltext_index() {
        assert_parses_to(
            "CREATE FULLTEXT INDEX ft FOR (n:Movie) ON EACH [n.title, n.description]",
            "CREATE FULLTEXT INDEX ft FOR (n:Movie) ON EACH [n.title, n.description]",
        );
    }

    #[test]
    fn parse_create_fulltext_multi_label() {
        assert_parses_to(
            "CREATE FULLTEXT INDEX ft FOR (n:Movie|Book) ON EACH [n.title, n.summary]",
            "CREATE FULLTEXT INDEX ft FOR (n:Movie|Book) ON EACH [n.title, n.summary]",
        );
    }

    #[test]
    fn parse_create_lookup_index_node() {
        assert_parses_to(
            "CREATE LOOKUP INDEX node_lookup FOR (n) ON EACH labels(n)",
            "CREATE LOOKUP INDEX node_lookup FOR (n) ON EACH labels(n)",
        );
    }

    #[test]
    fn parse_create_lookup_index_relationship() {
        assert_parses_to(
            "CREATE LOOKUP INDEX rel_lookup FOR ()-[r]-() ON EACH type(r)",
            "CREATE LOOKUP INDEX rel_lookup FOR ()-[r]-() ON EACH type(r)",
        );
    }

    #[test]
    fn parse_create_index_for_relationship() {
        assert_parses_to(
            "CREATE INDEX rel_idx FOR ()-[r:KNOWS]-() ON (r.since)",
            "CREATE INDEX rel_idx FOR ()-[r:KNOWS]-() ON (r.since)",
        );
    }

    #[test]
    fn parse_create_fulltext_index_relationship_multi_type() {
        assert_parses_to(
            "CREATE FULLTEXT INDEX ft_rels FOR ()-[r:KNOWS|WORKS_WITH]-() ON EACH [r.note]",
            "CREATE FULLTEXT INDEX ft_rels FOR ()-[r:KNOWS|WORKS_WITH]-() ON EACH [r.note]",
        );
    }

    // ── DROP INDEX ──

    #[test]
    fn parse_drop_index() {
        assert_parses_to("DROP INDEX my_index", "DROP INDEX my_index");
    }

    #[test]
    fn parse_drop_index_if_exists() {
        assert_parses_to(
            "DROP INDEX my_index IF EXISTS",
            "DROP INDEX my_index IF EXISTS",
        );
    }

    // ── CREATE CONSTRAINT ──

    #[test]
    fn parse_create_unique_constraint() {
        assert_parses_to(
            "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE",
            "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE",
        );
    }

    #[test]
    fn parse_create_unique_constraint_composite() {
        assert_parses_to(
            "CREATE CONSTRAINT unique_name FOR (n:Person) REQUIRE (n.firstName, n.lastName) IS UNIQUE",
            "CREATE CONSTRAINT unique_name FOR (n:Person) REQUIRE (n.firstName, n.lastName) IS UNIQUE",
        );
    }

    #[test]
    fn parse_create_existence_constraint() {
        assert_parses_to(
            "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.name IS NOT NULL",
            "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.name IS NOT NULL",
        );
    }

    #[test]
    fn parse_create_node_key_composite() {
        assert_parses_to(
            "CREATE CONSTRAINT person_key FOR (n:Person) REQUIRE (n.id, n.name) IS NODE KEY",
            "CREATE CONSTRAINT person_key FOR (n:Person) REQUIRE (n.id, n.name) IS NODE KEY",
        );
    }

    #[test]
    fn parse_create_relationship_key() {
        assert_parses_to(
            "CREATE CONSTRAINT rel_key FOR ()-[r:REVIEWED]-() REQUIRE r.id IS RELATIONSHIP KEY",
            "CREATE CONSTRAINT rel_key FOR ()-[r:REVIEWED]-() REQUIRE r.id IS RELATIONSHIP KEY",
        );
    }

    #[test]
    fn parse_create_property_type_constraint() {
        assert_parses_to(
            "CREATE CONSTRAINT score_type FOR ()-[r:REVIEWED]-() REQUIRE r.score IS :: FLOAT",
            "CREATE CONSTRAINT score_type FOR ()-[r:REVIEWED]-() REQUIRE r.score IS :: FLOAT",
        );
    }

    #[test]
    fn parse_create_existence_constraint_on_relationship() {
        assert_parses_to(
            "CREATE CONSTRAINT rel_date FOR ()-[r:WORKS_AT]-() REQUIRE r.startDate IS NOT NULL",
            "CREATE CONSTRAINT rel_date FOR ()-[r:WORKS_AT]-() REQUIRE r.startDate IS NOT NULL",
        );
    }

    // ── DROP CONSTRAINT ──

    #[test]
    fn parse_drop_constraint() {
        assert_parses_to(
            "DROP CONSTRAINT my_constraint",
            "DROP CONSTRAINT my_constraint",
        );
    }

    #[test]
    fn parse_drop_constraint_if_exists() {
        assert_parses_to(
            "DROP CONSTRAINT my_constraint IF EXISTS",
            "DROP CONSTRAINT my_constraint IF EXISTS",
        );
    }

    // ── SHOW ──

    #[test]
    fn parse_show_indexes() {
        assert_parses_to("SHOW INDEXES", "SHOW INDEXES");
    }

    #[test]
    fn parse_show_range_indexes_yield_all() {
        assert_parses_to("SHOW RANGE INDEXES YIELD *", "SHOW RANGE INDEXES YIELD *");
    }

    #[test]
    fn parse_show_constraints() {
        assert_parses_to("SHOW CONSTRAINTS", "SHOW CONSTRAINTS");
    }

    #[test]
    fn parse_show_unique_constraints() {
        assert_parses_to("SHOW UNIQUE CONSTRAINTS", "SHOW UNIQUE CONSTRAINTS");
    }

    #[test]
    fn parse_show_functions() {
        assert_parses_to("SHOW FUNCTIONS", "SHOW FUNCTIONS");
    }

    #[test]
    fn parse_show_built_in_functions() {
        assert_parses_to("SHOW BUILT IN FUNCTIONS", "SHOW BUILT IN FUNCTIONS");
    }

    #[test]
    fn parse_show_functions_executable() {
        assert_parses_to(
            "SHOW FUNCTIONS EXECUTABLE BY CURRENT USER",
            "SHOW FUNCTIONS EXECUTABLE BY CURRENT USER",
        );
    }

    #[test]
    fn parse_show_procedures() {
        assert_parses_to("SHOW PROCEDURES", "SHOW PROCEDURES");
    }

    #[test]
    fn parse_show_transactions() {
        assert_parses_to("SHOW TRANSACTIONS", "SHOW TRANSACTIONS");
    }

    #[test]
    fn parse_show_transactions_with_ids() {
        assert_parses_to(
            "SHOW TRANSACTIONS 'neo4j-tx-123'",
            "SHOW TRANSACTIONS 'neo4j-tx-123'",
        );
    }

    #[test]
    fn parse_show_transactions_multiple_ids() {
        assert_parses_to(
            "SHOW TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
            "SHOW TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
        );
    }

    #[test]
    fn parse_show_indexes_yield_fields_where() {
        assert_parses_to(
            "SHOW INDEXES YIELD name, type, state WHERE state = 'ONLINE'",
            "SHOW INDEXES YIELD name, type, state WHERE state = 'ONLINE'",
        );
    }

    #[test]
    fn parse_show_constraints_yield_all() {
        assert_parses_to("SHOW CONSTRAINTS YIELD *", "SHOW CONSTRAINTS YIELD *");
    }

    // ── TERMINATE TRANSACTIONS ──

    #[test]
    fn parse_terminate_transactions() {
        assert_parses_to(
            "TERMINATE TRANSACTIONS 'neo4j-tx-123'",
            "TERMINATE TRANSACTIONS 'neo4j-tx-123'",
        );
    }

    #[test]
    fn parse_terminate_transactions_multiple() {
        assert_parses_to(
            "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
            "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'",
        );
    }

    #[test]
    fn parse_terminate_with_yield() {
        assert_parses_to(
            "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD *",
            "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD *",
        );
    }

    #[test]
    fn parse_terminate_with_yield_and_where() {
        assert_parses_to(
            "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD transactionId, username WHERE username = 'bob'",
            "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD transactionId, username WHERE username = 'bob'",
        );
    }

    // ── Backtracking: non-admin CREATE/DROP should not interfere ──

    #[test]
    fn create_node_falls_through() {
        // CREATE (n:Person) should NOT be parsed as admin — it should fall through
        // to regular clause parsing
        assert_parses_to(
            "CREATE (n:`Person`) RETURN n",
            "CREATE (n:`Person`) RETURN n",
        );
    }
}
