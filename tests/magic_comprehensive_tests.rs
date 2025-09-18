//! Sprint 42: Comprehensive Magic Commands Test Coverage
//! Target: src/runtime/magic.rs (723 lines, 4 tests -> 20+ comprehensive tests)
//!
//! This module provides comprehensive test coverage for the REPL magic commands system,
//! including command parsing, execution, error handling, and edge cases.

use ruchy::runtime::Repl;
use ruchy::runtime::magic::{MagicRegistry, MagicResult, UnicodeExpander, ProfileData};
use std::{env, time::Duration;

// Helper function to create a mock repl for testing
fn create_test_repl() -> Repl {
    Repl::new(std::env::temp_dir()).expect("Failed to create test REPL")
}

// ============================================================================
// MagicRegistry Core Functionality Tests
// ============================================================================

#[test]
fn test_magic_registry_new() {
    let registry = MagicRegistry::new();
    let commands = registry.list_commands();

    // Verify all expected magic commands are registered
    let expected_commands = vec![
        "cd", "clear", "debug", "history", "load", "ls",
        "profile", "pwd", "reset", "run", "save", "time",
        "timeit", "whos"
    ];

    for cmd in expected_commands {
        assert!(commands.contains(&cmd.to_string()),
               "Command '{}' not found in registry", cmd);
    }

    assert_eq!(commands.len(), 14, "Expected 14 magic commands");
}

#[test]
fn test_magic_registry_default() {
    let registry = MagicRegistry::default();
    let commands = registry.list_commands();
    assert!(!commands.is_empty(), "Default registry should have commands");
    assert!(commands.contains(&"time".to_string()));
}

#[test]
fn test_magic_command_detection() {
    let registry = MagicRegistry::new();

    // Test line magic detection
    assert!(registry.is_magic("%time"));
    assert!(registry.is_magic("%debug"));
    assert!(registry.is_magic("%whos"));

    // Test cell magic detection
    assert!(registry.is_magic("%%time"));
    assert!(registry.is_magic("%%debug"));

    // Test non-magic commands
    assert!(!registry.is_magic("time"));
    assert!(!registry.is_magic("debug"));
    assert!(!registry.is_magic("regular_command"));
    assert!(!registry.is_magic(""));
}

#[test]
fn test_magic_registry_list_commands_sorted() {
    let registry = MagicRegistry::new();
    let commands = registry.list_commands();

    // Commands should be sorted alphabetically
    let mut sorted_commands = commands.clone();
    sorted_commands.sort();
    assert_eq!(commands, sorted_commands, "Commands should be sorted alphabetically");
}

// ============================================================================
// Magic Command Execution Tests
// ============================================================================

#[test]
fn test_magic_command_execution_empty() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test empty magic command
    let result = registry.execute(&mut repl, "%");
    assert!(result.is_err(), "Empty magic command should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Empty magic command"));
}

#[test]
fn test_magic_command_execution_unknown() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test unknown magic command
    let result = registry.execute(&mut repl, "%unknown_command");
    assert!(result.is_err(), "Unknown magic command should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unknown magic command"));
    assert!(error_msg.contains("unknown_command"));
}

#[test]
fn test_magic_command_execution_not_magic() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test non-magic input
    let result = registry.execute(&mut repl, "regular_command");
    assert!(result.is_err(), "Non-magic input should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Not a magic command"));
}

// ============================================================================
// MagicResult Display Tests
// ============================================================================

#[test]
fn test_magic_result_text_display() {
    let result = MagicResult::Text("Hello, World!".to_string());
    assert_eq!(format!("{}", result), "Hello, World!");
}

#[test]
fn test_magic_result_timed_display() {
    let result = MagicResult::Timed {
        output: "42".to_string(),
        duration: Duration::from_millis(150),
    };
    let display = format!("{}", result);
    assert!(display.contains("42"));
    assert!(display.contains("0.150s"));
    assert!(display.contains("Execution time"));
}

