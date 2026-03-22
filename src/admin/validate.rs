//! Validation helpers for admin command inputs.
//!
//! These functions enforce that user-supplied strings are safe to embed in
//! Cypher admin commands.  Invalid inputs cause a panic with a descriptive
//! message, matching the convention used by [`Parameter::new()`] and
//! [`Expression::function_invocation()`].

/// Returns `true` if `name` is a valid Cypher identifier.
///
/// Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`
///
/// Used for: variables, property names, schema object names, usernames.
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        unreachable!("guarded by is_empty check above");
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Returns `true` if `id` is a safe transaction ID.
///
/// Rules: non-empty, no single-quote `'`, no backslash `\`, no NUL byte.
pub fn is_valid_transaction_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('\'') && !id.contains('\\') && !id.contains('\0')
}

/// Panics if `name` is not a valid identifier.
pub fn assert_valid_identifier(name: &str, context: &str) {
    assert!(
        is_valid_identifier(name),
        "invalid {context} `{name}`: must match [a-zA-Z_][a-zA-Z0-9_]*"
    );
}

/// Panics if `id` is not a valid transaction ID.
pub fn assert_valid_transaction_id(id: &str) {
    assert!(
        is_valid_transaction_id(id),
        "invalid transaction ID `{id}`: must not contain single quotes, \
         backslashes, or NUL bytes, and must not be empty"
    );
}

#[cfg(test)]
#[allow(clippy::panic, reason = "tests use assert macros and #[should_panic]")]
mod tests {
    use super::*;

    // ── is_valid_identifier ──

    #[test]
    fn identifier_accepts_simple_name() {
        assert!(is_valid_identifier("foo"));
    }

    #[test]
    fn identifier_accepts_underscore_prefix() {
        assert!(is_valid_identifier("_bar"));
    }

    #[test]
    fn identifier_accepts_alphanumeric() {
        assert!(is_valid_identifier("x123"));
    }

    #[test]
    fn identifier_accepts_single_char() {
        assert!(is_valid_identifier("a"));
    }

    #[test]
    fn identifier_rejects_empty() {
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn identifier_rejects_leading_digit() {
        assert!(!is_valid_identifier("123"));
    }

    #[test]
    fn identifier_rejects_spaces() {
        assert!(!is_valid_identifier("a b"));
    }

    #[test]
    fn identifier_rejects_parenthesis() {
        assert!(!is_valid_identifier("a)b"));
    }

    #[test]
    fn identifier_rejects_nul() {
        assert!(!is_valid_identifier("a\0b"));
    }

    #[test]
    fn identifier_rejects_injection_payload() {
        assert!(!is_valid_identifier("idx IF EXISTS"));
    }

    #[test]
    fn identifier_rejects_backtick() {
        assert!(!is_valid_identifier("a`b"));
    }

    // ── is_valid_transaction_id ──

    #[test]
    fn tx_id_accepts_normal_id() {
        assert!(is_valid_transaction_id("neo4j-tx-123"));
    }

    #[test]
    fn tx_id_accepts_simple() {
        assert!(is_valid_transaction_id("abc"));
    }

    #[test]
    fn tx_id_rejects_empty() {
        assert!(!is_valid_transaction_id(""));
    }

    #[test]
    fn tx_id_rejects_single_quote() {
        assert!(!is_valid_transaction_id("tx'"));
    }

    #[test]
    fn tx_id_rejects_backslash() {
        assert!(!is_valid_transaction_id("tx\\"));
    }

    #[test]
    fn tx_id_rejects_nul() {
        assert!(!is_valid_transaction_id("tx\0"));
    }

    #[test]
    fn tx_id_rejects_quote_breakout_payload() {
        assert!(!is_valid_transaction_id("neo4j-tx-1' YIELD * WHERE username = 'alice"));
    }

    // ── assert helpers ──

    #[test]
    #[should_panic(expected = "invalid index name")]
    fn assert_identifier_panics_on_invalid() {
        assert_valid_identifier("idx IF EXISTS", "index name");
    }

    #[test]
    #[should_panic(expected = "invalid transaction ID")]
    fn assert_tx_id_panics_on_quote() {
        assert_valid_transaction_id("tx' YIELD *");
    }
}
