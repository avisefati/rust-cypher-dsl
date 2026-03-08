//! Pattern types: `Pattern`, `PatternElement`, `NamedPath`, and `IntoPattern`.
//!
//! Patterns represent Cypher MATCH patterns such as `(a)-[:KNOWS]->(b)`,
//! comma-separated multi-element patterns, and named paths like
//! `p = (a)-[:KNOWS]->(b)`.

use std::borrow::Cow;

use super::node::Node;
use super::relationship::{Relationship, RelationshipChain};

/// A Cypher pattern consisting of one or more pattern elements.
///
/// Patterns appear in MATCH clauses and can contain multiple
/// comma-separated elements. For example:
///
/// ```text
/// MATCH (a:Person), (a)-[:KNOWS]->(b)
/// ```
///
/// corresponds to a `Pattern` with two elements.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    /// The pattern elements.
    pub(crate) elements: Vec<PatternElement>,
}

/// A single element within a pattern.
///
/// Each variant represents a different kind of pattern element
/// that can appear in a Cypher MATCH clause.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternElement {
    /// A single node pattern: `(n:Person)`.
    Node(Node),
    /// A relationship between two nodes: `(a)-[:KNOWS]->(b)`.
    Relationship(Relationship),
    /// A multi-hop chain: `(a)-[:R1]->(b)-[:R2]->(c)`.
    Chain(RelationshipChain),
    /// A named path: `p = (a)-[:KNOWS]->(b)`.
    NamedPath(NamedPath),
    /// A quantified path pattern: `((a)-[:R]->(b)){1,3}`.
    QuantifiedPath(QuantifiedPath),
    /// A pattern with a path selector: `SHORTEST 1 (pattern)`.
    SelectedPath(PathSelector, Box<Self>),
}

// ---------------------------------------------------------------------------
// Quantified Path Patterns (Task 9.1)
// ---------------------------------------------------------------------------

/// A quantifier for path patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quantifier {
    /// `*` - zero or more repetitions.
    Star,
    /// `+` - one or more repetitions.
    Plus,
    /// `{n}` - exactly n repetitions.
    Exact(u32),
    /// `{min, max}` - range of repetitions.
    Range {
        /// Minimum repetitions (None = no lower bound).
        min: Option<u32>,
        /// Maximum repetitions (None = no upper bound).
        max: Option<u32>,
    },
}

/// A quantified path pattern: `(pattern){quantifier}`.
///
/// Represents repeated graph patterns such as `((a)-[:R]->(b))+`
/// or `((a)-[:R]->(b)){1,3}`.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantifiedPath {
    /// The inner pattern being quantified.
    pub(crate) pattern: Box<PatternElement>,
    /// The quantifier applied to the pattern.
    pub(crate) quantifier: Quantifier,
    /// Optional WHERE predicate inside the QPP.
    pub(crate) where_clause: Option<crate::types::expression::Expression>,
}

impl QuantifiedPath {
    /// Creates a quantified path from a pattern element and quantifier.
    pub fn new(pattern: impl Into<PatternElement>, quantifier: Quantifier) -> Self {
        Self {
            pattern: Box::new(pattern.into()),
            quantifier,
            where_clause: None,
        }
    }

    /// Adds a WHERE predicate to this quantified path.
    #[must_use]
    pub fn where_(mut self, predicate: impl Into<crate::types::expression::Expression>) -> Self {
        self.where_clause = Some(predicate.into());
        self
    }

    /// Returns the inner pattern.
    pub fn pattern(&self) -> &PatternElement {
        &self.pattern
    }

    /// Returns the quantifier.
    pub const fn quantifier(&self) -> &Quantifier {
        &self.quantifier
    }

    /// Returns the WHERE clause, if any.
    pub const fn where_clause(&self) -> Option<&crate::types::expression::Expression> {
        self.where_clause.as_ref()
    }
}

/// Builder for constructing a [`QuantifiedPath`].
///
/// Created by the [`quantified_path()`] free function.
#[derive(Debug, Clone)]
pub struct QuantifiedPathBuilder {
    pattern: PatternElement,
    where_clause: Option<crate::types::expression::Expression>,
}

impl QuantifiedPathBuilder {
    /// Applies a `*` (zero or more) quantifier.
    #[must_use]
    pub fn star(self) -> QuantifiedPath {
        self.build(Quantifier::Star)
    }

    /// Applies a `+` (one or more) quantifier.
    #[must_use]
    pub fn plus(self) -> QuantifiedPath {
        self.build(Quantifier::Plus)
    }

    /// Applies an exact `{n}` quantifier.
    #[must_use]
    pub fn exact(self, n: u32) -> QuantifiedPath {
        self.build(Quantifier::Exact(n))
    }

