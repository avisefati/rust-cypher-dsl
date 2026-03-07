//! `Relationship`, `RelationshipDetail`, and related types.
//!
//! Relationships represent edges in the property graph, connecting
//! two nodes with a direction, optional type(s), variable-length,
//! and properties.

use std::borrow::Cow;

use super::expression::Expression;
use super::node::Node;
use super::property::Property;

/// Relationship metadata: type(s), name, length, and properties.
///
/// Created via the [`rel()`] free function. Does **not** contain
/// direction or endpoint nodes — those are set when building a
/// [`Relationship`] through [`RelationshipBuilder`].
///
/// # Examples (conceptual)
///
/// ```text
/// rel("ACTED_IN").named("r").min(1).max(3)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipDetail {
    /// Relationship type(s).
    pub(crate) types: Vec<Cow<'static, str>>,
    /// Optional symbolic name.
    pub(crate) symbolic_name: Option<Cow<'static, str>>,
    /// Variable-length specification.
    pub(crate) length: Option<RelationshipLength>,
    /// Optional inline properties.
    pub(crate) properties: Option<Expression>,
}

/// A fully resolved relationship in a pattern.
///
/// Contains the left and right nodes, direction, and relationship
/// metadata. Created via [`RelationshipBuilder::to()`],
/// [`RelationshipBuilder::from()`], or [`RelationshipBuilder::between()`].
///
/// # Examples (conceptual)
///
/// ```text
/// // (a)-[:KNOWS]->(b)
/// let knows = node("Person").named("a").rel(rel("KNOWS")).to(node("Person").named("b"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    /// The left-hand node.
    pub(crate) left: Node,
    /// The right-hand node.
    pub(crate) right: Node,
    /// The direction of the relationship.
    pub(crate) direction: Direction,
    /// Relationship metadata (type, name, length, properties).
    pub(crate) details: RelationshipDetail,
}

/// An intermediate builder that knows its left node and relationship
/// details, awaiting a direction-setting terminal method.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipBuilder {
    /// The left-hand node.
    left: Node,
    /// The relationship metadata.
    details: RelationshipDetail,
}

/// Direction of a relationship in a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Outgoing: `(left)-[]->(right)`.
    Outgoing,
    /// Incoming: `(left)<-[]-(right)`.
    Incoming,
    /// Undirected: `(left)-[]-(right)`.
    Undirected,
}

/// Variable-length specification for a relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipLength {
    /// Unbounded: `[*]`.
    Unbounded,
    /// Exact length: `[*n]`.
    Exact(u32),
    /// Range: `[*min..max]`. Both bounds are optional.
    Range {
        /// Minimum hops (inclusive).
        min: Option<u32>,
        /// Maximum hops (inclusive).
        max: Option<u32>,
    },
}

// --- RelationshipDetail ---

impl RelationshipDetail {
    /// Creates a relationship detail with one type.
    pub fn new(type_name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            types: vec![type_name.into()],
            symbolic_name: None,
            length: None,
            properties: None,
        }
    }

    /// Creates a relationship detail with no types (anonymous).
    pub const fn untyped() -> Self {
        Self {
            types: Vec::new(),
            symbolic_name: None,
            length: None,
            properties: None,
        }
    }

    /// Assigns a symbolic name to this relationship.
    #[must_use]
    pub fn named(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.symbolic_name = Some(name.into());
        self
    }

    /// Adds inline properties to this relationship.
    #[must_use]
    pub fn with_properties(mut self, properties: impl Into<Expression>) -> Self {
        self.properties = Some(properties.into());
        self
    }

    /// Adds an additional type to this relationship detail.
    #[must_use]
    pub fn with_type(mut self, type_name: impl Into<Cow<'static, str>>) -> Self {
        self.types.push(type_name.into());
        self
    }

    /// Sets a minimum length bound: `[*min..]`.
    #[must_use]
    pub const fn min(mut self, min: u32) -> Self {
        self.length = Some(match self.length {
            Some(RelationshipLength::Range { max, .. }) => RelationshipLength::Range {
                min: Some(min),
                max,
            },
            _ => RelationshipLength::Range {
                min: Some(min),
                max: None,
            },
        });
        self
    }

    /// Sets a maximum length bound: `[*..max]`.
    #[must_use]
    pub const fn max(mut self, max: u32) -> Self {
        self.length = Some(match self.length {
            Some(RelationshipLength::Range { min, .. }) => RelationshipLength::Range {
                min,
                max: Some(max),
            },
            _ => RelationshipLength::Range {
                min: None,
                max: Some(max),
            },
        });
        self
    }

    /// Sets the length to unbounded: `[*]`.
    #[must_use]
    pub const fn unbounded(mut self) -> Self {
        self.length = Some(RelationshipLength::Unbounded);
        self
    }

    /// Sets an exact length: `[*n]`.
    #[must_use]
    pub const fn exact(mut self, n: u32) -> Self {
        self.length = Some(RelationshipLength::Exact(n));
        self
    }

    /// Returns the relationship type(s).
    pub fn types(&self) -> &[Cow<'static, str>] {
        &self.types
    }

    /// Returns the symbolic name, if any.
    pub fn symbolic_name(&self) -> Option<&str> {
        self.symbolic_name.as_deref()
    }

    /// Returns the length specification, if any.
    pub const fn length(&self) -> Option<&RelationshipLength> {
        self.length.as_ref()
    }

    /// Returns the inline properties expression, if any.
    pub const fn properties(&self) -> Option<&Expression> {
        self.properties.as_ref()
    }
}

