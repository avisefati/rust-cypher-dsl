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
    /// Optional quantifier for quantified relationships: `-[:R]->{1,5}`.
    pub(crate) quantifier: Option<crate::types::pattern::Quantifier>,
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
            quantifier: None,
        }
    }

    /// Creates a relationship detail with no types (anonymous).
    pub const fn untyped() -> Self {
        Self {
            types: Vec::new(),
            symbolic_name: None,
            length: None,
            properties: None,
            quantifier: None,
        }
    }

    /// Sets a quantifier for quantified relationships: `-[:R]->{1,5}`.
    #[must_use]
    pub const fn quantified(mut self, quantifier: crate::types::pattern::Quantifier) -> Self {
        self.quantifier = Some(quantifier);
        self
    }

    /// Shorthand for `.quantified(Quantifier::Exact(n))`: `-[:R]->{n}`.
    #[must_use]
    pub const fn times(self, n: u32) -> Self {
        self.quantified(crate::types::pattern::Quantifier::Exact(n))
    }

    /// Shorthand for `.quantified(Quantifier::Plus)`: `-[:R]->+`.
    #[must_use]
    pub const fn plus(self) -> Self {
        self.quantified(crate::types::pattern::Quantifier::Plus)
    }

    /// Shorthand for `.quantified(Quantifier::Star)`: `-[:R]->*`.
    #[must_use]
    pub const fn star(self) -> Self {
        self.quantified(crate::types::pattern::Quantifier::Star)
    }

    /// Returns the quantifier, if any.
    pub const fn quantifier(&self) -> Option<&crate::types::pattern::Quantifier> {
        self.quantifier.as_ref()
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

// --- From<&str> for RelationshipDetail ---

impl From<&'static str> for RelationshipDetail {
    fn from(type_name: &'static str) -> Self {
        Self::new(type_name)
    }
}

impl From<String> for RelationshipDetail {
    fn from(type_name: String) -> Self {
        Self::new(type_name)
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

    /// Applies an exact quantifier to this relationship: `-[:R]->{n}`.
    #[must_use]
    pub fn times(mut self, n: u32) -> Self {
        self.details = self.details.times(n);
        self
    }

    /// Applies a one-or-more quantifier: `-[:R]->+`.
    #[must_use]
    pub fn plus(mut self) -> Self {
        self.details = self.details.plus();
        self
    }

    /// Applies a zero-or-more quantifier: `-[:R]->*`.
    #[must_use]
    pub fn star(mut self) -> Self {
        self.details = self.details.star();
        self
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

    // --- Typed chaining shortcuts ---

    /// Chains an outgoing typed relationship from the right node: `...-[:R]->(target)`.
    pub fn to(self, detail: impl Into<RelationshipDetail>, target: Node) -> RelationshipChain {
        self.rel(detail.into()).to(target)
    }

    /// Chains an incoming typed relationship from the right node: `...<-[:R]-(source)`.
    pub fn from(self, detail: impl Into<RelationshipDetail>, source: Node) -> RelationshipChain {
        self.rel(detail.into()).from(source)
    }

    /// Chains an undirected typed relationship from the right node: `...-[:R]-(other)`.
    pub fn linked(self, detail: impl Into<RelationshipDetail>, other: Node) -> RelationshipChain {
        self.rel(detail.into()).between(other)
    }

    // --- Untyped chaining shortcuts ---

    /// Chains an untyped outgoing relationship: `...-->(target)`.
    pub fn link_to(self, target: Node) -> RelationshipChain {
        self.rel(RelationshipDetail::untyped()).to(target)
    }

    /// Chains an untyped incoming relationship: `...<--(source)`.
    pub fn link_from(self, source: Node) -> RelationshipChain {
        self.rel(RelationshipDetail::untyped()).from(source)
    }

    /// Chains an untyped undirected relationship: `...--(other)`.
    pub fn link(self, other: Node) -> RelationshipChain {
        self.rel(RelationshipDetail::untyped()).between(other)
    }

    /// Starts building a chained relationship from the right node.
    ///
    /// Creates a [`RelationshipChainBuilder`] that will extend this
    /// relationship into a multi-hop chain.
    pub fn rel(self, detail: RelationshipDetail) -> RelationshipChainBuilder {
        let chain = RelationshipChain {
            start: self.left,
            links: vec![ChainLink {
                details: self.details,
                direction: self.direction,
                target: self.right,
            }],
        };
        RelationshipChainBuilder { chain, pending: detail }
    }
}

// --- Relationship chaining ---

/// A single link in a relationship chain.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainLink {
    /// Relationship metadata for this hop.
    pub(crate) details: RelationshipDetail,
    /// Direction of this hop.
    pub(crate) direction: Direction,
    /// The target node of this hop.
    pub(crate) target: Node,
}

/// A multi-hop relationship chain: `(a)-[:R1]->(b)-[:R2]->(c)`.
///
/// Stores the starting node and a sequence of links, each containing
/// the relationship details, direction, and target node.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipChain {
    /// The starting node of the chain.
    pub(crate) start: Node,
    /// The sequence of hops.
    pub(crate) links: Vec<ChainLink>,
}

