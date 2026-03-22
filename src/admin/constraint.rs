//! Constraint management types: `CreateConstraint`, `DropConstraint`, `ConstraintType`, `ConstraintTarget`.

use std::borrow::Cow;

/// The type of constraint to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintType {
    /// `REQUIRE prop IS UNIQUE`
    Unique,
    /// `REQUIRE prop IS NOT NULL`
    Exists,
    /// `REQUIRE prop IS NODE KEY`
    NodeKey,
    /// `REQUIRE prop IS RELATIONSHIP KEY`
    RelationshipKey,
    /// `REQUIRE prop IS :: TYPE`
    PropertyType(Cow<'static, str>),
}

/// What the constraint is defined on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintTarget {
    /// Node constraint: `FOR (var:Label)`.
    Node {
        /// The variable name (e.g., "n").
        variable: Cow<'static, str>,
        /// The label.
        label: Cow<'static, str>,
    },
    /// Relationship constraint: `FOR ()-[var:TYPE]-()`.
    Relationship {
        /// The variable name (e.g., "r").
        variable: Cow<'static, str>,
        /// The relationship type.
        rel_type: Cow<'static, str>,
    },
}

/// A `CREATE CONSTRAINT` statement.
///
/// Renders as: `CREATE CONSTRAINT [name] [IF NOT EXISTS] FOR target REQUIRE specification`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateConstraint {
    /// Optional constraint name.
    pub(crate) name: Option<Cow<'static, str>>,
    /// Whether to include `IF NOT EXISTS`.
    pub(crate) if_not_exists: bool,
    /// What the constraint targets.
    pub(crate) target: ConstraintTarget,
    /// The properties involved.
    pub(crate) properties: Vec<Cow<'static, str>>,
    /// The constraint type.
    pub(crate) constraint_type: ConstraintType,
}

impl CreateConstraint {
    /// Creates a new `CreateConstraint`.
    pub(crate) const fn new(
        name: Option<Cow<'static, str>>,
        if_not_exists: bool,
        target: ConstraintTarget,
        properties: Vec<Cow<'static, str>>,
        constraint_type: ConstraintType,
    ) -> Self {
        Self {
            name,
            if_not_exists,
            target,
            properties,
            constraint_type,
        }
    }

    /// Returns the constraint name, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns whether `IF NOT EXISTS` is set.
    pub const fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    /// Returns the constraint target.
    pub const fn target(&self) -> &ConstraintTarget {
        &self.target
    }

    /// Returns the properties.
    pub fn properties(&self) -> &[Cow<'static, str>] {
        &self.properties
    }

    /// Returns the constraint type.
    pub const fn constraint_type(&self) -> &ConstraintType {
        &self.constraint_type
    }

    /// Returns the variable name from the target.
    pub fn variable(&self) -> &str {
        match &self.target {
            ConstraintTarget::Node { variable, .. }
            | ConstraintTarget::Relationship { variable, .. } => variable,
        }
    }
}

/// A `DROP CONSTRAINT` statement.
///
/// Renders as: `DROP CONSTRAINT name [IF EXISTS]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropConstraint {
    /// The constraint name to drop.
    pub(crate) name: Cow<'static, str>,
    /// Whether to include `IF EXISTS`.
    pub(crate) if_exists: bool,
}

impl DropConstraint {
    /// Creates a new `DropConstraint`.
    ///
    /// # Panics
    ///
    /// Panics if `name` is not a valid identifier.
    pub(crate) fn new(name: impl Into<Cow<'static, str>>, if_exists: bool) -> Self {
        let name = name.into();
        super::validate::assert_valid_identifier(&name, "constraint name");
        Self { name, if_exists }
    }

    /// Returns the constraint name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether `IF EXISTS` is set.
    pub const fn if_exists(&self) -> bool {
        self.if_exists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_unique_constraint() {
        let c = CreateConstraint::new(
            Some("unique_email".into()),
            false,
            ConstraintTarget::Node {
                variable: "n".into(),
                label: "Person".into(),
            },
            vec!["email".into()],
            ConstraintType::Unique,
        );
        assert_eq!(c.name(), Some("unique_email"));
        assert!(!c.if_not_exists());
        assert_eq!(c.variable(), "n");
        assert_eq!(c.properties().len(), 1);
    }

    #[test]
    fn create_existence_constraint() {
        let c = CreateConstraint::new(
            Some("exists_name".into()),
            true,
            ConstraintTarget::Node {
                variable: "n".into(),
                label: "Person".into(),
            },
            vec!["name".into()],
            ConstraintType::Exists,
        );
        assert!(c.if_not_exists());
        assert_eq!(c.constraint_type(), &ConstraintType::Exists);
    }

    #[test]
    fn create_node_key_constraint_composite() {
        let c = CreateConstraint::new(
            Some("person_key".into()),
            false,
            ConstraintTarget::Node {
                variable: "n".into(),
                label: "Person".into(),
            },
            vec!["id".into(), "name".into()],
            ConstraintType::NodeKey,
        );
        assert_eq!(c.properties().len(), 2);
        assert_eq!(c.constraint_type(), &ConstraintType::NodeKey);
    }

    #[test]
    fn create_relationship_key_constraint() {
        let c = CreateConstraint::new(
            Some("rel_key".into()),
            false,
            ConstraintTarget::Relationship {
                variable: "r".into(),
                rel_type: "REVIEWED".into(),
            },
            vec!["id".into()],
            ConstraintType::RelationshipKey,
        );
        assert!(matches!(c.target(), ConstraintTarget::Relationship { .. }));
    }

    #[test]
    fn create_property_type_constraint() {
        let c = CreateConstraint::new(
            Some("score_type".into()),
            false,
            ConstraintTarget::Relationship {
                variable: "r".into(),
                rel_type: "REVIEWED".into(),
            },
            vec!["score".into()],
            ConstraintType::PropertyType("FLOAT".into()),
        );
        assert_eq!(
            c.constraint_type(),
            &ConstraintType::PropertyType("FLOAT".into())
        );
    }

    #[test]
    fn drop_constraint_basic() {
        let dc = DropConstraint::new("my_constraint", false);
        assert_eq!(dc.name(), "my_constraint");
        assert!(!dc.if_exists());
    }

    #[test]
    fn drop_constraint_if_exists() {
        let dc = DropConstraint::new("my_constraint", true);
        assert!(dc.if_exists());
    }
}
