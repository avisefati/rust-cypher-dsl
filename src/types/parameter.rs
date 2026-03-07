//! `Parameter` type for Cypher `$name` references.
//!
//! Parameters represent named placeholders in Cypher queries,
//! rendered as `$paramName`.

use std::borrow::Cow;

use super::expression::Expression;

/// A named parameter reference (`$name`) in a Cypher query.
///
/// Parameters can optionally carry a bound value for use in
/// query execution. The bound value does not affect rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter name (without the `$` prefix).
    pub(crate) name: Cow<'static, str>,
    /// An optional bound value for query execution.
    pub(crate) value: Option<Expression>,
}

impl Parameter {
    /// Creates a new parameter with the given name.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }

    /// Creates a parameter with a bound value.
    pub fn with_value(name: impl Into<Cow<'static, str>>, value: impl Into<Expression>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
        }
    }

    /// Returns the parameter name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the bound value, if any.
    pub const fn value(&self) -> Option<&Expression> {
        self.value.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_parameter_without_value() {
        let param = Parameter::new("name");
        assert_eq!(param.name(), "name");
        assert!(param.value().is_none());
    }

    #[test]
    fn with_value_creates_parameter_with_bound_value() {
        let param = Parameter::with_value("age", 25_i32);
        assert_eq!(param.name(), "age");
        assert!(param.value().is_some());
    }

    #[test]
    fn name_returns_name_without_dollar_prefix() {
        let param = Parameter::new("userId");
        assert_eq!(param.name(), "userId");
    }

    #[test]
    fn static_str_and_string_both_work() {
        let static_param = Parameter::new("static");
        let owned_param = Parameter::new(String::from("owned"));
        assert_eq!(static_param.name(), "static");
        assert_eq!(owned_param.name(), "owned");
    }

    #[test]
    fn clone_preserves_value() {
        let param = Parameter::with_value("x", 42_i32);
        let cloned = param.clone();
        assert_eq!(param, cloned);
    }
}
