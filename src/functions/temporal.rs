//! Temporal functions: datetime, date, time, duration, and related utilities.

use std::borrow::Cow;

use crate::types::expression::Expression;

// ── Core temporal constructors ───────────────────────────────────────────────

/// `datetime(map)` - creates a datetime from a map. Call with no args for current datetime.
pub fn datetime_fn(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("datetime", args)
}

/// `localdatetime(map)` - creates a local datetime.
pub fn localdatetime(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("localdatetime", args)
}

/// `date(map)` - creates a date.
pub fn date_fn(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("date", args)
}

/// `localtime(map)` - creates a local time.
pub fn localtime(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("localtime", args)
}

/// `time(map)` - creates a time with timezone.
pub fn time_fn(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("time", args)
}

/// `duration(map)` - creates a duration from a map.
pub fn duration_fn(args: Vec<Expression>) -> Expression {
    Expression::function_invocation("duration", args)
}

// ── Duration utilities ───────────────────────────────────────────────────────

/// `duration.between(instant1, instant2)` - returns the duration between two instants.
pub fn duration_between(
    from: impl Into<Expression>,
    to: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("duration.between", vec![from.into(), to.into()])
}

/// `duration.inDays(from, to)` - returns the duration in days.
pub fn duration_in_days(
    from: impl Into<Expression>,
    to: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("duration.inDays", vec![from.into(), to.into()])
}

/// `duration.inMonths(from, to)` - returns the duration in months.
pub fn duration_in_months(
    from: impl Into<Expression>,
    to: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("duration.inMonths", vec![from.into(), to.into()])
}

/// `duration.inSeconds(from, to)` - returns the duration in seconds.
pub fn duration_in_seconds(
    from: impl Into<Expression>,
    to: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("duration.inSeconds", vec![from.into(), to.into()])
}

// ── Epoch ────────────────────────────────────────────────────────────────────

/// `datetime({epochSeconds: seconds})` - creates a datetime from epoch seconds.
pub fn datetime_from_epoch(seconds: impl Into<Expression>) -> Expression {
    Expression::function_invocation("datetime", vec![
        Expression::map_literal(vec![(Cow::Borrowed("epochSeconds"), seconds.into())]),
    ])
}

/// `datetime({epochMillis: millis})` - creates a datetime from epoch milliseconds.
pub fn datetime_from_epoch_millis(millis: impl Into<Expression>) -> Expression {
    Expression::function_invocation("datetime", vec![
        Expression::map_literal(vec![(Cow::Borrowed("epochMillis"), millis.into())]),
    ])
}

// ── Qualified method variants ────────────────────────────────────────────────

/// `datetime.realtime()` - returns the real-time clock datetime.
pub fn datetime_realtime() -> Expression {
    Expression::function_invocation("datetime.realtime", vec![])
}

/// `datetime.statement()` - returns the statement-time clock datetime.
pub fn datetime_statement() -> Expression {
    Expression::function_invocation("datetime.statement", vec![])
}

/// `datetime.transaction()` - returns the transaction-time clock datetime.
pub fn datetime_transaction() -> Expression {
    Expression::function_invocation("datetime.transaction", vec![])
}

/// `datetime.truncate(unit, input)` - truncates to the given temporal unit.
pub fn datetime_truncate(
    unit: impl Into<Expression>,
    input: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("datetime.truncate", vec![unit.into(), input.into()])
}

/// `date.realtime()` - returns the real-time clock date.
pub fn date_realtime() -> Expression {
    Expression::function_invocation("date.realtime", vec![])
}

/// `date.statement()` - returns the statement-time clock date.
pub fn date_statement() -> Expression {
    Expression::function_invocation("date.statement", vec![])
}

/// `date.transaction()` - returns the transaction-time clock date.
pub fn date_transaction() -> Expression {
    Expression::function_invocation("date.transaction", vec![])
}

/// `date.truncate(unit, input)` - truncates a date to the given unit.
pub fn date_truncate(
    unit: impl Into<Expression>,
    input: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("date.truncate", vec![unit.into(), input.into()])
}

