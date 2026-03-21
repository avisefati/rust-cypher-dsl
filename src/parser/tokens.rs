//! Token and keyword types produced by the lexer.

#![allow(dead_code, reason = "all types will be used by the lexer in subsequent tasks")]

use std::fmt;

/// A token produced by the lexer.
///
/// The lifetime `'a` borrows from the original input string for
/// identifiers (avoiding allocation for the common case).
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    // ── Keywords ──────────────────────────────────────────────
    /// A recognized Cypher keyword (case-insensitive).
    Keyword(Keyword),

    // ── Identifiers ──────────────────────────────────────────
    /// An unquoted identifier, e.g. `myVar`.
    Identifier(&'a str),
    /// A backtick-escaped identifier, e.g. `` `my var` ``.
    /// The inner string has backtick escaping already resolved.
    EscapedIdentifier(String),

    // ── Literals ─────────────────────────────────────────────
    /// Integer literal, e.g. `42`.
    IntegerLit(i64),
    /// Floating-point literal, e.g. `3.14`.
    FloatLit(f64),
    /// String literal (single- or double-quoted, unescaped).
    StringLit(String),

    // ── Operators and punctuation ────────────────────────────
    /// `=`
    Eq,
    /// `<>`
    Ne,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    Lte,
    /// `>=`
    Gte,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^`
    Caret,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `:`
    Colon,
    /// `|`
    Pipe,
    /// `&`
    Ampersand,
    /// `!`
    Bang,
    /// `~`
    Tilde,
    /// `$`
    Dollar,
    /// `->`
    Arrow,
    /// `<-`
    LeftArrow,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,

    // ── Special ──────────────────────────────────────────────
    /// `=~`
    RegexMatch,
    /// `+=`
    PlusAssign,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword(kw) => write!(f, "{kw}"),
            Self::Identifier(id) => write!(f, "{id}"),
            Self::EscapedIdentifier(id) => write!(f, "`{id}`"),
            Self::IntegerLit(n) => write!(f, "{n}"),
            Self::FloatLit(n) => write!(f, "{n}"),
            Self::StringLit(s) => write!(f, "'{s}'"),
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "<>"),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Lte => write!(f, "<="),
            Self::Gte => write!(f, ">="),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::Caret => write!(f, "^"),
            Self::Dot => write!(f, "."),
            Self::DotDot => write!(f, ".."),
            Self::Colon => write!(f, ":"),
            Self::Pipe => write!(f, "|"),
            Self::Ampersand => write!(f, "&"),
            Self::Bang => write!(f, "!"),
            Self::Tilde => write!(f, "~"),
            Self::Dollar => write!(f, "$"),
            Self::Arrow => write!(f, "->"),
            Self::LeftArrow => write!(f, "<-"),
            Self::Comma => write!(f, ","),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::RegexMatch => write!(f, "=~"),
            Self::PlusAssign => write!(f, "+="),
        }
    }
}

/// Cypher keywords (case-insensitive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    // ── Reading clauses ──
    /// `MATCH`
    Match,
    /// `OPTIONAL`
    Optional,
    /// `WHERE`
    Where,
    /// `RETURN`
    Return,
    /// `DISTINCT`
    Distinct,
    /// `WITH`
    With,
    /// `AS`
    As,

    // ── Writing clauses ──
    /// `CREATE`
    Create,
    /// `MERGE`
    Merge,
    /// `SET`
    Set,
    /// `DELETE`
    Delete,
    /// `DETACH`
    Detach,
    /// `REMOVE`
    Remove,

    // ── Ordering ──
    /// `ORDER`
    Order,
    /// `BY`
    By,
    /// `ASC`
    Asc,
    /// `ASCENDING`
    Ascending,
    /// `DESC`
    Desc,
    /// `DESCENDING`
    Descending,
    /// `SKIP`
    Skip,
    /// `LIMIT`
    Limit,

    // ── Other clauses ──
    /// `UNWIND`
    Unwind,
    /// `FOREACH`
    Foreach,
    /// `ON`
    On,
    /// `CALL`
    Call,
    /// `YIELD`
    Yield,

    // ── Boolean operators ──
    /// `AND`
    And,
    /// `OR`
    Or,
    /// `XOR`
    Xor,
    /// `NOT`
    Not,
    /// `IN`
    In,

    // ── Predicates ──
    /// `IS`
    Is,
    /// `NULL`
    Null,
    /// `TRUE`
    True,
    /// `FALSE`
    False,
    /// `STARTS`
    Starts,
    /// `ENDS`
    Ends,
    /// `CONTAINS`
    Contains,

    // ── CASE ──
    /// `CASE`
    Case,
    /// `WHEN`
    When,
    /// `THEN`
    Then,
    /// `ELSE`
    Else,
    /// `END`
    End,

    // ── Query composition ──
    /// `EXPLAIN`
    Explain,
    /// `PROFILE`
    Profile,
    /// `UNION`
    Union,
    /// `ALL`
    All,

    // ── LOAD CSV ──
    /// `LOAD`
    Load,
    /// `CSV`
    Csv,
    /// `FROM`
    From,
    /// `HEADERS`
    Headers,
    /// `FIELDTERMINATOR`
    FieldTerminator,

    // ── Hints ──
    /// `USING`
    Using,
    /// `INDEX`
    Index,
    /// `SEEK`
    Seek,
    /// `SCAN`
    Scan,
    /// `JOIN`
    Join,

    // ── Cypher 25 ──
    /// `FINISH`
    Finish,
    /// `FILTER`
    Filter,
    /// `LET`
    Let,
    /// `NEXT`
    Next,

    // ── Path selectors ──
    /// `SHORTEST`
    Shortest,
    /// `ANY`
    Any,
    /// `GROUPS`
    Groups,

    // ── Conditional ──
    /// `IF`
    If,

    // ── Subquery keywords ──
    /// `EXISTS`
    Exists,
    /// `COUNT`
    Count,
    /// `COLLECT`
    Collect,

    // ── Transactions ──
    /// `TRANSACTIONS`
    Transactions,
    /// `OF`
    Of,
    /// `ROWS`
    Rows,

    // ── Normalization ──
    /// `NORMALIZED`
    Normalized,
    /// `NFC`
    Nfc,
    /// `NFD`
    Nfd,
    /// `NFKC`
    Nfkc,
    /// `NFKD`
    Nfkd,

    // ── Type predicates ──
    /// `TYPED`
    Typed,

    // ── USE clause ──
    /// `USE`
    Use,

    // ── Periodic commit ──
    /// `PERIODIC`
    Periodic,
    /// `COMMIT`
    Commit,

    // ── Reduce ──
    /// `REDUCE`
    Reduce,

    // ── Admin commands ──
    /// `SHOW`
    Show,
    /// `DROP`
    Drop,
    /// `CONSTRAINT`
    Constraint,
    /// `CONSTRAINTS`
    Constraints,
    /// `INDEXES`
    Indexes,
    /// `FUNCTIONS`
    Functions,
    /// `PROCEDURES`
    Procedures,
    /// `TERMINATE`
    Terminate,
    /// `TEXT`
    Text,
    /// `POINT`
    Point,
    /// `FULLTEXT`
    Fulltext,
    /// `VECTOR`
    Vector,
    /// `LOOKUP`
    Lookup,
    /// `EXECUTABLE`
    Executable,
    /// `UNIQUE`
    Unique,
    /// `KEY`
    Key,
    /// `REQUIRE`
    Require,
    /// `EACH`
    Each,
    /// `TYPE`
    Type,
    /// `OPTIONS`
    Options,
    /// `BUILT`
    Built,
    /// `DEFINED`
    Defined,
    /// `USER`
    User,
    /// `CURRENT`
    Current,
    /// `RELATIONSHIP`
    Relationship,
    /// `NODE`
    Node,
    /// `FOR`
    For,
    /// `LABELS`
    Labels,
}

impl Keyword {
    /// Attempts to match a string to a keyword (case-insensitive).
    ///
    /// Returns `None` if the string is not a recognized keyword.
    #[allow(clippy::too_many_lines, reason = "keyword match table is flat and clear")]
    pub fn from_str_ci(s: &str) -> Option<Self> {
        // Use a match on the uppercase version for case-insensitive lookup.
        // For short strings this is faster than a HashMap.
        let upper: String = s.to_ascii_uppercase();
        match upper.as_str() {
            "MATCH" => Some(Self::Match),
            "OPTIONAL" => Some(Self::Optional),
            "WHERE" => Some(Self::Where),
            "RETURN" => Some(Self::Return),
            "DISTINCT" => Some(Self::Distinct),
            "WITH" => Some(Self::With),
            "AS" => Some(Self::As),
            "CREATE" => Some(Self::Create),
            "MERGE" => Some(Self::Merge),
            "SET" => Some(Self::Set),
            "DELETE" => Some(Self::Delete),
            "DETACH" => Some(Self::Detach),
            "REMOVE" => Some(Self::Remove),
            "ORDER" => Some(Self::Order),
            "BY" => Some(Self::By),
            "ASC" => Some(Self::Asc),
            "ASCENDING" => Some(Self::Ascending),
            "DESC" => Some(Self::Desc),
            "DESCENDING" => Some(Self::Descending),
            "SKIP" => Some(Self::Skip),
            "LIMIT" => Some(Self::Limit),
            "UNWIND" => Some(Self::Unwind),
            "FOREACH" => Some(Self::Foreach),
            "ON" => Some(Self::On),
            "CALL" => Some(Self::Call),
            "YIELD" => Some(Self::Yield),
            "AND" => Some(Self::And),
            "OR" => Some(Self::Or),
            "XOR" => Some(Self::Xor),
            "NOT" => Some(Self::Not),
            "IN" => Some(Self::In),
            "IS" => Some(Self::Is),
            "NULL" => Some(Self::Null),
            "TRUE" => Some(Self::True),
            "FALSE" => Some(Self::False),
            "STARTS" => Some(Self::Starts),
            "ENDS" => Some(Self::Ends),
            "CONTAINS" => Some(Self::Contains),
            "CASE" => Some(Self::Case),
            "WHEN" => Some(Self::When),
            "THEN" => Some(Self::Then),
            "ELSE" => Some(Self::Else),
            "END" => Some(Self::End),
            "EXPLAIN" => Some(Self::Explain),
            "PROFILE" => Some(Self::Profile),
            "UNION" => Some(Self::Union),
            "ALL" => Some(Self::All),
            "LOAD" => Some(Self::Load),
            "CSV" => Some(Self::Csv),
            "FROM" => Some(Self::From),
            "HEADERS" => Some(Self::Headers),
            "FIELDTERMINATOR" => Some(Self::FieldTerminator),
            "USING" => Some(Self::Using),
            "INDEX" => Some(Self::Index),
            "SEEK" => Some(Self::Seek),
            "SCAN" => Some(Self::Scan),
            "JOIN" => Some(Self::Join),
            "FINISH" => Some(Self::Finish),
            "FILTER" => Some(Self::Filter),
            "LET" => Some(Self::Let),
            "NEXT" => Some(Self::Next),
            "SHORTEST" => Some(Self::Shortest),
            "ANY" => Some(Self::Any),
            "GROUPS" => Some(Self::Groups),
            "IF" => Some(Self::If),
            "EXISTS" => Some(Self::Exists),
            "COUNT" => Some(Self::Count),
            "COLLECT" => Some(Self::Collect),
            "TRANSACTIONS" => Some(Self::Transactions),
            "OF" => Some(Self::Of),
            "ROWS" => Some(Self::Rows),
            "NORMALIZED" => Some(Self::Normalized),
            "NFC" => Some(Self::Nfc),
            "NFD" => Some(Self::Nfd),
            "NFKC" => Some(Self::Nfkc),
            "NFKD" => Some(Self::Nfkd),
            "TYPED" => Some(Self::Typed),
            "USE" => Some(Self::Use),
            "PERIODIC" => Some(Self::Periodic),
            "COMMIT" => Some(Self::Commit),
            "REDUCE" => Some(Self::Reduce),
            "SHOW" => Some(Self::Show),
            "DROP" => Some(Self::Drop),
            "CONSTRAINT" => Some(Self::Constraint),
            "CONSTRAINTS" => Some(Self::Constraints),
            "INDEXES" => Some(Self::Indexes),
            "FUNCTIONS" => Some(Self::Functions),
            "PROCEDURES" => Some(Self::Procedures),
            "TERMINATE" => Some(Self::Terminate),
            "TEXT" => Some(Self::Text),
            "POINT" => Some(Self::Point),
            "FULLTEXT" => Some(Self::Fulltext),
            "VECTOR" => Some(Self::Vector),
            "LOOKUP" => Some(Self::Lookup),
            "EXECUTABLE" => Some(Self::Executable),
            "UNIQUE" => Some(Self::Unique),
            "KEY" => Some(Self::Key),
            "REQUIRE" => Some(Self::Require),
            "EACH" => Some(Self::Each),
            "TYPE" => Some(Self::Type),
            "OPTIONS" => Some(Self::Options),
            "BUILT" => Some(Self::Built),
            "DEFINED" => Some(Self::Defined),
            "USER" => Some(Self::User),
            "CURRENT" => Some(Self::Current),
            "RELATIONSHIP" => Some(Self::Relationship),
            "NODE" => Some(Self::Node),
            "FOR" => Some(Self::For),
            "LABELS" => Some(Self::Labels),
            _ => None,
        }
    }
}

impl fmt::Display for Keyword {
    #[allow(clippy::too_many_lines, reason = "keyword display table is flat and clear")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Match => "MATCH",
            Self::Optional => "OPTIONAL",
            Self::Where => "WHERE",
            Self::Return => "RETURN",
            Self::Distinct => "DISTINCT",
            Self::With => "WITH",
            Self::As => "AS",
            Self::Create => "CREATE",
            Self::Merge => "MERGE",
            Self::Set => "SET",
            Self::Delete => "DELETE",
            Self::Detach => "DETACH",
            Self::Remove => "REMOVE",
            Self::Order => "ORDER",
            Self::By => "BY",
            Self::Asc => "ASC",
            Self::Ascending => "ASCENDING",
            Self::Desc => "DESC",
            Self::Descending => "DESCENDING",
            Self::Skip => "SKIP",
            Self::Limit => "LIMIT",
            Self::Unwind => "UNWIND",
            Self::Foreach => "FOREACH",
            Self::On => "ON",
            Self::Call => "CALL",
            Self::Yield => "YIELD",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::Not => "NOT",
            Self::In => "IN",
            Self::Is => "IS",
            Self::Null => "NULL",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Starts => "STARTS",
            Self::Ends => "ENDS",
            Self::Contains => "CONTAINS",
            Self::Case => "CASE",
            Self::When => "WHEN",
            Self::Then => "THEN",
            Self::Else => "ELSE",
            Self::End => "END",
            Self::Explain => "EXPLAIN",
            Self::Profile => "PROFILE",
            Self::Union => "UNION",
            Self::All => "ALL",
            Self::Load => "LOAD",
            Self::Csv => "CSV",
            Self::From => "FROM",
            Self::Headers => "HEADERS",
            Self::FieldTerminator => "FIELDTERMINATOR",
            Self::Using => "USING",
            Self::Index => "INDEX",
            Self::Seek => "SEEK",
            Self::Scan => "SCAN",
            Self::Join => "JOIN",
            Self::Finish => "FINISH",
            Self::Filter => "FILTER",
            Self::Let => "LET",
            Self::Next => "NEXT",
            Self::Shortest => "SHORTEST",
            Self::Any => "ANY",
            Self::Groups => "GROUPS",
            Self::If => "IF",
            Self::Exists => "EXISTS",
            Self::Count => "COUNT",
            Self::Collect => "COLLECT",
            Self::Transactions => "TRANSACTIONS",
            Self::Of => "OF",
            Self::Rows => "ROWS",
            Self::Normalized => "NORMALIZED",
            Self::Nfc => "NFC",
            Self::Nfd => "NFD",
            Self::Nfkc => "NFKC",
            Self::Nfkd => "NFKD",
            Self::Typed => "TYPED",
            Self::Use => "USE",
            Self::Periodic => "PERIODIC",
            Self::Commit => "COMMIT",
            Self::Reduce => "REDUCE",
            Self::Show => "SHOW",
            Self::Drop => "DROP",
            Self::Constraint => "CONSTRAINT",
            Self::Constraints => "CONSTRAINTS",
            Self::Indexes => "INDEXES",
            Self::Functions => "FUNCTIONS",
            Self::Procedures => "PROCEDURES",
            Self::Terminate => "TERMINATE",
            Self::Text => "TEXT",
            Self::Point => "POINT",
            Self::Fulltext => "FULLTEXT",
            Self::Vector => "VECTOR",
            Self::Lookup => "LOOKUP",
            Self::Executable => "EXECUTABLE",
            Self::Unique => "UNIQUE",
            Self::Key => "KEY",
            Self::Require => "REQUIRE",
            Self::Each => "EACH",
            Self::Type => "TYPE",
            Self::Options => "OPTIONS",
            Self::Built => "BUILT",
            Self::Defined => "DEFINED",
            Self::User => "USER",
            Self::Current => "CURRENT",
            Self::Relationship => "RELATIONSHIP",
            Self::Node => "NODE",
            Self::For => "FOR",
            Self::Labels => "LABELS",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_from_str_ci_uppercase() {
        assert_eq!(Keyword::from_str_ci("MATCH"), Some(Keyword::Match));
        assert_eq!(Keyword::from_str_ci("RETURN"), Some(Keyword::Return));
        assert_eq!(Keyword::from_str_ci("WHERE"), Some(Keyword::Where));
    }

    #[test]
    fn keyword_from_str_ci_lowercase() {
        assert_eq!(Keyword::from_str_ci("match"), Some(Keyword::Match));
        assert_eq!(Keyword::from_str_ci("return"), Some(Keyword::Return));
        assert_eq!(Keyword::from_str_ci("where"), Some(Keyword::Where));
    }

    #[test]
    fn keyword_from_str_ci_mixed_case() {
        assert_eq!(Keyword::from_str_ci("Match"), Some(Keyword::Match));
        assert_eq!(Keyword::from_str_ci("rEtUrN"), Some(Keyword::Return));
        assert_eq!(Keyword::from_str_ci("WheRe"), Some(Keyword::Where));
    }

    #[test]
    fn keyword_from_str_ci_not_a_keyword() {
        assert_eq!(Keyword::from_str_ci("person"), None);
        assert_eq!(Keyword::from_str_ci(""), None);
        assert_eq!(Keyword::from_str_ci("MATCHES"), None);
    }

    #[test]
    #[allow(clippy::too_many_lines, reason = "keyword test table is flat and clear")]
    fn all_keywords_recognized() {
        let keywords = [
            ("MATCH", Keyword::Match),
            ("OPTIONAL", Keyword::Optional),
            ("WHERE", Keyword::Where),
            ("RETURN", Keyword::Return),
            ("DISTINCT", Keyword::Distinct),
            ("WITH", Keyword::With),
            ("AS", Keyword::As),
            ("CREATE", Keyword::Create),
            ("MERGE", Keyword::Merge),
            ("SET", Keyword::Set),
            ("DELETE", Keyword::Delete),
            ("DETACH", Keyword::Detach),
            ("REMOVE", Keyword::Remove),
            ("ORDER", Keyword::Order),
            ("BY", Keyword::By),
            ("ASC", Keyword::Asc),
            ("ASCENDING", Keyword::Ascending),
            ("DESC", Keyword::Desc),
            ("DESCENDING", Keyword::Descending),
            ("SKIP", Keyword::Skip),
            ("LIMIT", Keyword::Limit),
            ("UNWIND", Keyword::Unwind),
            ("FOREACH", Keyword::Foreach),
            ("ON", Keyword::On),
            ("CALL", Keyword::Call),
            ("YIELD", Keyword::Yield),
            ("AND", Keyword::And),
            ("OR", Keyword::Or),
            ("XOR", Keyword::Xor),
            ("NOT", Keyword::Not),
            ("IN", Keyword::In),
            ("IS", Keyword::Is),
            ("NULL", Keyword::Null),
            ("TRUE", Keyword::True),
            ("FALSE", Keyword::False),
            ("STARTS", Keyword::Starts),
            ("ENDS", Keyword::Ends),
            ("CONTAINS", Keyword::Contains),
            ("CASE", Keyword::Case),
            ("WHEN", Keyword::When),
            ("THEN", Keyword::Then),
            ("ELSE", Keyword::Else),
            ("END", Keyword::End),
            ("EXPLAIN", Keyword::Explain),
            ("PROFILE", Keyword::Profile),
            ("UNION", Keyword::Union),
            ("ALL", Keyword::All),
            ("LOAD", Keyword::Load),
            ("CSV", Keyword::Csv),
            ("FROM", Keyword::From),
            ("HEADERS", Keyword::Headers),
            ("FIELDTERMINATOR", Keyword::FieldTerminator),
            ("USING", Keyword::Using),
            ("INDEX", Keyword::Index),
            ("SEEK", Keyword::Seek),
            ("SCAN", Keyword::Scan),
            ("JOIN", Keyword::Join),
            ("FINISH", Keyword::Finish),
            ("FILTER", Keyword::Filter),
            ("LET", Keyword::Let),
            ("NEXT", Keyword::Next),
            ("SHORTEST", Keyword::Shortest),
            ("ANY", Keyword::Any),
            ("GROUPS", Keyword::Groups),
            ("IF", Keyword::If),
            ("EXISTS", Keyword::Exists),
            ("COUNT", Keyword::Count),
            ("COLLECT", Keyword::Collect),
            ("TRANSACTIONS", Keyword::Transactions),
            ("OF", Keyword::Of),
            ("ROWS", Keyword::Rows),
            ("NORMALIZED", Keyword::Normalized),
            ("NFC", Keyword::Nfc),
            ("NFD", Keyword::Nfd),
            ("NFKC", Keyword::Nfkc),
            ("NFKD", Keyword::Nfkd),
            ("TYPED", Keyword::Typed),
            ("USE", Keyword::Use),
            ("PERIODIC", Keyword::Periodic),
            ("COMMIT", Keyword::Commit),
            ("REDUCE", Keyword::Reduce),
            ("SHOW", Keyword::Show),
            ("DROP", Keyword::Drop),
            ("CONSTRAINT", Keyword::Constraint),
            ("CONSTRAINTS", Keyword::Constraints),
            ("INDEXES", Keyword::Indexes),
            ("FUNCTIONS", Keyword::Functions),
            ("PROCEDURES", Keyword::Procedures),
            ("TERMINATE", Keyword::Terminate),
            ("TEXT", Keyword::Text),
            ("POINT", Keyword::Point),
            ("FULLTEXT", Keyword::Fulltext),
            ("VECTOR", Keyword::Vector),
            ("LOOKUP", Keyword::Lookup),
            ("EXECUTABLE", Keyword::Executable),
            ("UNIQUE", Keyword::Unique),
            ("KEY", Keyword::Key),
            ("REQUIRE", Keyword::Require),
            ("EACH", Keyword::Each),
            ("TYPE", Keyword::Type),
            ("OPTIONS", Keyword::Options),
            ("BUILT", Keyword::Built),
            ("DEFINED", Keyword::Defined),
            ("USER", Keyword::User),
            ("CURRENT", Keyword::Current),
            ("RELATIONSHIP", Keyword::Relationship),
            ("NODE", Keyword::Node),
            ("FOR", Keyword::For),
            ("LABELS", Keyword::Labels),
        ];
        for (s, expected) in keywords {
            assert_eq!(
                Keyword::from_str_ci(s),
                Some(expected),
                "Failed to recognize keyword: {s}"
            );
        }
    }

    #[test]
    fn keyword_display() {
        assert_eq!(Keyword::Match.to_string(), "MATCH");
        assert_eq!(Keyword::Return.to_string(), "RETURN");
        assert_eq!(Keyword::FieldTerminator.to_string(), "FIELDTERMINATOR");
    }

    #[test]
    fn token_display_keyword() {
        let token = Token::Keyword(Keyword::Match);
        assert_eq!(token.to_string(), "MATCH");
    }

    #[test]
    fn token_display_identifier() {
        let token = Token::Identifier("myVar");
        assert_eq!(token.to_string(), "myVar");
    }

    #[test]
    fn token_display_escaped_identifier() {
        let token = Token::EscapedIdentifier("my var".to_owned());
        assert_eq!(token.to_string(), "`my var`");
    }

    #[test]
    fn token_display_integer() {
        let token = Token::IntegerLit(42);
        assert_eq!(token.to_string(), "42");
    }

    #[test]
    fn token_display_float() {
        let token = Token::FloatLit(2.71);
        assert_eq!(token.to_string(), "2.71");
    }

    #[test]
    fn token_display_string() {
        let token = Token::StringLit("hello".to_owned());
        assert_eq!(token.to_string(), "'hello'");
    }

    #[test]
    fn token_display_operators() {
        assert_eq!(Token::Eq.to_string(), "=");
        assert_eq!(Token::Ne.to_string(), "<>");
        assert_eq!(Token::Lte.to_string(), "<=");
        assert_eq!(Token::Gte.to_string(), ">=");
        assert_eq!(Token::Arrow.to_string(), "->");
        assert_eq!(Token::LeftArrow.to_string(), "<-");
        assert_eq!(Token::RegexMatch.to_string(), "=~");
        assert_eq!(Token::PlusAssign.to_string(), "+=");
        assert_eq!(Token::DotDot.to_string(), "..");
    }

    #[test]
    fn token_clone_and_eq() {
        let token = Token::Keyword(Keyword::Match);
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }
}
