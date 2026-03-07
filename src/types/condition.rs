//! `Condition` enum for WHERE clause predicates.
//!
//! Conditions represent boolean expressions used in WHERE clauses,
//! pattern conditions, and other filtering contexts.

use std::borrow::Cow;

use super::expression::Expression;
use super::operator::{BooleanOp, ComparisonOp, StringPredicateOp};

/// A boolean condition for use in WHERE clauses.
///
/// Supports composition via `and()`, `or()`, `xor()`, and `not()`.
/// The `NoCondition` variant is a sentinel that collapses when combined.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Comparison: `left op right`.
    Comparison {
        /// Left-hand operand.
        left: Expression,
        /// Comparison operator.
        operator: ComparisonOp,
        /// Right-hand operand.
        right: Expression,
    },

    /// Boolean composition: AND, OR, XOR.
    Compound {
        /// Boolean operator.
        operator: BooleanOp,
        /// Constituent conditions.
        conditions: Vec<Self>,
    },

    /// Negation: `NOT condition`.
    Not(Box<Self>),

    /// Null check: `expr IS NULL`.
    IsNull(Expression),

    /// Non-null check: `expr IS NOT NULL`.
    IsNotNull(Expression),

    /// String predicate: `STARTS WITH`, `ENDS WITH`, `CONTAINS`, `=~`.
    StringPredicate {
        /// Left-hand expression.
        left: Expression,
        /// The string predicate operator.
        predicate: StringPredicateOp,
        /// Right-hand expression (the pattern/substring).
        right: Expression,
    },

    /// IN check: `left IN right`.
    In {
        /// The expression to check membership of.
        left: Expression,
        /// The list expression to check against.
        right: Expression,
    },

    /// Expression used as a truthy condition.
    ExpressionCondition(Expression),

    /// Boolean truth check: `expr IS TRUE`.
    IsTrue(Expression),

    /// Boolean falsity check: `expr IS FALSE`.
    IsFalse(Expression),

    /// Regex match: `expr =~ 'pattern'`.
    RegexMatch {
        /// The expression to match.
        left: Expression,
        /// The regex pattern expression.
        pattern: Expression,
    },

    /// Type predicate: `expr IS :: TYPE`.
    TypePredicate {
        /// The expression to type-check.
        expression: Expression,
        /// The expected Cypher type name.
        type_name: Cow<'static, str>,
    },

    /// Normalization check: `expr IS [NOT] NORMALIZED`.
    IsNormalized {
        /// The expression to check.
        expression: Expression,
        /// Whether the check is negated (`IS NOT NORMALIZED`).
        negated: bool,
    },

    /// No-op sentinel that collapses when combined.
    ///
    /// `NoCondition.and(x)` returns `x`. This matches Java DSL behavior
    /// where empty conditions disappear from output.
    NoCondition,
}

