//! Fluent builder types for admin commands.
//!
//! These builders are returned by [`Cypher`](crate::Cypher) entry-point methods
//! and guide the user through the typestate transitions needed to produce
//! valid [`Statement::Admin`](Statement::Admin) values.

use std::borrow::Cow;

use crate::statement::Statement;
use crate::types::condition::Condition;
use crate::types::expression::Expression;

use super::{AdminCommand, CreateIndex, IndexTarget, IndexType, ShowCommand};

// ── IndexBuilder ──

/// Builder for `CREATE INDEX` statements.
///
/// Follows the typestate pattern: type → target → build.
///
/// ```ignore
/// let stmt = Cypher::create_index("idx")
///     .text()
///     .for_node("n", "Person", vec!["name"])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct IndexBuilder {
    name: Option<Cow<'static, str>>,
    if_not_exists: bool,
    index_type: IndexType,
}

impl IndexBuilder {
    /// Creates a new `IndexBuilder` with the given name.
    pub(crate) fn new(
        name: impl Into<Cow<'static, str>>,
        if_not_exists: bool,
    ) -> Self {
        Self {
            name: Some(name.into()),
            if_not_exists,
            index_type: IndexType::Range,
        }
    }

    /// Sets the index type to `TEXT`.
    #[must_use]
    pub const fn text(mut self) -> Self {
        self.index_type = IndexType::Text;
        self
    }

    /// Sets the index type to `POINT`.
    #[must_use]
    pub const fn point(mut self) -> Self {
        self.index_type = IndexType::Point;
        self
    }

    /// Sets the index type to `FULLTEXT`.
    #[must_use]
    pub const fn fulltext(mut self) -> Self {
        self.index_type = IndexType::Fulltext;
        self
    }

    /// Sets the index type to `VECTOR`.
    #[must_use]
    pub const fn vector(mut self) -> Self {
        self.index_type = IndexType::Vector;
        self
    }

    /// Sets the index type to `LOOKUP`.
    #[must_use]
    pub const fn lookup(mut self) -> Self {
        self.index_type = IndexType::Lookup;
        self
    }

    /// Defines a node target: `FOR (var:Label) ON (var.prop1, var.prop2)`.
    ///
    /// For fulltext indexes, pass multiple labels to get `FOR (n:Label1|Label2)`.
    pub fn for_node(
        self,
        variable: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::Node {
                    variable: variable.into(),
                    labels: vec![label.into()],
                    properties: properties.into_iter().map(Into::into).collect(),
                },
            ),
        }
    }

    /// Defines a node target with multiple labels (for fulltext indexes).
    ///
    /// Renders as: `FOR (var:Label1|Label2) ON EACH [var.prop1, var.prop2]`
    pub fn for_node_multi_label(
        self,
        variable: impl Into<Cow<'static, str>>,
        labels: Vec<impl Into<Cow<'static, str>>>,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::Node {
                    variable: variable.into(),
                    labels: labels.into_iter().map(Into::into).collect(),
                    properties: properties.into_iter().map(Into::into).collect(),
                },
            ),
        }
    }

    /// Defines a relationship target: `FOR ()-[var:TYPE]-() ON (var.prop)`.
    pub fn for_relationship(
        self,
        variable: impl Into<Cow<'static, str>>,
        rel_type: impl Into<Cow<'static, str>>,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::Relationship {
                    variable: variable.into(),
                    types: vec![rel_type.into()],
                    properties: properties.into_iter().map(Into::into).collect(),
                },
            ),
        }
    }

    /// Defines a relationship target with multiple types (for fulltext).
    pub fn for_relationship_multi_type(
        self,
        variable: impl Into<Cow<'static, str>>,
        types: Vec<impl Into<Cow<'static, str>>>,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::Relationship {
                    variable: variable.into(),
                    types: types.into_iter().map(Into::into).collect(),
                    properties: properties.into_iter().map(Into::into).collect(),
                },
            ),
        }
    }

    /// Defines a node lookup target: `FOR (var) ON EACH labels(var)`.
    pub fn for_node_lookup(
        self,
        variable: impl Into<Cow<'static, str>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::NodeLookup {
                    variable: variable.into(),
                },
            ),
        }
    }

    /// Defines a relationship lookup target: `FOR ()-[var]-() ON EACH type(var)`.
    pub fn for_relationship_lookup(
        self,
        variable: impl Into<Cow<'static, str>>,
    ) -> IndexBuildable {
        IndexBuildable {
            inner: CreateIndex::new(
                self.index_type,
                self.name,
                self.if_not_exists,
                IndexTarget::RelationshipLookup {
                    variable: variable.into(),
                },
            ),
        }
    }
}

