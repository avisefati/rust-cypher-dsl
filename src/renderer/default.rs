//! Default single-line renderer for Cypher AST types.
//!
//! Renders expressions, conditions, and other AST nodes to
//! standard Cypher query strings. Uses the `RenderConfig` for
//! controlling identifier escaping.

use std::fmt::Write;

use super::{EscapeMode, RenderConfig};
use crate::types::condition::Condition;
use crate::types::expression::{Expression, ExpressionInner};
use crate::types::operator::{BooleanOp, ComparisonOp, MathOp, Operator, StringPredicateOp};

/// Single-line Cypher renderer.
///
/// Converts AST types into Cypher query strings using the
/// provided [`RenderConfig`].
#[derive(Debug, Clone)]
pub struct DefaultRenderer {
    config: RenderConfig,
}

impl DefaultRenderer {
    /// Creates a new renderer with the given configuration.
    pub const fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Creates a new renderer with the default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RenderConfig::default())
    }

    /// Renders an expression to a Cypher string.
    pub fn render_expression(&self, expr: &Expression) -> String {
        let mut buf = String::new();
        self.write_expression(&mut buf, expr);
        buf
    }

    /// Renders a condition to a Cypher string.
    pub fn render_condition(&self, cond: &Condition) -> String {
        let mut buf = String::new();
        self.write_condition(&mut buf, cond);
        buf
    }

    /// Writes an expression into the buffer.
    pub(crate) fn write_expression(&self, buf: &mut String, expr: &Expression) {
        match expr.inner() {
            ExpressionInner::StringLiteral(s) => {
                buf.push('\'');
                // Escape single quotes by doubling them
                for ch in s.chars() {
                    if ch == '\'' {
                        buf.push_str("''");
                    } else if ch == '\\' {
                        buf.push_str("\\\\");
                    } else {
                        buf.push(ch);
                    }
                }
                buf.push('\'');
            }
            ExpressionInner::IntegerLiteral(n) => {
                // Writing to a String is infallible; the fmt adaptor cannot fail.
                let _ = write!(buf, "{n}");
            }
            ExpressionInner::FloatLiteral(f) => {
                // Ensure there's always a decimal point
                if f.fract() == 0.0 {
                    let _ = write!(buf, "{f:.1}");
                } else {
                    let _ = write!(buf, "{f}");
                }
            }
            ExpressionInner::BooleanLiteral(b) => {
                buf.push_str(if *b { "true" } else { "false" });
            }
            ExpressionInner::NullLiteral => {
                buf.push_str("NULL");
            }
            ExpressionInner::ListLiteral(elements) => {
                buf.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    self.write_expression(buf, elem);
                }
                buf.push(']');
            }
            ExpressionInner::MapLiteral(entries) => {
                buf.push('{');
                for (i, (key, val)) in entries.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(key);
                    buf.push_str(": ");
                    self.write_expression(buf, val);
                }
                buf.push('}');
            }
            ExpressionInner::SymbolicName(name) => {
                buf.push_str(name);
            }
            ExpressionInner::Parameter(param) => {
                buf.push('$');
                buf.push_str(param.name());
            }
            ExpressionInner::Property(prop) => {
                self.write_expression(buf, prop.container());
                for name in prop.names() {
                    buf.push('.');
                    buf.push_str(name);
                }
            }
            ExpressionInner::Aliased { delegate, alias } => {
                self.write_expression(buf, delegate);
                buf.push_str(" AS ");
                buf.push_str(alias);
            }
            ExpressionInner::Operation {
                left,
                operator,
                right,
            } => {
                buf.push('(');
                self.write_expression(buf, left);
                buf.push(' ');
                Self::write_operator(buf, *operator);
                buf.push(' ');
                self.write_expression(buf, right);
                buf.push(')');
            }
            ExpressionInner::Node(node) => {
                self.write_node(buf, node);
            }
            ExpressionInner::Relationship(rel) => {
                self.write_relationship(buf, rel);
            }
            ExpressionInner::Condition(cond) => {
                self.write_condition(buf, cond);
            }
            ExpressionInner::RawExpression(raw) => {
                buf.push_str(raw);
            }
            ExpressionInner::Asterisk => {
                buf.push('*');
            }
        }
    }

    /// Writes a condition into the buffer.
    pub(crate) fn write_condition(&self, buf: &mut String, cond: &Condition) {
        match cond {
            Condition::Comparison {
                left,
                operator,
                right,
            } => {
                self.write_expression(buf, left);
                buf.push(' ');
                Self::write_comparison_op(buf, *operator);
                buf.push(' ');
                self.write_expression(buf, right);
            }
            Condition::Compound {
                operator,
                conditions,
            } => self.write_compound_condition(buf, *operator, conditions),
            Condition::Not(inner) => self.write_not_condition(buf, inner),
            Condition::IsNull(expr) => {
                self.write_expression(buf, expr);
                buf.push_str(" IS NULL");
            }
            Condition::IsNotNull(expr) => {
                self.write_expression(buf, expr);
                buf.push_str(" IS NOT NULL");
            }
            Condition::StringPredicate {
                left,
                predicate,
                right,
            } => self.write_string_predicate(buf, left, *predicate, right),
            Condition::In { left, right } => {
                self.write_expression(buf, left);
                buf.push_str(" IN ");
                self.write_expression(buf, right);
            }
            Condition::ExpressionCondition(expr)
            | Condition::IsTrue(expr)
            | Condition::IsFalse(expr) => {
                self.write_expression(buf, expr);
                match cond {
                    Condition::IsTrue(_) => buf.push_str(" IS TRUE"),
                    Condition::IsFalse(_) => buf.push_str(" IS FALSE"),
                    _ => {}
                }
            }
            Condition::RegexMatch { left, pattern } => {
                self.write_expression(buf, left);
                buf.push_str(" =~ ");
                self.write_expression(buf, pattern);
            }
            Condition::TypePredicate {
                expression,
                type_name,
            } => {
                self.write_expression(buf, expression);
                buf.push_str(" IS :: ");
                buf.push_str(type_name);
            }
            Condition::IsNormalized {
                expression,
                negated,
            } => {
                self.write_expression(buf, expression);
                buf.push_str(if *negated {
                    " IS NOT NORMALIZED"
                } else {
                    " IS NORMALIZED"
                });
            }
            Condition::HasLabels { node, labels } => {
                self.write_expression(buf, node);
                for label in labels {
                    buf.push(':');
                    self.write_escaped_name(buf, label);
                }
            }
            Condition::NoCondition => {}
        }
    }

    /// Writes a compound (AND/OR/XOR) condition into the buffer.
    fn write_compound_condition(
        &self,
        buf: &mut String,
        operator: BooleanOp,
        conditions: &[Condition],
    ) {
        let op_str = match operator {
            BooleanOp::And => " AND ",
            BooleanOp::Or => " OR ",
            BooleanOp::Xor => " XOR ",
        };
        for (i, cond) in conditions.iter().enumerate() {
            if i > 0 {
                buf.push_str(op_str);
            }
            let needs_parens = matches!(
                cond,
                Condition::Compound { operator: inner_op, .. }
                if *inner_op != operator
            );
            if needs_parens {
                buf.push('(');
            }
            self.write_condition(buf, cond);
            if needs_parens {
                buf.push(')');
            }
        }
    }

    /// Writes a NOT condition into the buffer.
    fn write_not_condition(&self, buf: &mut String, inner: &Condition) {
        buf.push_str("NOT ");
        let needs_parens = matches!(inner, Condition::Compound { .. });
        if needs_parens {
            buf.push('(');
        }
        self.write_condition(buf, inner);
        if needs_parens {
            buf.push(')');
        }
    }

    /// Writes a string predicate condition into the buffer.
    fn write_string_predicate(
        &self,
        buf: &mut String,
        left: &Expression,
        predicate: StringPredicateOp,
        right: &Expression,
    ) {
        self.write_expression(buf, left);
        match predicate {
            StringPredicateOp::StartsWith => buf.push_str(" STARTS WITH "),
            StringPredicateOp::EndsWith => buf.push_str(" ENDS WITH "),
            StringPredicateOp::Contains => buf.push_str(" CONTAINS "),
            StringPredicateOp::Matches => buf.push_str(" =~ "),
        }
        self.write_expression(buf, right);
    }

    /// Writes an operator symbol into the buffer.
    fn write_operator(buf: &mut String, op: Operator) {
        match op {
            Operator::Comparison(cmp) => Self::write_comparison_op(buf, cmp),
            Operator::Math(math) => Self::write_math_op(buf, math),
        }
    }

    /// Writes a comparison operator symbol.
    fn write_comparison_op(buf: &mut String, op: ComparisonOp) {
        buf.push_str(match op {
            ComparisonOp::Eq => "=",
            ComparisonOp::Ne => "<>",
            ComparisonOp::Lt => "<",
            ComparisonOp::Lte => "<=",
            ComparisonOp::Gt => ">",
            ComparisonOp::Gte => ">=",
        });
    }

    /// Writes a math operator symbol.
    fn write_math_op(buf: &mut String, op: MathOp) {
        buf.push_str(match op {
            MathOp::Add => "+",
            MathOp::Subtract => "-",
            MathOp::Multiply => "*",
            MathOp::Divide => "/",
            MathOp::Remainder => "%",
            MathOp::Pow => "^",
        });
    }

    /// Writes a name, backtick-escaped according to config.
    pub(crate) fn write_escaped_name(&self, buf: &mut String, name: &str) {
        match self.config.escape_names {
            EscapeMode::Always => {
                buf.push('`');
                // Escape backticks within the name by doubling them
                for ch in name.chars() {
                    if ch == '`' {
                        buf.push_str("``");
                    } else {
                        buf.push(ch);
                    }
                }
                buf.push('`');
            }
            EscapeMode::AsNeeded => {
                if needs_escaping(name) {
                    buf.push('`');
                    for ch in name.chars() {
                        if ch == '`' {
                            buf.push_str("``");
                        } else {
                            buf.push(ch);
                        }
                    }
                    buf.push('`');
                } else {
                    buf.push_str(name);
                }
            }
        }
    }

    // Placeholder stubs for node/relationship rendering (Task 3.2)

    /// Writes a node into the buffer.
    #[expect(clippy::unused_self, reason = "stub — will use self.config in Task 3.2")]
    pub(crate) fn write_node(
        &self,
        buf: &mut String,
        _node: &crate::types::node::Node,
    ) {
        buf.push_str("()");
    }

    /// Writes a relationship into the buffer.
    #[expect(clippy::unused_self, reason = "stub — will use self.config in Task 3.2")]
    pub(crate) fn write_relationship(
        &self,
        buf: &mut String,
        _rel: &crate::types::relationship::Relationship,
    ) {
        buf.push_str("()--()");
    }
}

