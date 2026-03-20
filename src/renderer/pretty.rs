//! Pretty-printing renderer for Cypher AST types.
//!
//! Produces indented, multi-line output with configurable indent
//! string. Each top-level clause starts on its own line.

use std::fmt::Write;

use super::default::DefaultRenderer;
use super::RenderConfig;

/// Multi-line, indented Cypher renderer.
///
/// Each clause is placed on its own line. Nested constructs such as
/// subqueries inside `CALL { }` or `FOREACH` bodies are indented
/// by one additional level.
#[derive(Debug, Clone)]
pub struct PrettyRenderer {
    /// The underlying default renderer for expressions, conditions,
    /// and patterns.
    inner: DefaultRenderer,
    /// The indent string (e.g. `"  "` or `"\t"`).
    indent: String,
}

impl PrettyRenderer {
    /// Creates a new pretty renderer from the given config.
    pub fn new(config: RenderConfig) -> Self {
        let indent = config.indent.to_string();
        Self {
            inner: DefaultRenderer::new(config),
            indent,
        }
    }

    /// Creates a new pretty renderer with default config.
    pub fn with_defaults() -> Self {
        Self::new(RenderConfig {
            pretty_print: true,
            ..RenderConfig::default()
        })
    }

    /// Renders a complete statement to a pretty-printed Cypher string.
    pub fn render_statement(&self, stmt: &crate::statement::Statement) -> String {
        let mut buf = String::new();
        self.write_statement(&mut buf, stmt, 0);
        buf
    }

    /// Writes a statement at the given indentation depth.
    fn write_statement(
        &self,
        buf: &mut String,
        stmt: &crate::statement::Statement,
        depth: usize,
    ) {
        match stmt {
            crate::statement::Statement::SinglePart(query) => {
                self.write_single_part_query(buf, query, depth);
            }
            crate::statement::Statement::Union(left, right) => {
                self.write_statement(buf, left, depth);
                buf.push('\n');
                self.write_indent(buf, depth);
                buf.push_str("UNION");
                buf.push('\n');
                self.write_statement(buf, right, depth);
            }
            crate::statement::Statement::UnionAll(left, right) => {
                self.write_statement(buf, left, depth);
                buf.push('\n');
                self.write_indent(buf, depth);
                buf.push_str("UNION ALL");
                buf.push('\n');
                self.write_statement(buf, right, depth);
            }
            crate::statement::Statement::Explain(inner) => {
                self.write_indent(buf, depth);
                buf.push_str("EXPLAIN");
                buf.push('\n');
                self.write_statement(buf, inner, depth);
            }
            crate::statement::Statement::Profile(inner) => {
                self.write_indent(buf, depth);
                buf.push_str("PROFILE");
                buf.push('\n');
                self.write_statement(buf, inner, depth);
            }
            // Cypher 25 composition
            crate::statement::Statement::Next(left, right) => {
                self.write_statement(buf, left, depth);
                buf.push('\n');
                self.write_indent(buf, depth);
                buf.push_str("NEXT");
                buf.push('\n');
                self.write_statement(buf, right, depth);
            }
            crate::statement::Statement::When {
                condition,
                then_branch,
                else_branch,
            } => {
                self.write_indent(buf, depth);
                buf.push_str("WHEN ");
                self.inner.write_condition(buf, condition);
                buf.push_str(" THEN");
                buf.push('\n');
                self.write_statement(buf, then_branch, depth);
                if let Some(else_stmt) = else_branch {
                    buf.push('\n');
                    self.write_indent(buf, depth);
                    buf.push_str("ELSE");
                    buf.push('\n');
                    self.write_statement(buf, else_stmt, depth);
                }
            }
        }
    }

    /// Writes a single-part query with each clause on its own line.
    fn write_single_part_query(
        &self,
        buf: &mut String,
        query: &crate::statement::SinglePartQuery,
        depth: usize,
    ) {
        for (i, clause) in query.clauses().iter().enumerate() {
            if i > 0 {
                buf.push('\n');
            }
            self.write_clause(buf, clause, depth);
        }
    }

