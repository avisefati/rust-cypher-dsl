//! Parse error type with source position and context.

use std::fmt;

/// Parse error with source position and context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset in the input where the error occurred.
    pub offset: usize,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
    /// What the parser expected at this position.
    pub expected: Vec<String>,
    /// Parsing context stack (e.g., `["RETURN clause", "expression"]`).
    pub context: Vec<String>,
    /// The portion of input near the error.
    pub snippet: String,
}

impl ParseError {
    /// Creates a placeholder error for unimplemented parser paths.
    #[allow(dead_code, reason = "kept for backwards compatibility")]
    pub(crate) fn not_yet_implemented() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
            expected: vec!["parser not yet implemented".to_owned()],
            context: Vec::new(),
            snippet: String::new(),
        }
    }

    /// Creates a validation error for invalid clause ordering.
    ///
    /// These errors don't have a source position since they are detected
    /// after syntactic parsing, based on clause sequence analysis.
    #[allow(dead_code, reason = "used by validate module")]
    pub(crate) fn validation_error(clause: &str, after: &str, expected: Vec<String>) -> Self {
        Self {
            offset: 0,
            line: 0,
            column: 0,
            expected,
            context: vec![format!("{clause} clause cannot appear after {after}")],
            snippet: String::new(),
        }
    }

    /// Creates a `ParseError` from a byte offset in the original input.
    ///
    /// Computes line/column from the offset and extracts a snippet
    /// of the surrounding source text.
    #[allow(dead_code, reason = "will be used by parser grammar in subsequent tasks")]
    pub(crate) fn from_offset(input: &str, offset: usize, expected: Vec<String>, context: Vec<String>) -> Self {
        let (line, column) = offset_to_line_column(input, offset);
        let snippet = extract_snippet(input, offset);
        Self {
            offset,
            line,
            column,
            expected,
            context,
            snippet,
        }
    }
}

/// Converts a byte offset to a 1-based (line, column) pair.
#[allow(dead_code, reason = "will be used by parser grammar in subsequent tasks")]
fn offset_to_line_column(input: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(input.len());
    let before = &input[..clamped];
    let line = before.chars().filter(|&c| c == '\n').count() + 1;
    let last_newline = before.rfind('\n').map_or(0, |pos| pos + 1);
    let column = clamped - last_newline + 1;
    (line, column)
}