/// An intermediate builder for extending a relationship chain.
///
/// Created by calling `.rel()` on a [`Relationship`] or [`RelationshipChain`].
/// Awaits a terminal method (`.to()`, `.from()`, `.between()`) to add the
/// next hop.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipChainBuilder {
    /// The chain built so far.
    chain: RelationshipChain,
    /// The pending relationship detail for the next hop.
    pending: RelationshipDetail,
}

impl RelationshipChainBuilder {
    /// Completes the next hop as outgoing and returns the chain.
    pub fn to(mut self, target: Node) -> RelationshipChain {
        self.chain.links.push(ChainLink {
            details: self.pending,
            direction: Direction::Outgoing,
            target,
        });
        self.chain
    }

    /// Completes the next hop as incoming and returns the chain.
    pub fn from(mut self, target: Node) -> RelationshipChain {
        self.chain.links.push(ChainLink {
            details: self.pending,
            direction: Direction::Incoming,
            target,
        });
        self.chain
    }

    /// Completes the next hop as undirected and returns the chain.
    pub fn between(mut self, target: Node) -> RelationshipChain {
        self.chain.links.push(ChainLink {
            details: self.pending,
            direction: Direction::Undirected,
            target,
        });
        self.chain
    }
}

impl RelationshipChain {
    // --- Typed chaining shortcuts ---

    /// Chains an outgoing typed relationship from the last node: `...-[:R]->(target)`.
    #[must_use]
    pub fn to(self, detail: impl Into<RelationshipDetail>, target: Node) -> Self {
        self.rel(detail.into()).to(target)
    }

    /// Chains an incoming typed relationship from the last node: `...<-[:R]-(source)`.
    #[must_use]
    pub fn from(self, detail: impl Into<RelationshipDetail>, source: Node) -> Self {
        self.rel(detail.into()).from(source)
    }

    /// Chains an undirected typed relationship from the last node: `...-[:R]-(other)`.
    #[must_use]
    pub fn linked(self, detail: impl Into<RelationshipDetail>, other: Node) -> Self {
        self.rel(detail.into()).between(other)
    }

    // --- Untyped chaining shortcuts ---

    /// Chains an untyped outgoing relationship: `...-->(target)`.
    #[must_use]
    pub fn link_to(self, target: Node) -> Self {
        self.rel(RelationshipDetail::untyped()).to(target)
    }

    /// Chains an untyped incoming relationship: `...<--(source)`.
    #[must_use]
    pub fn link_from(self, source: Node) -> Self {
        self.rel(RelationshipDetail::untyped()).from(source)
    }

    /// Chains an untyped undirected relationship: `...--(other)`.
    #[must_use]
    pub fn link(self, other: Node) -> Self {
        self.rel(RelationshipDetail::untyped()).between(other)
    }

    /// Starts building another hop from the last node in the chain.
    pub const fn rel(self, detail: RelationshipDetail) -> RelationshipChainBuilder {
        RelationshipChainBuilder {
            chain: self,
            pending: detail,
        }
    }

    /// Applies an exact quantifier to the last relationship in the chain.
    #[must_use]
    pub fn times(mut self, n: u32) -> Self {
        if let Some(last) = self.links.last_mut() {
            last.details = last.details.clone().times(n);
        }
        self
    }

    /// Applies a one-or-more quantifier to the last relationship in the chain.
    #[must_use]
    pub fn plus(mut self) -> Self {
        if let Some(last) = self.links.last_mut() {
            last.details = last.details.clone().plus();
        }
        self
    }

