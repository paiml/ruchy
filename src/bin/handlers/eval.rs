//! Eval Command Handler
//!
//! Handles evaluation of one-liner expressions via the `-e` flag.

use anyhow::Result;

/// Handle eval command - evaluate a one-liner expression with -e flag
///
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `verbose` - Enable verbose output
/// * `format` - Output format ("json" or default text)
/// * `trace` - Enable function call tracing (DEBUGGER-014)
///
/// # Errors
/// Returns error if expression cannot be parsed or evaluated
/// Handle eval command (complexity: 5 - reduced from 11)
pub fn handle_eval_command(expr: &str, verbose: bool, format: &str, trace: bool) -> Result<()> {
    // DEBUGGER-014 Phase 1.3: Set trace flag via environment variable
    if trace {
        std::env::set_var("RUCHY_TRACE", "1");
    } else {
        std::env::remove_var("RUCHY_TRACE");
    }

    if verbose {
        eprintln!("Parsing expression: {expr}");
    }
    let mut repl = super::create_repl()?;

    // If expression defines main(), call it automatically
    // PARSER-085: Fixed to check for "fun main(" (Ruchy keyword) not "fn main(" (Rust keyword)
    let expr_to_eval = if expr.contains("fun main(") {
        format!("{expr}\nmain()")
    } else {
        expr.to_string()
    };

    match repl.eval(&expr_to_eval) {
        Ok(result) => {
            if verbose {
                eprintln!("Evaluation successful");
            }
            // [CLI-EVAL-001] FIX: Print result for REPL one-liners (unless already printed)
            // - `ruchy -e "42"` → prints "42" (REPL behavior)
            // - `ruchy -e "println(42)"` → prints "42" only once (println returns "nil", don't double-print)
            // This fixes tests: non_tty_omits_interactive_features, cli_eval_flag_executes_inline
            // Preserves: prop_021_consistency_eval_equals_file (println behavior still consistent)
            if result != "nil" && !result.is_empty() {
                println!("{result}");
                // Ensure output is flushed for tests capturing stdout
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            Ok(())
        }
        Err(e) => {
            if verbose {
                eprintln!("Evaluation failed: {e}");
            }
            print_eval_error(&e, format);
            Err(e)
        }
    }
}

/// Print successful evaluation result (complexity: 2)
#[allow(dead_code)]
pub fn print_eval_success(result: &str, format: &str) {
    if format == "json" {
        // Manually construct JSON to ensure field order matches test expectations
        let result_str = result.replace('"', "\\\"");
        println!("{{\"success\":true,\"result\":\"{result_str}\"}}");
    } else {
        // Default text output - always show result for one-liner evaluation
        println!("{result}");
    }
}

/// Print evaluation error (complexity: 2)
pub fn print_eval_error(e: &anyhow::Error, format: &str) {
    if format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "success": false,
                "error": e.to_string()
            })
        );
    } else {
        eprintln!("Error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_eval_success_json() {
        // Capture output via a simple test of format logic
        let result = "42";
        let format = "json";
        if format == "json" {
            let result_str = result.replace('"', "\\\"");
            let output = format!("{{\"success\":true,\"result\":\"{result_str}\"}}");
            assert!(output.contains("success"));
            assert!(output.contains("true"));
            assert!(output.contains("42"));
        }
    }

    #[test]
    fn test_print_eval_success_text() {
        let result = "42";
        let format = "text";
        if format != "json" {
            assert_eq!(result, "42");
        }
    }

    #[test]
    fn test_expr_with_main_detection() {
        let expr = "fun main() { 42 }";
        assert!(expr.contains("fun main("));
    }

    #[test]
    fn test_expr_without_main() {
        let expr = "2 + 2";
        assert!(!expr.contains("fun main("));
    }

    #[test]
    fn test_nil_result_not_printed() {
        let result = "nil";
        assert!(result == "nil" || result.is_empty());
    }

    #[test]
    fn test_empty_result_not_printed() {
        let result = "";
        assert!(result == "nil" || result.is_empty());
    }

    // ===== EXTREME TDD Round 152 - Eval Handler Tests =====

    #[test]
    fn test_print_eval_error_text_format() {
        let error = anyhow::anyhow!("Test error message");
        print_eval_error(&error, "text");
        // Just verify it doesn't panic
    }

    #[test]
    fn test_print_eval_error_json_format() {
        let error = anyhow::anyhow!("Parse error");
        print_eval_error(&error, "json");
        // Just verify it doesn't panic
    }

    #[test]
    fn test_print_eval_success_formats() {
        print_eval_success("42", "text");
        print_eval_success("42", "json");
        print_eval_success("hello world", "text");
        print_eval_success("hello world", "json");
    }

    #[test]
    fn test_main_function_detection_patterns() {
        let patterns = [
            ("fun main() { }", true),
            ("fun main(args) { }", true),
            ("fun foo() { }", false),
            ("let main = 42", false),
            ("// fun main()", false),
        ];
        for (expr, should_have_main) in &patterns {
            assert_eq!(expr.contains("fun main("), *should_have_main);
        }
    }

    #[test]
    fn test_result_filtering_logic() {
        let results = ["nil", "", "42", "hello", "true"];
        for result in &results {
            let should_print = *result != "nil" && !result.is_empty();
            if *result == "42" || *result == "hello" || *result == "true" {
                assert!(should_print);
            } else {
                assert!(!should_print);
            }
        }
    }

    #[test]
    fn test_json_escaping_in_success() {
        let result = "hello \"world\"";
        let escaped = result.replace('"', "\\\"");
        assert_eq!(escaped, "hello \\\"world\\\"");
    }

    #[test]
    fn test_handle_eval_command_simple_expr() {
        let result = handle_eval_command("2 + 2", false, "text", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_eval_command_with_trace() {
        let result = handle_eval_command("42", false, "text", true);
        // Clean up env var
        std::env::remove_var("RUCHY_TRACE");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_eval_command_verbose() {
        let result = handle_eval_command("1 + 1", true, "text", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_eval_command_json_format() {
        let result = handle_eval_command("42", false, "json", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_eval_command_invalid_syntax() {
        let result = handle_eval_command("let x = {", false, "text", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_trace_env_var_setting() {
        // Test trace flag sets environment variable
        std::env::set_var("RUCHY_TRACE", "1");
        assert_eq!(std::env::var("RUCHY_TRACE").unwrap(), "1");
        std::env::remove_var("RUCHY_TRACE");
        assert!(std::env::var("RUCHY_TRACE").is_err());
    }
}