#[test]
fn test_magic_result_silent_display() {
    let result = MagicResult::Silent;
    assert_eq!(format!("{}", result), "");
}

#[test]
fn test_magic_result_profile_display() {
    let profile_data = ProfileData {
        total_time: Duration::from_millis(500),
        function_times: vec![
            ("main".to_string(), Duration::from_millis(300), 1),
            ("helper".to_string(), Duration::from_millis(200), 5),
        ],
    };
    let result = MagicResult::Profile(profile_data);
    let display = format!("{}", result);

    assert!(display.contains("Profile Results"));
    assert!(display.contains("Total time: 0.500s"));
    assert!(display.contains("main"));
    assert!(display.contains("helper"));
    assert!(display.contains("Function"));
    assert!(display.contains("Time (ms)"));
    assert!(display.contains("Count"));
    assert!(display.contains("Avg (ms)"));
}

// ============================================================================
// Workspace Magic Commands Tests
// ============================================================================

#[test]
fn test_whos_magic_empty_workspace() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Clear any existing bindings
    repl.clear_bindings();

    let result = registry.execute(&mut repl, "%whos").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("No variables in workspace"));
        }
        _ => panic!("Expected Text result for %whos"),
    }
}

#[test]
fn test_whos_magic_with_variables() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Add some variables
    repl.eval("let x = 42").unwrap();
    repl.eval("let message = \"hello\"").unwrap();
    repl.eval("let flag = true").unwrap();

    let result = registry.execute(&mut repl, "%whos").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("Variable"));
            assert!(output.contains("Type"));
            assert!(output.contains("Value"));
            // Variables should be listed
            assert!(output.contains("x") || output.contains("42"));
            assert!(output.contains("message") || output.contains("hello"));
            assert!(output.contains("flag") || output.contains("true"));
        }
        _ => panic!("Expected Text result for %whos"),
    }
}

#[test]
fn test_clear_magic_no_args() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%clear");
    assert!(result.is_err(), "%clear without arguments should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %clear <pattern>"));
}

#[test]
fn test_reset_magic() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Add some variables
    repl.eval("let x = 42").unwrap();
    repl.eval("let y = 24").unwrap();

    // Reset workspace
    let result = registry.execute(&mut repl, "%reset").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("Workspace reset"));
        }
        _ => panic!("Expected Text result for %reset"),
    }

    // Verify workspace is empty
    assert!(repl.get_bindings().is_empty(), "Workspace should be empty after reset");
}

// ============================================================================
// Shell Integration Magic Commands Tests
// ============================================================================

#[test]
fn test_pwd_magic() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%pwd").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(!output.is_empty(), "PWD output should not be empty");
            assert!(output.contains("/"), "PWD should contain path separators");
        }
        _ => panic!("Expected Text result for %pwd"),
    }
}

#[test]
fn test_ls_magic_current_dir() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%ls");
    // ls might succeed or fail depending on permissions, but should not panic
    match result {
        Ok(MagicResult::Text(_)) => {
            // Success case - directory listing
        }
        Err(_) => {
            // Error case - might not have permission or directory doesn't exist
        }
        _ => panic!("Expected Text result or error for %ls"),
    }
}

// ============================================================================
// File and Script Magic Commands Tests
// ============================================================================

#[test]
fn test_run_magic_no_args() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%run");
    assert!(result.is_err(), "%run without arguments should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %run <script.ruchy>"));
}

#[test]
fn test_run_magic_nonexistent_file() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%run nonexistent_file.ruchy");
    assert!(result.is_err(), "%run with nonexistent file should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read script"));
}

// ============================================================================
// Session Magic Commands Tests
// ============================================================================

#[test]
fn test_save_magic_no_args() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%save");
    assert!(result.is_err(), "%save without arguments should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %save <filename>"));
}

#[test]
fn test_load_magic_no_args() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%load");
    assert!(result.is_err(), "%load without arguments should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %load <filename>"));
}

