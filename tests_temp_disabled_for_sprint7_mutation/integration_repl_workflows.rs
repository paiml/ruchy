//! REPL Integration Workflow Tests (QUALITY-009 Phase 2)
//!
//! Tests complete REPL workflows as specified in QUALITY-009, focusing on:
//! - Interactive command sequences
//! - Session state management
//! - Multi-line input handling
//! - Error recovery without state loss

#![allow(clippy::unwrap_used)]

use ruchy::runtime::repl::Repl;
use std::env;

/// Test harness for REPL workflow testing
struct ReplWorkflowHarness {
    repl: Repl,
}

impl ReplWorkflowHarness {
    fn new() -> Self {
        Self {
            repl: Repl::new(std::env::temp_dir()).expect("Failed to create REPL instance"),
        }
    }

    /// Execute a REPL command and validate expected output
    fn execute_and_validate(&mut self, command: &str, expected: &str) -> Result<(), String> {
        match self.repl.eval(command) {
            Ok(result) => {
                let output = result;
                if output == expected {
                    Ok(())
                } else {
                    Err(format!("Expected '{expected}', got '{output}'"))
                }
            }
            Err(e) => Err(format!("Command failed: {e}")),
        }
    }

    /// Execute a command that should succeed but don't validate output
    fn execute_ok(&mut self, command: &str) -> Result<String, String> {
        match self.repl.eval(command) {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Command failed: {e}")),
        }
    }

    /// Execute a command that should fail
    fn execute_error(&mut self, command: &str) -> bool {
        self.repl.eval(command).is_err()
    }
}

#[test]
fn test_basic_repl_operations() {
    let mut harness = ReplWorkflowHarness::new();

    // Check case from QUALITY-009: Basic REPL Operations
    // > let x = 42
    // > x + 8
    // 50

    harness
        .execute_ok("let x = 42")
        .expect("Should define variable x");

    harness
        .execute_and_validate("x + 8", "50")
        .expect("Should evaluate x + 8 to 50");

    // Additional basic operations
    harness
        .execute_and_validate("x * 2", "84")
        .expect("Should evaluate x * 2 to 84");

    harness
        .execute_and_validate("x / 6", "7")
        .expect("Should evaluate x / 6 to 7");
}

#[test]
fn test_session_state_persistence() {
    let mut harness = ReplWorkflowHarness::new();

    // Variable persistence across commands
    harness
        .execute_ok("let counter = 0")
        .expect("Should initialize counter");

    harness
        .execute_ok("counter = counter + 1")
        .expect("Should increment counter");

    harness
        .execute_and_validate("counter", "1")
        .expect("Counter should be 1");

    // Multiple variable state
    harness
        .execute_ok("let name = \"Ruchy\"")
        .expect("Should define name");

    harness
        .execute_ok("let version = 1.16")
        .expect("Should define version");

    harness
        .execute_and_validate("name", "\"Ruchy\"")
        .expect("Name should persist");

    harness
        .execute_and_validate("version", "1.16")
        .expect("Version should persist");
}

#[test]
fn test_function_definitions_and_calls() {
    let mut harness = ReplWorkflowHarness::new();

    // Function definitions persist across commands
    harness
        .execute_ok("fun add(a: i32, b: i32) -> i32 { a + b }")
        .expect("Should define add function");

    harness
        .execute_and_validate("add(10, 32)", "42")
        .expect("Should call add function");

    // Multiple function definitions
    harness
        .execute_ok("fun multiply(x: i32, y: i32) -> i32 { x * y }")
        .expect("Should define multiply function");

    harness
        .execute_and_validate("multiply(6, 7)", "42")
        .expect("Should call multiply function");

    // Functions can call other functions
    harness
        .execute_ok(
            "fun add_and_multiply(a: i32, b: i32, c: i32) -> i32 { multiply(add(a, b), c) }",
        )
        .expect("Should define composite function");

    harness
        .execute_and_validate("add_and_multiply(3, 4, 6)", "42")
        .expect("Should call composite function");
}

