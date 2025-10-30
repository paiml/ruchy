// NOTEBOOK-004: Rich HTML Output Formatting
// Phase 4: Notebook Excellence - HTML Rendering for Rich Output
//
// This module provides HTML formatting for notebook output:
// - Code syntax highlighting
// - Error messages with formatting
// - HTML tables for DataFrames
// - Value formatting with type information
//
// Quality Requirements:
// - Cyclomatic Complexity: ≤10 per function (Toyota Way)
// - Line Coverage: ≥85%
// - Branch Coverage: ≥90%

/// HTML formatter for notebook output
///
/// Converts plain text output into formatted HTML with syntax highlighting,
/// tables, and rich error display.
///
/// # Examples
///
/// ```
/// use ruchy::notebook::html::HtmlFormatter;
///
/// let formatter = HtmlFormatter::new();
/// let html = formatter.format_value("42");
/// assert!(html.contains("42"));
/// ```
#[derive(Debug, Clone)]
pub struct HtmlFormatter {
    /// Enable syntax highlighting
    syntax_highlighting: bool,
    /// Enable line numbers in code blocks
    line_numbers: bool,
    /// CSS theme (light/dark)
    theme: String,
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlFormatter {
    /// Create a new HTML formatter with default settings
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::html::HtmlFormatter;
    ///
    /// let formatter = HtmlFormatter::new();
    /// assert_eq!(formatter.theme(), "light");
    /// ```
    pub fn new() -> Self {
        Self {
            syntax_highlighting: true,
            line_numbers: false,
            theme: "light".to_string(),
        }
    }

    /// Create a formatter with custom theme
    pub fn with_theme(theme: String) -> Self {
        Self {
            syntax_highlighting: true,
            line_numbers: false,
            theme,
        }
    }

    /// Enable or disable syntax highlighting
    pub fn set_syntax_highlighting(&mut self, enabled: bool) {
        self.syntax_highlighting = enabled;
    }

    /// Enable or disable line numbers
    pub fn set_line_numbers(&mut self, enabled: bool) {
        self.line_numbers = enabled;
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: String) {
        self.theme = theme;
    }

    /// Get the current theme
    pub fn theme(&self) -> &str {
        &self.theme
    }

    /// Format a plain value as HTML
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::html::HtmlFormatter;
    ///
    /// let formatter = HtmlFormatter::new();
    /// let html = formatter.format_value("42");
    /// assert!(html.contains("42"));
    /// assert!(html.contains("<span"));
    /// ```
    pub fn format_value(&self, value: &str) -> String {
        format!(
            r#"<div class="notebook-output"><span class="output-value">{}</span></div>"#,
            html_escape(value)
        )
    }

    /// Format code with syntax highlighting
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::html::HtmlFormatter;
    ///
    /// let formatter = HtmlFormatter::new();
    /// let html = formatter.format_code("let x = 42");
    /// assert!(html.contains("let"));
    /// assert!(html.contains("x"));
    /// ```
    pub fn format_code(&self, code: &str) -> String {
        if self.syntax_highlighting {
            self.format_code_with_highlighting(code)
        } else {
            format!(r#"<pre class="notebook-code">{}</pre>"#, html_escape(code))
        }
    }

    /// Format an error message with styling
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::html::HtmlFormatter;
    ///
    /// let formatter = HtmlFormatter::new();
    /// let html = formatter.format_error("Parse error: unexpected token");
    /// assert!(html.contains("error"));
    /// assert!(html.contains("Parse error"));
    /// ```
    pub fn format_error(&self, error: &str) -> String {
        format!(
            r#"<div class="notebook-error"><span class="error-icon">❌</span> <span class="error-message">{}</span></div>"#,
            html_escape(error)
        )
    }

    /// Format a table (for `DataFrames`)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::html::HtmlFormatter;
    ///
    /// let formatter = HtmlFormatter::new();
    /// let headers = vec!["Name", "Age"];
    /// let rows = vec![
    ///     vec!["Alice", "30"],
    ///     vec!["Bob", "25"],
    /// ];
    /// let html = formatter.format_table(&headers, &rows);
    /// assert!(html.contains("<table"));
    /// assert!(html.contains("Alice"));
    /// ```
    pub fn format_table(&self, headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut html = String::from(r#"<table class="notebook-table">"#);

        // Headers
        html.push_str("<thead><tr>");
        for header in headers {
            html.push_str(&format!("<th>{}</th>", html_escape(header)));
        }
        html.push_str("</tr></thead>");

        // Rows
        html.push_str("<tbody>");
        for row in rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str(&format!("<td>{}</td>", html_escape(cell)));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody>");

        html.push_str("</table>");
        html
    }

    /// Format a list as HTML
    pub fn format_list(&self, items: &[&str]) -> String {
        let mut html = String::from(r#"<ul class="notebook-list">"#);
        for item in items {
            html.push_str(&format!("<li>{}</li>", html_escape(item)));
        }
        html.push_str("</ul>");
        html
    }

    /// Format code with syntax highlighting (internal)
    fn format_code_with_highlighting(&self, code: &str) -> String {
        let mut html = String::from(r#"<pre class="notebook-code syntax-highlighted">"#);

        // Simple keyword highlighting (can be enhanced with proper syntax highlighter)
        let keywords = [
            "let", "fn", "if", "else", "for", "while", "match", "return", "struct", "enum", "impl",
            "trait", "pub", "mut", "const", "static",
        ];

        let mut highlighted = html_escape(code);
        for keyword in &keywords {
            // Simple replacement (not regex for simplicity)
            highlighted = highlighted.replace(
                &format!(" {keyword} "),
                &format!(r#" <span class="keyword">{keyword}</span> "#),
            );
        }

        html.push_str(&highlighted);
        html.push_str("</pre>");
        html
    }

    /// Check if syntax highlighting is enabled
    pub fn syntax_highlighting_enabled(&self) -> bool {
        self.syntax_highlighting
    }

    /// Check if line numbers are enabled
    pub fn line_numbers_enabled(&self) -> bool {
        self.line_numbers
    }
}

/// Escape HTML special characters
///
/// # Examples
///
/// ```
/// use ruchy::notebook::html::html_escape;
///
/// assert_eq!(html_escape("<script>"), "&lt;script&gt;");
/// assert_eq!(html_escape("a & b"), "a &amp; b");
/// ```
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write tests that define expected behavior

    #[test]
    fn test_notebook_004_html_formatter_creation() {
        let formatter = HtmlFormatter::new();
        assert_eq!(formatter.theme(), "light");
        assert!(formatter.syntax_highlighting_enabled());
        assert!(!formatter.line_numbers_enabled());
    }

    #[test]
    fn test_notebook_004_html_formatter_with_theme() {
        let formatter = HtmlFormatter::with_theme("dark".to_string());
        assert_eq!(formatter.theme(), "dark");
    }

    #[test]
    fn test_notebook_004_format_value() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_value("42");

        assert!(html.contains("notebook-output"));
        assert!(html.contains("42"));
        assert!(html.contains("<div"));
    }

    #[test]
    fn test_notebook_004_format_value_escapes_html() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_value("<script>alert('xss')</script>");

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_notebook_004_format_code() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_code("let x = 42");

        assert!(html.contains("<pre"));
        assert!(html.contains('x'));
        assert!(html.contains("42"));
    }

    #[test]
    fn test_notebook_004_format_code_with_highlighting() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_code("let x = 42");

        assert!(html.contains("syntax-highlighted"));
    }

    #[test]
    fn test_notebook_004_format_code_without_highlighting() {
        let mut formatter = HtmlFormatter::new();
        formatter.set_syntax_highlighting(false);
        let html = formatter.format_code("let x = 42");

        assert!(!html.contains("syntax-highlighted"));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn test_notebook_004_format_error() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_error("Parse error: unexpected token");

        assert!(html.contains("notebook-error"));
        assert!(html.contains("Parse error"));
        assert!(html.contains("❌"));
    }

    #[test]
    fn test_notebook_004_format_error_escapes_html() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_error("<script>alert('error')</script>");

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_notebook_004_format_table() {
        let formatter = HtmlFormatter::new();
        let headers = vec!["Name", "Age"];
        let rows = vec![vec!["Alice", "30"], vec!["Bob", "25"]];

        let html = formatter.format_table(&headers, &rows);

        assert!(html.contains("<table"));
        assert!(html.contains("<thead"));
        assert!(html.contains("<tbody"));
        assert!(html.contains("Name"));
        assert!(html.contains("Alice"));
        assert!(html.contains("30"));
    }

    #[test]
    fn test_notebook_004_format_empty_table() {
        let formatter = HtmlFormatter::new();
        let headers: Vec<&str> = vec![];
        let rows: Vec<Vec<&str>> = vec![];

        let html = formatter.format_table(&headers, &rows);

        assert!(html.contains("<table"));
        assert!(html.contains("<thead"));
        assert!(html.contains("<tbody"));
    }

    #[test]
    fn test_notebook_004_format_list() {
        let formatter = HtmlFormatter::new();
        let items = vec!["Item 1", "Item 2", "Item 3"];

        let html = formatter.format_list(&items);

        assert!(html.contains("<ul"));
        assert!(html.contains("<li>Item 1</li>"));
        assert!(html.contains("<li>Item 2</li>"));
    }

    #[test]
    fn test_notebook_004_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("'single'"), "&#39;single&#39;");
    }

