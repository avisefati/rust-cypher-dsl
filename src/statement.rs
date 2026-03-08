//! `Statement` AST root types.
//!
//! A [`Statement`] is the top-level node in a Cypher query AST.
//! It can be rendered to a Cypher string via [`Statement::render()`]
//! or the [`Display`](std::fmt::Display) trait.

use std::fmt;

use crate::clauses::Clause;
use crate::renderer::default::DefaultRenderer;
use crate::renderer::RenderConfig;

/// A complete Cypher statement.
///
/// Implements [`Display`](fmt::Display) by delegating to the default
/// single-line renderer with standard escaping.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A single-part query (one sequence of clauses).
    SinglePart(SinglePartQuery),
}

/// A single-part query: a sequence of clauses executed in order.
///
/// For example: `MATCH (n:Person) WHERE n.age > 21 RETURN n`
#[derive(Debug, Clone, PartialEq)]
pub struct SinglePartQuery {
    /// The ordered list of clauses.
    pub(crate) clauses: Vec<Clause>,
}

impl SinglePartQuery {
    /// Creates a new single-part query from clauses.
    pub const fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }

    /// Returns the clauses.
    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }
}

impl Statement {
    /// Renders this statement to a Cypher string using the default renderer.
    pub fn render(&self) -> String {
        let renderer = DefaultRenderer::with_defaults();
        renderer.render_statement(self)
    }