/// `time.realtime()` - returns the real-time clock time.
pub fn time_realtime() -> Expression {
    Expression::function_invocation("time.realtime", vec![])
}

/// `time.statement()` - returns the statement-time clock time.
pub fn time_statement() -> Expression {
    Expression::function_invocation("time.statement", vec![])
}

/// `time.transaction()` - returns the transaction-time clock time.
pub fn time_transaction() -> Expression {
    Expression::function_invocation("time.transaction", vec![])
}

/// `time.truncate(unit, input)` - truncates a time to the given unit.
pub fn time_truncate(
    unit: impl Into<Expression>,
    input: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("time.truncate", vec![unit.into(), input.into()])
}

/// `localdatetime.realtime()` - returns the real-time clock local datetime.
pub fn localdatetime_realtime() -> Expression {
    Expression::function_invocation("localdatetime.realtime", vec![])
}

/// `localdatetime.statement()` - returns the statement-time clock local datetime.
pub fn localdatetime_statement() -> Expression {
    Expression::function_invocation("localdatetime.statement", vec![])
}

/// `localdatetime.transaction()` - returns the transaction-time clock local datetime.
pub fn localdatetime_transaction() -> Expression {
    Expression::function_invocation("localdatetime.transaction", vec![])
}

/// `localdatetime.truncate(unit, input)` - truncates a local datetime.
pub fn localdatetime_truncate(
    unit: impl Into<Expression>,
    input: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("localdatetime.truncate", vec![unit.into(), input.into()])
}

/// `localtime.realtime()` - returns the real-time clock local time.
pub fn localtime_realtime() -> Expression {
    Expression::function_invocation("localtime.realtime", vec![])
}

/// `localtime.statement()` - returns the statement-time clock local time.
pub fn localtime_statement() -> Expression {
    Expression::function_invocation("localtime.statement", vec![])
}

/// `localtime.transaction()` - returns the transaction-time clock local time.
pub fn localtime_transaction() -> Expression {
    Expression::function_invocation("localtime.transaction", vec![])
}

/// `localtime.truncate(unit, input)` - truncates a local time.
pub fn localtime_truncate(
    unit: impl Into<Expression>,
    input: impl Into<Expression>,
) -> Expression {
    Expression::function_invocation("localtime.truncate", vec![unit.into(), input.into()])
}

// ── Format ───────────────────────────────────────────────────────────────────

