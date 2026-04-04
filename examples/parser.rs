//! Real-world query example: build a complex multi-WITH query using the DSL,
//! then round-trip it through the parser.
//!
//! Run with: `cargo run --example parser`

use rust_cypher_dsl::functions::aggregate::{collect_distinct, count_distinct, sum};
use rust_cypher_dsl::functions::scalar::{coalesce, size};
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::types::expression::case_when;

/// Builds a course-enrollment analytics query with dynamic labels.
///
/// Finds all students enrolled in a course (directly or via study groups),
/// deduplicates them, then cross-references plagiarism incidents.
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (course:Course {department_id: $departmentId, id: $courseId})
/// OPTIONAL MATCH (directStudent:Student {department_id: $departmentId})
///   -[:ENROLLED_IN]->(course)
/// OPTIONAL MATCH (groupStudent:Student {department_id: $departmentId})
///   -[:BELONGS_TO]->(sg:StudyGroup {department_id: $departmentId})
///   -[:ENROLLED_IN]->(course)
/// WITH course,
///      collect(DISTINCT directStudent) + collect(DISTINCT groupStudent) AS combined
/// UNWIND CASE WHEN size(combined) = 0 THEN [null] ELSE combined END AS s
/// WITH course, [x IN collect(DISTINCT s) WHERE x IS NOT NULL] AS studentList
/// WITH size(studentList) AS totalStudents
/// OPTIONAL MATCH (inc:Incident {department_id: $departmentId, type: 'PLAGIARISM'})
/// WHERE inc.description CONTAINS $incidentKeyword
/// WITH totalStudents, count(DISTINCT inc) AS plagiarismCases
/// RETURN totalStudents, plagiarismCases, 0 AS exemptStudents
/// ```
fn build_enrollment_analytics_query(
    course_label: &str,
    student_label: &str,
    group_label: &str,
    incident_label: &str,
) -> Statement {
    // --- Node patterns ---
    let course = node(course_label.to_owned())
        .named("course")
        .with_properties(props! {
            "department_id" => param("departmentId"),
            "id" => param("courseId")
        });

    let direct_student = node(student_label.to_owned())
        .named("directStudent")
        .with_properties(props! { "department_id" => param("departmentId") });

    let group_student = node(student_label.to_owned())
        .named("groupStudent")
        .with_properties(props! { "department_id" => param("departmentId") });

    let study_group = node(group_label.to_owned())
        .named("sg")
        .with_properties(props! { "department_id" => param("departmentId") });

    // Bare named node for re-referencing course in OPTIONAL MATCH targets.
    let course_ref = any_node_named("course");

    let incident = node(incident_label.to_owned())
        .named("inc")
        .with_properties(props! {
            "department_id" => param("departmentId"),
            "type" => lit("PLAGIARISM")
        });

    // --- Relationship patterns ---
    // (directStudent)-[:ENROLLED_IN]->(course)
    let direct_pattern = direct_student >> rel("ENROLLED_IN") >> course_ref.clone();

    // (groupStudent)-[:BELONGS_TO]->(sg)-[:ENROLLED_IN]->(course)
    let group_pattern = group_student >> rel("BELONGS_TO") >> study_group >> rel("ENROLLED_IN") >> course_ref;

    // --- Expressions ---
    // collect(DISTINCT directStudent) + collect(DISTINCT groupStudent) AS combined
    let combined = collect_distinct(name("directStudent"))
        .add(collect_distinct(name("groupStudent")))
        .alias("combined");

    // CASE WHEN size(combined) = 0 THEN [null] ELSE combined END
    let unwind_expr = case_when(size(name("combined")).eq(lit(0_i64)))
        .then(Expression::list_literal(vec![lit_null()]))
        .else_(name("combined"));

    // [x IN collect(DISTINCT s) WHERE x IS NOT NULL] AS studentList
    let student_list = Expression::list_comprehension(
        "x",
        collect_distinct(name("s")),
        Some(Expression::from(name("x").is_not_null())),
        None,
    )
    .alias("studentList");

    // --- Build the query ---
    Cypher::match_(course)
        .optional_match(direct_pattern)
        .optional_match(group_pattern)
        .with((name("course"), combined))
        .unwind(unwind_expr)
        .as_("s")
        .with((name("course"), student_list))
        .with(size(name("studentList")).alias("totalStudents"))
        .optional_match(incident)
        .where_(Expression::from(prop("inc", "description")).contains(param("incidentKeyword")))
        .with((
            name("totalStudents"),
            count_distinct(name("inc")).alias("plagiarismCases"),
        ))
        .returning((
            name("totalStudents"),
            name("plagiarismCases"),
            lit(0_i64).alias("exemptStudents"),
        ))
        .build()
}

