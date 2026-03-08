//! Math functions: numeric, logarithmic, and trigonometric.

use crate::types::expression::Expression;

// ── Numeric ──────────────────────────────────────────────────────────────────

/// `abs(expr)` - returns the absolute value.
pub fn abs(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("abs", vec![expr.into()])
}

/// `ceil(expr)` - rounds up to the nearest integer.
pub fn ceil(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("ceil", vec![expr.into()])
}

/// `ceiling(expr)` - alias for `ceil`.
pub fn ceiling(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("ceiling", vec![expr.into()])
}

/// `floor(expr)` - rounds down to the nearest integer.
pub fn floor(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("floor", vec![expr.into()])
}

/// `round(expr)` - rounds to the nearest integer.
pub fn round(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("round", vec![expr.into()])
}

/// `round(expr, precision)` - rounds to the given precision.
pub fn round_with_precision(
    expr: impl Into<Expression>,
    precision: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("round", vec![expr.into(), precision.into()])
}

/// `sign(expr)` - returns the signum of a number.
pub fn sign(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("sign", vec![expr.into()])
}

/// `rand()` - returns a random float between 0 (inclusive) and 1 (exclusive).
pub fn rand() -> Expression {
    Expression::function_invocation("rand", vec![])
}

/// `isNaN(expr)` - returns true if the value is NaN.
pub fn is_nan(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("isNaN", vec![expr.into()])
}

// ── Logarithmic ──────────────────────────────────────────────────────────────

/// `sqrt(expr)` - returns the square root.
pub fn sqrt(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("sqrt", vec![expr.into()])
}

/// `log(base, expr)` - returns the logarithm to the given base.
pub fn log(
    base: impl Into<Expression>,
    expr: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("log", vec![base.into(), expr.into()])
}

/// `log10(expr)` - returns the base-10 logarithm.
pub fn log10(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("log10", vec![expr.into()])
}

/// `ln(expr)` - returns the natural logarithm.
pub fn ln(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("ln", vec![expr.into()])
}

/// `exp(expr)` - returns e raised to the power of the value.
pub fn exp(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("exp", vec![expr.into()])
}

/// `e()` - returns Euler's number (e).
pub fn e_const() -> Expression {
    Expression::function_invocation("e", vec![])
}

// ── Trigonometric ────────────────────────────────────────────────────────────

/// `sin(expr)` - returns the sine.
pub fn sin(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("sin", vec![expr.into()])
}

/// `cos(expr)` - returns the cosine.
pub fn cos(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("cos", vec![expr.into()])
}

/// `tan(expr)` - returns the tangent.
pub fn tan(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("tan", vec![expr.into()])
}

/// `asin(expr)` - returns the arc sine.
pub fn asin(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("asin", vec![expr.into()])
}

/// `acos(expr)` - returns the arc cosine.
pub fn acos(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("acos", vec![expr.into()])
}

/// `atan(expr)` - returns the arc tangent.
pub fn atan(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("atan", vec![expr.into()])
}

/// `atan2(y, x)` - returns the arc tangent of y/x.
pub fn atan2(
    y: impl Into<Expression>,
    x: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("atan2", vec![y.into(), x.into()])
}

/// `cot(expr)` - returns the cotangent.
pub fn cot(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("cot", vec![expr.into()])
}

/// `cosh(expr)` - returns the hyperbolic cosine.
pub fn cosh(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("cosh", vec![expr.into()])
}

/// `sinh(expr)` - returns the hyperbolic sine.
pub fn sinh(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("sinh", vec![expr.into()])
}

/// `tanh(expr)` - returns the hyperbolic tangent.
pub fn tanh(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("tanh", vec![expr.into()])
}

/// `coth(expr)` - returns the hyperbolic cotangent.
pub fn coth(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("coth", vec![expr.into()])
}

/// `degrees(expr)` - converts radians to degrees.
pub fn degrees(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("degrees", vec![expr.into()])
}

/// `radians(expr)` - converts degrees to radians.
pub fn radians(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("radians", vec![expr.into()])
}

/// `haversin(expr)` - returns the half-versine (haversine).
pub fn haversin(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("haversin", vec![expr.into()])
}