// --- RelationshipBuilder ---

impl RelationshipBuilder {
    /// Creates a new builder from a left node and relationship details.
    pub(crate) const fn new(left: Node, details: RelationshipDetail) -> Self {
        Self { left, details }
    }

    /// Completes the relationship as outgoing: `(left)-[]->(right)`.
    pub fn to(self, right: Node) -> Relationship {
        Relationship {
            left: self.left,
            right,
            direction: Direction::Outgoing,
            details: self.details,
        }
    }

    /// Completes the relationship as incoming: `(left)<-[]-(right)`.
    pub fn from(self, right: Node) -> Relationship {
        Relationship {
            left: self.left,
            right,
            direction: Direction::Incoming,
            details: self.details,
        }
    }

    /// Completes the relationship as undirected: `(left)-[]-(right)`.
    pub fn between(self, right: Node) -> Relationship {
        Relationship {
            left: self.left,
            right,
            direction: Direction::Undirected,
            details: self.details,
        }
    }
}

// --- Relationship ---

impl Relationship {
    /// Reverses the direction of this relationship.
    ///
    /// Swaps left and right nodes and flips the direction.
    /// Undirected relationships are unchanged.
    #[must_use]
    pub fn inverse(self) -> Self {
        let direction = match self.direction {
            Direction::Outgoing => Direction::Incoming,
            Direction::Incoming => Direction::Outgoing,
            Direction::Undirected => Direction::Undirected,
        };
        Self {
            left: self.right,
            right: self.left,
            direction,
            details: self.details,
        }
    }

    /// Creates a `Property` access on this relationship.
    pub fn property(&self, name: impl Into<Cow<'static, str>>) -> Property {
        Property::new(Expression::from(self.clone()), name)
    }

    /// Returns the left node.
    pub const fn left(&self) -> &Node {
        &self.left
    }

    /// Returns the right node.
    pub const fn right(&self) -> &Node {
        &self.right
    }

    /// Returns the direction.
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the relationship details.
    pub const fn details(&self) -> &RelationshipDetail {
        &self.details
    }
}

// --- Node integration ---

impl Node {
    /// Starts building a relationship from this node.
    ///
    /// Returns a [`RelationshipBuilder`] that awaits a terminal
    /// method (`.to()`, `.from()`, `.between()`) to set the
    /// direction and target node.
    pub fn rel(&self, detail: RelationshipDetail) -> RelationshipBuilder {
        RelationshipBuilder::new(self.clone(), detail)
    }
}

// --- Free functions ---

/// Creates a relationship detail with one type (convenience free function).
///
/// Equivalent to `RelationshipDetail::new(type_name)`.
pub fn rel(type_name: impl Into<Cow<'static, str>>) -> RelationshipDetail {
    RelationshipDetail::new(type_name)
}