    #[test]
    fn test_notebook_004_formatter_settings() {
        let mut formatter = HtmlFormatter::new();

        formatter.set_syntax_highlighting(false);
        assert!(!formatter.syntax_highlighting_enabled());

        formatter.set_line_numbers(true);
        assert!(formatter.line_numbers_enabled());

        formatter.set_theme("dark".to_string());
        assert_eq!(formatter.theme(), "dark");
    }

    #[test]
    fn test_notebook_004_formatter_clone() {
        let formatter = HtmlFormatter::new();
        let cloned = formatter.clone();

        assert_eq!(formatter.theme(), cloned.theme());
        assert_eq!(
            formatter.syntax_highlighting_enabled(),
            cloned.syntax_highlighting_enabled()
        );
    }

    #[test]
    fn test_notebook_004_formatter_debug() {
        let formatter = HtmlFormatter::new();
        let debug_str = format!("{formatter:?}");

        assert!(debug_str.contains("HtmlFormatter"));
        assert!(debug_str.contains("light"));
    }

    #[test]
    fn test_notebook_004_format_multiline_code() {
        let formatter = HtmlFormatter::new();
        let code = "fn main() {\n    let x = 42;\n}";
        let html = formatter.format_code(code);

        assert!(html.contains("fn"));
        assert!(html.contains("main"));
        assert!(html.contains("42"));
    }

