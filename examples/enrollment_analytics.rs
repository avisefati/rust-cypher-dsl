//! Course-enrollment analytics query with multi-WITH, CASE WHEN, list
//! comprehension, UNWIND, and OPTIONAL MATCH chains.
//!
//! Demonstrates: consecutive WITH clauses, generic CASE, collect(DISTINCT),
//! list comprehension with IS NOT NULL filter, CONTAINS, and dynamic labels.
//!
//! Run with: `cargo run --example enrollment_analytics`

use rust_cypher_dsl::functions::aggregate::{collect_distinct, count_distinct};
use rust_cypher_dsl::functions::scalar::size;
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
    let group_pattern =
        group_student >> rel("BELONGS_TO") >> study_group >> rel("ENROLLED_IN") >> course_ref;

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

fn main() {
    let stmt = build_enrollment_analytics_query("Course", "Student", "StudyGroup", "Incident");
    let cypher = stmt.render();
    println!("Generated Cypher:\n{cypher}\n");

    match rust_cypher_dsl::parser::parse(&cypher) {
        Ok(parsed) => {
            let round_tripped = parsed.render();
            assert_eq!(cypher, round_tripped, "round-trip must be identical");
            println!("Round-trip OK");
        }
        Err(e) => {
            eprintln!("Parser failed: {e}");
            std::process::exit(1);
        }
    }
}