/// Creates an untyped relationship detail.
///
/// Equivalent to `RelationshipDetail::untyped()`.
pub const fn untyped_rel() -> RelationshipDetail {
    RelationshipDetail::untyped()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::node;

    fn person(name: &'static str) -> Node {
        node("Person").named(name)
    }

    // --- RelationshipDetail ---

    #[test]
    fn new_creates_typed_detail() {
        let d = RelationshipDetail::new("KNOWS");
        assert_eq!(d.types().len(), 1);
        assert_eq!(d.types()[0], "KNOWS");
        assert!(d.symbolic_name().is_none());
        assert!(d.length().is_none());
        assert!(d.properties().is_none());
    }

    #[test]
    fn untyped_creates_empty_detail() {
        let d = RelationshipDetail::untyped();
        assert!(d.types().is_empty());
    }

    #[test]
    fn named_sets_symbolic_name() {
        let d = rel("KNOWS").named("r");
        assert_eq!(d.symbolic_name(), Some("r"));
    }

    #[test]
    fn with_type_adds_additional_type() {
        let d = rel("KNOWS").with_type("FOLLOWS");
        assert_eq!(d.types().len(), 2);
        assert_eq!(d.types()[1], "FOLLOWS");
    }

    #[test]
    fn with_properties_sets_properties() {
        let props = Expression::map_literal(vec![
            (Cow::Borrowed("since"), Expression::from(2020_i32)),
        ]);
        let d = rel("KNOWS").with_properties(props);
        assert!(d.properties().is_some());
    }

    #[test]
    fn min_sets_minimum_length() {
        let d = rel("KNOWS").min(2);
        let Some(RelationshipLength::Range { min, max }) = d.length() else {
            unreachable!("Expected Range");
        };
        assert_eq!(*min, Some(2));
        assert_eq!(*max, None);
    }

    #[test]
    fn max_sets_maximum_length() {
        let d = rel("KNOWS").max(5);
        let Some(RelationshipLength::Range { min, max }) = d.length() else {
            unreachable!("Expected Range");
        };
        assert_eq!(*min, None);
        assert_eq!(*max, Some(5));
    }

    #[test]
    fn min_and_max_combine() {
        let d = rel("KNOWS").min(1).max(3);
        let Some(RelationshipLength::Range { min, max }) = d.length() else {
            unreachable!("Expected Range");
        };
        assert_eq!(*min, Some(1));
        assert_eq!(*max, Some(3));
    }

    #[test]
    fn unbounded_sets_unbounded_length() {
        let d = rel("KNOWS").unbounded();
        assert_eq!(d.length(), Some(&RelationshipLength::Unbounded));
    }

    #[test]
    fn exact_sets_exact_length() {
        let d = rel("KNOWS").exact(3);
        assert_eq!(d.length(), Some(&RelationshipLength::Exact(3)));
    }

    // --- RelationshipBuilder and Relationship ---

    #[test]
    fn to_creates_outgoing_relationship() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        assert_eq!(r.direction(), Direction::Outgoing);
        assert_eq!(r.left().symbolic_name(), Some("a"));
        assert_eq!(r.right().symbolic_name(), Some("b"));
        assert_eq!(r.details().types()[0], "KNOWS");
    }

    #[test]
    fn from_creates_incoming_relationship() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).from(b);
        assert_eq!(r.direction(), Direction::Incoming);
    }

    #[test]
    fn between_creates_undirected_relationship() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).between(b);
        assert_eq!(r.direction(), Direction::Undirected);
    }

    #[test]
    fn relationship_with_named_detail() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("ACTED_IN").named("r")).to(b);
        assert_eq!(r.details().symbolic_name(), Some("r"));
    }

    #[test]
    fn relationship_with_properties() {
        let a = person("a");
        let b = person("b");
        let props = Expression::map_literal(vec![
            (Cow::Borrowed("role"), Expression::from("Neo")),
        ]);
        let r = a.rel(rel("ACTED_IN").with_properties(props)).to(b);
        assert!(r.details().properties().is_some());
    }

    #[test]
    fn relationship_with_variable_length() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS").min(1).max(3)).to(b);
        let Some(RelationshipLength::Range { min, max }) = r.details().length() else {
            unreachable!("Expected Range");
        };
        assert_eq!(*min, Some(1));
        assert_eq!(*max, Some(3));
    }

    // --- inverse ---

    #[test]
    fn inverse_flips_outgoing_to_incoming() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b).inverse();
        assert_eq!(r.direction(), Direction::Incoming);
        assert_eq!(r.left().symbolic_name(), Some("b"));
        assert_eq!(r.right().symbolic_name(), Some("a"));
    }

    #[test]
    fn inverse_flips_incoming_to_outgoing() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).from(b).inverse();
        assert_eq!(r.direction(), Direction::Outgoing);
    }

    #[test]
    fn inverse_keeps_undirected() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).between(b).inverse();
        assert_eq!(r.direction(), Direction::Undirected);
    }

    // --- property access ---

    #[test]
    fn property_creates_property_access() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("ACTED_IN").named("r")).to(b);
        let prop = r.property("role");
        assert_eq!(prop.names()[0], "role");
    }

    // --- From<Relationship> for Expression ---

    #[test]
    fn from_relationship_produces_expression() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let expr = Expression::from(r);
        assert!(matches!(
            expr.inner(),
            crate::types::expression::ExpressionInner::Relationship(_)
        ));
    }

    // --- Free functions ---

    #[test]
    fn rel_free_function() {
        let d = rel("TYPE");
        assert_eq!(d.types()[0], "TYPE");
    }

    #[test]
    fn untyped_rel_free_function() {
        let d = untyped_rel();
        assert!(d.types().is_empty());
    }
}