    /// Applies a `{min, max}` range quantifier.
    #[must_use]
    pub fn range(self, min: Option<u32>, max: Option<u32>) -> QuantifiedPath {
        self.build(Quantifier::Range { min, max })
    }

    /// Adds a WHERE predicate before choosing the quantifier.
    #[must_use]
    pub fn where_(mut self, predicate: impl Into<crate::types::expression::Expression>) -> Self {
        self.where_clause = Some(predicate.into());
        self
    }

    fn build(self, quantifier: Quantifier) -> QuantifiedPath {
        let mut qp = QuantifiedPath::new(self.pattern, quantifier);
        qp.where_clause = self.where_clause;
        qp
    }
}

// ---------------------------------------------------------------------------
// Path Selectors (Task 9.3)
// ---------------------------------------------------------------------------

/// A path selector that filters which paths are returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSelector {
    /// `SHORTEST k` - returns k shortest paths.
    Shortest(u32),
    /// `ALL SHORTEST` - returns all shortest paths.
    AllShortest,
    /// `ANY` - returns any single path.
    Any,
    /// `SHORTEST k GROUPS` - returns k shortest groups.
    ShortestGroups(u32),
}

/// A named path: `p = <pattern>`.
///
/// Named paths assign a variable name to a pattern element,
/// allowing the path to be referenced elsewhere in the query.
///
/// # Examples (conceptual)
///
/// ```text
/// p = (a)-[:KNOWS]->(b)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NamedPath {
    /// The path variable name (e.g., `p`).
    pub(crate) name: Cow<'static, str>,
    /// The pattern element this path is defined by.
    pub(crate) pattern: Box<PatternElement>,
}

/// A builder for constructing a [`NamedPath`].
///
/// Created by the [`path()`] free function. Use `.defined_by()`
/// to set the pattern element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedPathBuilder {
    /// The path variable name.
    name: Cow<'static, str>,
}

/// Trait for types that can be converted into a [`Pattern`].
///
/// Implemented for `Node`, `Relationship`, `RelationshipChain`,
/// `PatternElement`, `NamedPath`, `Vec<PatternElement>`, arrays,
/// and tuples.
pub trait IntoPattern {
    /// Converts this value into a `Pattern`.
    fn into_pattern(self) -> Pattern;
}

// --- Pattern ---

impl Pattern {
    /// Creates a pattern from a single element.
    pub fn new(element: impl Into<PatternElement>) -> Self {
        Self {
            elements: vec![element.into()],
        }
    }

    /// Creates a pattern from multiple elements.
    pub const fn from_elements(elements: Vec<PatternElement>) -> Self {
        Self { elements }
    }

    /// Returns the pattern elements.
    pub fn elements(&self) -> &[PatternElement] {
        &self.elements
    }

    /// Returns the number of elements in this pattern.
    pub const fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if this pattern has no elements.
    pub const fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Adds another element to this pattern (comma-separated in Cypher).
    #[must_use]
    pub fn and(mut self, element: impl Into<PatternElement>) -> Self {
        self.elements.push(element.into());
        self
    }
}

// --- NamedPath ---

impl NamedPath {
    /// Creates a named path.
    pub fn new(name: impl Into<Cow<'static, str>>, pattern: impl Into<PatternElement>) -> Self {
        Self {
            name: name.into(),
            pattern: Box::new(pattern.into()),
        }
    }

    /// Returns the path variable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the pattern element this path is defined by.
    pub fn pattern(&self) -> &PatternElement {
        &self.pattern
    }
}

// --- NamedPathBuilder ---

impl NamedPathBuilder {
    /// Creates a new builder with the given path name.
    pub(crate) fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { name: name.into() }
    }

    /// Defines the pattern element for this named path.
    pub fn defined_by(self, element: impl Into<PatternElement>) -> NamedPath {
        NamedPath {
            name: self.name,
            pattern: Box::new(element.into()),
        }
    }
}

// --- Into<PatternElement> conversions ---

impl From<Node> for PatternElement {
    fn from(node: Node) -> Self {
        Self::Node(node)
    }
}

impl From<Relationship> for PatternElement {
    fn from(rel: Relationship) -> Self {
        Self::Relationship(rel)
    }
}

impl From<RelationshipChain> for PatternElement {
    fn from(chain: RelationshipChain) -> Self {
        Self::Chain(chain)
    }
}

impl From<NamedPath> for PatternElement {
    fn from(path: NamedPath) -> Self {
        Self::NamedPath(path)
    }
}

// --- From<T> for Pattern (enables `impl Into<Pattern>`) ---

