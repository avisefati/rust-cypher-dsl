//! `Parameter` type for Cypher `$name` references.
//!
//! Parameters represent named placeholders in Cypher queries,
//! rendered as `$paramName`.

use std::borrow::Cow;

use super::expression::Expression;

/// Returns `true` if the name is a valid Cypher identifier.
///
/// A valid identifier starts with an ASCII letter or underscore and
/// contains only ASCII alphanumeric characters or underscores.
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        unreachable!("guarded by is_empty check above");
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// A named parameter reference (`$name`) in a Cypher query.
///
/// Parameters can optionally carry a bound value for use in
/// query execution. The bound value does not affect rendering.
///
/// # Panics
///
/// Construction panics if the parameter name is not a valid
/// Cypher identifier (`[a-zA-Z_][a-zA-Z0-9_]*`).
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter name (without the `$` prefix).
    pub(crate) name: Cow<'static, str>,
    /// An optional bound value for query execution.
    pub(crate) value: Option<Expression>,
}

impl Parameter {
    /// Creates a new parameter with the given name.
    ///
    /// # Panics
    ///
    /// Panics if `name` is not a valid Cypher identifier
    /// (`[a-zA-Z_][a-zA-Z0-9_]*`).
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        assert!(
            is_valid_identifier(&name),
            "invalid parameter name `{name}`: must match [a-zA-Z_][a-zA-Z0-9_]*"
        );
        Self { name, value: None }
    }

    /// Creates a parameter with a bound value.
    ///
    /// # Panics
    ///
    /// Panics if `name` is not a valid Cypher identifier
    /// (`[a-zA-Z_][a-zA-Z0-9_]*`).
    pub fn with_value(name: impl Into<Cow<'static, str>>, value: impl Into<Expression>) -> Self {
        let name = name.into();
        assert!(
            is_valid_identifier(&name),
            "invalid parameter name `{name}`: must match [a-zA-Z_][a-zA-Z0-9_]*"
        );
        Self {
            name,
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

// ---------------------------------------------------------------------------
// Free functions for ergonomic parameter construction
// ---------------------------------------------------------------------------

/// Creates a parameter reference (`$name`).
///
/// Shorthand for `Parameter::new(n)`.
pub fn param(n: impl Into<Cow<'static, str>>) -> Parameter {
    Parameter::new(n)
}

/// Creates a parameter with a bound value.
///
/// Shorthand for `Parameter::with_value(n, value)`.
pub fn param_with_value(
    n: impl Into<Cow<'static, str>>,
    value: impl Into<Expression>,
) -> Parameter {
    Parameter::with_value(n, value)
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

    #[test]
    fn underscore_prefix_accepted() {
        let param = Parameter::new("_internal");
        assert_eq!(param.name(), "_internal");
    }

    #[test]
    #[should_panic(expected = "invalid parameter name")]
    fn rejects_name_with_spaces() {
        let _param = Parameter::new("bad name");
    }

    #[test]
    #[should_panic(expected = "invalid parameter name")]
    fn rejects_name_with_special_chars() {
        let _param = Parameter::new("$injected");
    }

    #[test]
    #[should_panic(expected = "invalid parameter name")]
    fn rejects_empty_name() {
        let _param = Parameter::new("");
    }

    #[test]
    #[should_panic(expected = "invalid parameter name")]
    fn rejects_injection_payload() {
        let _param = Parameter::new("x} RETURN 1 //");
    }
}