    /// Applies a zero-or-more quantifier to the last relationship in the chain.
    #[must_use]
    pub fn star(mut self) -> Self {
        if let Some(last) = self.links.last_mut() {
            last.details = last.details.clone().star();
        }
        self
    }

    /// Returns the starting node of the chain.
    pub const fn start(&self) -> &Node {
        &self.start
    }

    /// Returns the chain links.
    pub fn links(&self) -> &[ChainLink] {
        &self.links
    }

    /// Returns the last node in the chain (the target of the last link).
    pub fn end(&self) -> Option<&Node> {
        self.links.last().map(|link| &link.target)
    }

    /// Returns the number of hops (relationships) in the chain.
    pub const fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns `true` if the chain has no hops.
    pub const fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

impl ChainLink {
    /// Returns the relationship details for this hop.
    pub const fn details(&self) -> &RelationshipDetail {
        &self.details
    }

    /// Returns the direction of this hop.
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the target node of this hop.
    pub const fn target(&self) -> &Node {
        &self.target
    }
}

// --- Operator overloading (>> / <<) ---

/// Intermediate type produced by `Node >> RelationshipDetail`.
///
/// Complete the outgoing relationship with `>> target_node`.
#[derive(Debug, Clone, PartialEq)]
pub struct OutgoingHalf {
    /// The left-hand node.
    left: Node,
    /// The relationship metadata.
    details: RelationshipDetail,
}

/// Intermediate type produced by `Node << RelationshipDetail`.
///
/// Complete the incoming relationship with `<< source_node`.
#[derive(Debug, Clone, PartialEq)]
pub struct IncomingHalf {
    /// The left-hand node.
    left: Node,
    /// The relationship metadata.
    details: RelationshipDetail,
}

/// Intermediate type produced by `Relationship >> RelationshipDetail`
/// or `RelationshipChain >> RelationshipDetail`.
///
/// Complete the next outgoing hop with `>> target_node`.
#[derive(Debug, Clone, PartialEq)]
pub struct OutgoingChainHalf {
    /// The chain builder so far.
    builder: RelationshipChainBuilder,
}

/// Intermediate type produced by `Relationship << RelationshipDetail`
/// or `RelationshipChain << RelationshipDetail`.
///
/// Complete the next incoming hop with `<< source_node`.
#[derive(Debug, Clone, PartialEq)]
pub struct IncomingChainHalf {
    /// The chain builder so far.
    builder: RelationshipChainBuilder,
}

// Node >> RelationshipDetail → OutgoingHalf
impl std::ops::Shr<RelationshipDetail> for Node {
    type Output = OutgoingHalf;

    fn shr(self, rhs: RelationshipDetail) -> Self::Output {
        OutgoingHalf {
            left: self,
            details: rhs,
        }
    }
}

// OutgoingHalf >> Node → Relationship
impl std::ops::Shr<Node> for OutgoingHalf {
    type Output = Relationship;

    fn shr(self, rhs: Node) -> Self::Output {
        Relationship {
            left: self.left,
            right: rhs,
            direction: Direction::Outgoing,
            details: self.details,
        }
    }
}

// Node << RelationshipDetail → IncomingHalf
impl std::ops::Shl<RelationshipDetail> for Node {
    type Output = IncomingHalf;

    fn shl(self, rhs: RelationshipDetail) -> Self::Output {
        IncomingHalf {
            left: self,
            details: rhs,
        }
    }
}

// IncomingHalf << Node → Relationship
impl std::ops::Shl<Node> for IncomingHalf {
    type Output = Relationship;

    fn shl(self, rhs: Node) -> Self::Output {
        Relationship {
            left: self.left,
            right: rhs,
            direction: Direction::Incoming,
            details: self.details,
        }
    }
}

// Relationship >> RelationshipDetail → OutgoingChainHalf
impl std::ops::Shr<RelationshipDetail> for Relationship {
    type Output = OutgoingChainHalf;

    fn shr(self, rhs: RelationshipDetail) -> Self::Output {
        OutgoingChainHalf {
            builder: self.rel(rhs),
        }
    }
}

// OutgoingChainHalf >> Node → RelationshipChain
impl std::ops::Shr<Node> for OutgoingChainHalf {
    type Output = RelationshipChain;