/// A `CREATE INDEX` that has a target and is ready to build.
///
/// Optionally add `OPTIONS` before calling `.build()`.
#[derive(Debug, Clone)]
pub struct IndexBuildable {
    inner: CreateIndex,
}

impl IndexBuildable {
    /// Adds an `OPTIONS` clause to the index.
    #[must_use]
    pub fn options(mut self, opts: Expression) -> Self {
        self.inner = self.inner.with_options(opts);
        self
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::Admin(AdminCommand::CreateIndex(self.inner))
    }
}

// ── ShowBuilder ──

/// Which SHOW command variant to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShowKind {
    Indexes,
    Constraints,
    Functions,
    Procedures,
    Transactions,
}

/// Builder for `SHOW` commands (indexes, constraints, functions,
/// procedures, transactions).
///
/// ```ignore
/// let stmt = Cypher::show_indexes()
///     .type_filter("RANGE")
///     .yield_all()
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ShowBuilder {
    kind: ShowKind,
    inner: ShowCommand,
}

impl ShowBuilder {
    /// Creates a new `ShowBuilder` for the given command kind.
    pub(crate) const fn new(kind: ShowKind) -> Self {
        Self {
            kind,
            inner: ShowCommand::new(),
        }
    }

    /// Filters by type (e.g., `"RANGE"` for indexes, `"UNIQUE"` for
    /// constraints, `"BUILT IN"` for functions).
    #[must_use]
    pub fn type_filter(mut self, filter: impl Into<Cow<'static, str>>) -> Self {
        self.inner = self.inner.with_type_filter(filter);
        self
    }

    /// Adds `YIELD *` to the statement.
    #[must_use]
    pub fn yield_all(mut self) -> Self {
        self.inner = self.inner.with_yield_all();
        self
    }

    /// Adds `YIELD field1, field2, ...` to the statement.
    #[must_use]
    pub fn yield_fields(mut self, fields: Vec<Expression>) -> Self {
        self.inner = self.inner.with_yield_fields(fields);
        self
    }

    /// Adds a `WHERE` condition (requires `YIELD`).
    #[must_use]
    pub fn where_(mut self, condition: impl Into<Condition>) -> Self {
        self.inner = self.inner.with_where(condition.into());
        self
    }

    /// Adds `EXECUTABLE BY CURRENT USER` (for functions/procedures).
    #[must_use]
    pub fn executable_by_current_user(mut self) -> Self {
        self.inner = self
            .inner
            .with_executable(super::ExecutableFilter::CurrentUser);
        self
    }

    /// Adds `EXECUTABLE BY username` (for functions/procedures).
    #[must_use]
    pub fn executable_by(
        mut self,
        user: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.inner = self
            .inner
            .with_executable(super::ExecutableFilter::User(user.into()));
        self
    }

    /// Sets transaction IDs (for `SHOW TRANSACTIONS` only).
    #[must_use]
    pub fn ids(
        mut self,
        ids: Vec<impl Into<Cow<'static, str>>>,
    ) -> Self {
        self.inner = self
            .inner
            .with_transaction_ids(ids.into_iter().map(Into::into).collect());
        self
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        let cmd = match self.kind {
            ShowKind::Indexes => AdminCommand::ShowIndexes(self.inner),
            ShowKind::Constraints => AdminCommand::ShowConstraints(self.inner),
            ShowKind::Functions => AdminCommand::ShowFunctions(self.inner),
            ShowKind::Procedures => AdminCommand::ShowProcedures(self.inner),
            ShowKind::Transactions => AdminCommand::ShowTransactions(self.inner),
        };
        Statement::Admin(cmd)
    }
}

// ── ConstraintBuilder ──

/// Builder for `CREATE CONSTRAINT` statements.
///
/// Follows the typestate pattern: target → require → build.
///
/// ```ignore
/// let stmt = Cypher::create_constraint("unique_email")
///     .for_node("n", "Person")
///     .is_unique(vec!["email"]);
/// ```
#[derive(Debug, Clone)]
pub struct ConstraintBuilder {
    name: Option<Cow<'static, str>>,
    if_not_exists: bool,
}

impl ConstraintBuilder {
    /// Creates a new `ConstraintBuilder` with the given name.
    pub(crate) fn new(
        name: impl Into<Cow<'static, str>>,
        if_not_exists: bool,
    ) -> Self {
        Self {
            name: Some(name.into()),
            if_not_exists,
        }
    }