#[test]
fn test_load_magic_nonexistent_file() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%load nonexistent_file.json");
    assert!(result.is_err(), "%load with nonexistent file should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file"));
}

// ============================================================================
// History Magic Commands Tests
// ============================================================================

#[test]
fn test_history_magic_default() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%history").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("Last 10 commands"));
            assert!(output.contains("command"));
        }
        _ => panic!("Expected Text result for %history"),
    }
}

#[test]
fn test_history_magic_with_count() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%history 5").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("Last 5 commands"));
        }
        _ => panic!("Expected Text result for %history 5"),
    }
}

#[test]
fn test_history_magic_invalid_count() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%history invalid").unwrap();
    match result {
        MagicResult::Text(output) => {
            // Should default to 10 when invalid number provided
            assert!(output.contains("Last 10 commands"));
        }
        _ => panic!("Expected Text result for %history invalid"),
    }
}

// ============================================================================
// Debug Magic Commands Tests
// ============================================================================

#[test]
fn test_debug_magic_no_error() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%debug").unwrap();
    match result {
        MagicResult::Text(output) => {
            assert!(output.contains("No recent error to debug"));
        }
        _ => panic!("Expected Text result for %debug"),
    }
}

#[test]
fn test_profile_magic_no_args() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%profile");
    assert!(result.is_err(), "%profile without arguments should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %profile <expression>"));
}

// ============================================================================
// Unicode Expansion Tests
// ============================================================================

#[test]
fn test_unicode_expander_new() {
    let expander = UnicodeExpander::new();

    // Test basic Greek letters
    assert_eq!(expander.expand("alpha"), Some('α'));
    assert_eq!(expander.expand("beta"), Some('β'));
    assert_eq!(expander.expand("gamma"), Some('γ'));
    assert_eq!(expander.expand("pi"), Some('π'));
    assert_eq!(expander.expand("omega"), Some('ω'));
}

#[test]
fn test_unicode_expander_with_backslash() {
    let expander = UnicodeExpander::new();

    // Test LaTeX-style with backslash
    assert_eq!(expander.expand("\\alpha"), Some('α'));
    assert_eq!(expander.expand("\\pi"), Some('π'));
    assert_eq!(expander.expand("\\infty"), Some('∞'));
}

#[test]
fn test_unicode_expander_capital_letters() {
    let expander = UnicodeExpander::new();

    // Test capital Greek letters
    assert_eq!(expander.expand("Alpha"), Some('Α'));
    assert_eq!(expander.expand("Beta"), Some('Β'));
    assert_eq!(expander.expand("Gamma"), Some('Γ'));
    assert_eq!(expander.expand("Delta"), Some('Δ'));
    assert_eq!(expander.expand("Omega"), Some('Ω'));
}

#[test]
fn test_unicode_expander_mathematical_symbols() {
    let expander = UnicodeExpander::new();

    // Test mathematical symbols
    assert_eq!(expander.expand("infty"), Some('∞'));
    assert_eq!(expander.expand("sum"), Some('∑'));
    assert_eq!(expander.expand("int"), Some('∫'));
    assert_eq!(expander.expand("sqrt"), Some('√'));
    assert_eq!(expander.expand("partial"), Some('∂'));
    assert_eq!(expander.expand("forall"), Some('∀'));
    assert_eq!(expander.expand("exists"), Some('∃'));
    assert_eq!(expander.expand("in"), Some('∈'));
    assert_eq!(expander.expand("emptyset"), Some('∅'));
    assert_eq!(expander.expand("pm"), Some('±'));
    assert_eq!(expander.expand("times"), Some('×'));
    assert_eq!(expander.expand("div"), Some('÷'));
    assert_eq!(expander.expand("neq"), Some('≠'));
    assert_eq!(expander.expand("leq"), Some('≤'));
    assert_eq!(expander.expand("geq"), Some('≥'));
}

