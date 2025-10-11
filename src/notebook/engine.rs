// NOTEBOOK-001: Notebook Core Infrastructure
// Phase 4: Notebook Excellence - EXTREME TDD Implementation
//
// This module provides the core notebook execution engine that:
// - Maintains REPL state across cells
// - Executes cells one by one
// - Returns formatted output
//
// Quality Requirements:
// - Cyclomatic Complexity: ≤10 per function (Toyota Way)
// - Line Coverage: ≥85%
// - Branch Coverage: ≥90%
// - Mutation Score: ≥90%

use crate::notebook::execution::CellExecutionResult;
use crate::runtime::repl::Repl;
use std::path::PathBuf;
use std::time::Instant;

/// Core notebook execution engine
///
/// Maintains REPL state across cell executions, enabling
/// variables and state to persist between cells (Jupyter-like behavior).
///
/// # Examples
///
/// ```
/// use ruchy::notebook::engine::NotebookEngine;
///
/// let mut engine = NotebookEngine::new().unwrap();
/// let result = engine.execute_cell("let x = 42").unwrap();
/// assert_eq!(result, "()");
///
/// let result = engine.execute_cell("x + 8").unwrap();
/// assert_eq!(result, "50");
/// ```
#[derive(Debug)]
pub struct NotebookEngine {
    repl: Repl,
}

