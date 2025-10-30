// NOTEBOOK-002: Cell Execution Engine Enhancements
// Phase 4: Notebook Excellence - Rich Execution Results
//
// This module provides rich execution results that capture:
// - Return value (expression result)
// - Stdout (print statements)
// - Stderr (warnings, errors)
// - Execution time
// - Success/failure status
//
// Quality Requirements:
// - Cyclomatic Complexity: ≤10 per function (Toyota Way)
// - Line Coverage: ≥85%
// - Branch Coverage: ≥90%

use crate::notebook::html::HtmlFormatter;
use std::time::Duration;

/// Result of executing a notebook cell
///
/// Captures all output, timing, and status information from cell execution.
///
/// # Examples
///
/// ```
/// use ruchy::notebook::execution::CellExecutionResult;
/// use std::time::Duration;
///
/// let result = CellExecutionResult::success(
///     "42".to_string(),
///     String::new(),
///     String::new(),
///     Duration::from_millis(5)
/// );
///
/// assert!(result.is_success());
/// assert_eq!(result.output(), "42");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CellExecutionResult {
    /// The return value of the cell execution
    output: String,
    /// Captured stdout (print statements)
    stdout: String,
    /// Captured stderr (warnings, errors)
    stderr: String,
    /// Execution time
    duration: Duration,
    /// Whether execution succeeded
    success: bool,
    /// Error message if execution failed
    error: Option<String>,
}

