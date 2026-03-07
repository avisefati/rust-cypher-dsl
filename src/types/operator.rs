//! Operator enums: comparison, boolean, math, string predicates.

/// Comparison operators for Cypher expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// Equality: `=`
    Eq,
    /// Inequality: `<>`
    Ne,
    /// Less than: `<`
    Lt,
    /// Less than or equal: `<=`
    Lte,
    /// Greater than: `>`
    Gt,
    /// Greater than or equal: `>=`
    Gte,
}

/// Boolean composition operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
    /// Logical XOR.
    Xor,
}

/// Math operators for arithmetic expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOp {
    /// Addition: `+`
    Add,
    /// Subtraction: `-`
    Subtract,
    /// Multiplication: `*`
    Multiply,
    /// Division: `/`
    Divide,
    /// Remainder: `%`
    Remainder,
    /// Exponentiation: `^`
    Pow,
}

/// String predicate operators for WHERE conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringPredicateOp {
    /// `STARTS WITH`
    StartsWith,
    /// `ENDS WITH`
    EndsWith,
    /// `CONTAINS`
    Contains,
    /// `=~` (regex match)
    Matches,
}

/// Unified operator for use in `Expression::Operation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// A comparison operator.
    Comparison(ComparisonOp),
    /// A math operator.
    Math(MathOp),
}