#[test]
fn test_unicode_expander_unknown_sequence() {
    let expander = UnicodeExpander::new();

    assert_eq!(expander.expand("unknown"), None);
    assert_eq!(expander.expand("\\unknown"), None);
    assert_eq!(expander.expand(""), None);
}

#[test]
fn test_unicode_expander_list_expansions() {
    let expander = UnicodeExpander::new();
    let expansions = expander.list_expansions();

    // Should return all mappings with backslash prefix
    assert!(!expansions.is_empty());

    // Check that all expansions start with backslash
    for (sequence, _) in &expansions {
        assert!(sequence.starts_with('\\'),
               "Expansion '{}' should start with backslash", sequence);
    }

    // Check that expansions are sorted
    let sequences: Vec<String> = expansions.iter().map(|(s, _)| s.clone()).collect();
    let mut sorted_sequences = sequences.clone();
    sorted_sequences.sort();
    assert_eq!(sequences, sorted_sequences, "Expansions should be sorted");

    // Verify some known expansions are present
    assert!(expansions.iter().any(|(s, c)| s == "\\alpha" && *c == 'α'));
    assert!(expansions.iter().any(|(s, c)| s == "\\pi" && *c == 'π'));
    assert!(expansions.iter().any(|(s, c)| s == "\\infty" && *c == '∞'));
}

#[test]
fn test_unicode_expander_default() {
    let expander = UnicodeExpander::default();

    // Default should work the same as new()
    assert_eq!(expander.expand("alpha"), Some('α'));
    assert_eq!(expander.expand("pi"), Some('π'));
}

// ============================================================================
// Cell Magic vs Line Magic Tests
// ============================================================================

#[test]
fn test_cell_magic_vs_line_magic_parsing() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Both should be detected as magic
    assert!(registry.is_magic("%pwd"));
    assert!(registry.is_magic("%%pwd"));

    // Both should execute (though behavior might differ)
    let line_result = registry.execute(&mut repl, "%pwd");
    let cell_result = registry.execute(&mut repl, "%%pwd");

    // Both should succeed for pwd command
    assert!(line_result.is_ok(), "Line magic %pwd should succeed");
    assert!(cell_result.is_ok(), "Cell magic %%pwd should succeed");
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[test]
fn test_magic_command_with_whitespace() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test commands with various whitespace patterns
    let result1 = registry.execute(&mut repl, "%pwd ");
    let result2 = registry.execute(&mut repl, "% pwd");
    let _result3 = registry.execute(&mut repl, "%  pwd  ");

    assert!(result1.is_ok(), "Command with trailing space should work");
    assert!(result2.is_ok(), "Command with space after % works due to split_whitespace");
    // result2 works because "% pwd" -> command_line = " pwd" -> split_whitespace() = ["pwd"]
}

#[test]
fn test_magic_command_case_sensitivity() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Magic commands should be case sensitive
    let result1 = registry.execute(&mut repl, "%pwd");
    let result2 = registry.execute(&mut repl, "%PWD");
    let result3 = registry.execute(&mut repl, "%Pwd");

    assert!(result1.is_ok(), "Lowercase command should work");
    assert!(result2.is_err(), "Uppercase command should fail");
    assert!(result3.is_err(), "Mixed case command should fail");
}

#[test]
fn test_magic_registry_robustness() {
    let registry = MagicRegistry::new();

    // Test edge cases for is_magic
    assert!(!registry.is_magic(""));
    assert!(registry.is_magic("%"));   // "%" starts with % so is_magic returns true
    assert!(registry.is_magic("%%"));  // "%%" starts with %% so is_magic returns true
    assert!(registry.is_magic("%a"));
    assert!(registry.is_magic("%%a"));

    // Test that list_commands doesn't panic and returns valid data
    let commands = registry.list_commands();
    assert!(!commands.is_empty());

    for cmd in &commands {
        assert!(!cmd.is_empty(), "Command name should not be empty");
        assert!(!cmd.contains('%'), "Command name should not contain %");
    }
}