impl Condition {
    /// Composes this condition with another using AND.
    ///
    /// `NoCondition` collapses: `NoCondition.and(x)` returns `x`,
    /// and `x.and(NoCondition)` returns `x`.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self::compose(BooleanOp::And, self, other)
    }

    /// Composes this condition with another using OR.
    ///
    /// `NoCondition` collapses: `NoCondition.or(x)` returns `x`,
    /// and `x.or(NoCondition)` returns `x`.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        Self::compose(BooleanOp::Or, self, other)
    }

    /// Composes this condition with another using XOR.
    ///
    /// `NoCondition` collapses: `NoCondition.xor(x)` returns `x`,
    /// and `x.xor(NoCondition)` returns `x`.
    #[must_use]
    pub fn xor(self, other: Self) -> Self {
        Self::compose(BooleanOp::Xor, self, other)
    }

    /// Negates this condition: `NOT self`.
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "DSL method mirrors Cypher NOT semantics, not Rust std::ops::Not"
    )]
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Internal helper to compose two conditions with a boolean operator.
    ///
    /// Handles `NoCondition` collapsing and flattening of same-operator
    /// compound conditions.
    fn compose(op: BooleanOp, left: Self, right: Self) -> Self {
        match (left, right) {
            // NoCondition collapses
            (Self::NoCondition, other) | (other, Self::NoCondition) => other,
            // Flatten same-operator compounds on the left
            (
                Self::Compound {
                    operator,
                    mut conditions,
                },
                other,
            ) if operator == op => {
                conditions.push(other);
                Self::Compound {
                    operator,
                    conditions,
                }
            }
            // Default: create new compound
            (left, right) => Self::Compound {
                operator: op,
                conditions: vec![left, right],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_comparison() -> Condition {
        Condition::Comparison {
            left: Expression::from(1_i32),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
    }

    fn another_comparison() -> Condition {
        Condition::Comparison {
            left: Expression::from(2_i32),
            operator: ComparisonOp::Gt,
            right: Expression::from(0_i32),
        }
    }

    // --- Composition tests ---

    #[test]
    fn and_creates_compound() {
        let result = sample_comparison().and(another_comparison());
        let Condition::Compound {
            operator,
            conditions,
        } = &result
        else {
            unreachable!("Expected Compound");
        };
        assert_eq!(*operator, BooleanOp::And);
        assert_eq!(conditions.len(), 2);
    }

    #[test]
    fn or_creates_compound() {
        let result = sample_comparison().or(another_comparison());
        let Condition::Compound {
            operator,
            conditions,
        } = &result
        else {
            unreachable!("Expected Compound");
        };
        assert_eq!(*operator, BooleanOp::Or);
        assert_eq!(conditions.len(), 2);
    }

    #[test]
    fn xor_creates_compound() {
        let result = sample_comparison().xor(another_comparison());
        let Condition::Compound {
            operator,
            conditions,
        } = &result
        else {
            unreachable!("Expected Compound");
        };
        assert_eq!(*operator, BooleanOp::Xor);
        assert_eq!(conditions.len(), 2);
    }

    #[test]
    fn not_wraps_condition() {
        let result = sample_comparison().not();
        assert!(matches!(result, Condition::Not(_)));
    }

    // --- NoCondition collapsing ---

    #[test]
    fn no_condition_and_other_returns_other() {
        let other = sample_comparison();
        let result = Condition::NoCondition.and(other.clone());
        assert_eq!(result, other);
    }

    #[test]
    fn other_and_no_condition_returns_other() {
        let other = sample_comparison();
        let result = other.clone().and(Condition::NoCondition);
        assert_eq!(result, other);
    }

    #[test]
    fn no_condition_or_other_returns_other() {
        let other = sample_comparison();
        let result = Condition::NoCondition.or(other.clone());
        assert_eq!(result, other);
    }

    #[test]
    fn no_condition_xor_other_returns_other() {
        let other = sample_comparison();
        let result = Condition::NoCondition.xor(other.clone());
        assert_eq!(result, other);
    }

    // --- Flattening ---

    #[test]
    fn and_flattens_same_operator_compound() {
        let a = sample_comparison();
        let b = another_comparison();
        let c = Condition::IsNull(Expression::from("x"));

        // a AND b AND c should flatten to a single Compound with 3 conditions
        let result = a.and(b).and(c);
        let Condition::Compound {
            operator,
            conditions,
        } = &result
        else {
            unreachable!("Expected Compound");
        };
        assert_eq!(*operator, BooleanOp::And);
        assert_eq!(conditions.len(), 3);
    }

    #[test]
    fn or_does_not_flatten_different_operator() {
        let a = sample_comparison();
        let b = another_comparison();
        let c = Condition::IsNull(Expression::from("x"));

        // (a AND b) OR c should NOT flatten — different operators
        let result = a.and(b).or(c);
        let Condition::Compound {
            operator,
            conditions,
        } = &result
        else {
            unreachable!("Expected Compound");
        };
        assert_eq!(*operator, BooleanOp::Or);
        assert_eq!(conditions.len(), 2); // [Compound(AND, [a, b]), c]
    }

    // --- Condition variant tests ---

    #[test]
    fn is_null_variant() {
        let cond = Condition::IsNull(Expression::from("x"));
        assert!(matches!(cond, Condition::IsNull(_)));
    }

    #[test]
    fn is_not_null_variant() {
        let cond = Condition::IsNotNull(Expression::from("x"));
        assert!(matches!(cond, Condition::IsNotNull(_)));
    }

    #[test]
    fn string_predicate_variant() {
        let cond = Condition::StringPredicate {
            left: Expression::from("name"),
            predicate: StringPredicateOp::StartsWith,
            right: Expression::from("A"),
        };
        assert!(matches!(cond, Condition::StringPredicate { .. }));
    }

    #[test]
    fn regex_match_variant() {
        let cond = Condition::RegexMatch {
            left: Expression::from("name"),
            pattern: Expression::from(".*test.*"),
        };
        assert!(matches!(cond, Condition::RegexMatch { .. }));
    }

    #[test]
    fn type_predicate_variant() {
        let cond = Condition::TypePredicate {
            expression: Expression::from("x"),
            type_name: Cow::Borrowed("INTEGER"),
        };
        assert!(matches!(cond, Condition::TypePredicate { .. }));
    }

    #[test]
    fn is_normalized_variant() {
        let cond = Condition::IsNormalized {
            expression: Expression::from("s"),
            negated: false,
        };
        if let Condition::IsNormalized { negated, .. } = &cond {
            assert!(!negated);
        } else {
            unreachable!("Expected IsNormalized");
        }
    }

    #[test]
    fn is_not_normalized_variant() {
        let cond = Condition::IsNormalized {
            expression: Expression::from("s"),
            negated: true,
        };
        if let Condition::IsNormalized { negated, .. } = &cond {
            assert!(negated);
        } else {
            unreachable!("Expected IsNormalized");
        }
    }

    #[test]
    fn in_variant() {
        let cond = Condition::In {
            left: Expression::from(1_i32),
            right: Expression::list_literal(vec![
                Expression::from(1_i32),
                Expression::from(2_i32),
            ]),
        };
        assert!(matches!(cond, Condition::In { .. }));
    }

    #[test]
    fn expression_condition_variant() {
        let cond = Condition::ExpressionCondition(Expression::from(true));
        assert!(matches!(cond, Condition::ExpressionCondition(_)));
    }

    #[test]
    fn is_true_and_is_false_variants() {
        let t = Condition::IsTrue(Expression::from("x"));
        let f = Condition::IsFalse(Expression::from("x"));
        assert!(matches!(t, Condition::IsTrue(_)));
        assert!(matches!(f, Condition::IsFalse(_)));
    }
}