    /// Renders this statement with a custom configuration.
    pub fn render_with(&self, config: RenderConfig) -> String {
        let renderer = DefaultRenderer::new(config);
        renderer.render_statement(self)
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

impl From<SinglePartQuery> for Statement {
    fn from(query: SinglePartQuery) -> Self {
        Self::SinglePart(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clauses::{MatchClause, ReturnClause, WhereClause};
    use crate::types::condition::Condition;
    use crate::types::expression::Expression;
    use crate::types::node::node;
    use crate::types::relationship::rel;

    #[test]
    fn render_simple_match_return() {
        // MATCH (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN n");
    }

    #[test]
    fn render_match_where_return() {
        // MATCH (n:`Person`) WHERE n.age > 21 RETURN n
        let n = node("Person").named("n");
        let age = Expression::from(Expression::symbolic_name("n").property("age"));
        let cond = Condition::Comparison {
            left: age,
            operator: crate::types::operator::ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Where(WhereClause::new(cond)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WHERE n.age > 21 RETURN n"
        );
    }

    #[test]
    fn render_optional_match() {
        // OPTIONAL MATCH (n:`Person`) RETURN n
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::optional(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(
            stmt.render(),
            "OPTIONAL MATCH (n:`Person`) RETURN n"
        );
    }

    #[test]
    fn render_return_distinct() {
        // MATCH (n:`Person`) RETURN DISTINCT n
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::distinct(vec![
                Expression::symbolic_name("n"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN DISTINCT n"
        );
    }

    #[test]
    fn render_return_multiple_expressions() {
        // MATCH (n:`Person`) RETURN n, n.name
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("n"),
                Expression::symbolic_name("n").property("name").into(),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n, n.name"
        );
    }

    #[test]
    fn render_return_aliased() {
        // MATCH (n:`Person`) RETURN n.name AS name
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![
                Expression::from(
                    Expression::symbolic_name("n").property("name"),
                )
                .as_alias("name"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n.name AS name"
        );
    }

    #[test]
    fn render_match_relationship_return() {
        // MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b
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
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`)-[:`KNOWS`]->(b:`Person`) RETURN a, b"
        );
    }

    #[test]
    fn display_matches_render() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(format!("{stmt}"), stmt.render());
    }

    #[test]
    fn render_with_custom_config() {
        use crate::renderer::EscapeMode;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        let output = stmt.render_with(RenderConfig {
            escape_names: EscapeMode::AsNeeded,
            ..RenderConfig::default()
        });
        assert_eq!(output, "MATCH (n:Person) RETURN n");
    }

    #[test]
    fn render_return_asterisk() {
        // MATCH (n:`Person`) RETURN *
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::asterisk()])),
        ]));
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN *");
    }

    // --- ORDER BY, SKIP, LIMIT tests ---

    #[test]
    fn render_return_order_by_ascending() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name
        use crate::clauses::OrderByClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::OrderBy(OrderByClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name"
        );
    }

    #[test]
    fn render_return_order_by_descending() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.age DESC
        use crate::clauses::OrderByClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::OrderBy(OrderByClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.age DESC"
        );
    }

    #[test]
    fn render_return_order_by_multiple() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC
        use crate::clauses::OrderByClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::OrderBy(OrderByClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
                Expression::from(Expression::symbolic_name("n").property("age")).descending(),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name, n.age DESC"
        );
    }

    #[test]
    fn render_return_skip() {
        // MATCH (n:`Person`) RETURN n SKIP 10
        use crate::clauses::SkipClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::Skip(SkipClause::new(10_i32)),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n SKIP 10"
        );
    }

    #[test]
    fn render_return_limit() {
        // MATCH (n:`Person`) RETURN n LIMIT 25
        use crate::clauses::LimitClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::Limit(LimitClause::new(25_i32)),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n LIMIT 25"
        );
    }

    #[test]
    fn render_return_order_by_skip_limit() {
        // MATCH (n:`Person`) RETURN n ORDER BY n.name SKIP 5 LIMIT 10
        use crate::clauses::{LimitClause, OrderByClause, SkipClause};
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("n")])),
            Clause::OrderBy(OrderByClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).ascending(),
            ])),
            Clause::Skip(SkipClause::new(5_i32)),
            Clause::Limit(LimitClause::new(10_i32)),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN n ORDER BY n.name SKIP 5 LIMIT 10"
        );
    }

    // --- WITH and UNWIND tests ---

    #[test]
    fn render_with_single_expression() {
        // MATCH (n:`Person`) WITH n AS person RETURN person
        use crate::clauses::WithClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::With(WithClause::new(vec![
                Expression::symbolic_name("n").as_alias("person"),
            ])),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("person"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n AS person RETURN person"
        );
    }

    #[test]
    fn render_with_multiple_expressions() {
        // MATCH (n:`Person`) WITH n.name AS name, n.age AS age RETURN name, age
        use crate::clauses::WithClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::With(WithClause::new(vec![
                Expression::from(Expression::symbolic_name("n").property("name")).as_alias("name"),
                Expression::from(Expression::symbolic_name("n").property("age")).as_alias("age"),
            ])),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("name"),
                Expression::symbolic_name("age"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n.name AS name, n.age AS age RETURN name, age"
        );
    }

    #[test]
    fn render_with_distinct() {
        // MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city
        use crate::clauses::WithClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::With(WithClause::distinct(vec![
                Expression::from(Expression::symbolic_name("n").property("city")).as_alias("city"),
            ])),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("city"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH DISTINCT n.city AS city RETURN city"
        );
    }

    #[test]
    fn render_unwind_list() {
        // UNWIND [1, 2, 3] AS x RETURN x
        use crate::clauses::UnwindClause;
        let list = Expression::list_literal(vec![
            Expression::from(1_i32),
            Expression::from(2_i32),
            Expression::from(3_i32),
        ]);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Unwind(UnwindClause::new(list.as_alias("x"))),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("x"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "UNWIND [1, 2, 3] AS x RETURN x"
        );
    }

    #[test]
    fn render_match_unwind_return() {
        // MATCH (n:`Person`) UNWIND n.friends AS friend RETURN friend
        use crate::clauses::UnwindClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Unwind(UnwindClause::new(
                Expression::from(Expression::symbolic_name("n").property("friends")).as_alias("friend"),
            )),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("friend"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) UNWIND n.friends AS friend RETURN friend"
        );
    }

    #[test]
    fn render_with_where() {
        // MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person
        use crate::clauses::WithClause;
        let n = node("Person").named("n");
        let age = Expression::from(Expression::symbolic_name("person").property("age"));
        let cond = Condition::Comparison {
            left: age,
            operator: crate::types::operator::ComparisonOp::Gt,
            right: Expression::from(21_i32),
        };
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::With(WithClause::new(vec![
                Expression::symbolic_name("n").as_alias("person"),
            ])),
            Clause::Where(WhereClause::new(cond)),
            Clause::Return(ReturnClause::new(vec![
                Expression::symbolic_name("person"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) WITH n AS person WHERE person.age > 21 RETURN person"
        );
    }

    // --- CREATE and MERGE tests ---

    #[test]
    fn render_create_node() {
        // CREATE (n:`Person` {name: 'Alice'})
        use crate::clauses::CreateClause;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Create(CreateClause::new(n)),
        ]));
        assert_eq!(
            stmt.render(),
            "CREATE (n:`Person` {name: 'Alice'})"
        );
    }

    #[test]
    fn render_create_relationship() {
        // CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)
        use crate::clauses::CreateClause;
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Create(CreateClause::new(r)),
        ]));
        assert_eq!(
            stmt.render(),
            "CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person`)"
        );
    }

    #[test]
    fn render_merge_simple() {
        // MERGE (n:`Person` {name: 'Alice'})
        use crate::clauses::MergeClause;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::new(n)),
        ]));
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person` {name: 'Alice'})"
        );
    }

    #[test]
    fn render_merge_on_create() {
        // MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true
        use crate::clauses::{MergeAction, MergeClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::with_actions(
                n,
                vec![MergeAction::OnCreate(vec![
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "created"),
                        Expression::from(true),
                    ),
                ])],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true"
        );
    }

    #[test]
    fn render_merge_on_match() {
        // MERGE (n:`Person` {name: 'Alice'}) ON MATCH SET n.found = true
        use crate::clauses::{MergeAction, MergeClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::with_actions(
                n,
                vec![MergeAction::OnMatch(vec![
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "found"),
                        Expression::from(true),
                    ),
                ])],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person` {name: 'Alice'}) ON MATCH SET n.found = true"
        );
    }

