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
    /// Whether all inline property map values are parameters.
    fully_parameterized: bool,
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
        let fully_parameterized = check_property_maps_parameterized(stmt);
        Self {
            labels: walker.labels,
            relationship_types: walker.relationship_types,
            properties: walker.properties,
            parameters: walker.parameters,
            fully_parameterized,
        }
    }

    /// Returns `true` if every value inside inline property maps
    /// (on nodes and relationships) is a parameter — no literal
    /// strings, numbers, booleans, or other non-parameter expressions.
    ///
    /// Property maps are the `{key: value}` blocks attached to nodes
    /// and relationships (e.g. `(n:Person {name: $name})`).
    /// Literals appearing elsewhere (WHERE conditions, RETURN
    /// expressions, SET values, etc.) are **not** checked.
    ///
    /// Use this as a runtime guard or test assertion to enforce
    /// parameterized queries:
    ///
    /// ```rust
    /// use rust_cypher_dsl::prelude::*;
    /// use rust_cypher_dsl::catalog::StatementCatalog;
    ///
    /// let stmt = Cypher::match_(
    ///     node("Movie").named("m").with_properties(props!("title" => param("title")))
    /// )
    /// .returning(name("m"))
    /// .build();
    ///
    /// let catalog = StatementCatalog::from_statement(&stmt);
    /// assert!(catalog.is_fully_parameterized());
    /// ```
    #[must_use]
    pub const fn is_fully_parameterized(&self) -> bool {
        self.fully_parameterized
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
            Statement::Admin(cmd) => self.visit_admin_command(cmd),
        }
    }

    fn visit_admin_command(&mut self, cmd: &crate::admin::AdminCommand) {
        use crate::admin::{AdminCommand, ConstraintTarget, IndexTarget};
        match cmd {
            AdminCommand::CreateIndex(ci) => {
                match ci.target() {
                    IndexTarget::Node {
                        labels, properties, ..
                    } => {
                        for label in labels {
                            self.labels.insert(label.to_string());
                        }
                        for prop in properties {
                            let key = (prop.to_string(), None, None);
                            if self.seen_properties.insert(key) {
                                self.properties.push(CatalogProperty {
                                    name: prop.to_string(),
                                    owner_label: None,
                                    owner_type: None,
                                });
                            }
                        }
                    }
                    IndexTarget::Relationship {
                        types, properties, ..
                    } => {
                        for t in types {
                            self.relationship_types.insert(t.to_string());
                        }
                        for prop in properties {
                            let key = (prop.to_string(), None, None);
                            if self.seen_properties.insert(key) {
                                self.properties.push(CatalogProperty {
                                    name: prop.to_string(),
                                    owner_label: None,
                                    owner_type: None,
                                });
                            }
                        }
                    }
                    IndexTarget::NodeLookup { .. }
                    | IndexTarget::RelationshipLookup { .. } => {}
                }
                if let Some(opts) = ci.options() {
                    self.visit_expression(opts);
                }
            }
            AdminCommand::CreateConstraint(cc) => {
                match cc.target() {
                    ConstraintTarget::Node { label, .. } => {
                        self.labels.insert(label.to_string());
                    }
                    ConstraintTarget::Relationship { rel_type, .. } => {
                        self.relationship_types.insert(rel_type.to_string());
                    }
                }
                for prop in cc.properties() {
                    let key = (prop.to_string(), None, None);
                    if self.seen_properties.insert(key) {
                        self.properties.push(CatalogProperty {
                            name: prop.to_string(),
                            owner_label: None,
                            owner_type: None,
                        });
                    }
                }
            }
            // SHOW/DROP/TERMINATE commands don't contribute schema metadata.
            AdminCommand::DropIndex(_)
            | AdminCommand::DropConstraint(_)
            | AdminCommand::ShowIndexes(_)
            | AdminCommand::ShowConstraints(_)
            | AdminCommand::ShowFunctions(_)
            | AdminCommand::ShowProcedures(_)
            | AdminCommand::ShowTransactions(_)
            | AdminCommand::TerminateTransactions(_) => {}
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
            PatternElement::PathConcatenation(segments) => {
                for seg in segments {
                    self.visit_pattern_element(seg);
                }
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
            Condition::PatternPredicate(pattern) => {
                self.visit_pattern_elements(pattern.elements());
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

// ---------------------------------------------------------------------------
// Property-map parameterization check
// ---------------------------------------------------------------------------

/// Returns `true` when every value in every inline property map
/// (on nodes and relationships) is a `Parameter`.
fn check_property_maps_parameterized(stmt: &Statement) -> bool {
    match stmt {
        Statement::SinglePart(query) => query
            .clauses()
            .iter()
            .all(clause_props_parameterized),
        Statement::Union(l, r)
        | Statement::UnionAll(l, r)
        | Statement::Next(l, r) => {
            check_property_maps_parameterized(l)
                && check_property_maps_parameterized(r)
        }
        Statement::Explain(inner) | Statement::Profile(inner) => {
            check_property_maps_parameterized(inner)
        }
        Statement::When {
            then_branch,
            else_branch,
            ..
        } => {
            check_property_maps_parameterized(then_branch)
                && else_branch
                    .as_ref()
                    .is_none_or(|e| check_property_maps_parameterized(e))
        }
        // Admin commands don't have inline property maps.
        Statement::Admin(_) => true,
    }
}

/// Checks a single clause for non-parameterized inline property maps.
fn clause_props_parameterized(clause: &Clause) -> bool {
    match clause {
        Clause::Match(m) => pattern_elements_parameterized(m.pattern().elements()),
        Clause::Create(c) => pattern_elements_parameterized(c.pattern().elements()),
        Clause::Merge(m) => pattern_elements_parameterized(m.pattern().elements()),
        Clause::InQueryCall(c) => c.subquery().iter().all(clause_props_parameterized),
        Clause::Foreach(f) => f.clauses().iter().all(clause_props_parameterized),
        // All other clauses (WHERE, RETURN, SET, etc.) don't contain
        // node/relationship inline property maps.
        _ => true,
    }
}

fn pattern_elements_parameterized(elements: &[PatternElement]) -> bool {
    elements.iter().all(pattern_element_parameterized)
}

fn pattern_element_parameterized(elem: &PatternElement) -> bool {
    match elem {
        PatternElement::Node(node) => property_map_parameterized(node.properties()),
        PatternElement::Relationship(rel) => {
            property_map_parameterized(rel.left().properties())
                && property_map_parameterized(rel.right().properties())
                && property_map_parameterized(rel.details().properties())
        }
        PatternElement::Chain(chain) => {
            if !property_map_parameterized(chain.start().properties()) {
                return false;
            }
            chain.links().iter().all(|link| {
                property_map_parameterized(link.target().properties())
                    && property_map_parameterized(link.details().properties())
            })
        }
        PatternElement::NamedPath(named) => pattern_element_parameterized(&named.pattern),
        PatternElement::QuantifiedPath(qp) => pattern_element_parameterized(qp.pattern()),
        PatternElement::SelectedPath(_, inner) => pattern_element_parameterized(inner),
        PatternElement::PathConcatenation(segments) => {
            segments.iter().all(pattern_element_parameterized)
        }
    }
}

/// Checks that all values in a property map expression are parameters.
/// Returns `true` if the expression is `None` (no properties).
fn property_map_parameterized(expr: Option<&Expression>) -> bool {
    let Some(e) = expr else { return true };
    match e.inner() {
        ExpressionInner::MapLiteral(entries) => entries
            .iter()
            .all(|(_, val)| map_value_is_parameter(val)),
        // A bare parameter or symbolic name referencing a map variable
        // is not an inline literal — treat as safe.
        _ => true,
    }
}

/// Returns `true` if the expression is a parameter, or a list/map
/// whose leaves are all parameters.
fn map_value_is_parameter(expr: &Expression) -> bool {
    match expr.inner() {
        ExpressionInner::Parameter(_) => true,
        ExpressionInner::ListLiteral(elems) => elems.iter().all(map_value_is_parameter),
        ExpressionInner::MapLiteral(entries) => {
            entries.iter().all(|(_, v)| map_value_is_parameter(v))
        }
        _ => false,
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

    // ── Admin command catalog tests ──

    #[test]
    fn collects_labels_from_create_index_node() {
        let stmt = crate::cypher::Cypher::create_index("idx")
            .for_node("n", "Person", vec!["name", "email"])
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        let prop_names: HashSet<_> = catalog.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(prop_names.contains("name"));
        assert!(prop_names.contains("email"));
    }

    #[test]
    fn collects_types_from_create_index_relationship() {
        let stmt = crate::cypher::Cypher::create_index("idx")
            .for_relationship("r", "KNOWS", vec!["since"])
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.relationship_types.contains("KNOWS"));
        let prop_names: HashSet<_> = catalog.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(prop_names.contains("since"));
    }

    #[test]
    fn collects_labels_from_create_constraint() {
        let stmt = crate::cypher::Cypher::create_constraint("c")
            .for_node("n", "Person")
            .is_unique(vec!["email"]);
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.contains("Person"));
        let prop_names: HashSet<_> = catalog.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(prop_names.contains("email"));
    }

    #[test]
    fn collects_types_from_create_constraint_relationship() {
        let stmt = crate::cypher::Cypher::create_constraint("c")
            .for_relationship("r", "REVIEWED")
            .is_typed("score", "FLOAT");
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.relationship_types.contains("REVIEWED"));
        let prop_names: HashSet<_> = catalog.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(prop_names.contains("score"));
    }

    #[test]
    fn drop_and_show_contribute_nothing() {
        let stmt = crate::cypher::Cypher::drop_index("idx");
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.is_empty());
        assert!(catalog.relationship_types.is_empty());
        assert!(catalog.properties.is_empty());

        let stmt = crate::cypher::Cypher::show_indexes().build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.labels.is_empty());
    }

    // ── is_fully_parameterized tests ──

    #[test]
    fn parameterized_node_properties_passes() {
        // MATCH (m:Movie {title: $title}) RETURN m
        let stmt = crate::cypher::Cypher::match_(
            node("Movie")
                .named("m")
                .with_properties(crate::props!("title" => crate::types::parameter::param("title"))),
        )
        .returning(Expression::symbolic_name("m"))
        .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.is_fully_parameterized());
    }

    #[test]
    fn inline_string_in_node_properties_fails() {
        // MATCH (m:Movie {title: 'The Matrix'}) RETURN m
        let stmt = crate::cypher::Cypher::match_(
            node("Movie")
                .named("m")
                .with_properties(crate::props!("title" => "The Matrix")),
        )
        .returning(Expression::symbolic_name("m"))
        .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(!catalog.is_fully_parameterized());
    }

    #[test]
    fn inline_integer_in_node_properties_fails() {
        // CREATE (m:Movie {released: 2024}) RETURN m
        let stmt = crate::cypher::Cypher::create(
            node("Movie")
                .named("m")
                .with_properties(crate::props!("released" => 2024_i32)),
        )
        .returning(Expression::symbolic_name("m"))
        .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(!catalog.is_fully_parameterized());
    }

    #[test]
    fn inline_in_relationship_properties_fails() {
        // MATCH (a)-[:ACTED_IN {roles: ['Hero']}]->(b) RETURN a
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let r = rel("ACTED_IN").with_properties(
            crate::props!("roles" => Expression::list_literal(vec![Expression::from("Hero")])),
        );
        let stmt = crate::cypher::Cypher::match_(a.rel(r).to(b))
            .returning(Expression::symbolic_name("a"))
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(!catalog.is_fully_parameterized());
    }

    #[test]
    fn parameterized_relationship_properties_passes() {
        let a = node("Person").named("a");
        let b = node("Movie").named("b");
        let r = rel("ACTED_IN").with_properties(
            crate::props!("roles" => crate::types::parameter::param("roles")),
        );
        let stmt = crate::cypher::Cypher::match_(a.rel(r).to(b))
            .returning(Expression::symbolic_name("a"))
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.is_fully_parameterized());
    }

    #[test]
    fn no_properties_passes() {
        // MATCH (m:Movie) RETURN m
        let stmt = crate::cypher::Cypher::match_(node("Movie").named("m"))
            .returning(Expression::symbolic_name("m"))
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.is_fully_parameterized());
    }

    #[test]
    fn literals_in_where_do_not_affect_parameterization() {
        // MATCH (m:Movie) WHERE m.released > 2000 RETURN m
        // Inline literal in WHERE is fine -- only property maps matter
        let stmt = crate::cypher::Cypher::match_(node("Movie").named("m"))
            .where_(
                Expression::symbolic_name("m")
                    .property("released")
                    .gt(2000_i32),
            )
            .returning(Expression::symbolic_name("m"))
            .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(catalog.is_fully_parameterized());
    }

    #[test]
    fn inline_in_merge_node_properties_fails() {
        let stmt = crate::cypher::Cypher::merge(
            node("Person")
                .named("p")
                .with_properties(crate::props!("name" => "Tom Hanks")),
        )
        .returning(Expression::symbolic_name("p"))
        .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(!catalog.is_fully_parameterized());
    }

    #[test]
    fn mixed_param_and_literal_fails() {
        // {name: $name, born: 1956} -- one param, one literal -> fails
        let stmt = crate::cypher::Cypher::match_(
            node("Person").named("p").with_properties(crate::props!(
                "name" => crate::types::parameter::param("name"),
                "born" => 1956_i32
            )),
        )
        .returning(Expression::symbolic_name("p"))
        .build();
        let catalog = StatementCatalog::from_statement(&stmt);
        assert!(!catalog.is_fully_parameterized());
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