    /// Writes a single clause, indented to the given depth.
    fn write_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::Clause,
        depth: usize,
    ) {
        self.write_indent(buf, depth);
        match clause {
            crate::clauses::Clause::Match(m) => self.write_match_clause(buf, m),
            crate::clauses::Clause::Where(w) => self.write_where_clause(buf, w),
            crate::clauses::Clause::Return(r) => self.write_return_clause(buf, r),
            crate::clauses::Clause::OrderBy(o) => self.write_order_by_clause(buf, o),
            crate::clauses::Clause::Skip(s) => self.write_skip_clause(buf, s),
            crate::clauses::Clause::Limit(l) => self.write_limit_clause(buf, l),
            crate::clauses::Clause::With(w) => self.write_with_clause(buf, w),
            crate::clauses::Clause::Unwind(u) => self.write_unwind_clause(buf, u),
            crate::clauses::Clause::Create(c) => self.write_create_clause(buf, c),
            crate::clauses::Clause::Merge(m) => self.write_merge_clause(buf, m),
            crate::clauses::Clause::Set(s) => self.write_set_clause(buf, s),
            crate::clauses::Clause::Delete(d) => self.write_delete_clause(buf, d),
            crate::clauses::Clause::Remove(r) => self.write_remove_clause(buf, r),
            crate::clauses::Clause::Foreach(f) => self.write_foreach_clause(buf, f, depth),
            crate::clauses::Clause::Call(c) => self.write_call_clause(buf, c),
            crate::clauses::Clause::InQueryCall(c) => {
                self.write_in_query_call_clause(buf, c, depth);
            }
            crate::clauses::Clause::LoadCsv(l) => self.write_load_csv_clause(buf, l),
            crate::clauses::Clause::Use(u) => self.write_use_clause(buf, u),
            crate::clauses::Clause::UsingIndex(u) => self.write_using_index_clause(buf, u),
            crate::clauses::Clause::UsingScan(u) => self.write_using_scan_clause(buf, u),
            crate::clauses::Clause::UsingJoin(u) => self.write_using_join_clause(buf, u),
            crate::clauses::Clause::UsingPeriodicCommit(u) => {
                self.write_using_periodic_commit_clause(buf, u);
            }
            // Cypher 25
            crate::clauses::Clause::Filter(f) => self.write_filter_clause(buf, f),
            crate::clauses::Clause::Let(l) => self.write_let_clause(buf, l),
            crate::clauses::Clause::Finish => buf.push_str("FINISH"),
        }
    }

    /// Writes indentation for the given depth.
    fn write_indent(&self, buf: &mut String, depth: usize) {
        for _ in 0..depth {
            buf.push_str(&self.indent);
        }
    }

    // ── Clause writers ──
    // These delegate expression/condition/pattern writing to the inner
    // DefaultRenderer, keeping all inline rendering consistent.

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
        self.inner.write_pattern(buf, clause.pattern());
    }

    fn write_where_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::WhereClause,
    ) {
        buf.push_str("WHERE ");
        self.inner.write_condition(buf, clause.condition());
    }

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
            self.inner.write_expression(buf, expr);
        }
    }

    fn write_order_by_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::OrderByClause,
    ) {
        use crate::types::expression::SortDirection;
        buf.push_str("ORDER BY ");
        for (i, item) in clause.items().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.inner.write_expression(buf, &item.expression);
            match item.direction {
                SortDirection::Ascending => {}
                SortDirection::Descending => buf.push_str(" DESC"),
            }
        }
    }

    fn write_skip_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::SkipClause,
    ) {
        buf.push_str("SKIP ");
        self.inner.write_expression(buf, clause.value());
    }

    fn write_limit_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::LimitClause,
    ) {
        buf.push_str("LIMIT ");
        self.inner.write_expression(buf, clause.value());
    }

    fn write_with_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::WithClause,
    ) {
        if clause.is_distinct() {
            buf.push_str("WITH DISTINCT ");
        } else {
            buf.push_str("WITH ");
        }
        for (i, expr) in clause.expressions().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.inner.write_expression(buf, expr);
        }
    }

    fn write_unwind_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UnwindClause,
    ) {
        buf.push_str("UNWIND ");
        self.inner.write_expression(buf, clause.expression());
    }

    fn write_create_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::CreateClause,
    ) {
        buf.push_str("CREATE ");
        self.inner.write_pattern(buf, clause.pattern());
    }

    fn write_merge_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::MergeClause,
    ) {
        buf.push_str("MERGE ");
        self.inner.write_pattern(buf, clause.pattern());
        for action in clause.actions() {
            match action {
                crate::clauses::MergeAction::OnCreate(items) => {
                    buf.push_str(" ON CREATE SET ");
                    self.inner.write_set_items(buf, items.as_slice());
                }
                crate::clauses::MergeAction::OnMatch(items) => {
                    buf.push_str(" ON MATCH SET ");
                    self.inner.write_set_items(buf, items.as_slice());
                }
            }
        }
    }

    fn write_set_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::SetClause,
    ) {
        buf.push_str("SET ");
        self.inner.write_set_items(buf, clause.items());
    }

    fn write_delete_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::DeleteClause,
    ) {
        if clause.is_detach() {
            buf.push_str("DETACH DELETE ");
        } else {
            buf.push_str("DELETE ");
        }
        for (i, expr) in clause.expressions().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.inner.write_expression(buf, expr);
        }
    }

    fn write_remove_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::RemoveClause,
    ) {
        buf.push_str("REMOVE ");
        for (i, item) in clause.items().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            match item {
                crate::clauses::RemoveItem::Property(property) => {
                    self.inner.write_expression(
                        buf,
                        &crate::types::expression::Expression::from(property.clone()),
                    );
                }
                crate::clauses::RemoveItem::Label { node, labels } => {
                    self.inner.write_expression(buf, node);
                    for label in labels {
                        buf.push(':');
                        self.inner.write_escaped_name(buf, label);
                    }
                }
            }
        }
    }

    fn write_foreach_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::ForeachClause,
        depth: usize,
    ) {
        buf.push_str("FOREACH (");
        self.inner.write_safe_identifier(buf, clause.variable());
        buf.push_str(" IN ");
        self.inner.write_expression(buf, clause.list());
        buf.push_str(" |");
        // Indent inner clauses
        for inner_clause in clause.clauses() {
            buf.push('\n');
            self.write_clause(buf, inner_clause, depth + 1);
        }
        buf.push('\n');
        self.write_indent(buf, depth);
        buf.push(')');
    }

    fn write_call_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::CallClause,
    ) {
        buf.push_str("CALL ");
        buf.push_str(clause.procedure());
        buf.push('(');
        for (i, arg) in clause.arguments().iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            self.inner.write_expression(buf, arg);
        }
        buf.push(')');
        if !clause.yield_fields().is_empty() {
            buf.push_str(" YIELD ");
            for (i, field) in clause.yield_fields().iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                self.inner.write_expression(buf, field);
            }
        }
        if let Some(cond) = clause.where_cond() {
            buf.push_str(" WHERE ");
            self.inner.write_condition(buf, cond);
        }
    }

    fn write_in_query_call_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::InQueryCallClause,
        depth: usize,
    ) {
        buf.push_str("CALL {");
        // Indent inner clauses
        for inner_clause in clause.subquery() {
            buf.push('\n');
            self.write_clause(buf, inner_clause, depth + 1);
        }
        buf.push('\n');
        self.write_indent(buf, depth);
        buf.push('}');
        if clause.is_in_transactions() {
            buf.push_str(" IN TRANSACTIONS");
            if let Some(size) = clause.batch_size() {
                buf.push_str(" OF ");
                self.inner.write_expression(buf, size);
                buf.push_str(" ROWS");
            }
        }
    }

    fn write_load_csv_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::LoadCsvClause,
    ) {
        buf.push_str("LOAD CSV ");
        if clause.is_with_headers() {
            buf.push_str("WITH HEADERS ");
        }
        buf.push_str("FROM ");
        self.inner.write_expression(buf, clause.url());
        buf.push_str(" AS ");
        self.inner.write_safe_identifier(buf, clause.alias());
        if let Some(terminator) = clause.field_terminator_value() {
            buf.push_str(" FIELDTERMINATOR ");
            DefaultRenderer::write_single_quoted(buf, terminator);
        }
    }

    fn write_use_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UseClause,
    ) {
        buf.push_str("USE ");
        self.inner.write_expression(buf, clause.graph());
    }

    fn write_using_index_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UsingIndexClause,
    ) {
        if clause.is_seek() {
            buf.push_str("USING INDEX SEEK ");
        } else {
            buf.push_str("USING INDEX ");
        }
        self.inner.write_safe_identifier(buf, clause.variable());
        buf.push(':');
        self.inner.write_escaped_name(buf, clause.label());
        buf.push('(');
        self.inner.write_safe_identifier(buf, clause.property_name());
        buf.push(')');
    }

    fn write_using_scan_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UsingScanClause,
    ) {
        buf.push_str("USING SCAN ");
        self.inner.write_safe_identifier(buf, clause.variable());
        buf.push(':');
        self.inner.write_escaped_name(buf, clause.label());
    }

    fn write_using_join_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UsingJoinClause,
    ) {
        buf.push_str("USING JOIN ON ");
        self.inner.write_safe_identifier(buf, clause.variable());
    }

    #[expect(clippy::unused_self, reason = "consistent with other write_* methods")]
    fn write_using_periodic_commit_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::UsingPeriodicCommitClause,
    ) {
        buf.push_str("USING PERIODIC COMMIT");
        if let Some(size) = clause.size() {
            buf.push(' ');
            let _ = write!(buf, "{size}");
        }
    }

    // ── Cypher 25 clause writers ──

    fn write_filter_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::FilterClause,
    ) {
        buf.push_str("FILTER ");
        self.inner.write_condition(buf, clause.condition());
    }

    fn write_let_clause(
        &self,
        buf: &mut String,
        clause: &crate::clauses::LetClause,
    ) {
        buf.push_str("LET ");
        self.inner.write_safe_identifier(buf, clause.variable());
        buf.push_str(" = ");
        self.inner.write_expression(buf, clause.expression());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clauses::{
        CreateClause, DeleteClause, ForeachClause, InQueryCallClause,
        LimitClause, MatchClause, OrderByClause, ReturnClause, SetClause, SetItem,
        SkipClause, WhereClause, WithClause,
    };
    use crate::statement::{SinglePartQuery, Statement};
    use crate::types::condition::Condition;
    use crate::types::expression::Expression;
    use crate::types::node::node;
    use crate::types::operator::ComparisonOp;
    use crate::types::property::Property;
    use crate::types::relationship::rel;

    fn pretty() -> PrettyRenderer {
        PrettyRenderer::with_defaults()
    }

    // ── Basic multi-line output ──

    #[test]
    fn simple_match_return() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nRETURN n"
        );
    }

    #[test]
    fn match_where_return() {
        let n = node("Person").named("n");
        let age = Expression::from(Expression::symbolic_name("n").property("age"));
        let cond = Condition::Comparison {
            left: age,
            operator: ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::Where(WhereClause::new(cond)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nWHERE n.age > 21\nRETURN n"
        );
    }

    #[test]
    fn return_with_order_by_skip_limit() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
            crate::clauses::Clause::OrderBy(OrderByClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
            ])),
            crate::clauses::Clause::Skip(SkipClause::new(5_i32)),
            crate::clauses::Clause::Limit(LimitClause::new(10_i32)),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nRETURN n\nORDER BY n.name\nSKIP 5\nLIMIT 10"
        );
    }

    // ── WITH and multi-part queries ──

    #[test]
    fn match_with_match_return() {
        let n = node("Person").named("n");
        let m = node("Movie").named("m");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::With(WithClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
            crate::clauses::Clause::Match(MatchClause::new(m)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
                Expression::symbolic_name("m"),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nWITH n\nMATCH (m:`Movie`)\nRETURN n, m"
        );
    }

    // ── UNION ──

    #[test]
    fn union_two_queries() {
        let left = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Person").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let right = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Movie").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let stmt = left.union(right);
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nRETURN n\nUNION\nMATCH (n:`Movie`)\nRETURN n"
        );
    }

    #[test]
    fn union_all_two_queries() {
        let left = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Person").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let right = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Movie").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let stmt = left.union_all(right);
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nRETURN n\nUNION ALL\nMATCH (n:`Movie`)\nRETURN n"
        );
    }

    // ── EXPLAIN / PROFILE ──

    #[test]
    fn explain_prefix() {
        let inner = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Person").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let stmt = inner.explain();
        assert_eq!(
            pretty().render_statement(&stmt),
            "EXPLAIN\nMATCH (n:`Person`)\nRETURN n"
        );
    }

    #[test]
    fn profile_prefix() {
        let inner = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(node("Person").named("n"))),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let stmt = inner.profile();
        assert_eq!(
            pretty().render_statement(&stmt),
            "PROFILE\nMATCH (n:`Person`)\nRETURN n"
        );
    }

    // ── In-query CALL with indented subquery ──

    #[test]
    fn in_query_call_indented() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::InQueryCall(InQueryCallClause::new(vec![
                crate::clauses::Clause::Match(MatchClause::new(n)),
                crate::clauses::Clause::Return(ReturnClause::new(vec![
                    Expression::symbolic_name("n"),
                ])),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "CALL {\n  MATCH (n:`Person`)\n  RETURN n\n}"
        );
    }

    // ── FOREACH with indented body ──

    #[test]
    fn foreach_indented() {
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Foreach(ForeachClause::new(
                "n",
                Expression::raw_unchecked("nodes(p)"),
                vec![crate::clauses::Clause::Set(SetClause::new(vec![
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "visited"),
                        Expression::from(true),
                    ),
                ]))],
            )),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "FOREACH (n IN nodes(p) |\n  SET n.visited = true\n)"
        );
    }

    // ── Custom indent width ──

    #[test]
    fn custom_indent_tab() {
        let config = RenderConfig {
            pretty_print: true,
            indent: std::borrow::Cow::Borrowed("\t"),
            ..RenderConfig::default()
        };
        let r = PrettyRenderer::new(config);
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::InQueryCall(InQueryCallClause::new(vec![
                crate::clauses::Clause::Match(MatchClause::new(n)),
                crate::clauses::Clause::Return(ReturnClause::new(vec![
                    Expression::symbolic_name("n"),
                ])),
            ])),
        ]));
        assert_eq!(
            r.render_statement(&stmt),
            "CALL {\n\tMATCH (n:`Person`)\n\tRETURN n\n}"
        );
    }

    #[test]
    fn custom_indent_four_spaces() {
        let config = RenderConfig {
            pretty_print: true,
            indent: std::borrow::Cow::Borrowed("    "),
            ..RenderConfig::default()
        };
        let r = PrettyRenderer::new(config);
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::InQueryCall(InQueryCallClause::new(vec![
                crate::clauses::Clause::Match(MatchClause::new(n)),
                crate::clauses::Clause::Return(ReturnClause::new(vec![
                    Expression::symbolic_name("n"),
                ])),
            ])),
        ]));
        assert_eq!(
            r.render_statement(&stmt),
            "CALL {\n    MATCH (n:`Person`)\n    RETURN n\n}"
        );
    }

    // ── CREATE, DELETE, MERGE ──

    #[test]
    fn match_create_return() {
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(a)),
            crate::clauses::Clause::Create(CreateClause::new(r)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("b"),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (a:`Person`)\nCREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)\nRETURN b"
        );
    }

    #[test]
    fn match_detach_delete() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::Delete(DeleteClause::detach(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        assert_eq!(
            pretty().render_statement(&stmt),
            "MATCH (n:`Person`)\nDETACH DELETE n"
        );
    }

    // ── Statement.render_with pretty_print=true ──

    #[test]
    fn render_with_pretty_print_config() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            crate::clauses::Clause::Match(MatchClause::new(n)),
            crate::clauses::Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        let config = RenderConfig {
            pretty_print: true,
            ..RenderConfig::default()
        };
        assert_eq!(
            stmt.render_with(config),
            "MATCH (n:`Person`)\nRETURN n"
        );
    }
}
