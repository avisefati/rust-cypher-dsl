//! Default single-line renderer for Cypher AST types.
//!
//! Renders expressions, conditions, and other AST nodes to
//! standard Cypher query strings. Uses the `RenderConfig` for
//! controlling identifier escaping.

use std::fmt::Write;

use super::{EscapeMode, RenderConfig};
use crate::types::condition::Condition;
use crate::types::expression::{Expression, ExpressionInner};
use crate::types::node::{LabelExpression, Node};
use crate::types::operator::{BooleanOp, ComparisonOp, MathOp, Operator, StringPredicateOp};
use crate::types::pattern::{NamedPath, Pattern, PatternElement};
use crate::types::relationship::{
    Direction, Relationship, RelationshipChain, RelationshipDetail, RelationshipLength,
};

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
            } => self.write_compound_condition(buf, *operator, conditions.as_slice()),
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

    // --- Node / Relationship / Chain / Pattern rendering ---

    /// Renders a pattern to a Cypher string.
    pub fn render_pattern(&self, pattern: &Pattern) -> String {
        let mut buf = String::new();
        self.write_pattern(&mut buf, pattern);
        buf
    }

    /// Writes a pattern into the buffer (comma-separated elements).
    pub(crate) fn write_pattern(&self, buf: &mut String, pattern: &Pattern) {
        for (i, elem) in pattern.elements().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.write_pattern_element(buf, elem);
        }
    }

    /// Writes a single pattern element into the buffer.
    fn write_pattern_element(&self, buf: &mut String, elem: &PatternElement) {
        match elem {
            PatternElement::Node(node) => self.write_node(buf, node),
            PatternElement::Relationship(rel) => self.write_relationship(buf, rel),
            PatternElement::Chain(chain) => self.write_chain(buf, chain),
            PatternElement::NamedPath(named) => self.write_named_path(buf, named),
        }
    }

    /// Writes a named path: `p = <pattern>`.
    fn write_named_path(&self, buf: &mut String, named: &NamedPath) {
        buf.push_str(&named.name);
        buf.push_str(" = ");
        self.write_pattern_element(buf, &named.pattern);
    }

    /// Writes a node into the buffer: `(name:Label {props})`.
    pub(crate) fn write_node(&self, buf: &mut String, node: &Node) {
        buf.push('(');
        if let Some(name) = node.symbolic_name() {
            buf.push_str(name);
        }
        // Standard labels
        for label in node.labels() {
            buf.push(':');
            self.write_escaped_name(buf, label.value());
        }
        // Label expressions (AND, OR, NOT, wildcard)
        if let Some(label_expr) = node.label_expression() {
            buf.push(':');
            self.write_label_expression(buf, label_expr);
        }
        // Inline properties
        if let Some(props) = node.properties() {
            buf.push(' ');
            self.write_expression(buf, props);
        }
        buf.push(')');
    }

    /// Writes a label expression (recursive).
    fn write_label_expression(&self, buf: &mut String, expr: &LabelExpression) {
        match expr {
            LabelExpression::Label(name) => {
                self.write_escaped_name(buf, name);
            }
            LabelExpression::And(children) => {
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        buf.push('&');
                    }
                    self.write_label_expression(buf, child);
                }
            }
            LabelExpression::Or(children) => {
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        buf.push('|');
                    }
                    self.write_label_expression(buf, child);
                }
            }
            LabelExpression::Not(inner) => {
                buf.push('!');
                self.write_label_expression(buf, inner);
            }
            LabelExpression::Wildcard => {
                buf.push('%');
            }
        }
    }

    /// Writes a relationship into the buffer: `(left)-[details]->(right)`.
    pub(crate) fn write_relationship(&self, buf: &mut String, rel: &Relationship) {
        self.write_node(buf, rel.left());
        self.write_relationship_arrow(buf, rel.direction(), rel.details());
        self.write_node(buf, rel.right());
    }

    /// Writes the arrow portion of a relationship: `-[details]->`.
    fn write_relationship_arrow(
        &self,
        buf: &mut String,
        direction: Direction,
        details: &RelationshipDetail,
    ) {
        let has_content = details.symbolic_name().is_some()
            || !details.types().is_empty()
            || details.length().is_some()
            || details.properties().is_some();

        if has_content {
            // Left side of arrow
            match direction {
                Direction::Incoming => buf.push_str("<-"),
                Direction::Outgoing | Direction::Undirected => buf.push('-'),
            }
            buf.push('[');
            self.write_relationship_detail_body(buf, details);
            buf.push(']');
            // Right side of arrow
            match direction {
                Direction::Outgoing => buf.push_str("->"),
                Direction::Incoming | Direction::Undirected => buf.push('-'),
            }
        } else {
            // No bracket content: render as simple arrow
            match direction {
                Direction::Outgoing => buf.push_str("-->"),
                Direction::Incoming => buf.push_str("<--"),
                Direction::Undirected => buf.push_str("--"),
            }
        }
    }

    /// Writes the inner body of relationship brackets.
    fn write_relationship_detail_body(
        &self,
        buf: &mut String,
        details: &RelationshipDetail,
    ) {
        if let Some(name) = details.symbolic_name() {
            buf.push_str(name);
        }
        // Types, joined by |
        for (i, type_name) in details.types().iter().enumerate() {
            if i == 0 {
                buf.push(':');
            } else {
                buf.push('|');
            }
            self.write_escaped_name(buf, type_name);
        }
        // Variable length
        if let Some(length) = details.length() {
            Self::write_relationship_length(buf, length);
        }
        // Properties
        if let Some(props) = details.properties() {
            buf.push(' ');
            self.write_expression(buf, props);
        }
    }

    /// Writes a variable-length specification: `*`, `*3`, `*1..3`, etc.
    fn write_relationship_length(buf: &mut String, length: &RelationshipLength) {
        buf.push_str(" *");
        match length {
            RelationshipLength::Unbounded => {}
            RelationshipLength::Exact(n) => {
                let _ = write!(buf, "{n}");
            }
            RelationshipLength::Range { min, max } => {
                if let Some(m) = min {
                    let _ = write!(buf, "{m}");
                }
                buf.push_str("..");
                if let Some(m) = max {
                    let _ = write!(buf, "{m}");
                }
            }
        }
    }

    /// Writes a multi-hop chain: `(a)-[:R1]->(b)-[:R2]->(c)`.
    pub(crate) fn write_chain(&self, buf: &mut String, chain: &RelationshipChain) {
        self.write_node(buf, chain.start());
        for link in chain.links() {
            self.write_relationship_arrow(buf, link.direction(), link.details());
            self.write_node(buf, link.target());
        }
    }

    // --- Statement / Clause rendering ---

    /// Renders a complete statement to a Cypher string.
    pub fn render_statement(&self, stmt: &crate::statement::Statement) -> String {
        let mut buf = String::new();
        self.write_statement(&mut buf, stmt);
        buf
    }

    /// Writes a statement into the buffer.
    fn write_statement(
        &self,
        buf: &mut String,
        stmt: &crate::statement::Statement,
    ) {
        match stmt {
            crate::statement::Statement::SinglePart(query) => {
                self.write_single_part_query(buf, query);
            }
        }
    }

    /// Writes a single-part query (sequence of clauses).
    fn write_single_part_query(
        &self,
        buf: &mut String,
        query: &crate::statement::SinglePartQuery,
    ) {
        for (i, clause) in query.clauses().iter().enumerate() {
            if i > 0 {
                buf.push(' ');
            }
            self.write_clause(buf, clause);
        }
    }

    /// Writes a single clause into the buffer.
    fn write_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::Clause,
    ) {
        match clause {
            crate::clauses::Clause::Match(m) => self.write_match_clause(buf, m),
            crate::clauses::Clause::Where(w) => self.write_where_clause(buf, w),
            crate::clauses::Clause::Return(r) => self.write_return_clause(buf, r),
        }
    }

    /// Writes a MATCH or OPTIONAL MATCH clause.
    fn write_match_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::MatchClause,
    ) {
        if clause.is_optional() {
            buf.push_str("OPTIONAL MATCH ");
        } else {
            buf.push_str("MATCH ");
        }
        self.write_pattern(buf, clause.pattern());
    }

    /// Writes a WHERE clause.
    fn write_where_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::WhereClause,
    ) {
        buf.push_str("WHERE ");
        self.write_condition(buf, clause.condition());
    }

    /// Writes a RETURN clause.
    fn write_return_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::ReturnClause,
    ) {
        if clause.is_distinct() {
            buf.push_str("RETURN DISTINCT ");
        } else {
            buf.push_str("RETURN ");
        }
        for (i, expr) in clause.expressions().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.write_expression(buf, expr);
        }
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

    // --- Node rendering ---

    #[test]
    fn render_named_node_with_label() {
        // (m:`Movie`)
        let n = crate::types::node::node("Movie").named("m");
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(m:`Movie`)");
    }

    #[test]
    fn render_anonymous_node_with_label() {
        // (:`Person`)
        let n = crate::types::node::node("Person");
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(:`Person`)");
    }

    #[test]
    fn render_bare_named_node() {
        // (n)
        let n = crate::types::node::any_node_named("n");
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n)");
    }

    #[test]
    fn render_anonymous_bare_node() {
        // ()
        let n = crate::types::node::any_node();
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "()");
    }

    #[test]
    fn render_node_with_multiple_labels() {
        // (n:`Person`:`Actor`)
        let n = crate::types::node::node("Person")
            .named("n")
            .with_labels(["Actor"]);
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n:`Person`:`Actor`)");
    }

    #[test]
    fn render_node_with_properties() {
        // (p:`Person` {name: 'Alice', age: 30})
        let props = crate::props! { "name" => "Alice", "age" => 30_i32 };
        let n = crate::types::node::node("Person")
            .named("p")
            .with_properties(props);
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(p:`Person` {name: 'Alice', age: 30})");
    }

    #[test]
    fn render_node_with_label_expression_and() {
        // (n:`A`&`B`)  — label expression: A&B
        use crate::types::node::LabelExpression;
        let n = crate::types::node::any_node_named("n").with_label_expression(
            LabelExpression::and(vec![
                LabelExpression::label("A"),
                LabelExpression::label("B"),
            ]),
        );
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n:`A`&`B`)");
    }

    #[test]
    fn render_node_with_label_expression_or() {
        // (n:`A`|`B`)
        use crate::types::node::LabelExpression;
        let n = crate::types::node::any_node_named("n").with_label_expression(
            LabelExpression::or(vec![
                LabelExpression::label("A"),
                LabelExpression::label("B"),
            ]),
        );
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n:`A`|`B`)");
    }

    #[test]
    fn render_node_with_label_expression_not() {
        // (n:!`A`)
        use crate::types::node::LabelExpression;
        let n = crate::types::node::any_node_named("n")
            .with_label_expression(LabelExpression::label("A").not());
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n:!`A`)");
    }

    #[test]
    fn render_node_with_label_expression_wildcard() {
        // (n:%)
        use crate::types::node::LabelExpression;
        let n = crate::types::node::any_node_named("n")
            .with_label_expression(LabelExpression::wildcard());
        let r = renderer();
        let mut buf = String::new();
        r.write_node(&mut buf, &n);
        assert_eq!(buf, "(n:%)");
    }

    #[test]
    fn render_node_escape_as_needed() {
        // (m:Movie)  — simple label doesn't need escaping
        let renderer = DefaultRenderer::new(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        let n = crate::types::node::node("Movie").named("m");
        let mut buf = String::new();
        renderer.write_node(&mut buf, &n);
        assert_eq!(buf, "(m:Movie)");
    }

    #[test]
    fn render_node_escape_as_needed_special_chars() {
        // (m:`My Label`)  — label with space needs escaping
        let renderer = DefaultRenderer::new(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        let n = crate::types::node::node("My Label").named("m");
        let mut buf = String::new();
        renderer.write_node(&mut buf, &n);
        assert_eq!(buf, "(m:`My Label`)");
    }

    // --- Relationship rendering ---

    #[test]
    fn render_simple_outgoing_relationship() {
        // (a)-[:`KNOWS`]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS")).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS`]->(b)");
    }

    #[test]
    fn render_simple_incoming_relationship() {
        // (a)<-[:`KNOWS`]-(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS")).from(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)<-[:`KNOWS`]-(b)");
    }

    #[test]
    fn render_undirected_relationship() {
        // (a)-[:`KNOWS`]-(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS")).between(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS`]-(b)");
    }

    #[test]
    fn render_named_relationship() {
        // (a)-[r:`KNOWS`]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").named("r")).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[r:`KNOWS`]->(b)");
    }

    #[test]
    fn render_relationship_with_properties() {
        // (a)-[:`KNOWS` {since: 2020}]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let detail = crate::types::relationship::rel("KNOWS")
            .with_properties(crate::props! { "since" => 2020_i32 });
        let r = a.rel(detail).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` {since: 2020}]->(b)");
    }

    #[test]
    fn render_relationship_unbounded_length() {
        // (a)-[:`KNOWS` *]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").unbounded()).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` *]->(b)");
    }

    #[test]
    fn render_relationship_exact_length() {
        // (a)-[:`KNOWS` *3]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").exact(3)).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` *3]->(b)");
    }

    #[test]
    fn render_relationship_range_length() {
        // (a)-[:`KNOWS` *1..3]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").min(1).max(3)).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` *1..3]->(b)");
    }

    #[test]
    fn render_relationship_min_only_length() {
        // (a)-[:`KNOWS` *2..]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").min(2)).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` *2..]->(b)");
    }

    #[test]
    fn render_relationship_max_only_length() {
        // (a)-[:`KNOWS` *..5]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::rel("KNOWS").max(5)).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS` *..5]->(b)");
    }

    #[test]
    fn render_untyped_relationship() {
        // (a)-->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(crate::types::relationship::untyped_rel()).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-->(b)");
    }

    #[test]
    fn render_relationship_multiple_types() {
        // (a)-[:`KNOWS`|`LIKES`]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let detail = crate::types::relationship::rel("KNOWS").with_type("LIKES");
        let r = a.rel(detail).to(b);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_relationship(&mut buf, &r);
        assert_eq!(buf, "(a)-[:`KNOWS`|`LIKES`]->(b)");
    }

    // --- Chain rendering ---

    #[test]
    fn render_two_hop_chain() {
        // (a)-[:`R1`]->(b)-[:`R2`]->(c)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let c = crate::types::node::any_node_named("c");
        let chain = a
            .rel(crate::types::relationship::rel("R1"))
            .to(b)
            .rel(crate::types::relationship::rel("R2"))
            .to(c);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_chain(&mut buf, &chain);
        assert_eq!(buf, "(a)-[:`R1`]->(b)-[:`R2`]->(c)");
    }

    #[test]
    fn render_mixed_direction_chain() {
        // (a)-[:`R1`]->(b)<-[:`R2`]-(c)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let c = crate::types::node::any_node_named("c");
        let chain = a
            .rel(crate::types::relationship::rel("R1"))
            .to(b)
            .rel(crate::types::relationship::rel("R2"))
            .from(c);
        let renderer = renderer();
        let mut buf = String::new();
        renderer.write_chain(&mut buf, &chain);
        assert_eq!(buf, "(a)-[:`R1`]->(b)<-[:`R2`]-(c)");
    }

    // --- Pattern rendering ---

    #[test]
    fn render_single_node_pattern() {
        // (n:`Person`)
        let n = crate::types::node::node("Person").named("n");
        let pattern = crate::types::pattern::Pattern::new(n);
        let renderer = renderer();
        assert_eq!(renderer.render_pattern(&pattern), "(n:`Person`)");
    }

    #[test]
    fn render_multi_element_pattern() {
        // (a:`Person`), (b:`Movie`)
        let a = crate::types::node::node("Person").named("a");
        let b = crate::types::node::node("Movie").named("b");
        let pattern = crate::types::pattern::Pattern::new(a).and(b);
        let renderer = renderer();
        assert_eq!(
            renderer.render_pattern(&pattern),
            "(a:`Person`), (b:`Movie`)"
        );
    }

    #[test]
    fn render_named_path() {
        // p = (a)-[:`KNOWS`]->(b)
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let rel = a.rel(crate::types::relationship::rel("KNOWS")).to(b);
        let named = crate::types::pattern::path("p").defined_by(rel);
        let pattern = crate::types::pattern::Pattern::new(named);
        let renderer = renderer();
        assert_eq!(
            renderer.render_pattern(&pattern),
            "p = (a)-[:`KNOWS`]->(b)"
        );
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