/// `toString(temporal)` - formats a temporal value as a string.
///
/// Re-uses the generic `toString` function from the scalar module.
/// This is an alias provided for discoverability in temporal contexts.
pub fn format_fn(expr: impl Into<Expression>) -> Expression {
    Expression::function_invocation("toString", vec![expr.into()])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(expr: &Expression) -> String {
        let r = crate::renderer::default::DefaultRenderer::with_defaults();
        let mut buf = String::new();
        r.write_expression(&mut buf, expr);
        buf
    }

    #[test]
    fn render_datetime_no_args() {
        assert_eq!(render(&datetime_fn(vec![])), "datetime()");
    }

    #[test]
    fn render_datetime_with_arg() {
        assert_eq!(
            render(&datetime_fn(vec![Expression::symbolic_name("m")])),
            "datetime(m)"
        );
    }

    #[test]
    fn render_localdatetime_no_args() {
        assert_eq!(render(&localdatetime(vec![])), "localdatetime()");
    }

    #[test]
    fn render_date_fn() {
        assert_eq!(render(&date_fn(vec![])), "date()");
    }

    #[test]
    fn render_localtime() {
        assert_eq!(render(&localtime(vec![])), "localtime()");
    }

    #[test]
    fn render_time_fn() {
        assert_eq!(render(&time_fn(vec![])), "time()");
    }

    #[test]
    fn render_duration_fn() {
        assert_eq!(render(&duration_fn(vec![])), "duration()");
    }

    #[test]
    fn render_duration_between() {
        assert_eq!(
            render(&duration_between(
                Expression::symbolic_name("d1"),
                Expression::symbolic_name("d2"),
            )),
            "duration.between(d1, d2)"
        );
    }

    #[test]
    fn render_duration_in_days() {
        assert_eq!(
            render(&duration_in_days(
                Expression::symbolic_name("d1"),
                Expression::symbolic_name("d2"),
            )),
            "duration.inDays(d1, d2)"
        );
    }

    #[test]
    fn render_duration_in_months() {
        assert_eq!(
            render(&duration_in_months(
                Expression::symbolic_name("d1"),
                Expression::symbolic_name("d2"),
            )),
            "duration.inMonths(d1, d2)"
        );
    }

    #[test]
    fn render_duration_in_seconds() {
        assert_eq!(
            render(&duration_in_seconds(
                Expression::symbolic_name("d1"),
                Expression::symbolic_name("d2"),
            )),
            "duration.inSeconds(d1, d2)"
        );
    }

    #[test]
    fn render_datetime_from_epoch() {
        assert_eq!(
            render(&datetime_from_epoch(Expression::from(1_000_000_i32))),
            "datetime({epochSeconds: 1000000})"
        );
    }

    #[test]
    fn render_datetime_from_epoch_millis() {
        assert_eq!(
            render(&datetime_from_epoch_millis(Expression::symbolic_name("ms"))),
            "datetime({epochMillis: ms})"
        );
    }

    #[test]
    fn render_datetime_realtime() {
        assert_eq!(render(&datetime_realtime()), "datetime.realtime()");
    }

    #[test]
    fn render_datetime_statement() {
        assert_eq!(render(&datetime_statement()), "datetime.statement()");
    }

    #[test]
    fn render_datetime_transaction() {
        assert_eq!(render(&datetime_transaction()), "datetime.transaction()");
    }

    #[test]
    fn render_datetime_truncate() {
        assert_eq!(
            render(&datetime_truncate(Expression::from("day"), Expression::symbolic_name("d"))),
            "datetime.truncate('day', d)"
        );
    }

    #[test]
    fn render_date_clock_variants() {
        assert_eq!(render(&date_realtime()), "date.realtime()");
        assert_eq!(render(&date_statement()), "date.statement()");
        assert_eq!(render(&date_transaction()), "date.transaction()");
    }

    #[test]
    fn render_date_truncate() {
        assert_eq!(
            render(&date_truncate(Expression::from("month"), Expression::symbolic_name("d"))),
            "date.truncate('month', d)"
        );
    }

    #[test]
    fn render_time_clock_variants() {
        assert_eq!(render(&time_realtime()), "time.realtime()");
        assert_eq!(render(&time_statement()), "time.statement()");
        assert_eq!(render(&time_transaction()), "time.transaction()");
    }

    #[test]
    fn render_time_truncate() {
        assert_eq!(
            render(&time_truncate(Expression::from("second"), Expression::symbolic_name("t"))),
            "time.truncate('second', t)"
        );
    }

    #[test]
    fn render_localdatetime_clock_variants() {
        assert_eq!(render(&localdatetime_realtime()), "localdatetime.realtime()");
        assert_eq!(render(&localdatetime_statement()), "localdatetime.statement()");
        assert_eq!(render(&localdatetime_transaction()), "localdatetime.transaction()");
    }

    #[test]
    fn render_localdatetime_truncate() {
        assert_eq!(
            render(&localdatetime_truncate(Expression::from("day"), Expression::symbolic_name("dt"))),
            "localdatetime.truncate('day', dt)"
        );
    }

    #[test]
    fn render_localtime_clock_variants() {
        assert_eq!(render(&localtime_realtime()), "localtime.realtime()");
        assert_eq!(render(&localtime_statement()), "localtime.statement()");
        assert_eq!(render(&localtime_transaction()), "localtime.transaction()");
    }

    #[test]
    fn render_localtime_truncate() {
        assert_eq!(
            render(&localtime_truncate(Expression::from("minute"), Expression::symbolic_name("t"))),
            "localtime.truncate('minute', t)"
        );
    }

    #[test]
    fn render_format_fn() {
        assert_eq!(
            render(&format_fn(Expression::symbolic_name("d"))),
            "toString(d)"
        );
    }
}
