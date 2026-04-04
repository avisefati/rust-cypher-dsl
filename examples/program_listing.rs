//! Academic-program listing query with CALL {} subqueries, simple CASE,
//! map literals, and dynamic WHERE filters.
//!
//! Demonstrates: `Cypher::with()` for fluent subquery construction,
//! `call_subquery()` accepting `Statement` directly, simple CASE expression,
//! map literal in RETURN, `toLower()` + CONTAINS + AND, and
//! ORDER BY / SKIP / LIMIT with parameters.
//!
//! Run with: `cargo run --example program_listing`

use rust_cypher_dsl::functions::aggregate::{count, min};
use rust_cypher_dsl::functions::string::to_lower;
use rust_cypher_dsl::prelude::*;
use rust_cypher_dsl::types::expression::case;

/// Builds an academic-program listing query with three CALL {} subqueries.
///
/// Lists programs managed by departments, enriching each program with
/// enrollment count, cohort count, and accreditation audit findings
/// (including a severity ranking via simple CASE).
///
/// **Note:** Pattern-in-WHERE (e.g. `WHERE (dept)-[:MANAGES]->(prog)`) is
/// not yet supported by the DSL; this query uses explicit MATCH patterns.
///
/// Equivalent Cypher:
/// ```cypher
/// MATCH (dept:Department)-[:MANAGES]->(prog:Program)
/// WHERE toLower(prog.name) CONTAINS toLower($nameFilter)
///   AND toLower(dept.name) CONTAINS toLower($deptFilter)
/// WITH dept, prog
/// CALL { WITH prog OPTIONAL MATCH (enrollee:Student)-[:ENROLLED_IN]->(prog)
///        RETURN count(enrollee) AS enrolleeCount }
/// CALL { WITH prog OPTIONAL MATCH (cohort:Cohort)-[:ASSIGNED_TO]->(prog)
///        RETURN count(cohort) AS cohortCount }
/// CALL { WITH prog OPTIONAL MATCH (finding:AuditFinding)-[:PERTAINS_TO]->(prog)
///        WITH finding, CASE finding.severity WHEN 'CRITICAL' THEN 1
///          WHEN 'HIGH' THEN 2 WHEN 'MEDIUM' THEN 3 WHEN 'LOW' THEN 4 ELSE 5 END
///          AS severityRank
///        WHERE finding IS NOT NULL
///        RETURN count(finding) AS findingCount, min(severityRank) AS worstSeverity }
/// WITH dept, prog, enrolleeCount, cohortCount, findingCount, worstSeverity
/// RETURN {id: prog.id, name: prog.name, department: dept.name,
///   enrollees: enrolleeCount, cohorts: cohortCount,
///   findings: findingCount, worstSeverity: worstSeverity} AS result
/// ORDER BY toLower(prog.name) ASC
/// SKIP $skip LIMIT $limit
/// ```
fn build_program_listing_query(
    department_label: &str,
    program_label: &str,
    student_label: &str,
    cohort_label: &str,
    finding_label: &str,
) -> Statement {
    // --- Main MATCH pattern ---
    let dept = node(department_label.to_owned()).named("dept");
    let prog = node(program_label.to_owned()).named("prog");
    let main_pattern = dept >> rel("MANAGES") >> prog;

    // --- CALL subquery 1: enrollee count (fluent builder) ---
    let prog_ref = any_node_named("prog");
    let enrollee = node(student_label.to_owned()).named("enrollee");
    let enrollee_pattern = enrollee >> rel("ENROLLED_IN") >> prog_ref.clone();

    let sub1 = Cypher::with(name("prog"))
        .optional_match(enrollee_pattern)
        .returning(count(name("enrollee")).alias("enrolleeCount"))
        .build();

    // --- CALL subquery 2: cohort count (fluent builder) ---
    let cohort = node(cohort_label.to_owned()).named("cohort");
    let cohort_pattern = cohort >> rel("ASSIGNED_TO") >> prog_ref.clone();

    let sub2 = Cypher::with(name("prog"))
        .optional_match(cohort_pattern)
        .returning(count(name("cohort")).alias("cohortCount"))
        .build();

    // --- CALL subquery 3: audit findings with severity ranking (fluent builder) ---
    let finding = node(finding_label.to_owned()).named("finding");
    let finding_pattern = finding >> rel("PERTAINS_TO") >> prog_ref;

    // CASE finding.severity WHEN 'CRITICAL' THEN 1 WHEN 'HIGH' THEN 2 ...
    let severity_rank = case(prop("finding", "severity"))
        .when(lit("CRITICAL"))
        .then(lit(1_i64))
        .when(lit("HIGH"))
        .then(lit(2_i64))
        .when(lit("MEDIUM"))
        .then(lit(3_i64))
        .when(lit("LOW"))
        .then(lit(4_i64))
        .else_(lit(5_i64))
        .alias("severityRank");

    let sub3 = Cypher::with(name("prog"))
        .optional_match(finding_pattern)
        .with((name("finding"), severity_rank))
        .where_(name("finding").is_not_null())
        .returning((
            count(name("finding")).alias("findingCount"),
            min(name("severityRank")).alias("worstSeverity"),
        ))
        .build();

    // --- Map literal for RETURN ---
    let result_map = map_of(vec![
        ("id".into(), Expression::from(prop("prog", "id"))),
        ("name".into(), Expression::from(prop("prog", "name"))),
        (
            "department".into(),
            Expression::from(prop("dept", "name")),
        ),
        ("enrollees".into(), name("enrolleeCount")),
        ("cohorts".into(), name("cohortCount")),
        ("findings".into(), name("findingCount")),
        ("worstSeverity".into(), name("worstSeverity")),
    ])
    .alias("result");

    // --- Build the full query ---
    Cypher::match_(main_pattern)
        .where_(
            to_lower(prop("prog", "name")).contains(to_lower(param("nameFilter"))),
        )
        .and(
            to_lower(prop("dept", "name")).contains(to_lower(param("deptFilter"))),
        )
        .with((name("dept"), name("prog")))
        .call_subquery(sub1)
        .call_subquery(sub2)
        .call_subquery(sub3)
        .with((
            name("dept"),
            name("prog"),
            name("enrolleeCount"),
            name("cohortCount"),
            name("findingCount"),
            name("worstSeverity"),
        ))
        .returning(result_map)
        .order_by(to_lower(prop("prog", "name")).ascending())
        .skip(param("skip"))
        .limit(param("limit"))
        .build()
}

fn main() {
    let stmt = build_program_listing_query(
        "Department",
        "Program",
        "Student",
        "Cohort",
        "AuditFinding",
    );
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