impl From<Node> for Pattern {
    fn from(node: Node) -> Self {
        node.into_pattern()
    }
}

impl From<Relationship> for Pattern {
    fn from(rel: Relationship) -> Self {
        rel.into_pattern()
    }
}

impl From<RelationshipChain> for Pattern {
    fn from(chain: RelationshipChain) -> Self {
        chain.into_pattern()
    }
}

impl From<NamedPath> for Pattern {
    fn from(path: NamedPath) -> Self {
        path.into_pattern()
    }
}

impl From<PatternElement> for Pattern {
    fn from(elem: PatternElement) -> Self {
        elem.into_pattern()
    }
}

// --- IntoPattern implementations ---

impl IntoPattern for Pattern {
    fn into_pattern(self) -> Pattern {
        self
    }
}

impl IntoPattern for PatternElement {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![self],
        }
    }
}

impl IntoPattern for Node {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![PatternElement::Node(self)],
        }
    }
}

impl IntoPattern for Relationship {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![PatternElement::Relationship(self)],
        }
    }
}

impl IntoPattern for RelationshipChain {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![PatternElement::Chain(self)],
        }
    }
}

impl IntoPattern for NamedPath {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![PatternElement::NamedPath(self)],
        }
    }
}

impl IntoPattern for Vec<PatternElement> {
    fn into_pattern(self) -> Pattern {
        Pattern { elements: self }
    }
}

impl<const N: usize> IntoPattern for [PatternElement; N] {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: self.into(),
        }
    }
}

// Tuple implementations for combining pattern elements.

impl<A, B> IntoPattern for (A, B)
where
    A: Into<PatternElement>,
    B: Into<PatternElement>,
{
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![self.0.into(), self.1.into()],
        }
    }
}

impl<A, B, C> IntoPattern for (A, B, C)
where
    A: Into<PatternElement>,
    B: Into<PatternElement>,
    C: Into<PatternElement>,
{
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![self.0.into(), self.1.into(), self.2.into()],
        }
    }
}

impl<A, B, C, D> IntoPattern for (A, B, C, D)
where
    A: Into<PatternElement>,
    B: Into<PatternElement>,
    C: Into<PatternElement>,
    D: Into<PatternElement>,
{
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![self.0.into(), self.1.into(), self.2.into(), self.3.into()],
        }
    }
}

impl From<QuantifiedPath> for PatternElement {
    fn from(qp: QuantifiedPath) -> Self {
        Self::QuantifiedPath(qp)
    }
}

impl From<QuantifiedPath> for Pattern {
    fn from(qp: QuantifiedPath) -> Self {
        qp.into_pattern()
    }
}

impl IntoPattern for QuantifiedPath {
    fn into_pattern(self) -> Pattern {
        Pattern {
            elements: vec![PatternElement::QuantifiedPath(self)],
        }
    }
}

// --- Free functions ---

/// Creates a [`NamedPathBuilder`] with the given path variable name.
///
/// Use `.defined_by()` to set the pattern element.
///
/// # Examples (conceptual)
///
/// ```text
/// let p = path("p").defined_by(a.rel(rel("KNOWS")).to(b));
/// ```
pub fn path(name: impl Into<Cow<'static, str>>) -> NamedPathBuilder {
    NamedPathBuilder::new(name)
}

/// Creates a [`QuantifiedPathBuilder`] from a pattern element.
///
/// Use `.star()`, `.plus()`, `.exact(n)`, or `.range(min, max)` to choose
/// the quantifier.
pub fn quantified_path(pattern: impl Into<PatternElement>) -> QuantifiedPathBuilder {
    QuantifiedPathBuilder {
        pattern: pattern.into(),
        where_clause: None,
    }
}

/// Creates a `SHORTEST k` path selector wrapping a pattern element.
pub fn shortest(k: u32, pattern: impl Into<PatternElement>) -> PatternElement {
    PatternElement::SelectedPath(PathSelector::Shortest(k), Box::new(pattern.into()))
}

/// Creates an `ALL SHORTEST` path selector wrapping a pattern element.
pub fn all_shortest(pattern: impl Into<PatternElement>) -> PatternElement {
    PatternElement::SelectedPath(PathSelector::AllShortest, Box::new(pattern.into()))
}

/// Creates an `ANY` path selector wrapping a pattern element.
pub fn any_path(pattern: impl Into<PatternElement>) -> PatternElement {
    PatternElement::SelectedPath(PathSelector::Any, Box::new(pattern.into()))
}