/// `pi()` - returns the constant pi.
pub fn pi() -> Expression {
    Expression::function_invocation("pi", vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(expr: &Expression) -> String {
        let r = crate::renderer::default::DefaultRenderer::with_defaults();
        let mut buf = String::new();
        r.write_expression(&mut buf, expr);
        buf
    }

    // ── Numeric ──

    #[test]
    fn render_abs() {
        assert_eq!(render(&abs(Expression::symbolic_name("x"))), "abs(x)");
    }

    #[test]
    fn render_ceil_ceiling() {
        assert_eq!(render(&ceil(Expression::from(2.3_f64))), "ceil(2.3)");
        assert_eq!(render(&ceiling(Expression::from(2.3_f64))), "ceiling(2.3)");
    }

    #[test]
    fn render_floor() {
        assert_eq!(render(&floor(Expression::from(2.9_f64))), "floor(2.9)");
    }

    #[test]
    fn render_round() {
        assert_eq!(render(&round(Expression::from(3.56_f64))), "round(3.56)");
    }

    #[test]
    fn render_round_with_precision() {
        assert_eq!(
            render(&round_with_precision(Expression::from(3.567_f64), Expression::from(2_i32))),
            "round(3.567, 2)"
        );
    }

    #[test]
    fn render_sign() {
        assert_eq!(render(&sign(Expression::from(-5_i32))), "sign(-5)");
    }

    #[test]
    fn render_rand() {
        assert_eq!(render(&rand()), "rand()");
    }

    #[test]
    fn render_is_nan() {
        assert_eq!(render(&is_nan(Expression::symbolic_name("x"))), "isNaN(x)");
    }

    // ── Logarithmic ──

    #[test]
    fn render_sqrt() {
        assert_eq!(render(&sqrt(Expression::from(16_i32))), "sqrt(16)");
    }

    #[test]
    fn render_log() {
        assert_eq!(
            render(&log(Expression::from(2_i32), Expression::from(8_i32))),
            "log(2, 8)"
        );
    }

    #[test]
    fn render_log10() {
        assert_eq!(render(&log10(Expression::from(100_i32))), "log10(100)");
    }

    #[test]
    fn render_ln() {
        assert_eq!(render(&ln(Expression::symbolic_name("x"))), "ln(x)");
    }

    #[test]
    fn render_exp() {
        assert_eq!(render(&exp(Expression::from(1_i32))), "exp(1)");
    }

    #[test]
    fn render_e_const() {
        assert_eq!(render(&e_const()), "e()");
    }

    // ── Trigonometric ──

    #[test]
    fn render_trig_basic() {
        assert_eq!(render(&sin(Expression::symbolic_name("x"))), "sin(x)");
        assert_eq!(render(&cos(Expression::symbolic_name("x"))), "cos(x)");
        assert_eq!(render(&tan(Expression::symbolic_name("x"))), "tan(x)");
    }

    #[test]
    fn render_inverse_trig() {
        assert_eq!(render(&asin(Expression::symbolic_name("x"))), "asin(x)");
        assert_eq!(render(&acos(Expression::symbolic_name("x"))), "acos(x)");
        assert_eq!(render(&atan(Expression::symbolic_name("x"))), "atan(x)");
    }

    #[test]
    fn render_atan2() {
        assert_eq!(
            render(&atan2(Expression::symbolic_name("y"), Expression::symbolic_name("x"))),
            "atan2(y, x)"
        );
    }

    #[test]
    fn render_cot() {
        assert_eq!(render(&cot(Expression::symbolic_name("x"))), "cot(x)");
    }

    #[test]
    fn render_hyperbolic() {
        assert_eq!(render(&cosh(Expression::symbolic_name("x"))), "cosh(x)");
        assert_eq!(render(&sinh(Expression::symbolic_name("x"))), "sinh(x)");
        assert_eq!(render(&tanh(Expression::symbolic_name("x"))), "tanh(x)");
        assert_eq!(render(&coth(Expression::symbolic_name("x"))), "coth(x)");
    }

    #[test]
    fn render_degrees_radians() {
        assert_eq!(render(&degrees(Expression::symbolic_name("x"))), "degrees(x)");
        assert_eq!(render(&radians(Expression::symbolic_name("x"))), "radians(x)");
    }

    #[test]
    fn render_haversin() {
        assert_eq!(render(&haversin(Expression::symbolic_name("x"))), "haversin(x)");
    }

    #[test]
    fn render_pi() {
        assert_eq!(render(&pi()), "pi()");
    }
}