    fn shr(self, rhs: Node) -> Self::Output {
        self.builder.to(rhs)
    }
}

// Relationship << RelationshipDetail → IncomingChainHalf
impl std::ops::Shl<RelationshipDetail> for Relationship {
    type Output = IncomingChainHalf;

    fn shl(self, rhs: RelationshipDetail) -> Self::Output {
        IncomingChainHalf {
            builder: self.rel(rhs),
        }
    }
}

// IncomingChainHalf << Node → RelationshipChain
impl std::ops::Shl<Node> for IncomingChainHalf {
    type Output = RelationshipChain;

    fn shl(self, rhs: Node) -> Self::Output {
        self.builder.from(rhs)
    }
}

// RelationshipChain >> RelationshipDetail → OutgoingChainHalf
impl std::ops::Shr<RelationshipDetail> for RelationshipChain {
    type Output = OutgoingChainHalf;

    fn shr(self, rhs: RelationshipDetail) -> Self::Output {
        OutgoingChainHalf {
            builder: self.rel(rhs),
        }
    }
}

// OutgoingChainHalf from chain >> Node → RelationshipChain
// (already handled by OutgoingChainHalf >> Node above)

// RelationshipChain << RelationshipDetail → IncomingChainHalf
impl std::ops::Shl<RelationshipDetail> for RelationshipChain {
    type Output = IncomingChainHalf;

    fn shl(self, rhs: RelationshipDetail) -> Self::Output {
        IncomingChainHalf {
            builder: self.rel(rhs),
        }
    }
}

// IncomingChainHalf from chain << Node → RelationshipChain
// (already handled by IncomingChainHalf << Node above)

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

    // --- Typed relationship shortcuts ---

    /// Creates an outgoing typed relationship: `(self)-[:R]->(target)`.
    ///
    /// The first argument accepts a `&str` or a pre-built [`RelationshipDetail`].
    ///
    /// ```text
    /// a.to("KNOWS", b)              // (a)-[:KNOWS]->(b)
    /// a.to(rel("KNOWS").named("r"), b)  // (a)-[r:KNOWS]->(b)
    /// ```
    pub fn to(&self, detail: impl Into<RelationshipDetail>, target: Self) -> Relationship {
        self.rel(detail.into()).to(target)
    }

    /// Creates an incoming typed relationship: `(self)<-[:R]-(source)`.
    ///
    /// The first argument accepts a `&str` or a pre-built [`RelationshipDetail`].
    ///
    /// ```text
    /// a.from("DIRECTED", m)  // (a)<-[:DIRECTED]-(m)
    /// ```
    pub fn from(&self, detail: impl Into<RelationshipDetail>, source: Self) -> Relationship {
        self.rel(detail.into()).from(source)
    }

    /// Creates an undirected typed relationship: `(self)-[:R]-(other)`.
    ///
    /// The first argument accepts a `&str` or a pre-built [`RelationshipDetail`].
    ///
    /// ```text
    /// a.linked("KNOWS", b)  // (a)-[:KNOWS]-(b)
    /// ```
    pub fn linked(&self, detail: impl Into<RelationshipDetail>, other: Self) -> Relationship {
        self.rel(detail.into()).between(other)
    }

    // --- Untyped relationship shortcuts ---

    /// Creates an untyped outgoing relationship: `(self)-->(target)`.
    pub fn link_to(&self, target: Self) -> Relationship {
        self.rel(RelationshipDetail::untyped()).to(target)
    }

    /// Creates an untyped incoming relationship: `(self)<--(source)`.
    pub fn link_from(&self, source: Self) -> Relationship {
        self.rel(RelationshipDetail::untyped()).from(source)
    }

