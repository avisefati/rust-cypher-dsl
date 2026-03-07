//! Central `Expression` enum — the core AST type.
//!
//! Represents all possible Cypher expressions: literals, parameters,
//! properties, operations, function invocations, and more.

use std::borrow::Cow;
use std::rc::Rc;

/// Central AST type representing any Cypher expression.
///
/// Uses `Rc` internally for cheap cloning. All builder methods
/// return new values (immutable pattern).
#[derive(Debug, Clone, PartialEq)]
pub struct Expression(pub(crate) Rc<ExpressionInner>);

/// The inner representation of an expression.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionInner {
    // --- Literals ---
    /// A string literal, rendered with single quotes.
    StringLiteral(Cow<'static, str>),
    /// An integer literal.
    IntegerLiteral(i64),
    /// A floating-point literal.
    FloatLiteral(f64),
    /// A boolean literal (`true` or `false`).
    BooleanLiteral(bool),
    /// The `NULL` literal.
    NullLiteral,
    /// A list literal: `[elem1, elem2, ...]`.
    ListLiteral(Vec<Expression>),
    /// A map literal: `{key1: val1, key2: val2}`.
    MapLiteral(Vec<(Cow<'static, str>, Expression)>),

    // --- References ---
    /// A symbolic name reference.
    SymbolicName(Cow<'static, str>),

    // --- Aliases ---
    /// An aliased expression: `expr AS alias`.
    Aliased {
        /// The expression being aliased.
        delegate: Expression,
        /// The alias name.
        alias: Cow<'static, str>,
    },

    // --- Raw Cypher ---
    /// Raw Cypher string (escape hatch).
    RawExpression(Cow<'static, str>),

    // --- Wildcard ---
    /// The `*` wildcard.
    Asterisk,
}

impl Expression {
    /// Creates a string literal expression.
    pub fn string_literal(value: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(ExpressionInner::StringLiteral(value.into())))
    }

    /// Creates an integer literal expression.
    pub fn integer_literal(value: i64) -> Self {
        Self(Rc::new(ExpressionInner::IntegerLiteral(value)))
    }

    /// Creates a float literal expression.
    pub fn float_literal(value: f64) -> Self {
        Self(Rc::new(ExpressionInner::FloatLiteral(value)))
    }

    /// Creates a boolean literal expression.
    pub fn boolean_literal(value: bool) -> Self {
        Self(Rc::new(ExpressionInner::BooleanLiteral(value)))
    }

    /// Creates a `NULL` literal expression.
    pub fn null_literal() -> Self {
        Self(Rc::new(ExpressionInner::NullLiteral))
    }

    /// Creates a list literal expression.
    pub fn list_literal(elements: Vec<Self>) -> Self {
        Self(Rc::new(ExpressionInner::ListLiteral(elements)))
    }

    /// Creates a map literal expression.
    pub fn map_literal(entries: Vec<(Cow<'static, str>, Self)>) -> Self {
        Self(Rc::new(ExpressionInner::MapLiteral(entries)))
    }

    /// Creates a symbolic name expression.
    pub fn symbolic_name(name: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(ExpressionInner::SymbolicName(name.into())))
    }

    /// Creates a raw Cypher expression (escape hatch).
    pub fn raw(cypher: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(ExpressionInner::RawExpression(cypher.into())))
    }

    /// Creates the wildcard (`*`) expression.
    pub fn asterisk() -> Self {
        Self(Rc::new(ExpressionInner::Asterisk))
    }

    /// Aliases this expression: `self AS alias`.
    #[must_use]
    pub fn as_alias(self, alias: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(ExpressionInner::Aliased {
            delegate: self,
            alias: alias.into(),
        }))
    }

    /// Returns a reference to the inner enum variant.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "renderer and other modules will use it in later tasks"
        )
    )]
    pub(crate) fn inner(&self) -> &ExpressionInner {
        &self.0
    }
}

// --- From conversions for ergonomic literal construction ---

impl From<i32> for Expression {
    fn from(value: i32) -> Self {
        Self::integer_literal(i64::from(value))
    }
}

