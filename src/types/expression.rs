//! Central `Expression` enum — the core AST type.
//!
//! Represents all possible Cypher expressions: literals, parameters,
//! properties, operations, function invocations, and more.

use std::borrow::Cow;
use std::rc::Rc;

use super::condition::Condition;
use super::node::Node;
use super::operator::{ComparisonOp, MathOp, Operator, StringPredicateOp};
use super::parameter::Parameter;
use super::property::Property;
use super::relationship::Relationship;

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
    /// A parameter reference: `$name`.
    Parameter(Parameter),
    /// A property access: `container.name`.
    Property(Property),

    // --- Aliases ---
    /// An aliased expression: `expr AS alias`.
    Aliased {
        /// The expression being aliased.
        delegate: Expression,
        /// The alias name.
        alias: Cow<'static, str>,
    },

    // --- Operations ---
    /// An infix operation: `left operator right`.
    Operation {
        /// Left-hand operand.
        left: Expression,
        /// The operator.
        operator: Operator,
        /// Right-hand operand.
        right: Expression,
    },

    // --- Graph elements ---
    /// A node reference.
    Node(Node),
    /// A relationship reference.
    Relationship(Relationship),

    // --- Conditions (also expressions in Cypher) ---
    /// A condition used as an expression.
    Condition(Condition),

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

    /// Creates a node expression.
    pub fn node(node: Node) -> Self {
        Self(Rc::new(ExpressionInner::Node(node)))
    }

    /// Creates a relationship expression.
    pub fn relationship(rel: Relationship) -> Self {
        Self(Rc::new(ExpressionInner::Relationship(rel)))
    }

    /// Aliases this expression: `self AS alias`.
    #[must_use]
    pub fn as_alias(self, alias: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(ExpressionInner::Aliased {
            delegate: self,
            alias: alias.into(),
        }))
    }

    // --- Comparison methods ---

    /// Equality: `self = other`.
    #[must_use]
    pub fn eq(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Eq), other.into())
    }

    /// Inequality: `self <> other`.
    #[must_use]
    pub fn ne(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Ne), other.into())
    }

    /// Less than: `self < other`.
    #[must_use]
    pub fn lt(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Lt), other.into())
    }

    /// Less than or equal: `self <= other`.
    #[must_use]
    pub fn lte(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Lte), other.into())
    }

    /// Greater than: `self > other`.
    #[must_use]
    pub fn gt(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Gt), other.into())
    }

    /// Greater than or equal: `self >= other`.
    #[must_use]
    pub fn gte(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Comparison(ComparisonOp::Gte), other.into())
    }

    // --- Arithmetic methods ---

    /// Addition: `self + other`.
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "DSL method mirrors Cypher semantics, not Rust std::ops::Add"
    )]
    pub fn add(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Add), other.into())
    }

    /// Subtraction: `self - other`.
    #[must_use]
    pub fn subtract(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Subtract), other.into())
    }

    /// Multiplication: `self * other`.
    #[must_use]
    pub fn multiply(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Multiply), other.into())
    }

    /// Division: `self / other`.
    #[must_use]
    pub fn divide(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Divide), other.into())
    }

    /// Remainder: `self % other`.
    #[must_use]
    pub fn remainder(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Remainder), other.into())
    }

    /// Exponentiation: `self ^ other`.
    #[must_use]
    pub fn pow(self, other: impl Into<Self>) -> Self {
        self.operation(Operator::Math(MathOp::Pow), other.into())
    }

    // --- Property access ---

    /// Accesses a property on this expression: `self.name`.
    #[must_use]
    pub fn property(self, name: impl Into<Cow<'static, str>>) -> Property {
        Property::new(self, name)
    }

    // --- Condition-producing methods ---

    /// Null check: `self IS NULL`.
    #[must_use]
    pub const fn is_null(self) -> Condition {
        Condition::IsNull(self)
    }

    /// Non-null check: `self IS NOT NULL`.
    #[must_use]
    pub const fn is_not_null(self) -> Condition {
        Condition::IsNotNull(self)
    }

    /// String predicate: `self STARTS WITH other`.
    #[must_use]
    pub fn starts_with(self, other: impl Into<Self>) -> Condition {
        Condition::StringPredicate {
            left: self,
            predicate: StringPredicateOp::StartsWith,
            right: other.into(),
        }
    }

    /// String predicate: `self ENDS WITH other`.
    #[must_use]
    pub fn ends_with(self, other: impl Into<Self>) -> Condition {
        Condition::StringPredicate {
            left: self,
            predicate: StringPredicateOp::EndsWith,
            right: other.into(),
        }
    }

    /// String predicate: `self CONTAINS other`.
    #[must_use]
    pub fn contains(self, other: impl Into<Self>) -> Condition {
        Condition::StringPredicate {
            left: self,
            predicate: StringPredicateOp::Contains,
            right: other.into(),
        }
    }

    /// String predicate: `self =~ pattern` (regex match).
    #[must_use]
    pub fn matches(self, pattern: impl Into<Self>) -> Condition {
        Condition::StringPredicate {
            left: self,
            predicate: StringPredicateOp::Matches,
            right: pattern.into(),
        }
    }

    /// Regex match: `self =~ pattern`.
    #[must_use]
    pub fn regex_match(self, pattern: impl Into<Self>) -> Condition {
        Condition::RegexMatch {
            left: self,
            pattern: pattern.into(),
        }
    }

    /// IN check: `self IN list`.
    #[must_use]
    pub fn in_list(self, list: impl Into<Self>) -> Condition {
        Condition::In {
            left: self,
            right: list.into(),
        }
    }

    /// Type predicate: `self IS :: type_name`.
    #[must_use]
    pub fn is_type(self, type_name: impl Into<Cow<'static, str>>) -> Condition {
        Condition::TypePredicate {
            expression: self,
            type_name: type_name.into(),
        }
    }

    /// Normalization check: `self IS NORMALIZED`.
    #[must_use]
    pub const fn is_normalized(self) -> Condition {
        Condition::IsNormalized {
            expression: self,
            negated: false,
        }
    }

    /// Negated normalization check: `self IS NOT NORMALIZED`.
    #[must_use]
    pub const fn is_not_normalized(self) -> Condition {
        Condition::IsNormalized {
            expression: self,
            negated: true,
        }
    }

    /// Wraps this expression as a truthy condition.
    #[must_use]
    pub const fn as_condition(self) -> Condition {
        Condition::ExpressionCondition(self)
    }

    // --- Sort direction ---

    /// Marks this expression for ascending sort order.
    #[must_use]
    pub const fn ascending(self) -> SortExpression {
        SortExpression {
            expression: self,
            direction: SortDirection::Ascending,
        }
    }

    /// Marks this expression for descending sort order.
    #[must_use]
    pub const fn descending(self) -> SortExpression {
        SortExpression {
            expression: self,
            direction: SortDirection::Descending,
        }
    }

    // --- Internal helpers ---

    /// Creates an operation expression from two operands.
    fn operation(self, operator: Operator, right: Self) -> Self {
        Self(Rc::new(ExpressionInner::Operation {
            left: self,
            operator,
            right,
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

/// Sort direction for ORDER BY clauses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (`ASC`).
    Ascending,
    /// Descending order (`DESC`).
    Descending,
}

/// An expression with a sort direction, for use in ORDER BY.
#[derive(Debug, Clone, PartialEq)]
pub struct SortExpression {
    /// The expression to sort by.
    pub(crate) expression: Expression,
    /// The sort direction.
    pub(crate) direction: SortDirection,
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

impl From<Condition> for Expression {
    fn from(value: Condition) -> Self {
        Self(Rc::new(ExpressionInner::Condition(value)))
    }
}

impl From<Parameter> for Expression {
    fn from(value: Parameter) -> Self {
        Self(Rc::new(ExpressionInner::Parameter(value)))
    }
}

impl From<Property> for Expression {
    fn from(value: Property) -> Self {
        Self(Rc::new(ExpressionInner::Property(value)))
    }
}

impl From<Node> for Expression {
    fn from(value: Node) -> Self {
        Self::node(value)
    }
}

impl From<Relationship> for Expression {
    fn from(value: Relationship) -> Self {
        Self::relationship(value)
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

    // --- Comparison method tests ---

    #[test]
    fn eq_produces_comparison_operation() {
        let expr = Expression::from(5_i32).eq(3_i32);
        let ExpressionInner::Operation {
            left,
            operator,
            right,
        } = expr.inner()
        else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*left.inner(), ExpressionInner::IntegerLiteral(5));
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Eq));
        assert_eq!(*right.inner(), ExpressionInner::IntegerLiteral(3));
    }

    #[test]
    fn ne_produces_comparison_operation() {
        let expr = Expression::from("a").ne("b");
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Ne));
    }

    #[test]
    fn lt_produces_comparison_operation() {
        let expr = Expression::from(1_i32).lt(2_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Lt));
    }

    #[test]
    fn lte_produces_comparison_operation() {
        let expr = Expression::from(1_i32).lte(2_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Lte));
    }

    #[test]
    fn gt_produces_comparison_operation() {
        let expr = Expression::from(1_i32).gt(2_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Gt));
    }

    #[test]
    fn gte_produces_comparison_operation() {
        let expr = Expression::from(1_i32).gte(2_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Comparison(ComparisonOp::Gte));
    }

    // --- Arithmetic method tests ---

    #[test]
    fn add_produces_math_operation() {
        let expr = Expression::from(5_i32).add(3_i32);
        let ExpressionInner::Operation {
            left,
            operator,
            right,
        } = expr.inner()
        else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*left.inner(), ExpressionInner::IntegerLiteral(5));
        assert_eq!(*operator, Operator::Math(MathOp::Add));
        assert_eq!(*right.inner(), ExpressionInner::IntegerLiteral(3));
    }

    #[test]
    fn subtract_produces_math_operation() {
        let expr = Expression::from(10_i32).subtract(4_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Subtract));
    }

    #[test]
    fn multiply_produces_math_operation() {
        let expr = Expression::from(3_i32).multiply(7_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Multiply));
    }

    #[test]
    fn divide_produces_math_operation() {
        let expr = Expression::from(10_i32).divide(2_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Divide));
    }

    #[test]
    fn remainder_produces_math_operation() {
        let expr = Expression::from(10_i32).remainder(3_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Remainder));
    }

    #[test]
    fn pow_produces_math_operation() {
        let expr = Expression::from(2_i32).pow(8_i32);
        let ExpressionInner::Operation { operator, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Pow));
    }

    // --- Sort direction tests ---

    #[test]
    fn ascending_produces_sort_expression() {
        let sort = Expression::from(1_i32).ascending();
        assert_eq!(sort.direction, SortDirection::Ascending);
        assert_eq!(*sort.expression.inner(), ExpressionInner::IntegerLiteral(1));
    }

    #[test]
    fn descending_produces_sort_expression() {
        let sort = Expression::from("name").descending();
        assert_eq!(sort.direction, SortDirection::Descending);
    }

    // --- Chained operations ---

    #[test]
    fn operations_can_be_chained() {
        // Left-to-right chaining: 5.add(3).multiply(2) builds the AST
        // Operation(Operation(5, Add, 3), Multiply, 2)
        let expr = Expression::from(5_i32).add(3_i32).multiply(2_i32);
        let ExpressionInner::Operation {
            operator, right, ..
        } = expr.inner()
        else {
            unreachable!("Expected Operation");
        };
        assert_eq!(*operator, Operator::Math(MathOp::Multiply));
        assert_eq!(*right.inner(), ExpressionInner::IntegerLiteral(2));
    }

    #[test]
    fn comparison_with_implicit_conversion() {
        // Expression::from("title").eq("Neo") — rhs is &str converted via Into
        let expr = Expression::from("title").eq("Neo");
        let ExpressionInner::Operation { right, .. } = expr.inner() else {
            unreachable!("Expected Operation");
        };
        assert_eq!(
            *right.inner(),
            ExpressionInner::StringLiteral(Cow::Borrowed("Neo"))
        );
    }

    // --- Condition-producing method tests ---

    #[test]
    fn is_null_produces_condition() {
        let cond = Expression::from("x").is_null();
        assert!(matches!(cond, Condition::IsNull(_)));
    }

    #[test]
    fn is_not_null_produces_condition() {
        let cond = Expression::from("x").is_not_null();
        assert!(matches!(cond, Condition::IsNotNull(_)));
    }

    #[test]
    fn starts_with_produces_string_predicate() {
        let cond = Expression::from("name").starts_with("A");
        let Condition::StringPredicate { predicate, .. } = &cond else {
            unreachable!("Expected StringPredicate");
        };
        assert_eq!(*predicate, StringPredicateOp::StartsWith);
    }

    #[test]
    fn ends_with_produces_string_predicate() {
        let cond = Expression::from("name").ends_with("z");
        let Condition::StringPredicate { predicate, .. } = &cond else {
            unreachable!("Expected StringPredicate");
        };
        assert_eq!(*predicate, StringPredicateOp::EndsWith);
    }

    #[test]
    fn contains_produces_string_predicate() {
        let cond = Expression::from("name").contains("test");
        let Condition::StringPredicate { predicate, .. } = &cond else {
            unreachable!("Expected StringPredicate");
        };
        assert_eq!(*predicate, StringPredicateOp::Contains);
    }

    #[test]
    fn matches_produces_string_predicate() {
        let cond = Expression::from("name").matches(".*foo.*");
        let Condition::StringPredicate { predicate, .. } = &cond else {
            unreachable!("Expected StringPredicate");
        };
        assert_eq!(*predicate, StringPredicateOp::Matches);
    }

    #[test]
    fn regex_match_produces_regex_condition() {
        let cond = Expression::from("name").regex_match(".*test.*");
        assert!(matches!(cond, Condition::RegexMatch { .. }));
    }

    #[test]
    fn in_list_produces_in_condition() {
        let list = Expression::list_literal(vec![Expression::from(1_i32), Expression::from(2_i32)]);
        let cond = Expression::from(1_i32).in_list(list);
        assert!(matches!(cond, Condition::In { .. }));
    }

    #[test]
    fn is_type_produces_type_predicate() {
        let cond = Expression::from("x").is_type("INTEGER");
        let Condition::TypePredicate { type_name, .. } = &cond else {
            unreachable!("Expected TypePredicate");
        };
        assert_eq!(type_name, "INTEGER");
    }

    #[test]
    fn is_normalized_produces_condition() {
        let cond = Expression::from("s").is_normalized();
        let Condition::IsNormalized { negated, .. } = &cond else {
            unreachable!("Expected IsNormalized");
        };
        assert!(!negated);
    }

    #[test]
    fn is_not_normalized_produces_condition() {
        let cond = Expression::from("s").is_not_normalized();
        let Condition::IsNormalized { negated, .. } = &cond else {
            unreachable!("Expected IsNormalized");
        };
        assert!(negated);
    }

    #[test]
    fn as_condition_wraps_expression() {
        let cond = Expression::from(true).as_condition();
        assert!(matches!(cond, Condition::ExpressionCondition(_)));
    }

    #[test]
    fn from_condition_produces_condition_expression() {
        let cond = Condition::IsNull(Expression::from("x"));
        let expr = Expression::from(cond.clone());
        assert_eq!(*expr.inner(), ExpressionInner::Condition(cond));
    }

    // --- Parameter and Property integration ---

    #[test]
    fn from_parameter_produces_parameter_expression() {
        let param = Parameter::new("userId");
        let expr = Expression::from(param.clone());
        assert_eq!(*expr.inner(), ExpressionInner::Parameter(param));
    }

    #[test]
    fn from_property_produces_property_expression() {
        let prop = Property::new(Expression::symbolic_name("n"), "name");
        let expr = Expression::from(prop.clone());
        assert_eq!(*expr.inner(), ExpressionInner::Property(prop));
    }

    #[test]
    fn property_method_creates_property() {
        let prop = Expression::symbolic_name("n").property("title");
        assert_eq!(prop.names().len(), 1);
        assert_eq!(prop.names()[0], "title");
    }

    #[test]
    fn property_can_be_converted_to_expression() {
        let prop = Expression::symbolic_name("n").property("name");
        let expr = Expression::from(prop);
        assert!(matches!(expr.inner(), ExpressionInner::Property(_)));
    }

    #[test]
    fn parameter_with_value_converts_to_expression() {
        let param = Parameter::with_value("limit", 10_i32);
        let expr = Expression::from(param);
        let ExpressionInner::Parameter(p) = expr.inner() else {
            unreachable!("Expected Parameter");
        };
        assert_eq!(p.name(), "limit");
        assert!(p.value().is_some());
    }
}
