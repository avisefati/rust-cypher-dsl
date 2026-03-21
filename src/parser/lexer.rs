//! Lexer/tokenizer: converts Cypher source text into a token stream.
//!
//! The lexer handles:
//! - Whitespace and comment stripping (line `//` and block `/* */`)
//! - Keyword recognition (case-insensitive)
//! - Identifier tokenization (unquoted and backtick-escaped)
//! - Literal parsing (strings, integers, floats, booleans, null)
//! - Operator and punctuation tokenization (single and multi-char)

#![allow(dead_code, reason = "all items will be used by the parser grammar in subsequent tasks")]

use super::error::ParseError;
use super::tokens::{Keyword, Token};

/// A token with its byte offset in the original input.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'a> {
    /// The token value.
    pub token: Token<'a>,
    /// Byte offset where this token starts in the input.
    pub offset: usize,
}

/// Tokenizes a Cypher query string into a sequence of [`SpannedToken`]s.
///
/// # Errors
///
/// Returns [`ParseError`] if the input contains unrecognized characters
/// or malformed literals (e.g., unterminated strings).
#[allow(clippy::too_many_lines, reason = "dispatch loop is a natural flat structure")]
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken<'_>>, ParseError> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos < bytes.len() {
        // Skip whitespace
        if bytes[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }

        // Skip line comments: // ...
        if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'/' {
            pos += 2;
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        // Skip block comments: /* ... */
        if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
            let start = pos;
            pos += 2;
            let mut found_end = false;
            while pos + 1 < bytes.len() {
                if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                    pos += 2;
                    found_end = true;
                    break;
                }
                pos += 1;
            }
            if !found_end {
                return Err(ParseError::from_offset(
                    input,
                    start,
                    vec!["*/".to_owned()],
                    vec!["unterminated block comment".to_owned()],
                ));
            }
            continue;
        }

        let start = pos;

        // String literals (single or double quoted)
        if bytes[pos] == b'\'' || bytes[pos] == b'"' {
            let (s, end) = lex_string_literal(input, pos)?;
            tokens.push(SpannedToken {
                token: Token::StringLit(s),
                offset: start,
            });
            pos = end;
            continue;
        }

        // Backtick-escaped identifiers
        if bytes[pos] == b'`' {
            let (s, end) = lex_escaped_identifier(input, pos)?;
            tokens.push(SpannedToken {
                token: Token::EscapedIdentifier(s),
                offset: start,
            });
            pos = end;
            continue;
        }

        // Numbers (digits or dot followed by digit)
        if bytes[pos].is_ascii_digit() {
            let (tok, end) = lex_number(input, pos)?;
            tokens.push(SpannedToken {
                token: tok,
                offset: start,
            });
            pos = end;
            continue;
        }

        // Identifiers and keywords
        if bytes[pos].is_ascii_alphabetic() || bytes[pos] == b'_' {
            let end = lex_identifier_end(bytes, pos);
            let word = &input[pos..end];
            let token = Keyword::from_str_ci(word)
                .map_or(Token::Identifier(word), Token::Keyword);
            tokens.push(SpannedToken {
                token,
                offset: start,
            });
            pos = end;
            continue;
        }

        // Parameter marker
        if bytes[pos] == b'$' {
            tokens.push(SpannedToken {
                token: Token::Dollar,
                offset: start,
            });
            pos += 1;
            continue;
        }

        // Multi-char operators (must be checked before single-char)
        if let Some((tok, len)) = lex_multi_char_operator(bytes, pos) {
            tokens.push(SpannedToken {
                token: tok,
                offset: start,
            });
            pos += len;
            continue;
        }

        // Single-char operators and punctuation
        if let Some(tok) = lex_single_char_operator(bytes[pos]) {
            tokens.push(SpannedToken {
                token: tok,
                offset: start,
            });
            pos += 1;
            continue;
        }

        // Unrecognized character
        return Err(ParseError::from_offset(
            input,
            pos,
            vec!["valid token".to_owned()],
            vec![format!("unexpected character '{}'", input[pos..].chars().next().unwrap_or('?'))],
        ));
    }

    Ok(tokens)
}

