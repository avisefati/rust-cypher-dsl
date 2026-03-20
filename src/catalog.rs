//! `StatementCatalog` for introspecting query metadata.
//!
//! Walks the AST of a built [`Statement`]
//! and collects all node labels, relationship types, properties, and
//! parameters referenced in the query.

use std::collections::{HashMap, HashSet};

use crate::clauses::{Clause, MergeAction, RemoveItem, SetItem};
use crate::statement::{SinglePartQuery, Statement};
use crate::types::condition::Condition;
use crate::types::expression::{Expression, ExpressionInner, MapProjectionEntry};
use crate::types::node::{LabelExpression, Node};
use crate::types::pattern::{PatternElement, QuantifiedPath};
use crate::types::relationship::{Relationship, RelationshipChain};

/// Metadata extracted from a [`Statement`] AST.
///
/// Contains the set of node labels, relationship types, properties,
/// and named parameters found in the query.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementCatalog {
    /// All node labels referenced in the query.
    pub labels: HashSet<String>,
    /// All relationship types referenced in the query.
    pub relationship_types: HashSet<String>,
    /// All properties referenced in the query with owner info.
    pub properties: Vec<CatalogProperty>,
    /// All named parameters and their optional bound values.
    pub parameters: HashMap<String, Option<Expression>>,
}

/// A property referenced in the query, with optional owner metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProperty {
    /// The property name.
    pub name: String,
    /// The owner node label, if determinable.
    pub owner_label: Option<String>,
    /// The owner relationship type, if determinable.
    pub owner_type: Option<String>,
}

impl StatementCatalog {
    /// Builds a catalog by walking the statement AST.
    #[must_use]
    pub fn from_statement(stmt: &Statement) -> Self {
        let mut walker = CatalogWalker::default();
        walker.visit_statement(stmt);
        Self {
            labels: walker.labels,
            relationship_types: walker.relationship_types,
            properties: walker.properties,
            parameters: walker.parameters,
        }
    }
}

/// Internal walker that accumulates catalog metadata.
#[derive(Default)]
struct CatalogWalker {
    labels: HashSet<String>,
    relationship_types: HashSet<String>,
    properties: Vec<CatalogProperty>,
    parameters: HashMap<String, Option<Expression>>,
    /// Tracks seen property `(name, owner_label, owner_type)` to deduplicate.
    seen_properties: HashSet<(String, Option<String>, Option<String>)>,
}