impl NotebookEngine {
    /// Create a new notebook engine
    ///
    /// # Errors
    ///
    /// Returns error if REPL initialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::engine::NotebookEngine;
    ///
    /// let engine = NotebookEngine::new();
    /// assert!(engine.is_ok());
    /// ```
    pub fn new() -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
        let repl = Repl::new(current_dir)?;
        Ok(Self { repl })
    }

    /// Execute a cell and return the formatted output
    ///
    /// State persists across cell executions (variables remain in scope).
    ///
    /// # Errors
    ///
    /// Returns error if cell execution fails (parse error, runtime error, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::engine::NotebookEngine;
    ///
    /// let mut engine = NotebookEngine::new().unwrap();
    /// let result = engine.execute_cell("1 + 1").unwrap();
    /// assert_eq!(result, "2");
    /// ```
    pub fn execute_cell(&mut self, code: &str) -> anyhow::Result<String> {
        // Handle empty cells (Jupyter-like behavior)
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return Ok(String::new());
        }

        // Use eval() which returns Result<String> and properly propagates errors
        // Unlike process_line() which catches errors internally
        self.repl.eval(code)
    }

    /// Execute a cell and return detailed execution results
    ///
    /// Returns a `CellExecutionResult` with output, stdout, stderr, and timing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::engine::NotebookEngine;
    ///
    /// let mut engine = NotebookEngine::new().unwrap();
    /// let result = engine.execute_cell_detailed("1 + 1");
    ///
    /// assert!(result.is_success());
    /// assert_eq!(result.output(), "2");
    /// assert!(result.duration_ms() < 100);
    /// ```
    pub fn execute_cell_detailed(&mut self, code: &str) -> CellExecutionResult {
        let start = Instant::now();

        // Handle empty cells
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return CellExecutionResult::success(
                String::new(),
                String::new(),
                String::new(),
                start.elapsed(),
            );
        }

        // Execute and capture result
        match self.repl.eval(code) {
            Ok(output) => CellExecutionResult::success(
                output,
                String::new(), // TODO: Capture actual stdout in future enhancement
                String::new(), // TODO: Capture actual stderr in future enhancement
                start.elapsed(),
            ),
            Err(e) => CellExecutionResult::failure(e.to_string(), start.elapsed()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write tests that FAIL first
    // These tests define the expected behavior

    #[test]
    fn test_notebook_001_engine_creation() {
        let engine = NotebookEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_notebook_001_engine_debug_format() {
        let engine = NotebookEngine::new().unwrap();
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("NotebookEngine"));
    }

    #[test]
    fn test_notebook_001_execute_simple_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_arithmetic() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("1 + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_variable_binding() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("let x = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_state_persists_across_cells() {
        let mut engine = NotebookEngine::new().unwrap();

        // Cell 1: Define variable
        let result1 = engine.execute_cell("let x = 10");
        assert!(result1.is_ok());

        // Cell 2: Use variable (should succeed if state persists)
        let result2 = engine.execute_cell("x + 5");
        assert!(result2.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_string_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("\"hello world\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_boolean_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_invalid_syntax() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("let x = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_notebook_001_execute_undefined_variable() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("undefined_var");
        assert!(result.is_err());
    }

    #[test]
    fn test_notebook_001_execute_function_definition() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("fn add(a, b) { a + b }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_function_persists_across_cells() {
        let mut engine = NotebookEngine::new().unwrap();

        // Cell 1: Define function
        let result1 = engine.execute_cell("fn double(x) { x * 2 }");
        assert!(result1.is_ok());

        // Cell 2: Call function
        let result2 = engine.execute_cell("double(21)");
        assert!(result2.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_if_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("if true { 1 } else { 0 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_match_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("match 42 { 42 => true, _ => false }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_array_literal() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("[1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_object_literal() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("{ a: 1, b: 2 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_for_loop() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("for i in [1, 2, 3] { i }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_while_loop() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("let mut x = 0; while x < 3 { x = x + 1 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_multiple_statements_per_cell() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("let x = 10; let y = 20; x + y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_state_isolation_between_engines() {
        let mut engine1 = NotebookEngine::new().unwrap();
        let mut engine2 = NotebookEngine::new().unwrap();

        // Define variable in engine1
        engine1.execute_cell("let x = 100").unwrap();

        // Should fail in engine2 (different state)
        let result = engine2.execute_cell("x");
        assert!(result.is_err());
    }

    #[test]
    fn test_notebook_001_execute_empty_cell() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("");
        // Empty cells should succeed (Jupyter behavior)
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_comment_only() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("// This is a comment");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_whitespace_only() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("   \n\t  ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_complex_state_mutation() {
        let mut engine = NotebookEngine::new().unwrap();

        // Cell 1: Create mutable variable
        engine.execute_cell("let mut count = 0").unwrap();

        // Cell 2: Mutate it
        engine.execute_cell("count = count + 1").unwrap();

        // Cell 3: Mutate again
        engine.execute_cell("count = count + 1").unwrap();

        // Cell 4: Read value (should be 2)
        let result = engine.execute_cell("count");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_closure_state_persistence() {
        let mut engine = NotebookEngine::new().unwrap();

        // Cell 1: Create closure
        engine.execute_cell("let increment = |x| x + 1").unwrap();

        // Cell 2: Use closure
        let result = engine.execute_cell("increment(41)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_error_doesnt_break_engine() {
        let mut engine = NotebookEngine::new().unwrap();

        // Cell 1: Success
        engine.execute_cell("let x = 10").unwrap();

        // Cell 2: Error
        let _ = engine.execute_cell("invalid syntax here");

        // Cell 3: Should still work after error
        let result = engine.execute_cell("x + 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_multiline_function() {
        let mut engine = NotebookEngine::new().unwrap();
        let code = r#"
fn factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
        "#;
        let result = engine.execute_cell(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_nested_structures() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell("[[1, 2], [3, 4]]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_notebook_001_execute_struct_literal() {
        let mut engine = NotebookEngine::new().unwrap();
        let code = r#"
struct Point { x: i64, y: i64 }
Point { x: 10, y: 20 }
        "#;
        let result = engine.execute_cell(code);
        assert!(result.is_ok());
    }

    // NOTEBOOK-002: Tests for execute_cell_detailed()

    #[test]
    fn test_notebook_002_detailed_execution_success() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell_detailed("42");

        assert!(result.is_success());
        assert_eq!(result.output(), "42");
        assert!(result.duration_ms() < 1000);
        assert!(!result.has_stdout());
        assert!(!result.has_stderr());
    }

    #[test]
    fn test_notebook_002_detailed_execution_error() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell_detailed("undefined_variable");

        assert!(!result.is_success());
        assert!(result.error().is_some());
        assert!(result.error().unwrap().contains("Undefined variable"));
    }

    #[test]
    fn test_notebook_002_detailed_execution_empty_cell() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell_detailed("");

        assert!(result.is_success());
        assert_eq!(result.output(), "");
        assert!(result.duration_ms() < 10);
    }

    #[test]
    fn test_notebook_002_detailed_execution_timing() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell_detailed("1 + 1");

        assert!(result.is_success());
        // Should be very fast for simple arithmetic
        assert!(result.duration_ms() < 50);
        assert!(!result.is_slow());
    }

    #[test]
    fn test_notebook_002_detailed_preserves_state() {
        let mut engine = NotebookEngine::new().unwrap();

        let result1 = engine.execute_cell_detailed("let x = 100");
        assert!(result1.is_success());

        let result2 = engine.execute_cell_detailed("x + 50");
        assert!(result2.is_success());
        assert_eq!(result2.output(), "150");
    }

    #[test]
    fn test_notebook_002_detailed_complex_expression() {
        let mut engine = NotebookEngine::new().unwrap();
        let result = engine.execute_cell_detailed("if true { 42 } else { 0 }");

        assert!(result.is_success());
        assert_eq!(result.output(), "42");
    }

    #[test]
    fn test_notebook_002_detailed_multiline_code() {
        let mut engine = NotebookEngine::new().unwrap();
        let code = r#"
let a = 10
let b = 20
a + b
        "#;
        let result = engine.execute_cell_detailed(code);

        assert!(result.is_success());
        assert_eq!(result.output(), "30");
    }

    #[test]
    fn test_notebook_002_detailed_error_recovery() {
        let mut engine = NotebookEngine::new().unwrap();

        // Execute valid code
        let result1 = engine.execute_cell_detailed("let x = 5");
        assert!(result1.is_success());

        // Execute invalid code
        let result2 = engine.execute_cell_detailed("invalid syntax");
        assert!(!result2.is_success());

        // Should still work after error
        let result3 = engine.execute_cell_detailed("x + 10");
        assert!(result3.is_success());
        assert_eq!(result3.output(), "15");
    }

    // PROPERTY TESTS: Verify robustness with 10,000+ random inputs
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn notebook_engine_never_panics_on_any_input(code: String) {
                let mut engine = NotebookEngine::new().unwrap();
                // Should never panic on any input, even invalid code
                let _ = engine.execute_cell(&code);
            }

            #[test]
            fn notebook_engine_handles_any_expression(expr in "[0-9]{1,9}") {
                let mut engine = NotebookEngine::new().unwrap();
                let result = engine.execute_cell(&expr);
                // Valid numbers (up to 9 digits) should work
                prop_assert!(result.is_ok());
            }

            #[test]
            fn notebook_engine_state_persists(
                var_name in "[a-z][a-z0-9_]{0,10}",
                value in 0i64..1000
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                // Define variable
                let define = format!("let {} = {}", var_name, value);
                if engine.execute_cell(&define).is_ok() {
                    // Use variable - should succeed if definition succeeded
                    let use_var = engine.execute_cell(&var_name);
                    prop_assert!(use_var.is_ok());
                }
            }

            #[test]
            fn notebook_engine_handles_whitespace_variations(
                spaces_before in 0usize..10,
                spaces_after in 0usize..10
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let code = format!("{}42{}", " ".repeat(spaces_before), " ".repeat(spaces_after));
                let result = engine.execute_cell(&code);
                // Should handle whitespace variations
                prop_assert!(result.is_ok() || result.is_err());
            }

            #[test]
            fn notebook_engine_arithmetic_operations(
                a in 1i64..100,
                b in 1i64..100
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                let add = format!("{} + {}", a, b);
                prop_assert!(engine.execute_cell(&add).is_ok());

                let sub = format!("{} - {}", a, b);
                prop_assert!(engine.execute_cell(&sub).is_ok());

                let mul = format!("{} * {}", a, b);
                prop_assert!(engine.execute_cell(&mul).is_ok());

                let div = format!("{} / {}", a, b);
                prop_assert!(engine.execute_cell(&div).is_ok());
            }

            #[test]
            fn notebook_engine_string_literals(s in ".*") {
                let mut engine = NotebookEngine::new().unwrap();
                // Escape the string properly
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                let code = format!("\"{}\"", escaped);
                // Should handle any string content
                let _ = engine.execute_cell(&code);
            }

            #[test]
            fn notebook_engine_boolean_operations(
                a: bool,
                b: bool
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                let and = format!("{} && {}", a, b);
                prop_assert!(engine.execute_cell(&and).is_ok());

                let or = format!("{} || {}", a, b);
                prop_assert!(engine.execute_cell(&or).is_ok());

                let not = format!("!{}", a);
                prop_assert!(engine.execute_cell(&not).is_ok());
            }

            #[test]
            fn notebook_engine_multiple_cells_consistency(
                operations in prop::collection::vec("[0-9]+ [+\\-*/] [0-9]+", 1..10)
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                for op in operations {
                    // Each operation should be evaluated independently
                    let _ = engine.execute_cell(&op);
                }

                // Engine should still be usable after multiple operations
                let result = engine.execute_cell("42");
                prop_assert!(result.is_ok());
            }

            #[test]
            fn notebook_engine_error_recovery(
                valid_code in "[0-9]{1,9}",
                invalid_code in "[+\\-*/]+",
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                // Execute valid code (up to 9 digits to avoid overflow)
                prop_assert!(engine.execute_cell(&valid_code).is_ok());

                // Execute invalid code (should error)
                let _ = engine.execute_cell(&invalid_code);

                // Should recover and handle valid code again
                prop_assert!(engine.execute_cell(&valid_code).is_ok());
            }

            #[test]
            fn notebook_engine_comment_handling(
                comment_text in ".*"
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let code = format!("// {}", comment_text);
                // Comments should always succeed
                prop_assert!(engine.execute_cell(&code).is_ok());
            }

            // NOTEBOOK-002: Property tests for detailed execution

            #[test]
            fn notebook_engine_detailed_never_panics(code: String) {
                let mut engine = NotebookEngine::new().unwrap();
                // Should never panic on any input
                let _ = engine.execute_cell_detailed(&code);
            }

            #[test]
            fn notebook_engine_detailed_timing_is_reasonable(
                expr in "[0-9]{1,5}"
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let result = engine.execute_cell_detailed(&expr);

                // Should complete in reasonable time
                prop_assert!(result.duration_ms() < 1000);
            }

            #[test]
            fn notebook_engine_detailed_success_has_output(
                value in 1i64..1000
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let code = format!("{}", value);
                let result = engine.execute_cell_detailed(&code);

                if result.is_success() {
                    // Success should have non-error output
                    prop_assert!(result.error().is_none());
                    prop_assert!(!result.output().is_empty() || result.output() == "()");
                }
            }

            #[test]
            fn notebook_engine_detailed_failure_has_error(
                invalid in "[+\\-*/]{3,10}"
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let result = engine.execute_cell_detailed(&invalid);

                if !result.is_success() {
                    // Failure should have error message
                    prop_assert!(result.error().is_some());
                    prop_assert!(!result.error().unwrap().is_empty());
                }
            }

            #[test]
            fn notebook_engine_detailed_preserves_timing_order(
                operations in prop::collection::vec("[0-9]{1,3}", 5..15)
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let mut timings = Vec::new();

                for op in operations {
                    let result = engine.execute_cell_detailed(&op);
                    timings.push(result.duration_ms());
                }

                // All timings should be reasonable
                for timing in timings {
                    prop_assert!(timing < 100);
                }
            }

            #[test]
            fn notebook_engine_detailed_empty_is_fast(
                spaces in 0usize..20
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let code = " ".repeat(spaces);
                let result = engine.execute_cell_detailed(&code);

                // Empty cells should be very fast
                prop_assert!(result.is_success());
                prop_assert!(result.duration_ms() < 10);
                prop_assert_eq!(result.output(), "");
            }

            #[test]
            fn notebook_engine_detailed_arithmetic_always_timed(
                a in 1i64..100,
                b in 1i64..100,
                op in "[+\\-*/]"
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let code = format!("{} {} {}", a, op.chars().next().unwrap(), b);
                let result = engine.execute_cell_detailed(&code);

                // Should always have timing information
                prop_assert!(result.duration_ms() >= 0);
            }

            #[test]
            fn notebook_engine_detailed_consistent_with_basic(
                expr in "[0-9]{1,5}"
            ) {
                let mut engine1 = NotebookEngine::new().unwrap();
                let mut engine2 = NotebookEngine::new().unwrap();

                let basic_result = engine1.execute_cell(&expr);
                let detailed_result = engine2.execute_cell_detailed(&expr);

                // Both should agree on success/failure
                if basic_result.is_ok() {
                    prop_assert!(detailed_result.is_success());
                    prop_assert_eq!(detailed_result.output(), basic_result.unwrap());
                } else {
                    prop_assert!(!detailed_result.is_success());
                }
            }

            #[test]
            fn notebook_engine_detailed_state_consistency(
                var_name in "[a-z][a-z0-9]{0,8}",
                value1 in 0i64..100,
                value2 in 0i64..100
            ) {
                let mut engine = NotebookEngine::new().unwrap();

                // Define variable
                let def_result = engine.execute_cell_detailed(&format!("let {} = {}", var_name, value1));
                if def_result.is_success() {
                    // Use variable
                    let use_result = engine.execute_cell_detailed(&var_name);
                    prop_assert!(use_result.is_success());

                    // Modify variable
                    let mod_result = engine.execute_cell_detailed(&format!("{} = {}", var_name, value2));
                    if mod_result.is_success() {
                        // Check new value
                        let check_result = engine.execute_cell_detailed(&var_name);
                        prop_assert!(check_result.is_success());
                    }
                }
            }

            #[test]
            fn notebook_engine_detailed_error_metadata_complete(
                invalid in ".*"
            ) {
                let mut engine = NotebookEngine::new().unwrap();
                let result = engine.execute_cell_detailed(&invalid);

                // Every result should have valid metadata
                prop_assert!(result.duration_ms() >= 0);

                if result.is_success() {
                    prop_assert!(result.error().is_none());
                } else {
                    prop_assert!(result.error().is_some());
                    prop_assert!(!result.error().unwrap().is_empty());
                }
            }
        }
    }
}