    #[test]
    fn render_merge_on_create_and_on_match() {
        // MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true ON MATCH SET n.found = true
        use crate::clauses::{MergeAction, MergeClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::with_actions(
                n,
                vec![
                    MergeAction::OnCreate(vec![
                        SetItem::property(
                            Property::new(Expression::symbolic_name("n"), "created"),
                            Expression::from(true),
                        ),
                    ]),
                    MergeAction::OnMatch(vec![
                        SetItem::property(
                            Property::new(Expression::symbolic_name("n"), "found"),
                            Expression::from(true),
                        ),
                    ]),
                ],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true ON MATCH SET n.found = true"
        );
    }

    #[test]
    fn render_merge_multiple_set_items() {
        // MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true, n.age = 30
        use crate::clauses::{MergeAction, MergeClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person")
            .named("n")
            .with_properties(crate::props! { "name" => "Alice" });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Merge(MergeClause::with_actions(
                n,
                vec![MergeAction::OnCreate(vec![
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "created"),
                        Expression::from(true),
                    ),
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "age"),
                        Expression::from(30_i32),
                    ),
                ])],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "MERGE (n:`Person` {name: 'Alice'}) ON CREATE SET n.created = true, n.age = 30"
        );
    }

    #[test]
    fn render_match_create_return() {
        // MATCH (a:`Person`) CREATE (a)-[:`KNOWS`]->(b:`Person` {name: 'Bob'}) RETURN b
        use crate::clauses::CreateClause;
        let a = node("Person").named("a");
        let b = node("Person")
            .named("b")
            .with_properties(crate::props! { "name" => "Bob" });
        let match_clause = Clause::Match(MatchClause::new(a.clone()));
        let r = a.rel(rel("KNOWS")).to(b);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            match_clause,
            Clause::Create(CreateClause::new(r)),
            Clause::Return(ReturnClause::new(vec![Expression::symbolic_name("b")])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`) CREATE (a:`Person`)-[:`KNOWS`]->(b:`Person` {name: 'Bob'}) RETURN b"
        );
    }

    // --- SET, DELETE, REMOVE tests ---

    #[test]
    fn render_set_property() {
        // MATCH (n:`Person`) SET n.name = 'Bob'
        use crate::clauses::{SetClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Set(SetClause::new(vec![
                SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "name"),
                    Expression::from("Bob"),
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) SET n.name = 'Bob'"
        );
    }

    #[test]
    fn render_set_multiple_properties() {
        // MATCH (n:`Person`) SET n.name = 'Bob', n.age = 30
        use crate::clauses::{SetClause, SetItem};
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Set(SetClause::new(vec![
                SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "name"),
                    Expression::from("Bob"),
                ),
                SetItem::property(
                    Property::new(Expression::symbolic_name("n"), "age"),
                    Expression::from(30_i32),
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) SET n.name = 'Bob', n.age = 30"
        );
    }

    #[test]
    fn render_set_label() {
        // MATCH (n:`Person`) SET n:`Admin`
        use crate::clauses::{SetClause, SetItem};
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Set(SetClause::new(vec![
                SetItem::label(
                    Expression::symbolic_name("n"),
                    vec![std::borrow::Cow::Borrowed("Admin")],
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) SET n:`Admin`"
        );
    }

    #[test]
    fn render_set_mutate() {
        // MATCH (n:`Person`) SET n += {age: 30}
        use crate::clauses::{SetClause, SetItem};
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Set(SetClause::new(vec![
                SetItem::mutate(
                    Expression::symbolic_name("n"),
                    Expression::map_literal(vec![
                        (std::borrow::Cow::Borrowed("age"), Expression::from(30_i32)),
                    ]),
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) SET n += {age: 30}"
        );
    }

    #[test]
    fn render_delete() {
        // MATCH (n:`Person`) DELETE n
        use crate::clauses::DeleteClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Delete(DeleteClause::new(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) DELETE n"
        );
    }

    #[test]
    fn render_detach_delete() {
        // MATCH (n:`Person`) DETACH DELETE n
        use crate::clauses::DeleteClause;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Delete(DeleteClause::detach(vec![Expression::symbolic_name("n")])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) DETACH DELETE n"
        );
    }

    #[test]
    fn render_delete_multiple() {
        // MATCH (a)-[r]->(b) DELETE a, r, b
        use crate::clauses::DeleteClause;
        let a = node("Person").named("a");
        let b = node("Person").named("b");
        let r = a.rel(rel("KNOWS").named("r")).to(b);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(r)),
            Clause::Delete(DeleteClause::new(vec![
                Expression::symbolic_name("a"),
                Expression::symbolic_name("r"),
                Expression::symbolic_name("b"),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (a:`Person`)-[r:`KNOWS`]->(b:`Person`) DELETE a, r, b"
        );
    }

    #[test]
    fn render_remove_property() {
        // MATCH (n:`Person`) REMOVE n.age
        use crate::clauses::{RemoveClause, RemoveItem};
        use crate::types::property::Property;
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Remove(RemoveClause::new(vec![
                RemoveItem::property(
                    Property::new(Expression::symbolic_name("n"), "age"),
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) REMOVE n.age"
        );
    }

    #[test]
    fn render_remove_label() {
        // MATCH (n:`Person`) REMOVE n:`Admin`
        use crate::clauses::{RemoveClause, RemoveItem};
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Remove(RemoveClause::new(vec![
                RemoveItem::label(
                    Expression::symbolic_name("n"),
                    vec![std::borrow::Cow::Borrowed("Admin")],
                ),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) REMOVE n:`Admin`"
        );
    }

    // --- FOREACH tests ---

    #[test]
    fn render_foreach_with_set() {
        // MATCH p = (a)-[:`KNOWS`]->(b) FOREACH (n IN nodes(p) | SET n.visited = true)
        use crate::clauses::{ForeachClause, SetClause, SetItem};
        use crate::types::property::Property;
        let a = crate::types::node::any_node_named("a");
        let b = crate::types::node::any_node_named("b");
        let r = a.rel(rel("KNOWS")).to(b);
        let p = crate::types::pattern::path("p").defined_by(r);
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(p)),
            Clause::Foreach(ForeachClause::new(
                "n",
                Expression::raw("nodes(p)"),
                vec![Clause::Set(SetClause::new(vec![
                    SetItem::property(
                        Property::new(Expression::symbolic_name("n"), "visited"),
                        Expression::from(true),
                    ),
                ]))],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH p = (a)-[:`KNOWS`]->(b) FOREACH (n IN nodes(p) | SET n.visited = true)"
        );
    }

    #[test]
    fn render_foreach_with_create() {
        // FOREACH (name IN ['Alice', 'Bob'] | CREATE (:`Person` {name: name}))
        use crate::clauses::{CreateClause, ForeachClause};
        let list = Expression::list_literal(vec![
            Expression::from("Alice"),
            Expression::from("Bob"),
        ]);
        let n = crate::types::node::node("Person")
            .with_properties(crate::props! { "name" => Expression::symbolic_name("name") });
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Foreach(ForeachClause::new(
                "name",
                list,
                vec![Clause::Create(CreateClause::new(n))],
            )),
        ]));
        assert_eq!(
            stmt.render(),
            "FOREACH (name IN ['Alice', 'Bob'] | CREATE (:`Person` {name: name}))"
        );
    }
}
