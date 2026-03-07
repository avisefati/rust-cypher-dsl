//! `Node` type representing Cypher graph nodes.
//!
//! Nodes represent vertices in the property graph, rendered as
//! `(name:Label1:Label2 {props})`. Uses `Rc` internally for cheap cloning.

use std::borrow::Cow;
use std::rc::Rc;

use super::condition::Condition;
use super::expression::Expression;
use super::property::Property;

/// A graph node in a Cypher pattern.
///
/// Uses `Rc` internally so cloning is O(1). Nodes can be named,
/// labelled, and carry inline properties.
///
/// # Examples (conceptual)
///
/// ```text
/// let movie = node("Movie").named("m");        // (m:Movie)
/// let anon  = node("Person");                   // (:Person)
/// let bare  = any_node().named("n");            // (n)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Node(Rc<NodeInner>);

/// Inner representation of a node (behind `Rc`).
#[derive(Debug, Clone, PartialEq)]
struct NodeInner {
    /// Optional symbolic name (e.g., `m` in `(m:Movie)`).
    symbolic_name: Option<Cow<'static, str>>,
    /// Labels applied to the node.
    labels: Vec<NodeLabel>,
    /// Optional inline properties (a `MapLiteral` or `Parameter`).
    properties: Option<Expression>,
    /// Optional label expression for complex label predicates.
    label_expression: Option<LabelExpression>,
}

/// A single node label (e.g., `Person`, `Movie`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLabel(pub Cow<'static, str>);