    /// Sets the target to a node: `FOR (var:Label)`.
    pub fn for_node(
        self,
        variable: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
    ) -> ConstraintRequire {
        ConstraintRequire {
            name: self.name,
            if_not_exists: self.if_not_exists,
            target: super::ConstraintTarget::Node {
                variable: variable.into(),
                label: label.into(),
            },
        }
    }

    /// Sets the target to a relationship: `FOR ()-[var:TYPE]-()`.
    pub fn for_relationship(
        self,
        variable: impl Into<Cow<'static, str>>,
        rel_type: impl Into<Cow<'static, str>>,
    ) -> ConstraintRequire {
        ConstraintRequire {
            name: self.name,
            if_not_exists: self.if_not_exists,
            target: super::ConstraintTarget::Relationship {
                variable: variable.into(),
                rel_type: rel_type.into(),
            },
        }
    }
}

/// A constraint builder that has a target and needs a REQUIRE clause.
///
/// Call one of the `is_*` methods to specify the constraint type
/// and produce the final `Statement`.
#[derive(Debug, Clone)]
pub struct ConstraintRequire {
    name: Option<Cow<'static, str>>,
    if_not_exists: bool,
    target: super::ConstraintTarget,
}

#[allow(
    clippy::wrong_self_convention,
    reason = "is_* methods here are constraint-specification builders, not boolean predicates"
)]
impl ConstraintRequire {
    /// Builds a helper to create the final statement.
    fn finish(
        self,
        properties: Vec<Cow<'static, str>>,
        constraint_type: super::ConstraintType,
    ) -> Statement {
        Statement::Admin(AdminCommand::CreateConstraint(
            super::CreateConstraint::new(
                self.name,
                self.if_not_exists,
                self.target,
                properties,
                constraint_type,
            ),
        ))
    }

    /// `REQUIRE var.prop IS UNIQUE`.
    pub fn is_unique(
        self,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> Statement {
        self.finish(
            properties.into_iter().map(Into::into).collect(),
            super::ConstraintType::Unique,
        )
    }

    /// `REQUIRE var.prop IS NOT NULL`.
    pub fn is_not_null(
        self,
        property: impl Into<Cow<'static, str>>,
    ) -> Statement {
        self.finish(
            vec![property.into()],
            super::ConstraintType::Exists,
        )
    }

    /// `REQUIRE (var.prop1, var.prop2) IS NODE KEY`.
    pub fn is_node_key(
        self,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> Statement {
        self.finish(
            properties.into_iter().map(Into::into).collect(),
            super::ConstraintType::NodeKey,
        )
    }

    /// `REQUIRE (var.prop1, var.prop2) IS RELATIONSHIP KEY`.
    pub fn is_relationship_key(
        self,
        properties: Vec<impl Into<Cow<'static, str>>>,
    ) -> Statement {
        self.finish(
            properties.into_iter().map(Into::into).collect(),
            super::ConstraintType::RelationshipKey,
        )
    }

    /// `REQUIRE var.prop IS :: TYPE`.
    pub fn is_typed(
        self,
        property: impl Into<Cow<'static, str>>,
        type_name: impl Into<Cow<'static, str>>,
    ) -> Statement {
        self.finish(
            vec![property.into()],
            super::ConstraintType::PropertyType(type_name.into()),
        )
    }
}

// ── TerminateBuilder ──

/// Builder for `TERMINATE TRANSACTIONS` statements.
///
/// ```ignore
/// let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct TerminateBuilder {
    inner: super::TerminateTransactions,
}

impl TerminateBuilder {
    /// Creates a new `TerminateBuilder` with the given transaction IDs.
    pub(crate) const fn new(ids: Vec<Cow<'static, str>>) -> Self {
        Self {
            inner: super::TerminateTransactions::new(ids),
        }
    }

    /// Adds `YIELD *` to the statement.
    #[must_use]
    pub fn yield_all(mut self) -> Self {
        self.inner = self.inner.with_yield_all();
        self
    }

    /// Adds `YIELD field1, field2, ...` to the statement.
    #[must_use]
    pub fn yield_fields(mut self, fields: Vec<Expression>) -> Self {
        self.inner = self.inner.with_yield_fields(fields);
        self
    }

    /// Adds a `WHERE` condition (requires `YIELD`).
    #[must_use]
    pub fn where_(mut self, condition: impl Into<Condition>) -> Self {
        self.inner = self.inner.with_where(condition.into());
        self
    }