/// Lexes a single- or double-quoted string literal starting at `pos`.
/// Returns the unescaped string content and the byte position after the closing quote.
fn lex_string_literal(input: &str, pos: usize) -> Result<(String, usize), ParseError> {
    let bytes = input.as_bytes();
    let quote = bytes[pos];
    let mut result = String::new();
    let mut i = pos + 1;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Escape sequence
            i += 1;
            match bytes[i] {
                b'\'' => result.push('\''),
                b'"' => result.push('"'),
                b'\\' => result.push('\\'),
                b'n' => result.push('\n'),
                b'r' => result.push('\r'),
                b't' => result.push('\t'),
                b'u' => {
                    // \uXXXX unicode escape
                    if i + 4 < bytes.len() {
                        let hex = &input[i + 1..i + 5];
                        if let Ok(cp) = u32::from_str_radix(hex, 16)
                            && let Some(c) = char::from_u32(cp)
                        {
                            result.push(c);
                            i += 5;
                            continue;
                        }
                    }
                    // Invalid unicode escape — just include literally
                    result.push('\\');
                    result.push('u');
                }
                other => {
                    // Unknown escape — include the backslash and the char
                    result.push('\\');
                    result.push(other as char);
                }
            }
            i += 1;
        } else if bytes[i] == quote {
            // End of string
            return Ok((result, i + 1));
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    Err(ParseError::from_offset(
        input,
        pos,
        vec![format!("closing {}", quote as char)],
        vec!["unterminated string literal".to_owned()],
    ))
}

/// Lexes a backtick-escaped identifier starting at `pos`.
/// Returns the identifier content (with doubled backticks unescaped)
/// and the byte position after the closing backtick.
fn lex_escaped_identifier(input: &str, pos: usize) -> Result<(String, usize), ParseError> {
    let bytes = input.as_bytes();
    let mut result = String::new();
    let mut i = pos + 1;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            // Check for doubled backtick (escaped backtick inside identifier)
            if i + 1 < bytes.len() && bytes[i + 1] == b'`' {
                result.push('`');
                i += 2;
            } else {
                // End of identifier
                return Ok((result, i + 1));
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    Err(ParseError::from_offset(
        input,
        pos,
        vec!["closing `".to_owned()],
        vec!["unterminated escaped identifier".to_owned()],
    ))
}

/// Lexes a numeric literal (integer or float) starting at `pos`.
/// Returns the token and byte position after the number.
fn lex_number(input: &str, pos: usize) -> Result<(Token<'_>, usize), ParseError> {
    let bytes = input.as_bytes();
    let mut end = pos;
    let mut is_float = false;

    // Integer part
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }

    // Decimal part
    if end < bytes.len() && bytes[end] == b'.' {
        // Check the next char — if it's a digit, this is a float
        // If it's '..' (range operator), don't consume the dot
        if end + 1 < bytes.len() && bytes[end + 1].is_ascii_digit() {
            is_float = true;
            end += 1; // skip the dot
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        } else if end + 1 < bytes.len() && bytes[end + 1] == b'.' {
            // This is `N..` (range), don't consume the dot
        } else if end + 1 >= bytes.len() || !bytes[end + 1].is_ascii_alphabetic() {
            // Trailing dot like `3.` — treat as float
            is_float = true;
            end += 1;
        }
    }

    // Exponent part (e/E)
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        is_float = true;
        end += 1;
        if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
            end += 1;
        }
        let exp_start = end;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == exp_start {
            return Err(ParseError::from_offset(
                input,
                pos,
                vec!["exponent digits".to_owned()],
                vec!["invalid number literal".to_owned()],
            ));
        }
    }

    let text = &input[pos..end];
    if is_float {
        let value: f64 = text.parse().map_err(|_| {
            ParseError::from_offset(
                input,
                pos,
                vec!["valid float literal".to_owned()],
                vec!["invalid float literal".to_owned()],
            )
        })?;
        Ok((Token::FloatLit(value), end))
    } else {
        let value: i64 = text.parse().map_err(|_| {
            ParseError::from_offset(
                input,
                pos,
                vec!["valid integer literal".to_owned()],
                vec!["invalid integer literal".to_owned()],
            )
        })?;
        Ok((Token::IntegerLit(value), end))
    }
}

/// Returns the end position of an identifier starting at `pos`.
fn lex_identifier_end(bytes: &[u8], pos: usize) -> usize {
    let mut end = pos;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    end
}