impl From<i64> for Expression {
    fn from(value: i64) -> Self {
        Self::integer_literal(value)
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Self {
        Self::float_literal(value)
    }
}

impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Self::boolean_literal(value)
    }
}

impl From<&'static str> for Expression {
    fn from(value: &'static str) -> Self {
        Self::string_literal(value)
    }
}

impl From<String> for Expression {
    fn from(value: String) -> Self {
        Self::string_literal(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_i32_produces_integer_literal() {
        let expr = Expression::from(42_i32);
        assert_eq!(*expr.inner(), ExpressionInner::IntegerLiteral(42));
    }

    #[test]
    fn from_i64_produces_integer_literal() {
        let expr = Expression::from(100_i64);
        assert_eq!(*expr.inner(), ExpressionInner::IntegerLiteral(100));
    }

    #[test]
    fn from_f64_produces_float_literal() {
        let expr = Expression::from(2.72_f64);
        assert_eq!(*expr.inner(), ExpressionInner::FloatLiteral(2.72));
    }

    #[test]
    fn from_bool_produces_boolean_literal() {
        let expr_true = Expression::from(true);
        let expr_false = Expression::from(false);
        assert_eq!(*expr_true.inner(), ExpressionInner::BooleanLiteral(true));
        assert_eq!(*expr_false.inner(), ExpressionInner::BooleanLiteral(false));
    }

    #[test]
    fn from_static_str_produces_string_literal() {
        let expr = Expression::from("hello");
        assert_eq!(
            *expr.inner(),
            ExpressionInner::StringLiteral(Cow::Borrowed("hello"))
        );
    }

    #[test]
    fn from_string_produces_string_literal() {
        let expr = Expression::from(String::from("world"));
        assert_eq!(
            *expr.inner(),
            ExpressionInner::StringLiteral(Cow::Owned(String::from("world")))
        );
    }

    #[test]
    fn null_literal_creates_null() {
        let expr = Expression::null_literal();
        assert_eq!(*expr.inner(), ExpressionInner::NullLiteral);
    }

    #[test]
    fn list_literal_creates_list() {
        let list = Expression::list_literal(vec![Expression::from(1_i32), Expression::from(2_i32)]);
        let ExpressionInner::ListLiteral(elements) = list.inner() else {
            unreachable!("Expected ListLiteral");
        };
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn map_literal_creates_map() {
        let map = Expression::map_literal(vec![(Cow::Borrowed("key"), Expression::from(1_i32))]);
        let ExpressionInner::MapLiteral(entries) = map.inner() else {
            unreachable!("Expected MapLiteral");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "key");
    }

    #[test]
    fn symbolic_name_creates_name() {
        let expr = Expression::symbolic_name("n");
        assert_eq!(
            *expr.inner(),
            ExpressionInner::SymbolicName(Cow::Borrowed("n"))
        );
    }

    #[test]
    fn raw_expression_creates_raw() {
        let expr = Expression::raw("rand()");
        assert_eq!(
            *expr.inner(),
            ExpressionInner::RawExpression(Cow::Borrowed("rand()"))
        );
    }

    #[test]
    fn asterisk_creates_wildcard() {
        let expr = Expression::asterisk();
        assert_eq!(*expr.inner(), ExpressionInner::Asterisk);
    }

    #[test]
    fn as_alias_wraps_expression() {
        let expr = Expression::from(42_i32).as_alias("answer");
        let ExpressionInner::Aliased { delegate, alias } = expr.inner() else {
            unreachable!("Expected Aliased");
        };
        assert_eq!(*delegate.inner(), ExpressionInner::IntegerLiteral(42));
        assert_eq!(alias, "answer");
    }

    #[test]
    fn clone_is_cheap_rc_based() {
        let expr = Expression::from(42_i32);
        let cloned = expr.clone();
        // Both share the same Rc allocation
        assert!(Rc::ptr_eq(&expr.0, &cloned.0));
    }

    #[test]
    fn equality_works_across_clones() {
        let a = Expression::from("test");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn different_values_are_not_equal() {
        let a = Expression::from(1_i32);
        let b = Expression::from(2_i32);
        assert_ne!(a, b);
    }
}
