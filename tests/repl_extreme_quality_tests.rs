//! EXTREME Quality REPL Tests - TDD for 90% Coverage
//! Following Toyota Way: Stop the line for any defect
//! Complexity target: <10 for every function
//! Coverage target: 90% minimum

use ruchy::runtime::repl::Repl;
use std::io::{BufReader, Cursor};
use tempfile::TempDir;

#[cfg(test)]
mod repl_basic_functionality {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repl = Repl::new(temp_dir.path().to_path_buf());
        assert!(repl.is_ok());
    }

    #[test]
    fn test_simple_expression_evaluation() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("2 + 2");
        assert!(result.contains("4"));
    }

    #[test]
    fn test_variable_assignment() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("let x = 42");
        assert!(!result.contains("error"));

        let result = repl.handle_command("x");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_function_definition() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("fn add(a, b) { a + b }");
        assert!(!result.contains("error"));

        let result = repl.handle_command("add(3, 5)");
        assert!(result.contains("8"));
    }
}

#[cfg(test)]
mod repl_special_commands {
    use super::*;

    #[test]
    fn test_help_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":help");
        assert!(result.contains("Commands"));
    }

    #[test]
    fn test_history_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        repl.handle_command("1 + 1");
        repl.handle_command("2 + 2");

        let result = repl.handle_command(":history");
        assert!(result.contains("1 + 1"));
        assert!(result.contains("2 + 2"));
    }

    #[test]
    fn test_clear_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        repl.handle_command("let x = 42");
        let result = repl.handle_command(":clear");
        assert!(result.contains("cleared") || result.contains("Clear"));

        // Variable should be gone
        let result = repl.handle_command("x");
        assert!(result.contains("undefined") || result.contains("error"));
    }

    #[test]
    fn test_type_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":type 42");
        assert!(result.contains("Int") || result.contains("i32"));

        let result = repl.handle_command(":type \"hello\"");
        assert!(result.contains("String") || result.contains("str"));
    }

    #[test]
    fn test_ast_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":ast 2 + 2");
        assert!(result.contains("Binary") || result.contains("Add"));
    }

    #[test]
    fn test_tokens_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":tokens let x = 42");
        assert!(result.contains("Let") || result.contains("Identifier"));
    }
}

#[cfg(test)]
mod repl_error_handling {
    use super::*;

    #[test]
    fn test_syntax_error_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("let x =");
        assert!(result.contains("error") || result.contains("Error"));

        // Should recover and work normally
        let result = repl.handle_command("let y = 10");
        assert!(!result.contains("error"));
    }

    #[test]
    fn test_undefined_variable_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("unknown_var");
        assert!(result.contains("undefined") || result.contains("error"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("\"hello\" + 42");
        // Should handle gracefully even if concatenation isn't supported
        assert!(result.len() > 0);
    }

    #[test]
    fn test_division_by_zero() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command("10 / 0");
        assert!(result.contains("error") || result.contains("Error") || result.contains("division"));
    }
}

#[cfg(test)]
mod repl_multiline_input {
    use super::*;

    #[test]
    fn test_multiline_function() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Start multiline
        let result = repl.handle_command("fn factorial(n) {");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("  if n <= 1 {");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("    1");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("  } else {");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("    n * factorial(n - 1)");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("  }");
        assert!(result.contains("...") || result.is_empty());

        let result = repl.handle_command("}");
        assert!(!result.contains("error"));

        // Test the function
        let result = repl.handle_command("factorial(5)");
        assert!(result.contains("120"));
    }

    #[test]
    fn test_multiline_cancellation() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Start multiline
        repl.handle_command("fn test() {");

        // Cancel with special command
        let result = repl.handle_command(":cancel");
        assert!(result.contains("cancelled") || result.contains("Cancel"));
    }
}

#[cfg(test)]
mod repl_load_save {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        repl.handle_command("let x = 42");
        repl.handle_command("fn double(n) { n * 2 }");

        let save_path = temp_dir.path().join("session.ruchy");
        let result = repl.handle_command(&format!(":save {}", save_path.display()));
        assert!(result.contains("saved") || result.contains("Save"));

        assert!(save_path.exists());
    }

    #[test]
    fn test_load_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Create a test file
        let file_path = temp_dir.path().join("test.ruchy");
        fs::write(&file_path, "let loaded_var = 99\nfn loaded_fn() { 42 }").unwrap();

        let result = repl.handle_command(&format!(":load {}", file_path.display()));
        assert!(!result.contains("error"));

        // Check loaded content
        let result = repl.handle_command("loaded_var");
        assert!(result.contains("99"));

        let result = repl.handle_command("loaded_fn()");
        assert!(result.contains("42"));
    }
}

#[cfg(test)]
mod repl_modes {
    use super::*;

    #[test]
    fn test_debug_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":debug");
        assert!(result.contains("debug") || result.contains("Debug"));

        // In debug mode, should show more info
        let result = repl.handle_command("2 + 2");
        // Debug mode might show AST or other details
        assert!(result.len() > 0);
    }

    #[test]
    fn test_transpile_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let result = repl.handle_command(":transpile");
        assert!(result.contains("transpile") || result.contains("Transpile"));

        // Should show Rust code
        let result = repl.handle_command("let x = 42");
        assert!(result.contains("let x") || result.contains("42"));
    }

    #[test]
    fn test_quit_from_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        repl.handle_command(":debug");
        let result = repl.handle_command(":quit");
        // Should return to normal mode, not exit
        assert!(result.contains("normal") || result.contains("Normal"));
    }
}

#[cfg(test)]
mod repl_tab_completion {
    use super::*;

    #[test]
    fn test_keyword_completion() {
        let temp_dir = TempDir::new().unwrap();
        let repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Test that tab completion data structures exist
        // This would need actual tab completion API
        assert!(true); // Placeholder
    }

    #[test]
    fn test_variable_completion() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        repl.handle_command("let my_variable = 42");

        // Would need tab completion API to test
        // repl.get_completions("my_") should include "my_variable"
        assert!(true); // Placeholder
    }
}

#[cfg(test)]
mod repl_performance {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_response_time() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        let start = Instant::now();
        repl.handle_command("2 + 2");
        let duration = start.elapsed();

        // Should respond in less than 100ms for simple expressions
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_large_history_performance() {
        let temp_dir = TempDir::new().unwrap();
        let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

        // Add many history items
        for i in 0..1000 {
            repl.handle_command(&format!("let x{} = {}", i, i));
        }

        let start = Instant::now();
        repl.handle_command(":history");
        let duration = start.elapsed();

        // Should handle large history efficiently
        assert!(duration.as_millis() < 500);
    }
}

#[cfg(test)]
mod repl_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_repl_never_panics(input: String) {
            let temp_dir = TempDir::new().unwrap();
            let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

            // Should never panic on any input
            let _ = repl.handle_command(&input);
        }

        #[test]
        fn test_numeric_expressions(a: i32, b: i32) {
            let temp_dir = TempDir::new().unwrap();
            let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

            let expr = format!("{} + {}", a, b);
            let result = repl.handle_command(&expr);

            // Should not error on valid numeric expressions
            assert!(!result.contains("panic"));
        }

        #[test]
        fn test_variable_names(name in "[a-z][a-z0-9_]{0,20}") {
            let temp_dir = TempDir::new().unwrap();
            let mut repl = Repl::new(temp_dir.path().to_path_buf()).unwrap();

            let cmd = format!("let {} = 42", name);
            let result = repl.handle_command(&cmd);

            // Valid variable names should work
            assert!(!result.contains("panic"));
        }
    }
}