/// Returns `true` if the name contains characters that require backtick escaping.
fn needs_escaping(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // Must start with a letter or underscore
    // The `is_empty` guard above ensures `next()` always returns `Some`.
    let Some(first) = name.chars().next() else {
        unreachable!("guarded by is_empty check above");
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return true;
    }
    // Subsequent characters must be alphanumeric or underscore
    name.chars()
        .skip(1)
        .any(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::condition::Condition;
    use crate::types::expression::Expression;
    use crate::types::operator::ComparisonOp;
    use crate::types::parameter::Parameter;
    use crate::types::property::Property;
    use std::borrow::Cow;

    fn renderer() -> DefaultRenderer {
        DefaultRenderer::with_defaults()
    }

    // --- Expression rendering ---

    #[test]
    fn render_string_literal() {
        let expr = Expression::from("Alice");
        assert_eq!(renderer().render_expression(&expr), "'Alice'");
    }

    #[test]
    fn render_string_literal_with_single_quotes() {
        let expr = Expression::string_literal("it's");
        assert_eq!(renderer().render_expression(&expr), "'it''s'");
    }

    #[test]
    fn render_string_literal_with_backslash() {
        let expr = Expression::string_literal("a\\b");
        assert_eq!(renderer().render_expression(&expr), "'a\\\\b'");
    }

    #[test]
    fn render_integer_literal() {
        let expr = Expression::from(42_i32);
        assert_eq!(renderer().render_expression(&expr), "42");
    }

    #[test]
    fn render_negative_integer() {
        let expr = Expression::integer_literal(-7);
        assert_eq!(renderer().render_expression(&expr), "-7");
    }

    #[test]
    fn render_float_literal() {
        let expr = Expression::from(2.72_f64);
        assert_eq!(renderer().render_expression(&expr), "2.72");
    }

    #[test]
    fn render_float_whole_number() {
        let expr = Expression::from(5.0_f64);
        assert_eq!(renderer().render_expression(&expr), "5.0");
    }

    #[test]
    fn render_boolean_true() {
        let expr = Expression::from(true);
        assert_eq!(renderer().render_expression(&expr), "true");
    }

    #[test]
    fn render_boolean_false() {
        let expr = Expression::from(false);
        assert_eq!(renderer().render_expression(&expr), "false");
    }

    #[test]
    fn render_null_literal() {
        let expr = Expression::null_literal();
        assert_eq!(renderer().render_expression(&expr), "NULL");
    }

    #[test]
    fn render_list_literal() {
        let expr = Expression::list_literal(vec![
            Expression::from(1_i32),
            Expression::from(2_i32),
            Expression::from(3_i32),
        ]);
        assert_eq!(renderer().render_expression(&expr), "[1, 2, 3]");
    }

    #[test]
    fn render_empty_list() {
        let expr = Expression::list_literal(vec![]);
        assert_eq!(renderer().render_expression(&expr), "[]");
    }

    #[test]
    fn render_map_literal() {
        let expr = Expression::map_literal(vec![
            (Cow::Borrowed("name"), Expression::from("Alice")),
            (Cow::Borrowed("age"), Expression::from(30_i32)),
        ]);
        assert_eq!(
            renderer().render_expression(&expr),
            "{name: 'Alice', age: 30}"
        );
    }

    #[test]
    fn render_empty_map() {
        let expr = Expression::map_literal(vec![]);
        assert_eq!(renderer().render_expression(&expr), "{}");
    }

    #[test]
    fn render_symbolic_name() {
        let expr = Expression::symbolic_name("n");
        assert_eq!(renderer().render_expression(&expr), "n");
    }

    #[test]
    fn render_parameter() {
        let expr = Expression::from(Parameter::new("userId"));
        assert_eq!(renderer().render_expression(&expr), "$userId");
    }

    #[test]
    fn render_property_access() {
        let expr = Expression::from(Property::new(
            Expression::symbolic_name("n"),
            "name",
        ));
        assert_eq!(renderer().render_expression(&expr), "n.name");
    }

    #[test]
    fn render_nested_property_access() {
        let prop = Property::new(Expression::symbolic_name("n"), "address")
            .property("city");
        let expr = Expression::from(prop);
        assert_eq!(renderer().render_expression(&expr), "n.address.city");
    }

    #[test]
    fn render_aliased_expression() {
        let expr = Expression::from(42_i32).as_alias("answer");
        assert_eq!(renderer().render_expression(&expr), "42 AS answer");
    }

    #[test]
    fn render_comparison_operation() {
        let expr = Expression::from(5_i32).eq(3_i32);
        assert_eq!(renderer().render_expression(&expr), "(5 = 3)");
    }

    #[test]
    fn render_math_operation() {
        let expr = Expression::from(5_i32).add(3_i32);
        assert_eq!(renderer().render_expression(&expr), "(5 + 3)");
    }

    #[test]
    fn render_chained_operations() {
        let expr = Expression::from(5_i32).add(3_i32).multiply(2_i32);
        assert_eq!(renderer().render_expression(&expr), "((5 + 3) * 2)");
    }

    #[test]
    fn render_all_comparison_ops() {
        let cases = [
            (Expression::from(1_i32).eq(2_i32), "(1 = 2)"),
            (Expression::from(1_i32).ne(2_i32), "(1 <> 2)"),
            (Expression::from(1_i32).lt(2_i32), "(1 < 2)"),
            (Expression::from(1_i32).lte(2_i32), "(1 <= 2)"),
            (Expression::from(1_i32).gt(2_i32), "(1 > 2)"),
            (Expression::from(1_i32).gte(2_i32), "(1 >= 2)"),
        ];
        let renderer = renderer();
        for (expr, expected) in cases {
            assert_eq!(renderer.render_expression(&expr), expected);
        }
    }

    #[test]
    fn render_all_math_ops() {
        let cases = [
            (Expression::from(1_i32).add(2_i32), "(1 + 2)"),
            (Expression::from(1_i32).subtract(2_i32), "(1 - 2)"),
            (Expression::from(1_i32).multiply(2_i32), "(1 * 2)"),
            (Expression::from(1_i32).divide(2_i32), "(1 / 2)"),
            (Expression::from(1_i32).remainder(2_i32), "(1 % 2)"),
            (Expression::from(1_i32).pow(2_i32), "(1 ^ 2)"),
        ];
        let renderer = renderer();
        for (expr, expected) in cases {
            assert_eq!(renderer.render_expression(&expr), expected);
        }
    }

    #[test]
    fn render_raw_expression() {
        let expr = Expression::raw("rand()");
        assert_eq!(renderer().render_expression(&expr), "rand()");
    }

    #[test]
    fn render_asterisk() {
        let expr = Expression::asterisk();
        assert_eq!(renderer().render_expression(&expr), "*");
    }

    #[test]
    fn render_condition_as_expression() {
        let cond = Condition::IsNull(Expression::symbolic_name("n"));
        let expr = Expression::from(cond);
        assert_eq!(renderer().render_expression(&expr), "n IS NULL");
    }

    // --- Condition rendering ---

    #[test]
    fn render_comparison_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("n").property("age").into(),
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        assert_eq!(renderer().render_condition(&cond), "n.age > 21");
    }

    #[test]
    fn render_and_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .and(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        });
        assert_eq!(renderer().render_condition(&cond), "a = 1 AND b = 2");
    }

    #[test]
    fn render_or_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .or(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        });
        assert_eq!(renderer().render_condition(&cond), "a = 1 OR b = 2");
    }

    #[test]
    fn render_xor_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .xor(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        });
        assert_eq!(renderer().render_condition(&cond), "a = 1 XOR b = 2");
    }

    #[test]
    fn render_mixed_and_or_condition() {
        // (a = 1 AND b = 2) OR c = 3
        let and_cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .and(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        });
        let cond = and_cond.or(Condition::Comparison {
            left: Expression::symbolic_name("c"),
            operator: ComparisonOp::Eq,
            right: Expression::from(3_i32),
        });
        assert_eq!(
            renderer().render_condition(&cond),
            "(a = 1 AND b = 2) OR c = 3"
        );
    }

    #[test]
    fn render_not_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .not();
        assert_eq!(renderer().render_condition(&cond), "NOT a = 1");
    }

    #[test]
    fn render_not_compound_condition() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .and(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        })
        .not();
        assert_eq!(
            renderer().render_condition(&cond),
            "NOT (a = 1 AND b = 2)"
        );
    }

    #[test]
    fn render_is_null() {
        let cond = Expression::symbolic_name("n").is_null();
        assert_eq!(renderer().render_condition(&cond), "n IS NULL");
    }

    #[test]
    fn render_is_not_null() {
        let cond = Expression::symbolic_name("n").is_not_null();
        assert_eq!(renderer().render_condition(&cond), "n IS NOT NULL");
    }

    #[test]
    fn render_starts_with() {
        let cond = Expression::symbolic_name("name").starts_with("A");
        assert_eq!(
            renderer().render_condition(&cond),
            "name STARTS WITH 'A'"
        );
    }

    #[test]
    fn render_ends_with() {
        let cond = Expression::symbolic_name("name").ends_with("z");
        assert_eq!(renderer().render_condition(&cond), "name ENDS WITH 'z'");
    }

    #[test]
    fn render_contains() {
        let cond = Expression::symbolic_name("name").contains("test");
        assert_eq!(renderer().render_condition(&cond), "name CONTAINS 'test'");
    }

    #[test]
    fn render_matches() {
        let cond = Expression::symbolic_name("name").matches(".*foo.*");
        assert_eq!(renderer().render_condition(&cond), "name =~ '.*foo.*'");
    }

    #[test]
    fn render_regex_match() {
        let cond = Expression::symbolic_name("name").regex_match(".*test.*");
        assert_eq!(
            renderer().render_condition(&cond),
            "name =~ '.*test.*'"
        );
    }

    #[test]
    fn render_in_condition() {
        let list = Expression::list_literal(vec![
            Expression::from(1_i32),
            Expression::from(2_i32),
            Expression::from(3_i32),
        ]);
        let cond = Expression::symbolic_name("x").in_list(list);
        assert_eq!(renderer().render_condition(&cond), "x IN [1, 2, 3]");
    }

    #[test]
    fn render_expression_condition() {
        let cond = Condition::ExpressionCondition(Expression::from(true));
        assert_eq!(renderer().render_condition(&cond), "true");
    }

    #[test]
    fn render_is_true() {
        let cond = Condition::IsTrue(Expression::symbolic_name("x"));
        assert_eq!(renderer().render_condition(&cond), "x IS TRUE");
    }

    #[test]
    fn render_is_false() {
        let cond = Condition::IsFalse(Expression::symbolic_name("x"));
        assert_eq!(renderer().render_condition(&cond), "x IS FALSE");
    }

    #[test]
    fn render_type_predicate() {
        let cond = Expression::symbolic_name("x").is_type("INTEGER");
        assert_eq!(renderer().render_condition(&cond), "x IS :: INTEGER");
    }

    #[test]
    fn render_is_normalized() {
        let cond = Expression::symbolic_name("s").is_normalized();
        assert_eq!(renderer().render_condition(&cond), "s IS NORMALIZED");
    }

    #[test]
    fn render_is_not_normalized() {
        let cond = Expression::symbolic_name("s").is_not_normalized();
        assert_eq!(
            renderer().render_condition(&cond),
            "s IS NOT NORMALIZED"
        );
    }

    #[test]
    fn render_has_labels() {
        let cond = Condition::HasLabels {
            node: Expression::symbolic_name("n"),
            labels: vec![Cow::Borrowed("Person"), Cow::Borrowed("Actor")],
        };
        assert_eq!(
            renderer().render_condition(&cond),
            "n:`Person`:`Actor`"
        );
    }

    #[test]
    fn render_has_labels_as_needed() {
        let renderer = DefaultRenderer::new(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        let cond = Condition::HasLabels {
            node: Expression::symbolic_name("n"),
            labels: vec![Cow::Borrowed("Person")],
        };
        assert_eq!(renderer.render_condition(&cond), "n:Person");
    }

    #[test]
    fn render_no_condition_produces_empty() {
        let cond = Condition::NoCondition;
        assert_eq!(renderer().render_condition(&cond), "");
    }

    // --- Escaping ---

    #[test]
    fn escape_name_always_mode() {
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_escaped_name(&mut buf, "Person");
        assert_eq!(buf, "`Person`");
    }

    #[test]
    fn escape_name_as_needed_simple() {
        let renderer = DefaultRenderer::new(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        let mut buf = String::new();
        renderer.write_escaped_name(&mut buf, "Person");
        assert_eq!(buf, "Person");
    }

    #[test]
    fn escape_name_as_needed_with_spaces() {
        let renderer = DefaultRenderer::new(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        let mut buf = String::new();
        renderer.write_escaped_name(&mut buf, "My Label");
        assert_eq!(buf, "`My Label`");
    }

    #[test]
    fn escape_name_with_backtick_in_name() {
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_escaped_name(&mut buf, "has`tick");
        assert_eq!(buf, "`has``tick`");
    }

    #[test]
    fn render_three_condition_and() {
        let cond = Condition::Comparison {
            left: Expression::symbolic_name("a"),
            operator: ComparisonOp::Eq,
            right: Expression::from(1_i32),
        }
        .and(Condition::Comparison {
            left: Expression::symbolic_name("b"),
            operator: ComparisonOp::Eq,
            right: Expression::from(2_i32),
        })
        .and(Condition::Comparison {
            left: Expression::symbolic_name("c"),
            operator: ComparisonOp::Eq,
            right: Expression::from(3_i32),
        });
        assert_eq!(
            renderer().render_condition(&cond),
            "a = 1 AND b = 2 AND c = 3"
        );
    }
}