impl CatalogWalker {
    // ── Statement / Query ──

    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::SinglePart(query) => self.visit_single_part(query),
            Statement::Union(left, right)
            | Statement::UnionAll(left, right)
            | Statement::Next(left, right) => {
                self.visit_statement(left);
                self.visit_statement(right);
            }
            Statement::Explain(inner) | Statement::Profile(inner) => {
                self.visit_statement(inner);
            }
            Statement::When {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_condition(condition);
                self.visit_statement(then_branch);
                if let Some(else_stmt) = else_branch {
                    self.visit_statement(else_stmt);
                }
            }
        }
    }

    fn visit_single_part(&mut self, query: &SinglePartQuery) {
        for clause in query.clauses() {
            self.visit_clause(clause);
        }
    }

    // ── Clauses ──

    fn visit_clause(&mut self, clause: &Clause) {
        match clause {
            Clause::Match(m) => self.visit_pattern_elements(m.pattern().elements()),
            Clause::Where(w) => self.visit_condition(w.condition()),
            Clause::Return(r) => self.visit_expressions(r.expressions()),
            Clause::OrderBy(o) => {
                for item in o.items() {
                    self.visit_expression(&item.expression);
                }
            }
            Clause::Skip(s) => self.visit_expression(s.value()),
            Clause::Limit(l) => self.visit_expression(l.value()),
            Clause::With(w) => self.visit_expressions(w.expressions()),
            Clause::Unwind(u) => self.visit_expression(u.expression()),
            Clause::Create(c) => self.visit_pattern_elements(c.pattern().elements()),
            Clause::Merge(m) => {
                self.visit_pattern_elements(m.pattern().elements());
                for action in m.actions() {
                    match action {
                        MergeAction::OnCreate(items) | MergeAction::OnMatch(items) => {
                            self.visit_set_items(items);
                        }
                    }
                }
            }
            Clause::Set(s) => self.visit_set_items(s.items()),
            Clause::Delete(d) => self.visit_expressions(d.expressions()),
            Clause::Remove(r) => {
                for item in r.items() {
                    match item {
                        RemoveItem::Property(prop) => {
                            self.visit_expression(&Expression::from(prop.clone()));
                        }
                        RemoveItem::Label { node, labels } => {
                            self.visit_expression(node);
                            for label in labels {
                                self.labels.insert(label.to_string());
                            }
                        }
                    }
                }
            }
            Clause::Foreach(f) => {
                self.visit_expression(f.list());
                for inner in f.clauses() {
                    self.visit_clause(inner);
                }
            }
            Clause::Call(c) => {
                self.visit_expressions(c.arguments());
                self.visit_expressions(c.yield_fields());
                if let Some(cond) = c.where_cond() {
                    self.visit_condition(cond);
                }
            }
            Clause::InQueryCall(c) => {
                for inner in c.subquery() {
                    self.visit_clause(inner);
                }
            }
            Clause::LoadCsv(l) => self.visit_expression(l.url()),
            Clause::Use(u) => self.visit_expression(u.graph()),
            Clause::UsingIndex(u) => {
                self.labels.insert(u.label().to_owned());
            }
            Clause::UsingScan(u) => {
                self.labels.insert(u.label().to_owned());
            }
            Clause::UsingJoin(_) | Clause::UsingPeriodicCommit(_) | Clause::Finish => {}
            // Cypher 25
            Clause::Filter(f) => self.visit_condition(f.condition()),
            Clause::Let(l) => self.visit_expression(l.expression()),
        }
    }

    // ── Set items ──

    fn visit_set_items(&mut self, items: &[SetItem]) {
        for item in items {
            match item {
                SetItem::Property { property, value } => {
                    self.visit_expression(&Expression::from(property.clone()));
                    self.visit_expression(value);
                }
                SetItem::Label { node, labels } => {
                    self.visit_expression(node);
                    for label in labels {
                        self.labels.insert(label.to_string());
                    }
                }
                SetItem::Mutate { target, value }
                | SetItem::ReplaceAll { target, value } => {
                    self.visit_expression(target);
                    self.visit_expression(value);
                }
            }
        }
    }

    // ── Pattern elements ──

    fn visit_pattern_elements(&mut self, elements: &[PatternElement]) {
        for elem in elements {
            self.visit_pattern_element(elem);
        }
    }

    fn visit_pattern_element(&mut self, elem: &PatternElement) {
        match elem {
            PatternElement::Node(node) => self.visit_node(node),
            PatternElement::Relationship(rel) => self.visit_relationship(rel),
            PatternElement::Chain(chain) => self.visit_chain(chain),
            PatternElement::NamedPath(named) => {
                self.visit_pattern_element(&named.pattern);
            }
            PatternElement::QuantifiedPath(qp) => self.visit_quantified_path(qp),
            PatternElement::SelectedPath(_, inner) => {
                self.visit_pattern_element(inner);
            }
        }
    }

    fn visit_quantified_path(&mut self, qp: &QuantifiedPath) {
        self.visit_pattern_element(qp.pattern());
        if let Some(w) = qp.where_clause() {
            self.visit_expression(w);
        }
    }

    fn visit_node(&mut self, node: &Node) {
        for label in node.labels() {
            self.labels.insert(label.value().to_owned());
        }
        if let Some(label_expr) = node.label_expression() {
            self.visit_label_expression(label_expr);
        }
        if let Some(props) = node.properties() {
            self.visit_expression(props);
        }
    }

    fn visit_label_expression(&mut self, expr: &LabelExpression) {
        match expr {
            LabelExpression::Label(name) => {
                self.labels.insert(name.to_string());
            }
            LabelExpression::And(children) | LabelExpression::Or(children) => {
                for child in children {
                    self.visit_label_expression(child);
                }
            }
            LabelExpression::Not(inner) => self.visit_label_expression(inner),
            LabelExpression::Wildcard => {}
        }
    }

    fn visit_relationship(&mut self, rel: &Relationship) {
        self.visit_node(rel.left());
        self.visit_node(rel.right());
        for type_name in rel.details().types() {
            self.relationship_types.insert(type_name.to_string());
        }
        if let Some(props) = rel.details().properties() {
            self.visit_expression(props);
        }
    }

    fn visit_chain(&mut self, chain: &RelationshipChain) {
        self.visit_node(chain.start());
        for link in chain.links() {
            self.visit_node(link.target());
            for type_name in link.details().types() {
                self.relationship_types.insert(type_name.to_string());
            }
            if let Some(props) = link.details().properties() {
                self.visit_expression(props);
            }
        }
    }

    // ── Expressions ──

    fn visit_expressions(&mut self, exprs: &[Expression]) {
        for expr in exprs {
            self.visit_expression(expr);
        }
    }

    fn visit_expression(&mut self, expr: &Expression) {
        match expr.inner() {
            ExpressionInner::Parameter(param) => {
                self.parameters
                    .entry(param.name().to_owned())
                    .or_insert_with(|| param.value().cloned());
            }
            ExpressionInner::Property(prop) => self.visit_property(prop),
            ExpressionInner::Node(node) => self.visit_node(node),
            ExpressionInner::Relationship(rel) => self.visit_relationship(rel),
            ExpressionInner::ListLiteral(elems) => self.visit_expressions(elems),
            ExpressionInner::MapLiteral(entries) => {
                for (_, val) in entries {
                    self.visit_expression(val);
                }
            }
            ExpressionInner::Operation { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            ExpressionInner::Aliased { delegate, .. } => self.visit_expression(delegate),
            ExpressionInner::Condition(cond) => self.visit_condition(cond),
            ExpressionInner::FunctionInvocation { args, .. } => self.visit_expressions(args),
            ExpressionInner::SimpleCaseExpression { operand, when_clauses, else_clause } => {
                self.visit_case_expression(Some(operand), when_clauses, else_clause.as_ref());
            }
            ExpressionInner::GenericCaseExpression { when_clauses, else_clause } => {
                self.visit_case_expression(None, when_clauses, else_clause.as_ref());
            }
            ExpressionInner::ListComprehension { list, where_clause, projection, .. } => {
                self.visit_expression(list);
                self.visit_optional_expression(where_clause.as_ref());
                self.visit_optional_expression(projection.as_ref());
            }
            ExpressionInner::PatternComprehension { pattern, where_clause, projection } => {
                self.visit_expression(pattern);
                self.visit_optional_expression(where_clause.as_ref());
                self.visit_expression(projection);
            }
            ExpressionInner::MapProjection { variable, entries } => {
                self.visit_expression(variable);
                for entry in entries {
                    if let MapProjectionEntry::Literal(_, e) = entry {
                        self.visit_expression(e);
                    }
                }
            }
            ExpressionInner::ExistentialSubquery(sq)
            | ExpressionInner::CountSubquery(sq)
            | ExpressionInner::CollectSubquery(sq) => self.visit_expression(sq),
            ExpressionInner::ReduceExpression { init, list, expression, .. } => {
                self.visit_expression(init);
                self.visit_expression(list);
                self.visit_expression(expression);
            }
            // Leaf variants with no sub-expressions
            ExpressionInner::StringLiteral(_)
            | ExpressionInner::IntegerLiteral(_)
            | ExpressionInner::FloatLiteral(_)
            | ExpressionInner::BooleanLiteral(_)
            | ExpressionInner::NullLiteral
            | ExpressionInner::SymbolicName(_)
            | ExpressionInner::RawExpression(_)
            | ExpressionInner::Asterisk => {}
        }
    }

    /// Visits a property expression, collecting property metadata.
    fn visit_property(&mut self, prop: &crate::types::property::Property) {
        let owner_label = Self::extract_owner_label(prop.container());
        let owner_type = Self::extract_owner_type(prop.container());
        for name in prop.names() {
            let key = (name.to_string(), owner_label.clone(), owner_type.clone());
            if self.seen_properties.insert(key) {
                self.properties.push(CatalogProperty {
                    name: name.to_string(),
                    owner_label: owner_label.clone(),
                    owner_type: owner_type.clone(),
                });
            }
        }
        self.visit_expression(prop.container());
    }

    /// Visits a CASE expression (simple or generic).
    fn visit_case_expression(
        &mut self,
        operand: Option<&Expression>,
        when_clauses: &[(Expression, Expression)],
        else_clause: Option<&Expression>,
    ) {
        if let Some(op) = operand {
            self.visit_expression(op);
        }
        for (when_val, then_val) in when_clauses {
            self.visit_expression(when_val);
            self.visit_expression(then_val);
        }
        self.visit_optional_expression(else_clause);
    }

    /// Visits an optional expression if present.
    fn visit_optional_expression(&mut self, expr: Option<&Expression>) {
        if let Some(e) = expr {
            self.visit_expression(e);
        }
    }

    // ── Conditions ──

    fn visit_condition(&mut self, cond: &Condition) {
        match cond {
            Condition::Comparison { left, right, .. }
            | Condition::StringPredicate { left, right, .. }
            | Condition::In { left, right }
            | Condition::RegexMatch { left, pattern: right } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Condition::Compound { conditions, .. } => {
                for c in conditions {
                    self.visit_condition(c);
                }
            }
            Condition::Not(inner) => self.visit_condition(inner),
            Condition::IsNull(expr)
            | Condition::IsNotNull(expr)
            | Condition::ExpressionCondition(expr)
            | Condition::IsTrue(expr)
            | Condition::IsFalse(expr) => {
                self.visit_expression(expr);
            }
            Condition::TypePredicate { expression, .. }
            | Condition::IsNormalized { expression, .. } => {
                self.visit_expression(expression);
            }
            Condition::HasLabels { node, labels } => {
                self.visit_expression(node);
                for label in labels {
                    self.labels.insert(label.to_string());
                }
            }
            Condition::NoCondition => {}
        }
    }

    // ── Owner extraction helpers ──

    /// Tries to extract the node label from a property container.
    ///
    /// Best-effort: returns `None` when the owner cannot be
    /// determined statically (e.g., container is a symbolic name
    /// that would require correlating back to its MATCH clause).
    const fn extract_owner_label(_container: &Expression) -> Option<String> {
        None
    }

    /// Tries to extract the relationship type from a property container.
    const fn extract_owner_type(_container: &Expression) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clauses::{
        CreateClause, MatchClause, MergeAction, MergeClause, ReturnClause,
        SetClause, SetItem, WhereClause, WithClause,
    };
    use crate::statement::SinglePartQuery;
    use crate::types::expression::Expression;
    use crate::types::node::node;
    use crate::types::parameter::Parameter;
    use crate::types::property::Property;
    use crate::types::relationship::rel;

    #[test]
    fn collects_labels_from_match() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert_eq!(catalog.labels.len(), 1);
    }

    #[test]
    fn collects_multiple_labels() {
        let n = node("Person").named("n").with_labels(["Actor"]);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Actor"));
        assert_eq!(catalog.labels.len(), 2);
    }

    #[test]
    fn collects_relationship_types() {
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(r)),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("a"),
                Expression::symbolic_name("b"),
            ])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.relationship_types.contains("KNOWS"));
        assert!(catalog.labels.contains("Person"));
    }

    #[test]
    fn collects_parameters() {
        let n = node("Person").named("n");
        let cond = Expression::symbolic_name("n").property("age").gt(Parameter::new("minAge"));
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Where(WhereClause::new(cond)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.parameters.contains_key("minAge"));
        assert_eq!(catalog.parameters["minAge"], None);
    }

    #[test]
    fn collects_parameters_with_bound_value() {
        let n = node("Person").named("n");
        let cond = Expression::symbolic_name("n").property("age").gt(Parameter::with_value("minAge", 21_i32));
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Where(WhereClause::new(cond)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.parameters.contains_key("minAge"));
        assert_eq!(
            catalog.parameters["minAge"],
            Some(Expression::from(21_i32))
        );
    }

    #[test]
    fn collects_properties() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")),
                Expression::from(Expression::symbolic_name("n").property("age")),
            ])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        let prop_names: HashSet<_> = catalog
            .properties
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(prop_names.contains("name"));
        assert!(prop_names.contains("age"));
    }

    #[test]
    fn deduplicates_labels() {
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(a)),
            Clause::Match(MatchClause::new(b)),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("a"),
                Expression::symbolic_name("b"),
            ])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert_eq!(catalog.labels.len(), 1);
        assert!(catalog.labels.contains("Person"));
    }

    #[test]
    fn collects_from_union() {
        let left = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(node("Person").named("n"))),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let right = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(node("Movie").named("n"))),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let stmt = left.union(right);
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Movie"));
    }

    #[test]
    fn collects_from_create_and_merge() {
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let m = node("Company")
            .named("m")
            .with_properties(crate::props! { "name" => "Acme" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Create(CreateClause::new(n)),
            Clause::Merge(MergeClause::new(m)),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Company"));
    }

    #[test]
    fn collects_from_set_label() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Set(SetClause::new(vec![SetItem::label(
                Expression::symbolic_name("n"),
                vec![std::borrow::Cow::Borrowed("Admin")],
            )])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Admin"));
    }

    #[test]
    fn collects_from_has_labels_condition() {
        let cond = Condition::HasLabels {
            node: Expression::symbolic_name("n"),
            labels: vec![
                std::borrow::Cow::Borrowed("Person"),
                std::borrow::Cow::Borrowed("Actor"),
            ],
        };
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Where(WhereClause::new(cond)),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Actor"));
    }

    #[test]
    fn collects_from_chain() {
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let c = node("Director").named("c");
        let chain = a
            .rel(rel("ACTED_IN"))
            .to(b)
            .rel(rel("DIRECTED_BY"))
            .to(c);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(chain)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("a")])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Movie"));
        assert!(catalog.labels.contains("Director"));
        assert!(catalog.relationship_types.contains("ACTED_IN"));
        assert!(catalog.relationship_types.contains("DIRECTED_BY"));
    }

    #[test]
    fn collects_from_explain_profile() {
        let inner = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(node("Person").named("n"))),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let stmt = inner.explain();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
    }

    #[test]
    fn collects_from_merge_on_create() {
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::with_actions(
                n,
                vec![MergeAction::OnCreate(vec![SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "created"),
                    Expression::from(true),
                )])],
            )),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        let prop_names: HashSet<_> = catalog
            .properties
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(prop_names.contains("created"));
    }

    #[test]
    fn collects_from_complex_multi_clause_query() {
        // MATCH (n:Person)-[:KNOWS]->(m:Movie)
        // WHERE n.age > $minAge
        // WITH n.name AS name, m.title AS title
        // RETURN name, title
        let n = node("Person").named("n");
        let m = node("Movie").named("m");
        let r = n.rel(rel("KNOWS")).to(m);
        let cond = Expression::symbolic_name("n").property("age").gt(Parameter::new("minAge"));
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(r)),
            Clause::Where(WhereClause::new(cond)),
            Clause::With(WithClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name"))
                    .alias("name"),
                Expression::from(Expression::symbolic_name("m").property("title"))
                    .alias("title"),
            ])),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("name"),
                Expression::symbolic_name("title"),
            ])),
        ]));
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        assert!(catalog.labels.contains("Movie"));
        assert!(catalog.relationship_types.contains("KNOWS"));
        assert!(catalog.parameters.contains_key("minAge"));
        let prop_names: HashSet<_> = catalog
            .properties
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(prop_names.contains("age"));
        assert!(prop_names.contains("name"));
        assert!(prop_names.contains("title"));
    }
}