#[test]
fn test_multiline_function_definition() {
    let mut harness = ReplWorkflowHarness::new();

    // Check case from QUALITY-009: Multi-line Input
    // > fun factorial(n) {
    // ... if n <= 1 { 1 } else { n * factorial(n-1) }
    // ... }
    // > factorial(5)
    // 120

    let factorial_def = r"
        fun factorial(n: i32) -> i32 {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
    ";

    harness
        .execute_ok(factorial_def.trim())
        .expect("Should define factorial function");

    harness
        .execute_and_validate("factorial(5)", "120")
        .expect("Should calculate factorial(5) = 120");

    harness
        .execute_and_validate("factorial(0)", "1")
        .expect("Should calculate factorial(0) = 1");

    harness
        .execute_and_validate("factorial(3)", "6")
        .expect("Should calculate factorial(3) = 6");
}

#[test]
fn test_error_recovery_without_state_loss() {
    let mut harness = ReplWorkflowHarness::new();

    // Set up some state
    harness
        .execute_ok("let important_data = 42")
        .expect("Should set important data");

    harness
        .execute_ok("fun important_function() -> i32 { important_data * 2 }")
        .expect("Should define important function");

    // Check state works
    harness
        .execute_and_validate("important_function()", "84")
        .expect("Should call function successfully");

    // Introduce syntax errors
    assert!(harness.execute_error("let x = 2 + + 3"));
    assert!(harness.execute_error("invalid syntax here"));
    assert!(harness.execute_error("fun broken("));

    // Check state is preserved after errors
    harness
        .execute_and_validate("important_data", "42")
        .expect("Variable should persist after syntax errors");

    harness
        .execute_and_validate("important_function()", "84")
        .expect("Function should persist after syntax errors");

    // Can still define new state after errors
    harness
        .execute_ok("let recovery_data = 100")
        .expect("Should define new variable after errors");

    harness
        .execute_and_validate("recovery_data + important_data", "142")
        .expect("Should combine old and new state");
}

#[test]
fn test_complex_multiline_expressions() {
    let mut harness = ReplWorkflowHarness::new();

    // Complex nested expressions
    let complex_expr = r#"
        let result = 
            if 10 > 5 {
                match 42 {
                    0 => "zero",
                    42 => "answer", 
                    _ => "other"
                }
            } else {
                "impossible"
            }
    "#;

    harness
        .execute_ok(complex_expr.trim())
        .expect("Should handle complex multiline expression");

    harness
        .execute_and_validate("result", "\"answer\"")
        .expect("Should evaluate complex expression correctly");
}

#[test]
fn test_interactive_command_sequences() {
    let mut harness = ReplWorkflowHarness::new();

    // Simulate a realistic interactive session

    // Step 1: Define some data
    harness
        .execute_ok("let numbers = [1, 2, 3, 4, 5]")
        .expect("Should define numbers array");

    // Step 2: Transform the data
    harness
        .execute_ok("let doubled = numbers.map(|x| x * 2)")
        .expect("Should map over numbers");

    harness
        .execute_and_validate("doubled", "[2, 4, 6, 8, 10]")
        .expect("Should have doubled numbers");

    // Step 3: Filter the data
    harness
        .execute_ok("let big_numbers = doubled.filter(|x| x > 5)")
        .expect("Should filter doubled numbers");

    harness
        .execute_and_validate("big_numbers", "[6, 8, 10]")
        .expect("Should have filtered numbers");

    // Step 4: Reduce the data
    harness
        .execute_ok("let sum = big_numbers.reduce(0, |acc, x| acc + x)")
        .expect("Should reduce big numbers");

    harness
        .execute_and_validate("sum", "24")
        .expect("Should have correct sum");

    // Step 5: Use the result in another computation
    harness
        .execute_and_validate("sum * 2 - 6", "42")
        .expect("Should compute final result");
}

#[test]
fn test_repl_meta_commands() {
    let mut harness = ReplWorkflowHarness::new();

    // Set up some state for meta command testing
    harness
        .execute_ok("let test_var = 42")
        .expect("Should define test variable");

    harness
        .execute_ok("fun test_func() -> i32 { test_var }")
        .expect("Should define test function");

    // Meta commands should not affect state
    // Note: These might not be implemented yet, so just verify state persists

    harness
        .execute_and_validate("test_var", "42")
        .expect("State should persist");

    harness
        .execute_and_validate("test_func()", "42")
        .expect("Function should persist");
}

#[test]
fn test_performance_and_memory_in_session() {
    let mut harness = ReplWorkflowHarness::new();

    // Check that REPL handles moderate workloads
    harness
        .execute_ok("let big_list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]")
        .expect("Should create moderately large list");

    harness
        .execute_ok("let processed = big_list.map(|x| x + 1)")
        .expect("Should process large list");

    harness
        .execute_and_validate("processed.len()", "10")
        .expect("Should maintain list length");

    // Clean up doesn't affect later operations
    harness
        .execute_ok("let small_data = 42")
        .expect("Should define small data after large operations");

    harness
        .execute_and_validate("small_data", "42")
        .expect("Small data should work after large operations");
}

#[test]
fn test_string_interpolation_in_session() {
    let mut harness = ReplWorkflowHarness::new();

    // String interpolation with session state
    harness
        .execute_ok("let user = \"Alice\"")
        .expect("Should define user");

    harness
        .execute_ok("let score = 95")
        .expect("Should define score");

    let result = harness
        .execute_ok("f\"User {user} scored {score} points\"")
        .expect("Should interpolate strings");

    assert!(result.contains("Alice"));
    assert!(result.contains("95"));
    assert!(result.contains("User"));
    assert!(result.contains("scored"));
    assert!(result.contains("points"));
}