/// Builds a shared-reading analytics query with dynamic labels.
///
/// Finds pairs of readers who borrowed the same books (incoming
/// relationship pattern), aggregates the overlap, and returns the top
/// pairs by shared count.
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (r1:Reader)-[:BORROWED]->(b:Book)<-[:BORROWED]-(r2:Reader)
/// WHERE r1.id <> r2.id
/// WITH r1, r2, count(DISTINCT b) AS sharedCount
/// WHERE sharedCount > 0
/// WITH COALESCE(r1.name, r1.id, '') AS readerName, sum(sharedCount) AS totalShared
/// RETURN readerName, totalShared AS sharedCount
/// ORDER BY totalShared DESC
/// LIMIT $limit
/// ```
fn build_shared_reading_query(
    reader_label: &str,
    book_label: &str,
) -> Statement {
    let r1 = node(reader_label.to_owned()).named("r1");
    let b = node(book_label.to_owned()).named("b");
    let r2 = node(reader_label.to_owned()).named("r2");

    // (r1)-[:BORROWED]->(b)<-[:BORROWED]-(r2)
    let pattern = r1 >> rel("BORROWED") >> b << rel("BORROWED") << r2;

    Cypher::match_(pattern)
        // WHERE r1.id <> r2.id
        .where_(prop("r1", "id").ne(prop("r2", "id")))
        // WITH r1, r2, count(DISTINCT b) AS sharedCount
        .with((
            name("r1"),
            name("r2"),
            count_distinct(name("b")).alias("sharedCount"),
        ))
        // WHERE sharedCount > 0
        .where_(name("sharedCount").gt(lit(0_i64)))
        // WITH COALESCE(r1.name, r1.id, '') AS readerName, sum(sharedCount) AS totalShared
        .with((
            coalesce(vec![
                Expression::from(prop("r1", "name")),
                Expression::from(prop("r1", "id")),
                lit(""),
            ])
            .alias("readerName"),
            sum(name("sharedCount")).alias("totalShared"),
        ))
        // RETURN readerName, totalShared AS sharedCount
        .returning((
            name("readerName"),
            name("totalShared").alias("sharedCount"),
        ))
        // ORDER BY totalShared DESC
        .order_by(name("totalShared").descending())
        // LIMIT $limit
        .limit(param("limit"))
        .build()
}

fn round_trip(label: &str, stmt: &Statement) {
    let cypher = stmt.render();
    println!("[{label}] Generated Cypher:\n{cypher}\n");

    match rust_cypher_dsl::parser::parse(&cypher) {
        Ok(parsed) => {
            let round_tripped = parsed.render();
            assert_eq!(cypher, round_tripped, "round-trip must be identical");
            println!("[{label}] Round-trip OK\n");
        }
        Err(e) => {
            eprintln!("[{label}] Parser failed: {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    println!("=== Builder — Real-world Cypher DSL Examples ===\n");

    // Query 1: enrollment analytics (multi-WITH, CASE, list comprehension).
    let q1 = build_enrollment_analytics_query("Course", "Student", "StudyGroup", "Incident");
    round_trip("enrollment", &q1);

    // Query 2: shared-reading analytics (incoming rel, COALESCE, WHERE after WITH).
    let q2 = build_shared_reading_query("Reader", "Book");
    round_trip("shared-reading", &q2);
}
