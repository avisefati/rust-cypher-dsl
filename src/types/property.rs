//! `Property` type for accessing node/relationship properties.
//!
//! Properties represent property access on graph elements,
//! rendered as `container.name` or `container.a.b.c` for nested access.

use std::borrow::Cow;

use super::condition::Condition;
use super::expression::Expression;

/// A property access on a container expression.
///
/// Supports nested property chains: `n.address.city` is represented
/// as a single `Property` with `names: ["address", "city"]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The container expression (typically a Node or Relationship).
    pub(crate) container: Expression,
    /// The property name chain (supports nested access).
    pub(crate) names: Vec<Cow<'static, str>>,
}

impl Property {
    /// Creates a property access on a container expression.
    pub fn new(container: Expression, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            container,
            names: vec![name.into()],
        }
    }

    /// Adds a nested property access: `self.name`.
    #[must_use]
    pub fn property(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.names.push(name.into());
        self
    }

    /// Returns the container expression.
    pub const fn container(&self) -> &Expression {
        &self.container
    }

    /// Returns the property name chain.
    pub fn names(&self) -> &[Cow<'static, str>] {
        &self.names
    }

    // --- Comparison methods (delegate to Expression) ---

    /// Equality: `self = other`.
    #[must_use]
    pub fn eq(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).eq(other)
    }

    /// Inequality: `self <> other`.
    #[must_use]
    pub fn ne(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).ne(other)
    }

    /// Less than: `self < other`.
    #[must_use]
    pub fn lt(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).lt(other)
    }

    /// Less than or equal: `self <= other`.
    #[must_use]
    pub fn lte(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).lte(other)
    }

    /// Greater than: `self > other`.
    #[must_use]
    pub fn gt(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).gt(other)
    }

    /// Greater than or equal: `self >= other`.
    #[must_use]
    pub fn gte(self, other: impl Into<Expression>) -> Condition {
        Expression::from(self).gte(other)
    }

    // --- Convenience methods (delegate to Expression) ---

    /// Aliases this property: `self AS alias`.
    #[must_use]
    pub fn alias(self, alias: impl Into<Cow<'static, str>>) -> Expression {
        Expression::from(self).alias(alias)
    }
}

// ---------------------------------------------------------------------------
// Free functions for ergonomic property construction
// ---------------------------------------------------------------------------

/// Creates a property access from a symbolic name and property name.
///
/// `prop("n", "age")` is equivalent to `Property::new(Expression::symbolic_name("n"), "age")`.
pub fn prop(
    container: impl Into<Cow<'static, str>>,
    property_name: impl Into<Cow<'static, str>>,
) -> Property {
    Property::new(
        Expression::symbolic_name(container),
        property_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_single_property() {
        let prop = Property::new(Expression::symbolic_name("n"), "name");
        assert_eq!(prop.names().len(), 1);
        assert_eq!(prop.names()[0], "name");
    }

    #[test]
    fn property_chains_nested_access() {
        let prop = Property::new(Expression::symbolic_name("n"), "address").property("city");
        assert_eq!(prop.names().len(), 2);
        assert_eq!(prop.names()[0], "address");
        assert_eq!(prop.names()[1], "city");
    }

    #[test]
    fn container_returns_the_base_expression() {
        let container = Expression::symbolic_name("person");
        let prop = Property::new(container.clone(), "age");
        assert_eq!(*prop.container(), container);
    }

    #[test]
    fn triple_nested_property() {
        let prop = Property::new(Expression::symbolic_name("n"), "a")
            .property("b")
            .property("c");
        assert_eq!(prop.names().len(), 3);
        assert_eq!(
            prop.names().iter().map(AsRef::as_ref).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }
}