// ============================================================================
// Timing Magic Commands Specific Tests
// ============================================================================

#[test]
fn test_time_magic_with_expression() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test %time with a simple expression
    let result = registry.execute(&mut repl, "%time 1 + 1");
    assert!(result.is_ok(), "%time with valid expression should succeed");

    match result.unwrap() {
        MagicResult::Timed { output, duration } => {
            assert_eq!(output, "2", "Expression result should be correct");
            assert!(duration.as_nanos() > 0, "Duration should be positive");
        }
        _ => panic!("Expected Timed result for %time"),
    }
}

#[test]
fn test_time_magic_empty_expression() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%time");
    assert!(result.is_err(), "%time without expression should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %time <expression>"));
}

#[test]
fn test_time_magic_whitespace_only() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%time   ");
    assert!(result.is_err(), "%time with only whitespace should fail");
}

#[test]
fn test_timeit_magic_default_runs() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test %timeit with a simple expression (uses default 1000 runs)
    let result = registry.execute(&mut repl, "%timeit 1 + 1");
    assert!(result.is_ok(), "%timeit with valid expression should succeed");

    match result.unwrap() {
        MagicResult::Text(output) => {
            assert!(output.contains("1000 loops"), "Should show 1000 loops by default");
            assert!(output.contains("best of 1000"), "Should show best of 1000");
            assert!(output.contains("per loop"), "Should show per loop timing");
            assert!(output.contains("min:"), "Should show minimum time");
            assert!(output.contains("median:"), "Should show median time");
            assert!(output.contains("max:"), "Should show maximum time");
        }
        _ => panic!("Expected Text result for %timeit"),
    }
}

#[test]
fn test_timeit_magic_custom_runs() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test %timeit with custom number of runs
    let result = registry.execute(&mut repl, "%timeit -n 10 1 + 1");
    assert!(result.is_ok(), "%timeit with -n flag should succeed");

    match result.unwrap() {
        MagicResult::Text(output) => {
            assert!(output.contains("10 loops"), "Should show 10 loops");
            assert!(output.contains("best of 10"), "Should show best of 10");
        }
        _ => panic!("Expected Text result for %timeit with -n"),
    }
}

#[test]
fn test_timeit_magic_invalid_runs() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test %timeit with invalid number of runs
    let result = registry.execute(&mut repl, "%timeit -n invalid 1 + 1");
    assert!(result.is_err(), "%timeit with invalid -n should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid number of runs"));
}

#[test]
fn test_timeit_magic_incomplete_n_flag() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    // Test %timeit with incomplete -n flag
    let result = registry.execute(&mut repl, "%timeit -n");
    assert!(result.is_err(), "%timeit with incomplete -n should fail");

    let error_msg = result.unwrap_err().to_string();
    // The "-n" gets treated as an expression, which would cause evaluation error
    // rather than "Invalid -n syntax" since it doesn't match the "-n " pattern
    assert!(error_msg.contains("Undefined variable: 'n'"),
            "Error should indicate undefined variable 'n', got: {}", error_msg);
}

#[test]
fn test_timeit_magic_empty_expression() {
    let mut registry = MagicRegistry::new();
    let mut repl = create_test_repl();

    let result = registry.execute(&mut repl, "%timeit");
    assert!(result.is_err(), "%timeit without expression should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Usage: %timeit [-n RUNS] <expression>"));
}

// ============================================================================
// Property Tests for Magic Commands
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_magic_registry_is_magic_never_panics(input: String) {
            let registry = MagicRegistry::new();
            let _ = registry.is_magic(&input);
            // Should never panic
        }

        #[test]
        fn test_unicode_expander_never_panics(input: String) {
            let expander = UnicodeExpander::new();
            let _ = expander.expand(&input);
            // Should never panic
        }

        #[test]
        fn test_magic_result_display_never_panics(text: String) {
            let result = MagicResult::Text(text);
            let _ = format!("{}", result);
            // Should never panic
        }
    }
}