impl CellExecutionResult {
    /// Create a successful execution result
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::execution::CellExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = CellExecutionResult::success(
    ///     "hello".to_string(),
    ///     "debug output".to_string(),
    ///     String::new(),
    ///     Duration::from_micros(100)
    /// );
    ///
    /// assert!(result.is_success());
    /// assert_eq!(result.output(), "hello");
    /// assert_eq!(result.stdout(), "debug output");
    /// ```
    pub fn success(output: String, stdout: String, stderr: String, duration: Duration) -> Self {
        Self {
            output,
            stdout,
            stderr,
            duration,
            success: true,
            error: None,
        }
    }

    /// Create a failed execution result
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::execution::CellExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = CellExecutionResult::failure(
    ///     "Parse error: unexpected token".to_string(),
    ///     Duration::from_millis(1)
    /// );
    ///
    /// assert!(!result.is_success());
    /// assert_eq!(result.error().unwrap(), "Parse error: unexpected token");
    /// ```
    pub fn failure(error: String, duration: Duration) -> Self {
        Self {
            output: String::new(),
            stdout: String::new(),
            stderr: String::new(),
            duration,
            success: false,
            error: Some(error),
        }
    }

    /// Check if execution succeeded
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the output value
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Get captured stdout
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Get captured stderr
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Get execution duration
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Get error message if execution failed
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get duration in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::execution::CellExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = CellExecutionResult::success(
    ///     "42".to_string(),
    ///     String::new(),
    ///     String::new(),
    ///     Duration::from_millis(250)
    /// );
    ///
    /// assert_eq!(result.duration_ms(), 250);
    /// ```
    pub fn duration_ms(&self) -> u128 {
        self.duration.as_millis()
    }

    /// Check if execution was slow (>100ms)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::execution::CellExecutionResult;
    /// use std::time::Duration;
    ///
    /// let fast = CellExecutionResult::success(
    ///     "42".to_string(), String::new(), String::new(),
    ///     Duration::from_millis(50)
    /// );
    /// assert!(!fast.is_slow());
    ///
    /// let slow = CellExecutionResult::success(
    ///     "42".to_string(), String::new(), String::new(),
    ///     Duration::from_millis(150)
    /// );
    /// assert!(slow.is_slow());
    /// ```
    pub fn is_slow(&self) -> bool {
        self.duration > Duration::from_millis(100)
    }

    /// Check if there's any stdout output
    pub fn has_stdout(&self) -> bool {
        !self.stdout.is_empty()
    }

    /// Check if there's any stderr output
    pub fn has_stderr(&self) -> bool {
        !self.stderr.is_empty()
    }

    /// Format the execution result as HTML
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::execution::CellExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = CellExecutionResult::success(
    ///     "42".to_string(),
    ///     String::new(),
    ///     String::new(),
    ///     Duration::from_millis(5)
    /// );
    ///
    /// let html = result.as_html();
    /// assert!(html.contains("42"));
    /// assert!(html.contains("<div"));
    /// ```
    pub fn as_html(&self) -> String {
        let formatter = HtmlFormatter::new();

        if self.success {
            let mut html = String::new();

            // Add output
            if !self.output.is_empty() {
                html.push_str(&formatter.format_value(&self.output));
            }

            // Add stdout if present
            if self.has_stdout() {
                html.push_str(r#"<div class="notebook-stdout">"#);
                html.push_str(&formatter.format_value(&self.stdout));
                html.push_str("</div>");
            }

            // Add stderr if present
            if self.has_stderr() {
                html.push_str(r#"<div class="notebook-stderr">"#);
                html.push_str(&formatter.format_value(&self.stderr));
                html.push_str("</div>");
            }

            // Add timing info if slow
            if self.is_slow() {
                html.push_str(&format!(
                    r#"<div class="notebook-timing">⏱️ {}ms</div>"#,
                    self.duration_ms()
                ));
            }

            html
        } else {
            // Error formatting
            formatter.format_error(self.error.as_deref().unwrap_or("Unknown error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write tests that define expected behavior

    #[test]
    fn test_notebook_002_success_result_creation() {
        let result = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        assert!(result.is_success());
        assert_eq!(result.output(), "42");
        assert_eq!(result.stdout(), "");
        assert_eq!(result.stderr(), "");
        assert_eq!(result.duration_ms(), 5);
        assert!(result.error().is_none());
    }

    #[test]
    fn test_notebook_002_failure_result_creation() {
        let result =
            CellExecutionResult::failure("Parse error".to_string(), Duration::from_millis(1));

        assert!(!result.is_success());
        assert_eq!(result.error().unwrap(), "Parse error");
        assert_eq!(result.output(), "");
        assert_eq!(result.duration_ms(), 1);
    }

    #[test]
    fn test_notebook_002_result_with_stdout() {
        let result = CellExecutionResult::success(
            "result".to_string(),
            "Hello from print".to_string(),
            String::new(),
            Duration::from_micros(100),
        );

        assert!(result.has_stdout());
        assert!(!result.has_stderr());
        assert_eq!(result.stdout(), "Hello from print");
    }

    #[test]
    fn test_notebook_002_result_with_stderr() {
        let result = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            "Warning: deprecated".to_string(),
            Duration::from_millis(10),
        );

        assert!(!result.has_stdout());
        assert!(result.has_stderr());
        assert_eq!(result.stderr(), "Warning: deprecated");
    }

    #[test]
    fn test_notebook_002_result_with_both_streams() {
        let result = CellExecutionResult::success(
            "output".to_string(),
            "stdout message".to_string(),
            "stderr message".to_string(),
            Duration::from_millis(20),
        );

        assert!(result.has_stdout());
        assert!(result.has_stderr());
        assert_eq!(result.stdout(), "stdout message");
        assert_eq!(result.stderr(), "stderr message");
    }

    #[test]
    fn test_notebook_002_slow_execution_detection() {
        let fast = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(50),
        );
        assert!(!fast.is_slow());

        let slow = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(150),
        );
        assert!(slow.is_slow());

        let at_threshold = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(100),
        );
        assert!(!at_threshold.is_slow());
    }

    #[test]
    fn test_notebook_002_duration_conversion() {
        let result = CellExecutionResult::success(
            "test".to_string(),
            String::new(),
            String::new(),
            Duration::from_secs(2),
        );

        assert_eq!(result.duration_ms(), 2000);
        assert!(result.is_slow());
    }

    #[test]
    fn test_notebook_002_clone_result() {
        let original = CellExecutionResult::success(
            "data".to_string(),
            "stdout".to_string(),
            "stderr".to_string(),
            Duration::from_millis(42),
        );

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.output(), "data");
        assert_eq!(cloned.stdout(), "stdout");
    }

    #[test]
    fn test_notebook_002_debug_format() {
        let result = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("CellExecutionResult"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_notebook_002_empty_output() {
        let result = CellExecutionResult::success(
            String::new(),
            String::new(),
            String::new(),
            Duration::from_micros(10),
        );

        assert!(result.is_success());
        assert_eq!(result.output(), "");
        assert!(!result.has_stdout());
        assert!(!result.has_stderr());
    }

    #[test]
    fn test_notebook_002_error_with_details() {
        let result = CellExecutionResult::failure(
            "Runtime error: Division by zero at line 42".to_string(),
            Duration::from_millis(3),
        );

        assert!(!result.is_success());
        assert!(result
            .error()
            .unwrap()
            .contains("Runtime error: Division by zero"));
        assert!(result.error().unwrap().contains("line 42"));
    }

    #[test]
    fn test_notebook_002_multiline_output() {
        let result = CellExecutionResult::success(
            "line1\nline2\nline3".to_string(),
            "debug1\ndebug2".to_string(),
            "warn1\nwarn2".to_string(),
            Duration::from_millis(15),
        );

        assert!(result.output().contains('\n'));
        assert!(result.stdout().contains('\n'));
        assert!(result.stderr().contains('\n'));
    }

    #[test]
    fn test_notebook_002_unicode_output() {
        let result = CellExecutionResult::success(
            "Hello 世界 🌍".to_string(),
            "stdout 日本語".to_string(),
            "stderr Ελληνικά".to_string(),
            Duration::from_millis(8),
        );

        assert_eq!(result.output(), "Hello 世界 🌍");
        assert_eq!(result.stdout(), "stdout 日本語");
        assert_eq!(result.stderr(), "stderr Ελληνικά");
    }

    #[test]
    fn test_notebook_002_large_output() {
        let large_output = "x".repeat(10000);
        let result = CellExecutionResult::success(
            large_output,
            String::new(),
            String::new(),
            Duration::from_millis(25),
        );

        assert_eq!(result.output().len(), 10000);
        assert!(result.is_success());
    }

    #[test]
    fn test_notebook_002_zero_duration() {
        let result = CellExecutionResult::success(
            "instant".to_string(),
            String::new(),
            String::new(),
            Duration::ZERO,
        );

        assert_eq!(result.duration_ms(), 0);
        assert!(!result.is_slow());
    }

    #[test]
    fn test_notebook_002_getters_immutability() {
        let result = CellExecutionResult::success(
            "data".to_string(),
            "log".to_string(),
            "warn".to_string(),
            Duration::from_millis(10),
        );

        // Getters return references, not ownership
        let _out1 = result.output();
        let _out2 = result.output(); // Can call multiple times

        assert_eq!(result.output(), "data");
    }

    #[test]
    fn test_notebook_002_partial_eq() {
        let result1 = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        let result2 = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        let result3 = CellExecutionResult::success(
            "43".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_notebook_002_special_characters_in_error() {
        let result = CellExecutionResult::failure(
            "Error: \"quoted\" 'text' with\nnewlines\tand\ttabs".to_string(),
            Duration::from_millis(2),
        );

        assert!(result.error().unwrap().contains("\"quoted\""));
        assert!(result.error().unwrap().contains('\n'));
        assert!(result.error().unwrap().contains('\t'));
    }

    #[test]
    fn test_notebook_002_very_fast_execution() {
        let result = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_nanos(500),
        );

        assert!(result.duration_ms() < 1);
        assert!(!result.is_slow());
    }

    // NOTEBOOK-004: Tests for HTML formatting integration

    #[test]
    fn test_notebook_004_as_html_success() {
        let result = CellExecutionResult::success(
            "42".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        let html = result.as_html();
        assert!(html.contains("42"));
        assert!(html.contains("<div"));
    }

    #[test]
    fn test_notebook_004_as_html_error() {
        let result =
            CellExecutionResult::failure("Parse error".to_string(), Duration::from_millis(1));

        let html = result.as_html();
        assert!(html.contains("Parse error"));
        assert!(html.contains("error"));
        assert!(html.contains("❌"));
    }

    #[test]
    fn test_notebook_004_as_html_with_stdout() {
        let result = CellExecutionResult::success(
            "result".to_string(),
            "debug output".to_string(),
            String::new(),
            Duration::from_millis(10),
        );

        let html = result.as_html();
        assert!(html.contains("result"));
        assert!(html.contains("debug output"));
        assert!(html.contains("notebook-stdout"));
    }

    #[test]
    fn test_notebook_004_as_html_with_stderr() {
        let result = CellExecutionResult::success(
            "value".to_string(),
            String::new(),
            "warning message".to_string(),
            Duration::from_millis(10),
        );

        let html = result.as_html();
        assert!(html.contains("value"));
        assert!(html.contains("warning message"));
        assert!(html.contains("notebook-stderr"));
    }

    #[test]
    fn test_notebook_004_as_html_slow_execution() {
        let result = CellExecutionResult::success(
            "slow result".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(150),
        );

        let html = result.as_html();
        assert!(html.contains("slow result"));
        assert!(html.contains("150ms"));
        assert!(html.contains("notebook-timing"));
    }

    #[test]
    fn test_notebook_004_as_html_escapes_dangerous_content() {
        let result = CellExecutionResult::success(
            "<script>alert('xss')</script>".to_string(),
            String::new(),
            String::new(),
            Duration::from_millis(5),
        );

        let html = result.as_html();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
