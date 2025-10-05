//! Code formatting utilities
//!
//! This module provides utilities for formatting Swift code, including:
//!
//! - Indentation management
//! - Doc comment formatting
//! - String escaping
//! - Line wrapping
//!
//! These utilities ensure consistent formatting across all generated Swift code.

/// Add indentation to each line of a string.
///
/// # Arguments
///
/// * `s` - The string to indent
/// * `level` - The indentation level (number of spaces = level * 4)
///
/// # Returns
///
/// The indented string with proper formatting
///
/// # Examples
///
/// ```rust
/// # use specta_swift::utils::formatting::indent;
/// let code = "let x = 5\nlet y = 10";
/// let indented = indent(code, 1);
/// assert_eq!(indented, "    let x = 5\n    let y = 10");
/// ```
pub fn indent(s: &str, level: usize) -> String {
    let indent_str = "    ".repeat(level);
    s.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent_str, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a doc comment with proper Swift syntax.
///
/// Converts multi-line comments into Swift's `///` format.
///
/// # Arguments
///
/// * `docs` - The documentation string
///
/// # Returns
///
/// Formatted doc comment with `///` prefix on each line
///
/// # Examples
///
/// ```rust
/// # use specta_swift::utils::formatting::format_doc_comment;
/// let docs = "This is a type.\nIt does things.";
/// let formatted = format_doc_comment(docs);
/// assert_eq!(formatted, "/// This is a type.\n/// It does things.\n");
/// ```
pub fn format_doc_comment(docs: &str) -> String {
    let mut result = String::new();
    for line in docs.lines() {
        result.push_str("/// ");
        result.push_str(line.trim_start());
        result.push('\n');
    }
    result
}

/// Escape a string for use in Swift code.
///
/// Handles common escape sequences like quotes, newlines, etc.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// The escaped string safe for Swift code
///
/// # Examples
///
/// ```rust
/// # use specta_swift::utils::formatting::escape_string;
/// assert_eq!(escape_string("hello"), "hello");
/// assert_eq!(escape_string("say \"hi\""), "say \\\"hi\\\"");
/// assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
/// ```
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Join multiple strings with a separator, filtering out empty strings.
///
/// # Arguments
///
/// * `items` - Iterator of string references
/// * `separator` - The separator to use between items
///
/// # Returns
///
/// Joined string with empty items filtered out
///
/// # Examples
///
/// ```rust
/// # use specta_swift::utils::formatting::join_non_empty;
/// let items = vec!["a", "", "b", "c"];
/// assert_eq!(join_non_empty(items.iter().copied(), ", "), "a, b, c");
/// ```
pub fn join_non_empty<'a, I>(items: I, separator: &str) -> String
where
    I: Iterator<Item = &'a str>,
{
    items
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Format a deprecation message for Swift's `@available` attribute.
///
/// # Arguments
///
/// * `message` - The deprecation message
///
/// # Returns
///
/// Formatted `@available` attribute string
///
/// # Examples
///
/// ```rust
/// # use specta_swift::utils::formatting::format_deprecated;
/// let attr = format_deprecated("Use NewType instead");
/// assert_eq!(attr, "@available(*, deprecated, message: \"Use NewType instead\")\n");
/// ```
pub fn format_deprecated(message: &str) -> String {
    format!("@available(*, deprecated, message: \"{}\")\n", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_single_line() {
        assert_eq!(indent("let x = 5", 0), "let x = 5");
        assert_eq!(indent("let x = 5", 1), "    let x = 5");
        assert_eq!(indent("let x = 5", 2), "        let x = 5");
    }

    #[test]
    fn test_indent_multiple_lines() {
        let code = "let x = 5\nlet y = 10";
        assert_eq!(indent(code, 1), "    let x = 5\n    let y = 10");
    }

    #[test]
    fn test_indent_preserves_empty_lines() {
        let code = "let x = 5\n\nlet y = 10";
        assert_eq!(indent(code, 1), "    let x = 5\n\n    let y = 10");
    }

    #[test]
    fn test_format_doc_comment_single_line() {
        assert_eq!(format_doc_comment("This is a doc"), "/// This is a doc\n");
    }

    #[test]
    fn test_format_doc_comment_multiple_lines() {
        let docs = "Line 1\nLine 2\nLine 3";
        let expected = "/// Line 1\n/// Line 2\n/// Line 3\n";
        assert_eq!(format_doc_comment(docs), expected);
    }

    #[test]
    fn test_format_doc_comment_trims_leading_whitespace() {
        let docs = "  Indented line\n    More indented";
        let expected = "/// Indented line\n/// More indented\n";
        assert_eq!(format_doc_comment(docs), expected);
    }

    #[test]
    fn test_escape_string_quotes() {
        assert_eq!(escape_string("say \"hi\""), "say \\\"hi\\\"");
    }

    #[test]
    fn test_escape_string_backslash() {
        assert_eq!(escape_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_string_newline() {
        assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_string_tab() {
        assert_eq!(escape_string("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_escape_string_carriage_return() {
        assert_eq!(escape_string("line1\rline2"), "line1\\rline2");
    }

    #[test]
    fn test_escape_string_combined() {
        assert_eq!(
            escape_string("path\\to\\\"file\"\nline2"),
            "path\\\\to\\\\\\\"file\\\"\\nline2"
        );
    }

    #[test]
    fn test_join_non_empty_basic() {
        let items = vec!["a", "b", "c"];
        assert_eq!(join_non_empty(items.iter().copied(), ", "), "a, b, c");
    }

    #[test]
    fn test_join_non_empty_filters_empty() {
        let items = vec!["a", "", "b", "", "c"];
        assert_eq!(join_non_empty(items.iter().copied(), ", "), "a, b, c");
    }

    #[test]
    fn test_join_non_empty_all_empty() {
        let items = vec!["", "", ""];
        assert_eq!(join_non_empty(items.iter().copied(), ", "), "");
    }

    #[test]
    fn test_join_non_empty_single_item() {
        let items = vec!["a"];
        assert_eq!(join_non_empty(items.iter().copied(), ", "), "a");
    }

    #[test]
    fn test_format_deprecated() {
        let result = format_deprecated("Use NewType instead");
        assert_eq!(
            result,
            "@available(*, deprecated, message: \"Use NewType instead\")\n"
        );
    }

    #[test]
    fn test_format_deprecated_empty() {
        let result = format_deprecated("");
        assert_eq!(result, "@available(*, deprecated, message: \"\")\n");
    }
}