/// Creates a `SHORTEST k GROUPS` path selector wrapping a pattern element.
pub fn shortest_groups(k: u32, pattern: impl Into<PatternElement>) -> PatternElement {
    PatternElement::SelectedPath(PathSelector::ShortestGroups(k), Box::new(pattern.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::node::node;
    use crate::types::relationship::rel;

    fn person(name: &'static str) -> Node {
        node("Person").named(name)
    }

    // --- Pattern creation ---

    #[test]
    fn single_node_pattern() {
        let p = Pattern::new(node("Person").named("n"));
        assert_eq!(p.len(), 1);
        assert!(!p.is_empty());
        assert!(matches!(&p.elements()[0], PatternElement::Node(_)));
    }

    #[test]
    fn single_relationship_pattern() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let p = Pattern::new(r);
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::Relationship(_)));
    }

    #[test]
    fn chain_pattern() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c);
        let p = Pattern::new(chain);
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::Chain(_)));
    }

    #[test]
    fn multi_element_pattern_via_and() {
        let alice = person("a");
        let bob = person("b");
        let knows = alice.rel(rel("KNOWS")).to(bob);
        let movie = node("Movie").named("m");
        let pat = Pattern::new(knows).and(movie);
        assert_eq!(pat.len(), 2);
        assert!(matches!(&pat.elements()[0], PatternElement::Relationship(_)));
        assert!(matches!(&pat.elements()[1], PatternElement::Node(_)));
    }

    #[test]
    fn from_elements_creates_pattern() {
        let elements = vec![
            PatternElement::Node(person("a")),
            PatternElement::Node(person("b")),
        ];
        let p = Pattern::from_elements(elements);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn empty_pattern() {
        let p = Pattern::from_elements(vec![]);
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    // --- NamedPath ---

    #[test]
    fn named_path_creation() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let np = NamedPath::new("p", r);
        assert_eq!(np.name(), "p");
        assert!(matches!(np.pattern(), PatternElement::Relationship(_)));
    }

    #[test]
    fn named_path_via_builder() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let np = path("p").defined_by(r);
        assert_eq!(np.name(), "p");
        assert!(matches!(np.pattern(), PatternElement::Relationship(_)));
    }

    #[test]
    fn named_path_with_chain() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c);
        let np = path("route").defined_by(chain);
        assert_eq!(np.name(), "route");
        assert!(matches!(np.pattern(), PatternElement::Chain(_)));
    }

    #[test]
    fn named_path_as_pattern_element() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let np = path("p").defined_by(r);
        let p = Pattern::new(np);
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::NamedPath(_)));
    }

    // --- IntoPattern ---

    #[test]
    fn node_into_pattern() {
        let p = person("a").into_pattern();
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::Node(_)));
    }

    #[test]
    fn relationship_into_pattern() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let p = r.into_pattern();
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::Relationship(_)));
    }

    #[test]
    fn chain_into_pattern() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let chain = a.rel(rel("R1")).to(b).rel(rel("R2")).to(c);
        let p = chain.into_pattern();
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::Chain(_)));
    }

    #[test]
    fn named_path_into_pattern() {
        let a = person("a");
        let b = person("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let np = path("p").defined_by(r);
        let p = np.into_pattern();
        assert_eq!(p.len(), 1);
        assert!(matches!(&p.elements()[0], PatternElement::NamedPath(_)));
    }

    #[test]
    fn vec_into_pattern() {
        let elements = vec![
            PatternElement::Node(person("a")),
            PatternElement::Node(person("b")),
        ];
        let p = elements.into_pattern();
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn array_into_pattern() {
        let p = [PatternElement::Node(person("a")), PatternElement::Node(person("b"))].into_pattern();
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn tuple_2_into_pattern() {
        let alice = person("a");
        let bob = person("b");
        let knows = alice.rel(rel("KNOWS")).to(bob);
        let charlie = person("c");
        let pat = (charlie, knows).into_pattern();
        assert_eq!(pat.len(), 2);
        assert!(matches!(&pat.elements()[0], PatternElement::Node(_)));
        assert!(matches!(&pat.elements()[1], PatternElement::Relationship(_)));
    }

    #[test]
    fn tuple_3_into_pattern() {
        let a = person("a");
        let b = person("b");
        let c = person("c");
        let p = (a, b, c).into_pattern();
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn tuple_4_into_pattern() {
        let pat = (person("a"), person("b"), person("c"), person("d")).into_pattern();
        assert_eq!(pat.len(), 4);
    }

    #[test]
    fn pattern_into_pattern_is_identity() {
        let original = Pattern::new(person("a"));
        let roundtripped = original.clone().into_pattern();
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn pattern_element_into_pattern() {
        let elem = PatternElement::Node(person("a"));
        let p = elem.into_pattern();
        assert_eq!(p.len(), 1);
    }
}
