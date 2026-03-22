//! # Real-World Examples
//!
//! Examples based on the [Neo4j Movies Graph](https://neo4j.com/docs/getting-started/appendix/example-data/)
//! — a dataset of movies, actors, directors, and reviewers.
//!
//! ## The Movies Graph
//!
//! ```text
//! (:Person {name, born}) -[:ACTED_IN {roles}]->  (:Movie {title, released, tagline})
//! (:Person)              -[:DIRECTED]->           (:Movie)
//! (:Person)              -[:PRODUCED]->           (:Movie)
//! (:Person)              -[:REVIEWED {summary, rating}]-> (:Movie)
//! ```
//!
//! ---
//!
//! ## Reading Data
//!
//! ### Find all movies
//!
//! ```cypher
//! MATCH (m:Movie) RETURN m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::match_(node("Movie").named("m"))
//!     .returning(name("m"))
//!     .build();
//!
//! assert_eq!(stmt.render(), "MATCH (m:`Movie`) RETURN m");
//! ```
//!
//! ### Find a movie by title
//!
//! ```cypher
//! MATCH (m:Movie {title: 'The Matrix'}) RETURN m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::match_(
//!         node("Movie").named("m").with_properties(props!("title" => "The Matrix"))
//!     )
//!     .returning(name("m"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (m:`Movie` {title: 'The Matrix'}) RETURN m"
//! );
//! ```
//!
//! ### Find actors in a movie (outgoing relationship)
//!
//! ```cypher
//! MATCH (p:Person)-[:ACTED_IN]->(m:Movie {title: 'The Matrix'})
//! RETURN p.name
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m").with_properties(props!("title" => "The Matrix"));
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .returning(Expression::from(prop("p", "name")))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie` {title: 'The Matrix'}) RETURN p.name"
//! );
//! ```
//!
//! ### Find who directed a movie (using `.to()` shorthand)
//!
//! ```cypher
//! MATCH (p:Person)-[:DIRECTED]->(m:Movie {title: 'The Matrix'})
//! RETURN p.name
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m").with_properties(props!("title" => "The Matrix"));
//!
//! let stmt = Cypher::match_(p.to("DIRECTED", m))
//!     .returning(Expression::from(prop("p", "name")))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`DIRECTED`]->(m:`Movie` {title: 'The Matrix'}) RETURN p.name"
//! );
//! ```
//!
//! ### Find movies released after 2000
//!
//! ```cypher
//! MATCH (m:Movie) WHERE m.released > 2000 RETURN m.title, m.released
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::match_(node("Movie").named("m"))
//!     .where_(prop("m", "released").gt(2000_i32))
//!     .returning((
//!         Expression::from(prop("m", "title")),
//!         Expression::from(prop("m", "released")),
//!     ))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (m:`Movie`) WHERE m.released > 2000 RETURN m.title, m.released"
//! );
//! ```
//!
//! ### Find co-actors (multi-hop pattern)
//!
//! ```cypher
//! MATCH (p:Person)-[:ACTED_IN]->(m:Movie)<-[:ACTED_IN]-(coActor:Person)
//! WHERE p.name = 'Tom Hanks'
//! RETURN coActor.name, m.title
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m");
//! let co = node("Person").named("coActor");
//!
//! let pattern = (p >> rel("ACTED_IN") >> m) << rel("ACTED_IN") << co;
//!
//! let stmt = Cypher::match_(pattern)
//!     .where_(prop("p", "name").eq("Tom Hanks"))
//!     .returning((
//!         Expression::from(prop("coActor", "name")),
//!         Expression::from(prop("m", "title")),
//!     ))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`)<-[:`ACTED_IN`]-(coActor:`Person`) \
//!      WHERE p.name = 'Tom Hanks' RETURN coActor.name, m.title"
//! );
//! ```
//!
//! ### Aggregate: count movies per actor
//!
//! ```cypher
//! MATCH (p:Person)-[:ACTED_IN]->(m:Movie)
//! RETURN p.name, count(m) AS movieCount
//! ORDER BY movieCount DESC
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! use rust_cypher_dsl::functions::aggregate;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m");
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .returning((
//!         Expression::from(prop("p", "name")),
//!         aggregate::count(name("m")).alias("movieCount"),
//!     ))
//!     .order_by(name("movieCount").descending())
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`) \
//!      RETURN p.name, count(m) AS movieCount \
//!      ORDER BY movieCount DESC"
//! );
//! ```
//!
//! ### Collect: list all actors per movie
//!
//! ```cypher
//! MATCH (p:Person)-[:ACTED_IN]->(m:Movie)
//! RETURN m.title, collect(p.name) AS cast
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! use rust_cypher_dsl::functions::aggregate;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m");
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .returning((
//!         Expression::from(prop("m", "title")),
//!         aggregate::collect(prop("p", "name")).alias("cast"),
//!     ))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`) \
//!      RETURN m.title, collect(p.name) AS cast"
//! );
//! ```
//!
//! ### Parameterized query
//!
//! ```cypher
//! MATCH (p:Person {name: $name})-[:ACTED_IN]->(m:Movie) RETURN m.title
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p").with_properties(props!("name" => param("name")));
//! let m = node("Movie").named("m");
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .returning(Expression::from(prop("m", "title")))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person` {name: $name})-[:`ACTED_IN`]->(m:`Movie`) RETURN m.title"
//! );
//! ```
//!
//! ---
//!
//! ## Writing Data
//!
//! ### Create a movie
//!
//! ```cypher
//! CREATE (m:Movie {title: 'New Movie', released: 2024}) RETURN m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::create(
//!         node("Movie").named("m").with_properties(
//!             props!("title" => "New Movie", "released" => 2024_i32)
//!         )
//!     )
//!     .returning(name("m"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "CREATE (m:`Movie` {title: 'New Movie', released: 2024}) RETURN m"
//! );
//! ```
//!
//! ### Create a relationship between existing nodes
//!
//! ```cypher
//! MATCH (p:Person {name: 'Tom Hanks'}), (m:Movie {title: 'New Movie'})
//! CREATE (p)-[:ACTED_IN {roles: ['Hero']}]->(m)
//! RETURN p, m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p").with_properties(props!("name" => "Tom Hanks"));
//! let m = node("Movie").named("m").with_properties(props!("title" => "New Movie"));
//!
//! let acted = rel("ACTED_IN")
//!     .with_properties(props!("roles" => list_of(vec![lit("Hero")])));
//!
//! // Use bare node references in CREATE to avoid repeating properties
//! let p_ref = any_node_named("p");
//! let m_ref = any_node_named("m");
//!
//! let stmt = Cypher::match_((p, m))
//!     .create(p_ref.rel(acted).to(m_ref))
//!     .returning((name("p"), name("m")))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person` {name: 'Tom Hanks'}), (m:`Movie` {title: 'New Movie'}) \
//!      CREATE (p)-[:`ACTED_IN` {roles: ['Hero']}]->(m) \
//!      RETURN p, m"
//! );
//! ```
//!
//! ### Merge: create if not exists
//!
//! ```cypher
//! MERGE (p:Person {name: 'Tom Hanks'})
//! ON CREATE SET p.born = 1956
//! RETURN p
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let p = node("Person").named("p").with_properties(props!("name" => "Tom Hanks"));
//!
//! let stmt = Cypher::merge(p)
//!     .on_create(vec![SetItem::property(prop("p", "born"), 1956_i32)])
//!     .returning(name("p"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MERGE (p:`Person` {name: 'Tom Hanks'}) \
//!      ON CREATE SET p.born = 1956 \
//!      RETURN p"
//! );
//! ```
//!
//! ### Update properties with SET
//!
//! ```cypher
//! MATCH (m:Movie {title: 'The Matrix'})
//! SET m.tagline = 'Welcome to the Real World'
//! RETURN m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let m = node("Movie").named("m").with_properties(props!("title" => "The Matrix"));
//!
//! let stmt = Cypher::match_(m)
//!     .set(vec![SetItem::property(prop("m", "tagline"), "Welcome to the Real World")])
//!     .returning(name("m"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (m:`Movie` {title: 'The Matrix'}) \
//!      SET m.tagline = 'Welcome to the Real World' \
//!      RETURN m"
//! );
//! ```
//!
//! ### Delete a node
//!
//! ```cypher
//! MATCH (m:Movie {title: 'New Movie'}) DETACH DELETE m
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let m = node("Movie").named("m").with_properties(props!("title" => "New Movie"));
//!
//! let stmt = Cypher::match_(m)
//!     .detach_delete(name("m"))
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (m:`Movie` {title: 'New Movie'}) DETACH DELETE m"
//! );
//! ```
//!
//! ---
//!
//! ## Multi-Part Queries (WITH)
//!
//! ### Pipeline: find prolific actors
//!
//! ```cypher
//! MATCH (p:Person)-[:ACTED_IN]->(m:Movie)
//! WITH p, count(m) AS movieCount
//! WHERE movieCount > 5
//! RETURN p.name, movieCount
//! ORDER BY movieCount DESC
//! ```
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! use rust_cypher_dsl::functions::aggregate;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m");
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .with((name("p"), aggregate::count(name("m")).alias("movieCount")))
//!     .where_(name("movieCount").gt(5_i32))
//!     .returning((Expression::from(prop("p", "name")), name("movieCount")))
//!     .order_by(name("movieCount").descending())
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`) \
//!      WITH p, count(m) AS movieCount \
//!      WHERE movieCount > 5 \
//!      RETURN p.name, movieCount \
//!      ORDER BY movieCount DESC"
//! );
//! ```
//!
//! ---
//!
//! ## Administration
//!
//! ### Create an index
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::create_index("movie_title")
//!     .for_node("m", "Movie", vec!["title"])
//!     .build();
//!
//! assert_eq!(
//!     stmt.render(),
//!     "CREATE INDEX movie_title FOR (m:Movie) ON (m.title)"
//! );
//! ```
//!
//! ### Create a uniqueness constraint
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::create_constraint("unique_movie_title")
//!     .for_node("m", "Movie")
//!     .is_unique(vec!["title"]);
//!
//! assert_eq!(
//!     stmt.render(),
//!     "CREATE CONSTRAINT unique_movie_title FOR (m:Movie) REQUIRE m.title IS UNIQUE"
//! );
//! ```
//!
//! ### Show indexes with filter
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//!
//! let stmt = Cypher::show_indexes()
//!     .filter(IndexFilter::Range)
//!     .yield_all()
//!     .build();
//!
//! assert_eq!(stmt.render(), "SHOW RANGE INDEXES YIELD *");
//! ```
//!
//! ---
//!
//! ## Pretty Printing
//!
//! All examples above use single-line rendering. For multi-line output:
//!
//! ```rust
//! use rust_cypher_dsl::prelude::*;
//! use rust_cypher_dsl::renderer::RenderConfig;
//! use rust_cypher_dsl::functions::aggregate;
//!
//! let p = node("Person").named("p");
//! let m = node("Movie").named("m");
//!
//! let stmt = Cypher::match_(p >> rel("ACTED_IN") >> m)
//!     .returning((
//!         Expression::from(prop("p", "name")),
//!         aggregate::count(name("m")).alias("movies"),
//!     ))
//!     .order_by(name("movies").descending())
//!     .build();
//!
//! let pretty = stmt.render_with(RenderConfig {
//!     pretty_print: true,
//!     ..RenderConfig::default()
//! });
//!
//! assert_eq!(pretty, "\
//! MATCH (p:`Person`)-[:`ACTED_IN`]->(m:`Movie`)
//! RETURN p.name, count(m) AS movies
//! ORDER BY movies DESC");
//! ```