/// Attempts to match a multi-char operator starting at `pos`.
/// Returns the token and its byte length, or `None`.
fn lex_multi_char_operator<'a>(bytes: &[u8], pos: usize) -> Option<(Token<'a>, usize)> {
    if pos + 1 >= bytes.len() {
        return None;
    }

    let two = [bytes[pos], bytes[pos + 1]];
    match &two {
        b"<>" => Some((Token::Ne, 2)),
        b"<=" => Some((Token::Lte, 2)),
        b">=" => Some((Token::Gte, 2)),
        b"<-" => Some((Token::LeftArrow, 2)),
        b"->" => Some((Token::Arrow, 2)),
        b"=~" => Some((Token::RegexMatch, 2)),
        b"+=" => Some((Token::PlusAssign, 2)),
        b".." => Some((Token::DotDot, 2)),
        _ => None,
    }
}

/// Maps a single byte to a single-char operator/punctuation token.
const fn lex_single_char_operator(b: u8) -> Option<Token<'static>> {
    match b {
        b'=' => Some(Token::Eq),
        b'<' => Some(Token::Lt),
        b'>' => Some(Token::Gt),
        b'+' => Some(Token::Plus),
        b'-' => Some(Token::Minus),
        b'*' => Some(Token::Star),
        b'/' => Some(Token::Slash),
        b'%' => Some(Token::Percent),
        b'^' => Some(Token::Caret),
        b'.' => Some(Token::Dot),
        b':' => Some(Token::Colon),
        b'|' => Some(Token::Pipe),
        b'&' => Some(Token::Ampersand),
        b'!' => Some(Token::Bang),
        b'~' => Some(Token::Tilde),
        b',' => Some(Token::Comma),
        b'(' => Some(Token::LParen),
        b')' => Some(Token::RParen),
        b'[' => Some(Token::LBracket),
        b']' => Some(Token::RBracket),
        b'{' => Some(Token::LBrace),
        b'}' => Some(Token::RBrace),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, reason = "tests use unwrap/panic for assertions")]
mod tests {
    use super::*;

