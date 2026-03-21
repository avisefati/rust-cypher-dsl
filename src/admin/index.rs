//! Index management types: `CreateIndex`, `DropIndex`, `IndexType`, `IndexTarget`.

use std::borrow::Cow;

use crate::types::expression::Expression;

/// The type of index to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexType {
    /// Default range index (no type keyword in CREATE).
    Range,
    /// Text index: `CREATE TEXT INDEX ...`
    Text,
    /// Point index: `CREATE POINT INDEX ...`
    Point,
    /// Full-text index: `CREATE FULLTEXT INDEX ...`
    Fulltext,
    /// Vector index: `CREATE VECTOR INDEX ...`
    Vector,
    /// Token lookup index: `CREATE LOOKUP INDEX ...`
    Lookup,
}

/// What the index is defined on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexTarget {
    /// Node index: `FOR (var:Label) ON (var.prop1, var.prop2)`.
    Node {
        /// The variable name used in the pattern (e.g., "n").
        variable: Cow<'static, str>,
        /// The label(s). For fulltext, multiple labels separated by `|`.
        labels: Vec<Cow<'static, str>>,
        /// The properties indexed.
        properties: Vec<Cow<'static, str>>,
    },
    /// Relationship index: `FOR ()-[var:TYPE]-() ON (var.prop)`.
    Relationship {
        /// The variable name used in the pattern (e.g., "r").
        variable: Cow<'static, str>,
        /// The relationship type(s). For fulltext, multiple types separated by `|`.
        types: Vec<Cow<'static, str>>,
        /// The properties indexed.
        properties: Vec<Cow<'static, str>>,
    },
    /// Token lookup for nodes: `FOR (var) ON EACH labels(var)`.
    NodeLookup {
        /// The variable name.
        variable: Cow<'static, str>,
    },
    /// Token lookup for relationships: `FOR ()-[var]-() ON EACH type(var)`.
    RelationshipLookup {
        /// The variable name.
        variable: Cow<'static, str>,
    },
}

/// A `CREATE INDEX` statement.
///
/// Renders as: `CREATE [type] INDEX [name] [IF NOT EXISTS] FOR target ON properties [OPTIONS {...}]`
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndex {
    /// The index type (Range, Text, Point, Fulltext, Vector, Lookup).
    pub(crate) index_type: IndexType,
    /// Optional index name.
    pub(crate) name: Option<Cow<'static, str>>,
    /// Whether to include `IF NOT EXISTS`.
    pub(crate) if_not_exists: bool,
    /// What the index targets (node/relationship pattern and properties).
    pub(crate) target: IndexTarget,
    /// Optional OPTIONS map (rendered as `OPTIONS { key: value, ... }`).
    pub(crate) options: Option<Expression>,
}

impl CreateIndex {
    /// Creates a new `CreateIndex`.
    pub(crate) const fn new(
        index_type: IndexType,
        name: Option<Cow<'static, str>>,
        if_not_exists: bool,
        target: IndexTarget,
    ) -> Self {
        Self {
            index_type,
            name,
            if_not_exists,
            target,
            options: None,
        }
    }

    /// Adds OPTIONS to the index.
    #[must_use]
    pub(crate) fn with_options(mut self, options: Expression) -> Self {
        self.options = Some(options);
        self
    }

    /// Returns the index type.
    pub const fn index_type(&self) -> &IndexType {
        &self.index_type
    }

    /// Returns the index name, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns whether `IF NOT EXISTS` is set.
    pub const fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    /// Returns the index target.
    pub const fn target(&self) -> &IndexTarget {
        &self.target
    }

    /// Returns the options, if any.
    pub const fn options(&self) -> Option<&Expression> {
        self.options.as_ref()
    }
}

/// A `DROP INDEX` statement.
///
/// Renders as: `DROP INDEX name [IF EXISTS]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropIndex {
    /// The index name to drop.
    pub(crate) name: Cow<'static, str>,
    /// Whether to include `IF EXISTS`.
    pub(crate) if_exists: bool,
}

impl DropIndex {
    /// Creates a new `DropIndex`.
    pub(crate) fn new(name: impl Into<Cow<'static, str>>, if_exists: bool) -> Self {
        Self {
            name: name.into(),
            if_exists,
        }
    }

    /// Returns the index name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether `IF EXISTS` is set.
    pub const fn if_exists(&self) -> bool {
        self.if_exists
    }
}

#[cfg(test)]
#[allow(clippy::panic, reason = "tests use assert macros")]
mod tests {
    use super::*;

    #[test]
    fn create_range_index_for_node() {
        let idx = CreateIndex::new(
            IndexType::Range,
            Some("idx_name".into()),
            false,
            IndexTarget::Node {
                variable: "n".into(),
                labels: vec!["Person".into()],
                properties: vec!["name".into()],
            },
        );
        assert_eq!(idx.index_type(), &IndexType::Range);
        assert_eq!(idx.name(), Some("idx_name"));
        assert!(!idx.if_not_exists());
    }

    #[test]
    fn create_fulltext_index_multiple_labels() {
        let idx = CreateIndex::new(
            IndexType::Fulltext,
            Some("ft_idx".into()),
            true,
            IndexTarget::Node {
                variable: "n".into(),
                labels: vec!["Movie".into(), "Book".into()],
                properties: vec!["title".into(), "description".into()],
            },
        );
        assert_eq!(idx.index_type(), &IndexType::Fulltext);
        assert!(idx.if_not_exists());
        let IndexTarget::Node { labels, properties, .. } = idx.target() else {
            panic!("Expected node target");
        };
        assert_eq!(labels.len(), 2);
        assert_eq!(properties.len(), 2);
    }

    #[test]
    fn create_vector_index_with_options() {
        let opts = Expression::raw_unchecked(
            "{`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}",
        );
        let idx = CreateIndex::new(
            IndexType::Vector,
            Some("vec_idx".into()),
            true,
            IndexTarget::Node {
                variable: "n".into(),
                labels: vec!["Document".into()],
                properties: vec!["embedding".into()],
            },
        )
        .with_options(opts);
        assert!(idx.options().is_some());
    }

    #[test]
    fn create_lookup_index() {
        let idx = CreateIndex::new(
            IndexType::Lookup,
            Some("lookup_idx".into()),
            false,
            IndexTarget::NodeLookup {
                variable: "n".into(),
            },
        );
        assert_eq!(idx.index_type(), &IndexType::Lookup);
        assert!(matches!(idx.target(), IndexTarget::NodeLookup { .. }));
    }

    #[test]
    fn create_relationship_index() {
        let idx = CreateIndex::new(
            IndexType::Range,
            Some("rel_idx".into()),
            false,
            IndexTarget::Relationship {
                variable: "r".into(),
                types: vec!["KNOWS".into()],
                properties: vec!["since".into()],
            },
        );
        assert!(matches!(idx.target(), IndexTarget::Relationship { .. }));
    }

    #[test]
    fn drop_index_basic() {
        let di = DropIndex::new("my_index", false);
        assert_eq!(di.name(), "my_index");
        assert!(!di.if_exists());
    }

    #[test]
    fn drop_index_if_exists() {
        let di = DropIndex::new("my_index", true);
        assert!(di.if_exists());
    }
}