    /// Builds the final `Statement`.
    pub fn build(self) -> Statement {
        Statement::Admin(AdminCommand::TerminateTransactions(self.inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::Cypher;

    // ── IndexBuilder tests ──

    #[test]
    fn create_range_index_for_node() {
        let stmt = Cypher::create_index("person_name_idx")
            .for_node("n", "Person", vec!["name"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE INDEX person_name_idx FOR (n:Person) ON (n.name)"
        );
    }

    #[test]
    fn create_text_index_if_not_exists() {
        let stmt = Cypher::create_index_if_not_exists("bio_idx")
            .text()
            .for_node("n", "Person", vec!["bio"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE TEXT INDEX bio_idx IF NOT EXISTS FOR (n:Person) ON (n.bio)"
        );
    }

    #[test]
    fn create_point_index() {
        let stmt = Cypher::create_index("loc_idx")
            .point()
            .for_node("n", "Place", vec!["location"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE POINT INDEX loc_idx FOR (n:Place) ON (n.location)"
        );
    }

    #[test]
    fn create_fulltext_index_single_label() {
        let stmt = Cypher::create_index("ft_idx")
            .fulltext()
            .for_node("n", "Movie", vec!["title", "description"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE FULLTEXT INDEX ft_idx FOR (n:Movie) ON EACH [n.title, n.description]"
        );
    }

    #[test]
    fn create_fulltext_index_multi_label() {
        let stmt = Cypher::create_index("ft_idx")
            .fulltext()
            .for_node_multi_label("n", vec!["Movie", "Book"], vec!["title", "description"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE FULLTEXT INDEX ft_idx FOR (n:Movie|Book) ON EACH [n.title, n.description]"
        );
    }

    #[test]
    fn create_vector_index_with_options() {
        let opts = Expression::raw_unchecked(
            "{`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}",
        );
        let stmt = Cypher::create_index("vec_idx")
            .vector()
            .for_node("n", "Document", vec!["embedding"])
            .options(opts)
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE VECTOR INDEX vec_idx FOR (n:Document) ON (n.embedding) OPTIONS {`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}"
        );
    }

    #[test]
    fn create_lookup_index_node() {
        let stmt = Cypher::create_index("node_lookup")
            .lookup()
            .for_node_lookup("n")
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE LOOKUP INDEX node_lookup FOR (n) ON EACH labels(n)"
        );
    }

    #[test]
    fn create_lookup_index_relationship() {
        let stmt = Cypher::create_index("rel_lookup")
            .lookup()
            .for_relationship_lookup("r")
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE LOOKUP INDEX rel_lookup FOR ()-[r]-() ON EACH type(r)"
        );
    }

    #[test]
    fn create_index_for_relationship() {
        let stmt = Cypher::create_index("rel_idx")
            .for_relationship("r", "KNOWS", vec!["since"])
            .build();
        assert_eq!(
            stmt.render(),
            "CREATE INDEX rel_idx FOR ()-[r:KNOWS]-() ON (r.since)"
        );
    }

    #[test]
    fn drop_index_builder() {
        let stmt = Cypher::drop_index("my_index");
        assert_eq!(stmt.render(), "DROP INDEX my_index");
    }

    #[test]
    fn drop_index_if_exists_builder() {
        let stmt = Cypher::drop_index_if_exists("my_index");
        assert_eq!(stmt.render(), "DROP INDEX my_index IF EXISTS");
    }

    #[test]
    fn show_indexes_basic() {
        let stmt = Cypher::show_indexes().build();
        assert_eq!(stmt.render(), "SHOW INDEXES");
    }

    #[test]
    fn show_indexes_with_type_filter_and_yield() {
        let stmt = Cypher::show_indexes()
            .type_filter("RANGE")
            .yield_all()
            .build();
        assert_eq!(stmt.render(), "SHOW RANGE INDEXES YIELD *");
    }

    // ── ConstraintBuilder tests ──

    #[test]
    fn create_unique_constraint() {
        let stmt = Cypher::create_constraint("unique_email")
            .for_node("n", "Person")
            .is_unique(vec!["email"]);
        assert_eq!(
            stmt.render(),
            "CREATE CONSTRAINT unique_email FOR (n:Person) REQUIRE n.email IS UNIQUE"
        );
    }

    #[test]
    fn create_constraint_if_not_exists() {
        let stmt = Cypher::create_constraint_if_not_exists("exists_name")
            .for_node("n", "Person")
            .is_not_null("name");
        assert_eq!(
            stmt.render(),
            "CREATE CONSTRAINT exists_name IF NOT EXISTS FOR (n:Person) REQUIRE n.name IS NOT NULL"
        );
    }

    #[test]
    fn create_node_key_composite() {
        let stmt = Cypher::create_constraint("person_key")
            .for_node("n", "Person")
            .is_node_key(vec!["id", "name"]);
        assert_eq!(
            stmt.render(),
            "CREATE CONSTRAINT person_key FOR (n:Person) REQUIRE (n.id, n.name) IS NODE KEY"
        );
    }

    #[test]
    fn create_relationship_key_constraint() {
        let stmt = Cypher::create_constraint("rel_key")
            .for_relationship("r", "REVIEWED")
            .is_relationship_key(vec!["id"]);
        assert_eq!(
            stmt.render(),
            "CREATE CONSTRAINT rel_key FOR ()-[r:REVIEWED]-() REQUIRE r.id IS RELATIONSHIP KEY"
        );
    }

    #[test]
    fn create_property_type_constraint() {
        let stmt = Cypher::create_constraint("score_type")
            .for_relationship("r", "REVIEWED")
            .is_typed("score", "FLOAT");
        assert_eq!(
            stmt.render(),
            "CREATE CONSTRAINT score_type FOR ()-[r:REVIEWED]-() REQUIRE r.score IS :: FLOAT"
        );
    }

    #[test]
    fn drop_constraint_builder() {
        let stmt = Cypher::drop_constraint("my_constraint");
        assert_eq!(stmt.render(), "DROP CONSTRAINT my_constraint");
    }

    #[test]
    fn drop_constraint_if_exists_builder() {
        let stmt = Cypher::drop_constraint_if_exists("my_constraint");
        assert_eq!(stmt.render(), "DROP CONSTRAINT my_constraint IF EXISTS");
    }

    #[test]
    fn show_constraints_basic() {
        let stmt = Cypher::show_constraints().build();
        assert_eq!(stmt.render(), "SHOW CONSTRAINTS");
    }

    #[test]
    fn show_constraints_with_filter() {
        let stmt = Cypher::show_constraints()
            .type_filter("UNIQUE")
            .build();
        assert_eq!(stmt.render(), "SHOW UNIQUE CONSTRAINTS");
    }

    // ── ShowBuilder tests ──

    #[test]
    fn show_functions_basic() {
        let stmt = Cypher::show_functions().build();
        assert_eq!(stmt.render(), "SHOW FUNCTIONS");
    }

    #[test]
    fn show_functions_built_in_executable() {
        let stmt = Cypher::show_functions()
            .type_filter("BUILT IN")
            .executable_by_current_user()
            .build();
        assert_eq!(
            stmt.render(),
            "SHOW BUILT IN FUNCTIONS EXECUTABLE BY CURRENT USER"
        );
    }

    #[test]
    fn show_functions_executable_by_user() {
        let stmt = Cypher::show_functions()
            .executable_by("alice")
            .build();
        assert_eq!(stmt.render(), "SHOW FUNCTIONS EXECUTABLE BY alice");
    }

    #[test]
    fn show_procedures_with_yield_and_where() {
        let stmt = Cypher::show_procedures()
            .yield_fields(vec![
                Expression::symbolic_name("name"),
                Expression::symbolic_name("signature"),
            ])
            .where_(Expression::symbolic_name("name").starts_with("db."))
            .build();
        assert_eq!(
            stmt.render(),
            "SHOW PROCEDURES YIELD name, signature WHERE name STARTS WITH 'db.'"
        );
    }

    #[test]
    fn show_transactions_basic() {
        let stmt = Cypher::show_transactions().build();
        assert_eq!(stmt.render(), "SHOW TRANSACTIONS");
    }

    #[test]
    fn show_transactions_with_ids() {
        let stmt = Cypher::show_transactions()
            .ids(vec!["neo4j-tx-123"])
            .build();
        assert_eq!(stmt.render(), "SHOW TRANSACTIONS 'neo4j-tx-123'");
    }

    // ── TerminateBuilder tests ──

    #[test]
    fn terminate_transactions_basic() {
        let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"]).build();
        assert_eq!(stmt.render(), "TERMINATE TRANSACTIONS 'neo4j-tx-123'");
    }

    #[test]
    fn terminate_transactions_multiple() {
        let stmt =
            Cypher::terminate_transactions(vec!["neo4j-tx-1", "neo4j-tx-2"]).build();
        assert_eq!(
            stmt.render(),
            "TERMINATE TRANSACTIONS 'neo4j-tx-1', 'neo4j-tx-2'"
        );
    }

    #[test]
    fn terminate_transactions_with_yield_and_where() {
        let stmt = Cypher::terminate_transactions(vec!["neo4j-tx-123"])
            .yield_all()
            .where_(
                Expression::symbolic_name("username")
                    .eq(Expression::string_literal("alice")),
            )
            .build();
        assert_eq!(
            stmt.render(),
            "TERMINATE TRANSACTIONS 'neo4j-tx-123' YIELD * WHERE username = 'alice'"
        );
    }
}