    #[test]
    fn test_notebook_004_format_special_characters() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_value("a < b && c > d");

        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_notebook_004_format_unicode() {
        let formatter = HtmlFormatter::new();
        let html = formatter.format_value("Hello 世界 🌍");

        assert!(html.contains("Hello 世界 🌍"));
    }

    #[test]
    fn test_notebook_004_table_with_special_chars() {
        let formatter = HtmlFormatter::new();
        let headers = vec!["<Name>", "&Age"];
        let rows = vec![vec!["Alice & Bob", "<30>"]];

        let html = formatter.format_table(&headers, &rows);

        assert!(html.contains("&lt;Name&gt;"));
        assert!(html.contains("&amp;Age"));
        assert!(html.contains("Alice &amp; Bob"));
    }

    // PROPERTY TESTS: Verify robustness with random inputs
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn html_formatter_never_panics_on_value(value: String) {
                let formatter = HtmlFormatter::new();
                let _ = formatter.format_value(&value);
            }

            #[test]
            fn html_formatter_never_panics_on_code(code: String) {
                let formatter = HtmlFormatter::new();
                let _ = formatter.format_code(&code);
            }

            #[test]
            fn html_formatter_never_panics_on_error(error: String) {
                let formatter = HtmlFormatter::new();
                let _ = formatter.format_error(&error);
            }

            #[test]
            fn html_escape_handles_any_string(input: String) {
                let escaped = html_escape(&input);
                // Should not panic and should not contain unescaped special chars
                assert!(!escaped.contains("<script>"));
            }