/// Label expressions for complex label predicates.
///
/// Supports AND (`&`), OR (`|`), NOT (`!`), and wildcard (`%`).
///
/// # Examples (conceptual)
///
/// ```text
/// // (n:A&B)       → LabelExpression::And(vec![Label("A"), Label("B")])
/// // (n:A|B)       → LabelExpression::Or(vec![Label("A"), Label("B")])
/// // (n:!A)        → LabelExpression::Not(Box::new(Label("A")))
/// // (n:%)         → LabelExpression::Wildcard
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum LabelExpression {
    /// A single label.
    Label(Cow<'static, str>),
    /// Conjunction: all labels must match (`A&B&C`).
    And(Vec<Self>),
    /// Disjunction: any label matches (`A|B|C`).
    Or(Vec<Self>),
    /// Negation: label must not match (`!A`).
    Not(Box<Self>),
    /// Wildcard: matches any label (`%`).
    Wildcard,
}

impl Node {
    /// Creates a node with a single label and no symbolic name.
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(NodeInner {
            symbolic_name: None,
            labels: vec![NodeLabel(label.into())],
            properties: None,
            label_expression: None,
        }))
    }

    /// Creates an anonymous node with no labels and no name.
    pub fn any() -> Self {
        Self(Rc::new(NodeInner {
            symbolic_name: None,
            labels: Vec::new(),
            properties: None,
            label_expression: None,
        }))
    }

    /// Creates a named node with no labels.
    pub fn any_named(name: impl Into<Cow<'static, str>>) -> Self {
        Self(Rc::new(NodeInner {
            symbolic_name: Some(name.into()),
            labels: Vec::new(),
            properties: None,
            label_expression: None,
        }))
    }

    /// Returns a copy of this node with the given symbolic name.
    #[must_use]
    pub fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        self.with_inner(|inner| NodeInner {
            symbolic_name: Some(name.into()),
            ..inner
        })
    }

    /// Returns a copy of this node with additional labels.
    #[must_use]
    pub fn with_labels(self, labels: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) -> Self {
        self.with_inner(|mut inner| {
            inner.labels.extend(labels.into_iter().map(|l| NodeLabel(l.into())));
            inner
        })
    }

    /// Returns a copy of this node with inline properties.
    ///
    /// The expression should be a `MapLiteral` or `Parameter`.
    #[must_use]
    pub fn with_properties(self, properties: impl Into<Expression>) -> Self {
        self.with_inner(|inner| NodeInner {
            properties: Some(properties.into()),
            ..inner
        })
    }

    /// Returns a copy of this node with a label expression.
    #[must_use]
    pub fn with_label_expression(self, expr: LabelExpression) -> Self {
        self.with_inner(|inner| NodeInner {
            label_expression: Some(expr),
            ..inner
        })
    }

    /// Creates a `Property` access on this node: `node.propName`.
    pub fn property(&self, name: impl Into<Cow<'static, str>>) -> Property {
        Property::new(Expression::from(self.clone()), name)
    }

    /// Creates a `HasLabels` condition: `node:Label`.
    pub fn has_labels(&self, labels: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) -> Condition {
        Condition::HasLabels {
            node: Expression::from(self.clone()),
            labels: labels.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the symbolic name, if any.
    pub fn symbolic_name(&self) -> Option<&str> {
        self.0.symbolic_name.as_deref()
    }

    /// Returns the labels.
    pub fn labels(&self) -> &[NodeLabel] {
        &self.0.labels
    }

    /// Returns the inline properties expression, if any.
    pub fn properties(&self) -> Option<&Expression> {
        self.0.properties.as_ref()
    }

    /// Returns the label expression, if any.
    pub fn label_expression(&self) -> Option<&LabelExpression> {
        self.0.label_expression.as_ref()
    }

    /// Internal helper: clone the inner data, apply a transformation, and rewrap.
    fn with_inner(self, f: impl FnOnce(NodeInner) -> NodeInner) -> Self {
        let inner = Rc::unwrap_or_clone(self.0);
        Self(Rc::new(f(inner)))
    }
}

/// Creates a node with a single label (convenience free function).
///
/// Equivalent to `Node::new(label)`.
pub fn node(label: impl Into<Cow<'static, str>>) -> Node {
    Node::new(label)
}

/// Creates an anonymous node with no labels or name.
///
/// Equivalent to `Node::any()`.
pub fn any_node() -> Node {
    Node::any()
}

/// Creates a named node with no labels.
///
/// Equivalent to `Node::any_named(name)`.
pub fn any_node_named(name: impl Into<Cow<'static, str>>) -> Node {
    Node::any_named(name)
}

impl NodeLabel {
    /// Returns the label name.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl LabelExpression {
    /// Creates a label expression from a single label name.
    pub fn label(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Label(name.into())
    }

    /// Creates an AND expression from multiple label expressions.
    pub const fn and(exprs: Vec<Self>) -> Self {
        Self::And(exprs)
    }

    /// Creates an OR expression from multiple label expressions.
    pub const fn or(exprs: Vec<Self>) -> Self {
        Self::Or(exprs)
    }

    /// Negates this label expression.
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "DSL method mirrors Cypher NOT semantics, not Rust std::ops::Not"
    )]
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Creates a wildcard label expression (`%`).
    pub const fn wildcard() -> Self {
        Self::Wildcard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Node creation ---

    #[test]
    fn new_creates_node_with_label() {
        let n = Node::new("Person");
        assert!(n.symbolic_name().is_none());
        assert_eq!(n.labels().len(), 1);
        assert_eq!(n.labels()[0].value(), "Person");
        assert!(n.properties().is_none());
    }

    #[test]
    fn any_creates_anonymous_node() {
        let n = Node::any();
        assert!(n.symbolic_name().is_none());
        assert!(n.labels().is_empty());
    }

    #[test]
    fn any_named_creates_named_node_without_labels() {
        let n = Node::any_named("n");
        assert_eq!(n.symbolic_name(), Some("n"));
        assert!(n.labels().is_empty());
    }

    #[test]
    fn named_sets_symbolic_name() {
        let n = Node::new("Movie").named("m");
        assert_eq!(n.symbolic_name(), Some("m"));
        assert_eq!(n.labels().len(), 1);
        assert_eq!(n.labels()[0].value(), "Movie");
    }

    // --- Free functions ---

    #[test]
    fn node_free_function_creates_labelled_node() {
        let n = node("Actor");
        assert_eq!(n.labels()[0].value(), "Actor");
    }

    #[test]
    fn any_node_free_function_creates_anonymous_node() {
        let n = any_node();
        assert!(n.labels().is_empty());
        assert!(n.symbolic_name().is_none());
    }

    #[test]
    fn any_node_named_free_function() {
        let n = any_node_named("x");
        assert_eq!(n.symbolic_name(), Some("x"));
        assert!(n.labels().is_empty());
    }

    // --- Labels ---

    #[test]
    fn with_labels_adds_additional_labels() {
        let n = node("Person").with_labels(["Actor", "Director"]);
        assert_eq!(n.labels().len(), 3);
        assert_eq!(n.labels()[0].value(), "Person");
        assert_eq!(n.labels()[1].value(), "Actor");
        assert_eq!(n.labels()[2].value(), "Director");
    }

    #[test]
    fn with_labels_on_anonymous_node() {
        let n = any_node().with_labels(["Foo"]);
        assert_eq!(n.labels().len(), 1);
    }

    // --- Properties ---

    #[test]
    fn with_properties_sets_inline_properties() {
        let props = Expression::map_literal(vec![
            (Cow::Borrowed("name"), Expression::from("Alice")),
        ]);
        let n = node("Person").named("p").with_properties(props);
        assert!(n.properties().is_some());
    }

    #[test]
    fn property_creates_property_access() {
        let n = node("Person").named("p");
        let prop = n.property("name");
        assert_eq!(prop.names()[0], "name");
    }

    // --- Label expressions ---

    #[test]
    fn with_label_expression_sets_expression() {
        let expr = LabelExpression::and(vec![
            LabelExpression::label("A"),
            LabelExpression::label("B"),
        ]);
        let n = node("Person").with_label_expression(expr);
        assert!(n.label_expression().is_some());
    }

    #[test]
    fn label_expression_label_variant() {
        let expr = LabelExpression::label("Person");
        assert!(matches!(expr, LabelExpression::Label(_)));
    }

    #[test]
    fn label_expression_and_variant() {
        let expr = LabelExpression::and(vec![
            LabelExpression::label("A"),
            LabelExpression::label("B"),
        ]);
        let LabelExpression::And(children) = &expr else {
            unreachable!("Expected And");
        };
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn label_expression_or_variant() {
        let expr = LabelExpression::or(vec![
            LabelExpression::label("X"),
            LabelExpression::label("Y"),
        ]);
        assert!(matches!(expr, LabelExpression::Or(_)));
    }

    #[test]
    fn label_expression_not_variant() {
        let expr = LabelExpression::label("A").not();
        assert!(matches!(expr, LabelExpression::Not(_)));
    }

    #[test]
    fn label_expression_wildcard() {
        let expr = LabelExpression::wildcard();
        assert!(matches!(expr, LabelExpression::Wildcard));
    }

    #[test]
    fn label_expression_complex_nesting() {
        // (A & !B) | C
        let expr = LabelExpression::or(vec![
            LabelExpression::and(vec![
                LabelExpression::label("A"),
                LabelExpression::label("B").not(),
            ]),
            LabelExpression::label("C"),
        ]);
        let LabelExpression::Or(children) = &expr else {
            unreachable!("Expected Or");
        };
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], LabelExpression::And(_)));
    }

    // --- has_labels condition ---

    #[test]
    fn has_labels_produces_condition() {
        let n = node("Person").named("p");
        let cond = n.has_labels(["Person", "Actor"]);
        let Condition::HasLabels { labels, .. } = &cond else {
            unreachable!("Expected HasLabels");
        };
        assert_eq!(labels.len(), 2);
    }

    // --- From<Node> for Expression ---

    #[test]
    fn from_node_produces_expression() {
        let n = node("Movie").named("m");
        let expr = Expression::from(n);
        assert!(matches!(expr.inner(), super::super::expression::ExpressionInner::Node(_)));
    }

    // --- Cheap cloning ---

    #[test]
    fn clone_is_cheap_rc_based() {
        let n = node("Person").named("p");
        let cloned = n.clone();
        assert!(Rc::ptr_eq(&n.0, &cloned.0));
    }

    #[test]
    fn named_creates_new_rc_not_mutating_original() {
        let original = node("Person");
        let named = original.clone().named("p");
        assert!(original.symbolic_name().is_none());
        assert_eq!(named.symbolic_name(), Some("p"));
    }
}