    /// Helper: tokenize and return just the token values (no spans).
    fn tokens(input: &str) -> Vec<Token<'_>> {
        tokenize(input)
            .unwrap_or_else(|e| panic!("tokenize failed: {e}"))
            .into_iter()
            .map(|st| st.token)
            .collect()
    }

    // ── Whitespace and comments ─────────────────────────────

    #[test]
    fn empty_input() {
        assert_eq!(tokens(""), Vec::<Token<'_>>::new());
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(tokens("   \t\n  "), Vec::<Token<'_>>::new());
    }

    #[test]
    fn line_comment_stripped() {
        let result = tokens("MATCH // this is a comment\n(n)");
        assert_eq!(
            result,
            vec![
                Token::Keyword(Keyword::Match),
                Token::LParen,
                Token::Identifier("n"),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn block_comment_stripped() {
        let result = tokens("MATCH /* block comment */ (n)");
        assert_eq!(
            result,
            vec![
                Token::Keyword(Keyword::Match),
                Token::LParen,
                Token::Identifier("n"),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn unterminated_block_comment_errors() {
        let err = tokenize("MATCH /* never closed").unwrap_err();
        assert!(err.context[0].contains("unterminated block comment"));
    }

    // ── Punctuation ─────────────────────────────────────────

    #[test]
    fn single_char_operators() {
        let result = tokens("( ) [ ] { } , . : | & ! ~ + - * / % ^");
        assert_eq!(
            result,
            vec![
                Token::LParen,
                Token::RParen,
                Token::LBracket,
                Token::RBracket,
                Token::LBrace,
                Token::RBrace,
                Token::Comma,
                Token::Dot,
                Token::Colon,
                Token::Pipe,
                Token::Ampersand,
                Token::Bang,
                Token::Tilde,
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent,
                Token::Caret,
            ]
        );
    }

    #[test]
    fn multi_char_operators() {
        let result = tokens("<> <= >= <- -> =~ += ..");
        assert_eq!(
            result,
            vec![
                Token::Ne,
                Token::Lte,
                Token::Gte,
                Token::LeftArrow,
                Token::Arrow,
                Token::RegexMatch,
                Token::PlusAssign,
                Token::DotDot,
            ]
        );
    }

    #[test]
    fn eq_operator() {
        let result = tokens("a = b");
        assert_eq!(
            result,
            vec![
                Token::Identifier("a"),
                Token::Eq,
                Token::Identifier("b"),
            ]
        );
    }

    #[test]
    fn dollar_sign() {
        let result = tokens("$param");
        assert_eq!(
            result,
            vec![Token::Dollar, Token::Identifier("param")]
        );
    }

    // ── Identifiers and keywords ────────────────────────────

    #[test]
    fn identifier_simple() {
        let result = tokens("myVar");
        assert_eq!(result, vec![Token::Identifier("myVar")]);
    }

    #[test]
    fn identifier_with_underscore() {
        let result = tokens("_my_var_2");
        assert_eq!(result, vec![Token::Identifier("_my_var_2")]);
    }

    #[test]
    fn keyword_case_insensitive() {
        assert_eq!(tokens("MATCH"), vec![Token::Keyword(Keyword::Match)]);
        assert_eq!(tokens("match"), vec![Token::Keyword(Keyword::Match)]);
        assert_eq!(tokens("Match"), vec![Token::Keyword(Keyword::Match)]);
    }

    #[test]
    fn keyword_vs_identifier_disambiguation() {
        // "MATCHES" is not a keyword — it's an identifier
        assert_eq!(tokens("MATCHES"), vec![Token::Identifier("MATCHES")]);
    }

    #[test]
    fn escaped_identifier() {
        let result = tokens("`my var`");
        assert_eq!(
            result,
            vec![Token::EscapedIdentifier("my var".to_owned())]
        );
    }

    #[test]
    fn escaped_identifier_with_doubled_backtick() {
        let result = tokens("`my``var`");
        assert_eq!(
            result,
            vec![Token::EscapedIdentifier("my`var".to_owned())]
        );
    }

    #[test]
    fn unterminated_escaped_identifier_errors() {
        let err = tokenize("`never closed").unwrap_err();
        assert!(err.context[0].contains("unterminated escaped identifier"));
    }

    // ── String literals ─────────────────────────────────────

    #[test]
    fn single_quoted_string() {
        let result = tokens("'hello world'");
        assert_eq!(
            result,
            vec![Token::StringLit("hello world".to_owned())]
        );
    }

    #[test]
    fn double_quoted_string() {
        let result = tokens("\"hello world\"");
        assert_eq!(
            result,
            vec![Token::StringLit("hello world".to_owned())]
        );
    }

    #[test]
    fn string_escape_sequences() {
        let result = tokens(r"'a\nb\tc\\'");
        assert_eq!(
            result,
            vec![Token::StringLit("a\nb\tc\\".to_owned())]
        );
    }

    #[test]
    fn string_escaped_quotes() {
        let result = tokens(r"'it\'s'");
        assert_eq!(
            result,
            vec![Token::StringLit("it's".to_owned())]
        );
    }

    #[test]
    fn string_unicode_escape() {
        let result = tokens(r"'\u0041'");
        assert_eq!(
            result,
            vec![Token::StringLit("A".to_owned())]
        );
    }

    #[test]
    fn empty_string() {
        let result = tokens("''");
        assert_eq!(
            result,
            vec![Token::StringLit(String::new())]
        );
    }

    #[test]
    fn unterminated_string_errors() {
        let err = tokenize("'never closed").unwrap_err();
        assert!(err.context[0].contains("unterminated string literal"));
    }

    // ── Numeric literals ────────────────────────────────────

    #[test]
    fn integer_literal() {
        assert_eq!(tokens("42"), vec![Token::IntegerLit(42)]);
        assert_eq!(tokens("0"), vec![Token::IntegerLit(0)]);
        assert_eq!(tokens("1234567890"), vec![Token::IntegerLit(1_234_567_890)]);
    }

    #[test]
    fn float_literal_with_decimal() {
        assert_eq!(tokens("1.5"), vec![Token::FloatLit(1.5)]);
        assert_eq!(tokens("0.123"), vec![Token::FloatLit(0.123)]);
    }

    #[test]
    fn float_literal_with_exponent() {
        assert_eq!(tokens("1e10"), vec![Token::FloatLit(1e10)]);
        assert_eq!(tokens("2.5E3"), vec![Token::FloatLit(2.5e3)]);
        assert_eq!(tokens("1e-2"), vec![Token::FloatLit(0.01)]);
        assert_eq!(tokens("3e+4"), vec![Token::FloatLit(3e4)]);
    }

    #[test]
    fn number_followed_by_dotdot_is_integer() {
        // In `1..5`, the `1` should be an integer, `..` is DotDot
        let result = tokens("1..5");
        assert_eq!(
            result,
            vec![
                Token::IntegerLit(1),
                Token::DotDot,
                Token::IntegerLit(5),
            ]
        );
    }

    #[test]
    fn negative_number_is_minus_then_integer() {
        // Negative numbers are handled by the parser via unary minus
        let result = tokens("-42");
        assert_eq!(
            result,
            vec![Token::Minus, Token::IntegerLit(42)]
        );
    }

    // ── Boolean and null (as keywords) ──────────────────────

    #[test]
    fn true_false_null_are_keywords() {
        assert_eq!(tokens("true"), vec![Token::Keyword(Keyword::True)]);
        assert_eq!(tokens("false"), vec![Token::Keyword(Keyword::False)]);
        assert_eq!(tokens("null"), vec![Token::Keyword(Keyword::Null)]);
        assert_eq!(tokens("TRUE"), vec![Token::Keyword(Keyword::True)]);
        assert_eq!(tokens("FALSE"), vec![Token::Keyword(Keyword::False)]);
        assert_eq!(tokens("NULL"), vec![Token::Keyword(Keyword::Null)]);
    }

    // ── Full query tokenization ─────────────────────────────

    #[test]
    fn full_query_tokenization() {
        let result = tokens("MATCH (n:Person) WHERE n.age > 21 RETURN n");
        assert_eq!(
            result,
            vec![
                Token::Keyword(Keyword::Match),
                Token::LParen,
                Token::Identifier("n"),
                Token::Colon,
                Token::Identifier("Person"),
                Token::RParen,
                Token::Keyword(Keyword::Where),
                Token::Identifier("n"),
                Token::Dot,
                Token::Identifier("age"),
                Token::Gt,
                Token::IntegerLit(21),
                Token::Keyword(Keyword::Return),
                Token::Identifier("n"),
            ]
        );
    }

    #[test]
    fn query_with_string_literal_and_parameter() {
        let result = tokens("WHERE n.name = $name AND n.city = 'NYC'");
        assert_eq!(
            result,
            vec![
                Token::Keyword(Keyword::Where),
                Token::Identifier("n"),
                Token::Dot,
                Token::Identifier("name"),
                Token::Eq,
                Token::Dollar,
                Token::Identifier("name"),
                Token::Keyword(Keyword::And),
                Token::Identifier("n"),
                Token::Dot,
                Token::Identifier("city"),
                Token::Eq,
                Token::StringLit("NYC".to_owned()),
            ]
        );
    }

    #[test]
    fn query_with_relationship() {
        let result = tokens("(a)-[:KNOWS]->(b)");
        assert_eq!(
            result,
            vec![
                Token::LParen,
                Token::Identifier("a"),
                Token::RParen,
                Token::Minus,
                Token::LBracket,
                Token::Colon,
                Token::Identifier("KNOWS"),
                Token::RBracket,
                Token::Arrow,
                Token::LParen,
                Token::Identifier("b"),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn query_with_variable_length() {
        let result = tokens("[*1..5]");
        assert_eq!(
            result,
            vec![
                Token::LBracket,
                Token::Star,
                Token::IntegerLit(1),
                Token::DotDot,
                Token::IntegerLit(5),
                Token::RBracket,
            ]
        );
    }

    #[test]
    fn spanned_tokens_have_correct_offsets() {
        let result = tokenize("MATCH (n)").unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(result[0].offset, 0); // MATCH
        assert_eq!(result[1].offset, 6); // (
        assert_eq!(result[2].offset, 7); // n
        assert_eq!(result[3].offset, 8); // )
    }

    #[test]
    fn unrecognized_character_errors() {
        let err = tokenize("MATCH #invalid").unwrap_err();
        assert!(err.context[0].contains("unexpected character"));
    }
}