/// Extracts the line containing the error offset for display.
#[allow(dead_code, reason = "will be used by parser grammar in subsequent tasks")]
fn extract_snippet(input: &str, offset: usize) -> String {
    let clamped = offset.min(input.len());
    let line_start = input[..clamped].rfind('\n').map_or(0, |pos| pos + 1);
    let line_end = input[clamped..]
        .find('\n')
        .map_or(input.len(), |pos| clamped + pos);
    let line = &input[line_start..line_end];
    // Truncate very long lines for readability.
    if line.len() > 80 {
        format!("{}...", &line[..77])
    } else {
        line.to_owned()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}",
            self.line, self.column
        )?;
        if !self.snippet.is_empty() {
            write!(f, ":\n  {}\n  ", self.snippet)?;
            for _ in 0..self.column.saturating_sub(1) {
                write!(f, " ")?;
            }
            write!(f, "^")?;
        }
        if !self.expected.is_empty() {
            write!(f, "\nExpected: {}", self.expected.join(", "))?;
        }
        if !self.context.is_empty() {
            write!(f, "\nContext: {}", self.context.join(" > "))?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_to_line_column_first_char() {
        let (line, col) = offset_to_line_column("MATCH (n) RETURN n", 0);
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn offset_to_line_column_same_line() {
        // offset 6 = 'n' in "(n)"
        let (line, col) = offset_to_line_column("MATCH (n) RETURN n", 6);
        assert_eq!(line, 1);
        assert_eq!(col, 7);
    }

    #[test]
    fn offset_to_line_column_second_line() {
        let input = "MATCH (n)\nRETURN n";
        // offset 10 = start of "RETURN"
        let (line, col) = offset_to_line_column(input, 10);
        assert_eq!(line, 2);
        assert_eq!(col, 1);
    }

    #[test]
    fn offset_to_line_column_third_line() {
        let input = "MATCH (n)\nWHERE n.age > 21\nRETURN n";
        // offset 27 = start of "RETURN"
        let (line, col) = offset_to_line_column(input, 27);
        assert_eq!(line, 3);
        assert_eq!(col, 1);
    }

    #[test]
    fn offset_to_line_column_end_of_input() {
        let input = "MATCH (n)";
        let (line, col) = offset_to_line_column(input, input.len());
        assert_eq!(line, 1);
        assert_eq!(col, 10);
    }

    #[test]
    fn offset_to_line_column_beyond_input_clamps() {
        let input = "abc";
        let (line, col) = offset_to_line_column(input, 100);
        assert_eq!(line, 1);
        assert_eq!(col, 4);
    }

    #[test]
    fn extract_snippet_single_line() {
        let input = "MATCH (n) RETURN n";
        let snippet = extract_snippet(input, 6);
        assert_eq!(snippet, "MATCH (n) RETURN n");
    }

    #[test]
    fn extract_snippet_multiline_picks_correct_line() {
        let input = "MATCH (n)\nRETURN n";
        let snippet = extract_snippet(input, 10);
        assert_eq!(snippet, "RETURN n");
    }

    #[test]
    fn extract_snippet_truncates_long_lines() {
        let long_line = "A".repeat(100);
        let snippet = extract_snippet(&long_line, 0);
        assert_eq!(snippet.len(), 80); // 77 chars + "..."
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn from_offset_builds_correct_error() {
        let input = "MATCH (n)\nRETURN";
        let err = ParseError::from_offset(
            input,
            16,
            vec!["expression".to_owned()],
            vec!["RETURN clause".to_owned()],
        );
        assert_eq!(err.line, 2);
        assert_eq!(err.column, 7);
        assert_eq!(err.offset, 16);
        assert_eq!(err.expected, vec!["expression"]);
        assert_eq!(err.context, vec!["RETURN clause"]);
    }

    #[test]
    fn display_format_without_snippet() {
        let err = ParseError {
            offset: 0,
            line: 1,
            column: 1,
            expected: vec!["MATCH".to_owned()],
            context: Vec::new(),
            snippet: String::new(),
        };
        let msg = err.to_string();
        assert!(msg.contains("line 1, column 1"));
        assert!(msg.contains("Expected: MATCH"));
        assert!(!msg.contains("Context:"));
    }

    #[test]
    fn display_format_with_snippet_and_context() {
        let err = ParseError::from_offset(
            "MATCH (n) RETURN",
            16,
            vec!["expression".to_owned()],
            vec!["RETURN clause".to_owned()],
        );
        let msg = err.to_string();
        assert!(msg.contains("line 1, column 17"));
        assert!(msg.contains("MATCH (n) RETURN"));
        assert!(msg.contains("Expected: expression"));
        assert!(msg.contains("Context: RETURN clause"));
    }

    #[test]
    fn display_format_caret_points_to_column() {
        let err = ParseError {
            offset: 5,
            line: 1,
            column: 6,
            expected: vec!["(".to_owned()],
            context: Vec::new(),
            snippet: "MATCH x RETURN n".to_owned(),
        };
        let msg = err.to_string();
        // The caret should be indented: 2 (leading spaces) + 5 (column-1) = 7 chars before ^
        assert!(msg.contains("     ^"));
    }

    #[test]
    fn parse_error_implements_error_trait() {
        let err = ParseError::not_yet_implemented();
        // Verify it can be used as a std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn display_with_multiple_expected() {
        let err = ParseError {
            offset: 0,
            line: 1,
            column: 1,
            expected: vec!["MATCH".to_owned(), "CREATE".to_owned(), "RETURN".to_owned()],
            context: Vec::new(),
            snippet: String::new(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Expected: MATCH, CREATE, RETURN"));
    }

    #[test]
    fn display_with_nested_context() {
        let err = ParseError {
            offset: 0,
            line: 1,
            column: 1,
            expected: vec!["identifier".to_owned()],
            context: vec!["RETURN clause".to_owned(), "expression".to_owned()],
            snippet: String::new(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Context: RETURN clause > expression"));
    }
}