            #[test]
            fn html_formatter_escapes_dangerous_tags(
                tag in "<(script|iframe|object|embed)[^>]*>"
            ) {
                let formatter = HtmlFormatter::new();
                let html = formatter.format_value(&tag);
                // Should be escaped
                assert!(!html.contains(&tag));
                assert!(html.contains("&lt;"));
            }

            #[test]
            fn html_table_handles_any_headers(
                headers in prop::collection::vec("[a-zA-Z0-9]{1,20}", 0..10)
            ) {
                let formatter = HtmlFormatter::new();
                let header_refs: Vec<&str> = headers.iter().map(std::string::String::as_str).collect();
                let rows: Vec<Vec<&str>> = vec![];
                let html = formatter.format_table(&header_refs, &rows);
                assert!(html.contains("<table"));
            }

            #[test]
            fn html_list_handles_any_items(
                items in prop::collection::vec(".*", 0..20)
            ) {
                let formatter = HtmlFormatter::new();
                let item_refs: Vec<&str> = items.iter().map(std::string::String::as_str).collect();
                let html = formatter.format_list(&item_refs);
                assert!(html.contains("<ul"));
            }

            #[test]
            fn html_formatter_theme_preserved(
                theme in "[a-z]{4,10}"
            ) {
                let formatter = HtmlFormatter::with_theme(theme.clone());
                assert_eq!(formatter.theme(), theme);
            }

            #[test]
            fn html_escape_reversible_safe_chars(
                input in "[a-zA-Z0-9 ]{1,100}"
            ) {
                let escaped = html_escape(&input);
                // Safe characters should remain unchanged
                assert_eq!(escaped, input);
            }

            #[test]
            fn html_formatter_output_always_valid_structure(
                value in ".*"
            ) {
                let formatter = HtmlFormatter::new();
                let html = formatter.format_value(&value);
                // Should have proper HTML structure
                assert!(html.starts_with("<div"));
                assert!(html.ends_with("</div>"));
            }

            #[test]
            fn html_code_output_always_has_pre_tag(
                code in ".*"
            ) {
                let formatter = HtmlFormatter::new();
                let html = formatter.format_code(&code);
                assert!(html.contains("<pre"));
                assert!(html.contains("</pre>"));
            }

            #[test]
            fn html_error_always_has_error_class(
                error in ".*"
            ) {
                let formatter = HtmlFormatter::new();
                let html = formatter.format_error(&error);
                assert!(html.contains("notebook-error"));
                assert!(html.contains("❌"));
            }

            #[test]
            fn html_formatter_settings_are_independent(
                highlighting: bool,
                line_numbers: bool
            ) {
                let mut formatter = HtmlFormatter::new();
                formatter.set_syntax_highlighting(highlighting);
                formatter.set_line_numbers(line_numbers);

                assert_eq!(formatter.syntax_highlighting_enabled(), highlighting);
                assert_eq!(formatter.line_numbers_enabled(), line_numbers);
            }

            #[test]
            fn html_table_structure_valid_for_any_size(
                row_count in 0usize..20,
                col_count in 0usize..10
            ) {
                let formatter = HtmlFormatter::new();
                let headers: Vec<&str> = (0..col_count).map(|_| "H").collect();
                let rows: Vec<Vec<&str>> = (0..row_count)
                    .map(|_| (0..col_count).map(|_| "C").collect())
                    .collect();

                let html = formatter.format_table(&headers, &rows);
                assert!(html.contains("<table"));
                assert!(html.contains("</table>"));
            }

            #[test]
            fn html_escape_ampersand_first(
                input in ".*&.*"
            ) {
                let escaped = html_escape(&input);
                // Ampersand should be escaped
                if input.contains('&') {
                    assert!(escaped.contains("&amp;"));
                }
            }

            #[test]
            fn html_formatter_clone_preserves_settings(
                theme in "[a-z]{4,8}",
                highlighting: bool
            ) {
                let mut formatter = HtmlFormatter::with_theme(theme.clone());
                formatter.set_syntax_highlighting(highlighting);

                let cloned = formatter.clone();

                assert_eq!(cloned.theme(), theme);
                assert_eq!(cloned.syntax_highlighting_enabled(), highlighting);
            }
        }
    }
}
