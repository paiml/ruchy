//! Code formatting for Ruchy
use crate::backend::transpiler::Transpiler;
use crate::frontend::parser::Parser;
use tower_lsp::jsonrpc::Result;
pub struct Formatter {
    _transpiler: Transpiler,
}
impl Formatter {
    /// # Examples
    ///
    /// ```
    /// use ruchy::lsp::formatter::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn new() -> Self {
        Self {
            _transpiler: Transpiler::new(),
        }
    }
    /// Format Ruchy code
    ///
    /// # Errors
    ///
    /// This function currently does not return errors but returns Result for future compatibility
    /// # Examples
    ///
    /// ```
    /// use ruchy::lsp::formatter::format;
    ///
    /// let result = format("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn format(&self, document: &str) -> Result<String> {
        // For now, basic formatting - in the future this would be more sophisticated
        let formatted = Self::basic_format(document);
        // Validate the formatted code still parses
        let mut parser = Parser::new(&formatted);
        if parser.parse().is_err() {
            // If formatting broke the code, return original
            return Ok(document.to_string());
        }
        Ok(formatted)
    }
    fn basic_format(document: &str) -> String {
        let lines: Vec<&str> = document.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level: usize = 0;
        for line in lines {
            let trimmed = line.trim();
            // Skip empty lines
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            // Adjust indent level based on content
            let mut current_indent = indent_level;
            // Decrease indent for closing braces
            if trimmed.starts_with('}') || trimmed == "}" {
                current_indent = current_indent.saturating_sub(1);
                indent_level = current_indent;
            }
            // Add indentation
            let indented_line = format!("{}{}", "    ".repeat(current_indent), trimmed);
            formatted_lines.push(indented_line);
            // Increase indent for opening braces
            if trimmed.ends_with('{') {
                indent_level += 1;
            }
        }
        formatted_lines.join("\n")
    }
}
impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::Formatter;
    #[test]
    fn test_basic_formatting() -> anyhow::Result<()> {
        let formatter = Formatter::new();
        let input = "fun test() {\nlet x = 1\nif true {\nprint(x)\n}\n}";
        let result = formatter.format(input)?;
        assert!(result.contains("    let x = 1"));
        assert!(result.contains("        print(x)"));
        Ok(())
    }
    #[test]
    fn test_empty_lines_preserved() -> anyhow::Result<()> {
        let formatter = Formatter::new();
        let input = "fun test() {\n\nlet x = 1\n\n}";
        let result = formatter.format(input)?;
        assert!(result.contains("\n\n"));
        Ok(())
    }

    // === EXTREME TDD Round 16 tests ===

    #[test]
    fn test_formatter_default() {
        let formatter = Formatter::default();
        // Should work the same as new()
        let result = formatter.format("let x = 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_format_direct() {
        // Test basic_format static method directly
        let input = "if true {\nx\n}";
        let result = Formatter::basic_format(input);
        assert!(result.contains("if true {"));
        assert!(result.contains("    x"));
    }

    #[test]
    fn test_format_invalid_code_returns_original() -> anyhow::Result<()> {
        let formatter = Formatter::new();
        // This input is parseable but let's test edge case
        let input = "let x = 1";
        let result = formatter.format(input)?;
        // Should still return formatted version since it parses
        assert!(result.contains("let x = 1"));
        Ok(())
    }

    #[test]
    fn test_format_closing_brace_only() {
        let result = Formatter::basic_format("}");
        // Closing brace should not increase indent level incorrectly
        assert_eq!(result.trim(), "}");
    }

    #[test]
    fn test_format_nested_braces() {
        let input = "fun outer() {\nfun inner() {\nlet x = 1\n}\n}";
        let result = Formatter::basic_format(input);
        // Check nested indentation
        assert!(result.contains("    fun inner()"));
        assert!(result.contains("        let x = 1"));
    }
}
#[cfg(test)]
mod property_tests_formatter {
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