    /// Creates an untyped undirected relationship: `(self)--(other)`.
    pub fn link(&self, other: Self) -> Relationship {
        self.rel(RelationshipDetail::untyped()).between(other)
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

    // --- Quantifier shorthand on RelationshipDetail ---

    #[test]
    fn times_sets_exact_quantifier() {
        use crate::types::pattern::Quantifier;
        let d = rel("KNOWS").times(3);
        assert_eq!(d.quantifier(), Some(&Quantifier::Exact(3)));
    }

    #[test]
    fn plus_sets_one_or_more_quantifier() {
        use crate::types::pattern::Quantifier;
        let d = rel("KNOWS").plus();
        assert_eq!(d.quantifier(), Some(&Quantifier::Plus));
    }

    #[test]
    fn star_sets_zero_or_more_quantifier() {
        use crate::types::pattern::Quantifier;
        let d = rel("KNOWS").star();
        assert_eq!(d.quantifier(), Some(&Quantifier::Star));
    }

    // --- Quantifier shorthand on Relationship ---

    #[test]
    fn relationship_times_sets_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b).times(2);
        assert_eq!(r.details().quantifier(), Some(&Quantifier::Exact(2)));
    }

    #[test]
    fn relationship_plus_sets_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b).plus();
        assert_eq!(r.details().quantifier(), Some(&Quantifier::Plus));
    }

    #[test]
    fn relationship_star_sets_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b).star();
        assert_eq!(r.details().quantifier(), Some(&Quantifier::Star));
    }

    // --- Quantifier shorthand on RelationshipChain ---

    #[test]
    fn chain_times_sets_last_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c).times(3);
        assert!(chain.links()[0].details().quantifier().is_none());
        assert_eq!(chain.links()[1].details().quantifier(), Some(&Quantifier::Exact(3)));
    }

    #[test]
    fn chain_plus_sets_last_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c).plus();
        assert!(chain.links()[0].details().quantifier().is_none());
        assert_eq!(chain.links()[1].details().quantifier(), Some(&Quantifier::Plus));
    }

    #[test]
    fn chain_star_sets_last_quantifier() {
        use crate::types::pattern::Quantifier;
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c).star();
        assert!(chain.links()[0].details().quantifier().is_none());
        assert_eq!(chain.links()[1].details().quantifier(), Some(&Quantifier::Star));
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

    // --- From<&str> for RelationshipDetail ---

    #[test]
    fn from_str_creates_typed_detail() {
        let d: RelationshipDetail = "KNOWS".into();
        assert_eq!(d.types()[0], "KNOWS");
    }

    #[test]
    fn from_string_creates_typed_detail() {
        let d: RelationshipDetail = String::from("ACTED_IN").into();
        assert_eq!(d.types()[0], "ACTED_IN");
    }

    // --- Node shorthand: typed ---

    #[test]
    fn node_to_str_creates_outgoing() {
        let a = person("a");
        let b = person("b");
        let r = a.to("KNOWS", b);
        assert_eq!(r.direction(), Direction::Outgoing);
        assert_eq!(r.details().types()[0], "KNOWS");
        assert_eq!(r.left().symbolic_name(), Some("a"));
        assert_eq!(r.right().symbolic_name(), Some("b"));
    }

    #[test]
    fn node_to_prebuilt_creates_outgoing_with_detail() {
        let a = person("a");
        let b = person("b");
        let r = a.to(rel("KNOWS").named("r"), b);
        assert_eq!(r.direction(), Direction::Outgoing);
        assert_eq!(r.details().types()[0], "KNOWS");
        assert_eq!(r.details().symbolic_name(), Some("r"));
    }

    #[test]
    fn node_from_str_creates_incoming() {
        let a = person("a");
        let b = person("b");
        let r = a.from("DIRECTED", b);
        assert_eq!(r.direction(), Direction::Incoming);
        assert_eq!(r.details().types()[0], "DIRECTED");
    }

    #[test]
    fn node_linked_str_creates_undirected() {
        let a = person("a");
        let b = person("b");
        let r = a.linked("KNOWS", b);
        assert_eq!(r.direction(), Direction::Undirected);
        assert_eq!(r.details().types()[0], "KNOWS");
    }

    // --- Node shorthand: untyped ---

    #[test]
    fn node_link_to_creates_untyped_outgoing() {
        let a = person("a");
        let b = person("b");
        let r = a.link_to(b);
        assert_eq!(r.direction(), Direction::Outgoing);
        assert!(r.details().types().is_empty());
    }

    #[test]
    fn node_link_from_creates_untyped_incoming() {
        let a = person("a");
        let b = person("b");
        let r = a.link_from(b);
        assert_eq!(r.direction(), Direction::Incoming);
        assert!(r.details().types().is_empty());
    }

    #[test]
    fn node_link_creates_untyped_undirected() {
        let a = person("a");
        let b = person("b");
        let r = a.link(b);
        assert_eq!(r.direction(), Direction::Undirected);
        assert!(r.details().types().is_empty());
    }

    // --- Relationship chaining shortcuts ---

    #[test]
    fn relationship_to_chains_outgoing() {
        // a.to("R1", b).to("R2", c) == a-[:R1]->b-[:R2]->c
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.to("R1", b).to("R2", c);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].details().types()[0], "R1");
        assert_eq!(chain.links()[0].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[1].details().types()[0], "R2");
        assert_eq!(chain.links()[1].direction(), Direction::Outgoing);
    }

    #[test]
    fn relationship_from_chains_incoming() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.to("R1", b).from("R2", c);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[1].direction(), Direction::Incoming);
    }

    #[test]
    fn relationship_linked_chains_undirected() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.to("R1", b).linked("R2", c);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[1].direction(), Direction::Undirected);
    }

    #[test]
    fn relationship_link_to_chains_untyped_outgoing() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.to("R1", b).link_to(c);
        assert_eq!(chain.len(), 2);
        assert!(chain.links()[1].details().types().is_empty());
        assert_eq!(chain.links()[1].direction(), Direction::Outgoing);
    }

    // --- RelationshipChain chaining shortcuts ---

    #[test]
    fn chain_to_extends_chain() {
        // a.to("R1", b).to("R2", c).to("R3", d)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = a.to("R1", b).to("R2", c).to("R3", d);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.links()[2].details().types()[0], "R3");
        assert_eq!(chain.links()[2].direction(), Direction::Outgoing);
    }

    #[test]
    fn chain_from_extends_chain() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = a.to("R1", b).to("R2", c).from("R3", d);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.links()[2].direction(), Direction::Incoming);
    }

    #[test]
    fn chain_linked_extends_chain() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = a.to("R1", b).to("R2", c).linked("R3", d);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.links()[2].direction(), Direction::Undirected);
    }

    #[test]
    fn chain_link_to_extends_chain() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = a.to("R1", b).to("R2", c).link_to(d);
        assert_eq!(chain.len(), 3);
        assert!(chain.links()[2].details().types().is_empty());
        assert_eq!(chain.links()[2].direction(), Direction::Outgoing);
    }

    #[test]
    fn chain_with_prebuilt_detail() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let r1 = rel("KNOWS").named("r");
        let chain = a.to(r1, b).to("FOLLOWS", c);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].details().symbolic_name(), Some("r"));
        assert_eq!(chain.links()[0].details().types()[0], "KNOWS");
        assert_eq!(chain.links()[1].details().types()[0], "FOLLOWS");
    }

    // --- Relationship chaining ---

    #[test]
    fn two_hop_chain() {
        // (a)-[:R1]->(b)-[:R2]->(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c);
        assert_eq!(chain.start().symbolic_name(), Some("a"));
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].details().types()[0], "R1");
        assert_eq!(chain.links()[0].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[0].target().symbolic_name(), Some("b"));
        assert_eq!(chain.links()[1].details().types()[0], "R2");
        assert_eq!(chain.links()[1].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[1].target().symbolic_name(), Some("c"));
        assert_eq!(chain.end().and_then(Node::symbolic_name), Some("c"));
    }

    #[test]
    fn three_hop_chain() {
        // (a)-[:R1]->(b)-[:R2]->(c)-[:R3]->(d)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = a
            .rel(rel("R1"))
            .to(b)
            .rel(rel("R2"))
            .to(c)
            .rel(rel("R3"))
            .to(d);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.start().symbolic_name(), Some("a"));
        assert_eq!(chain.end().and_then(Node::symbolic_name), Some("d"));
    }

    #[test]
    fn mixed_direction_chain() {
        // (a)-[:R1]->(b)<-[:R2]-(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).from(c);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[1].direction(), Direction::Incoming);
    }

    #[test]
    fn chain_with_undirected() {
        // (a)-[:R1]-(b)-[:R2]->(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).between(b).rel(rel("R2")).to(c);
        assert_eq!(chain.links()[0].direction(), Direction::Undirected);
        assert_eq!(chain.links()[1].direction(), Direction::Outgoing);
    }

    #[test]
    fn chain_is_empty_never_true_after_construction() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c);
        assert!(!chain.is_empty());
    }

    // --- Operator overloading (>> / <<) ---

    #[test]
    fn shr_creates_outgoing_relationship() {
        let a = person("a");
        let b = person("b");
        let r = a >> rel("KNOWS") >> b;
        assert_eq!(r.direction(), Direction::Outgoing);
        assert_eq!(r.left().symbolic_name(), Some("a"));
        assert_eq!(r.right().symbolic_name(), Some("b"));
        assert_eq!(r.details().types()[0], "KNOWS");
    }

    #[test]
    fn shl_creates_incoming_relationship() {
        let a = person("a");
        let b = person("b");
        let r = a << rel("DIRECTED") << b;
        assert_eq!(r.direction(), Direction::Incoming);
        assert_eq!(r.left().symbolic_name(), Some("a"));
        assert_eq!(r.right().symbolic_name(), Some("b"));
        assert_eq!(r.details().types()[0], "DIRECTED");
    }

    #[test]
    fn shr_equals_method_syntax() {
        let a = person("a");
        let b = person("b");
        let via_op = a.clone() >> rel("KNOWS") >> b.clone();
        let via_method = a.rel(rel("KNOWS")).to(b);
        assert_eq!(via_op.direction(), via_method.direction());
        assert_eq!(
            via_op.left().symbolic_name(),
            via_method.left().symbolic_name()
        );
        assert_eq!(
            via_op.right().symbolic_name(),
            via_method.right().symbolic_name()
        );
        assert_eq!(via_op.details().types(), via_method.details().types());
    }

    #[test]
    fn shl_equals_method_syntax() {
        let a = person("a");
        let b = person("b");
        let via_op = a.clone() << rel("DIRECTED") << b.clone();
        let via_method = a.rel(rel("DIRECTED")).from(b);
        assert_eq!(via_op.direction(), via_method.direction());
        assert_eq!(
            via_op.left().symbolic_name(),
            via_method.left().symbolic_name()
        );
        assert_eq!(
            via_op.right().symbolic_name(),
            via_method.right().symbolic_name()
        );
    }

    #[test]
    fn shr_with_pre_built_detail() {
        let acted_in = rel("ACTED_IN").named("r").min(1).max(3);
        let a = person("a");
        let m = node("Movie").named("m");
        let r = a >> acted_in >> m;
        assert_eq!(r.details().symbolic_name(), Some("r"));
        assert_eq!(r.details().types()[0], "ACTED_IN");
        let Some(RelationshipLength::Range { min, max }) = r.details().length() else {
            unreachable!("Expected Range");
        };
        assert_eq!(*min, Some(1));
        assert_eq!(*max, Some(3));
    }

    #[test]
    fn shr_chain_two_hops() {
        // (a)-[:R1]->(b)-[:R2]->(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = (a >> rel("R1") >> b) >> rel("R2") >> c;
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.start().symbolic_name(), Some("a"));
        assert_eq!(chain.links()[0].details().types()[0], "R1");
        assert_eq!(chain.links()[0].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[0].target().symbolic_name(), Some("b"));
        assert_eq!(chain.links()[1].details().types()[0], "R2");
        assert_eq!(chain.links()[1].direction(), Direction::Outgoing);
        assert_eq!(chain.end().and_then(Node::symbolic_name), Some("c"));
    }

    #[test]
    fn shl_chain_two_hops() {
        // (a)<-[:R1]-(b)<-[:R2]-(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = (a << rel("R1") << b) << rel("R2") << c;
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].direction(), Direction::Incoming);
        assert_eq!(chain.links()[1].direction(), Direction::Incoming);
    }

    #[test]
    fn mixed_shr_shl_chain() {
        // (a)-[:R1]->(b)<-[:R2]-(c)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = (a >> rel("R1") >> b) << rel("R2") << c;
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.links()[0].direction(), Direction::Outgoing);
        assert_eq!(chain.links()[1].direction(), Direction::Incoming);
    }

    #[test]
    fn three_hop_chain_via_operators() {
        // (a)-[:R1]->(b)-[:R2]->(c)-[:R3]->(d)
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let d = person("d");
        let chain = ((a >> rel("R1") >> b) >> rel("R2") >> c) >> rel("R3") >> d;
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.start().symbolic_name(), Some("a"));
        assert_eq!(chain.end().and_then(Node::symbolic_name), Some("d"));
    }
}
