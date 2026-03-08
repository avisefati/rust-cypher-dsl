//! Aggregation functions: count, sum, avg, min, max, collect, etc.

use crate::types::expression::Expression;

/// `count(expr)` - counts non-null values.
pub fn count(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("count", vec![expr.into()])
}

/// `count(DISTINCT expr)` - counts distinct non-null values.
pub fn count_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("count", vec![expr.into()])
}

/// `sum(expr)` - sums numeric values.
pub fn sum(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("sum", vec![expr.into()])
}

/// `sum(DISTINCT expr)` - sums distinct numeric values.
pub fn sum_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("sum", vec![expr.into()])
}

/// `avg(expr)` - averages numeric values.
pub fn avg(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("avg", vec![expr.into()])
}

/// `avg(DISTINCT expr)` - averages distinct numeric values.
pub fn avg_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("avg", vec![expr.into()])
}

/// `min(expr)` - returns the minimum value.
pub fn min(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("min", vec![expr.into()])
}

/// `min(DISTINCT expr)` - returns the minimum of distinct values.
pub fn min_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("min", vec![expr.into()])
}

/// `max(expr)` - returns the maximum value.
pub fn max(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("max", vec![expr.into()])
}

/// `max(DISTINCT expr)` - returns the maximum of distinct values.
pub fn max_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("max", vec![expr.into()])
}

/// `collect(expr)` - collects values into a list.
pub fn collect(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("collect", vec![expr.into()])
}

/// `collect(DISTINCT expr)` - collects distinct values into a list.
pub fn collect_distinct(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation_distinct("collect", vec![expr.into()])
}

/// `percentileCont(expr, percentile)` - continuous percentile.
pub fn percentile_cont(
    expr: impl Into<Expression>,
    percentile: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("percentileCont", vec![expr.into(), percentile.into()])
}

/// `percentileDisc(expr, percentile)` - discrete percentile.
pub fn percentile_disc(
    expr: impl Into<Expression>,
    percentile: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("percentileDisc", vec![expr.into(), percentile.into()])
}

/// `stDev(expr)` - standard deviation (sample).
pub fn st_dev(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("stDev", vec![expr.into()])
}

/// `stDevP(expr)` - standard deviation (population).
pub fn st_dev_p(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("stDevP", vec![expr.into()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statement::{SinglePartQuery, Statement};
    use crate::clauses::{Clause, MatchClause, ReturnClause};
    use crate::types::node::node;

    #[test]
    fn render_count() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![count(Expression::symbolic_name("n"))])),
        ]));
        assert_eq!(stmt.render(), "MATCH (n:`Person`) RETURN count(n)");
    }

    #[test]
    fn render_count_distinct() {
        let n = node("Person").named("n");
        let stmt = Statement::SinglePart(SinglePartQuery::new(vec![
            Clause::Match(MatchClause::new(n)),
            Clause::Return(ReturnClause::new(vec![
                count_distinct(Expression::symbolic_name("n")),
            ])),
        ]));
        assert_eq!(
            stmt.render(),
            "MATCH (n:`Person`) RETURN count(DISTINCT n)"
        );
    }

    #[test]
    fn render_sum() {
        let expr = Expression::from(Expression::symbolic_name("n").property("amount"));
        assert_eq!(render_fn_expr(&sum(expr)), "sum(n.amount)");
    }

    #[test]
    fn render_sum_distinct() {
        let expr = Expression::from(Expression::symbolic_name("n").property("amount"));
        assert_eq!(render_fn_expr(&sum_distinct(expr)), "sum(DISTINCT n.amount)");
    }

    #[test]
    fn render_avg() {
        let expr = Expression::from(Expression::symbolic_name("n").property("score"));
        assert_eq!(render_fn_expr(&avg(expr)), "avg(n.score)");
    }

    #[test]
    fn render_avg_distinct() {
        let expr = Expression::from(Expression::symbolic_name("n").property("score"));
        assert_eq!(render_fn_expr(&avg_distinct(expr)), "avg(DISTINCT n.score)");
    }

    #[test]
    fn render_min() {
        assert_eq!(render_fn_expr(&min(Expression::symbolic_name("n"))), "min(n)");
    }

    #[test]
    fn render_min_distinct() {
        assert_eq!(
            render_fn_expr(&min_distinct(Expression::symbolic_name("n"))),
            "min(DISTINCT n)"
        );
    }

    #[test]
    fn render_max() {
        assert_eq!(render_fn_expr(&max(Expression::symbolic_name("n"))), "max(n)");
    }

    #[test]
    fn render_max_distinct() {
        assert_eq!(
            render_fn_expr(&max_distinct(Expression::symbolic_name("n"))),
            "max(DISTINCT n)"
        );
    }

    #[test]
    fn render_collect() {
        assert_eq!(
            render_fn_expr(&collect(Expression::symbolic_name("n"))),
            "collect(n)"
        );
    }

    #[test]
    fn render_collect_distinct() {
        assert_eq!(
            render_fn_expr(&collect_distinct(Expression::symbolic_name("n"))),
            "collect(DISTINCT n)"
        );
    }

    #[test]
    fn render_percentile_cont() {
        assert_eq!(
            render_fn_expr(&percentile_cont(
                Expression::from(Expression::symbolic_name("n").property("age")),
                Expression::from(0.5_f64),
            )),
            "percentileCont(n.age, 0.5)"
        );
    }

    #[test]
    fn render_percentile_disc() {
        assert_eq!(
            render_fn_expr(&percentile_disc(
                Expression::from(Expression::symbolic_name("n").property("age")),
                Expression::from(0.5_f64),
            )),
            "percentileDisc(n.age, 0.5)"
        );
    }

    #[test]
    fn render_st_dev() {
        assert_eq!(render_fn_expr(&st_dev(Expression::symbolic_name("x"))), "stDev(x)");
    }

    #[test]
    fn render_st_dev_p() {
        assert_eq!(render_fn_expr(&st_dev_p(Expression::symbolic_name("x"))), "stDevP(x)");
    }

    /// Helper: renders a single expression using the default renderer.
    fn render_fn_expr(expr: &Expression) -> String {
        let renderer = crate::renderer::default::DefaultRenderer::with_defaults();
        let mut buf = String::new();
        renderer.write_expression(&mut buf, expr);
        buf
    }
}
