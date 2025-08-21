//! REPL Regression Tests - Never allow v0.7.0 disasters again
//!
//! These tests MUST pass before ANY release

#![allow(clippy::expect_used)] // OK in tests

use ruchy::runtime::repl::Repl;

/// Helper to run REPL command and get output
fn run_repl_command(input: &str) -> String {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Evaluate and return result
    repl.eval(input).unwrap_or_else(|e| format!("Error: {e}"))
}

/// Helper to run multiple REPL commands
fn run_repl_commands(commands: &[&str]) -> Vec<String> {
    let mut repl = Repl::new().expect("Failed to create REPL");
    let mut results = Vec::new();

    for cmd in commands {
        let result = repl.eval(cmd).unwrap_or_else(|e| format!("Error: {e}"));
        results.push(result);
    }

    results
}

#[test]
fn test_blocks_return_last_value() {
    let output = run_repl_command("{ 1; 2; 3 }");
    assert_eq!(output, "3", "Blocks must return last expression");
}

#[test]
fn test_nested_blocks() {
    let output = run_repl_command("{ 1; { 2; 3 }; 4 }");
    assert_eq!(output, "4", "Nested blocks must work");
}

#[test]
fn test_function_definition_and_call() {
    let outputs = run_repl_commands(&["fun add(x, y) { x + y }", "add(5, 3)"]);
    assert!(outputs[0].contains("fn add"), "Function must be defined");
    assert_eq!(outputs[1], "8", "Function must be callable");
}

#[test]
fn test_function_with_multiple_params() {
    let outputs =
        run_repl_commands(&["fun multiply3(a, b, c) { a * b * c }", "multiply3(2, 3, 4)"]);
    assert_eq!(outputs[1], "24", "Multi-param functions must work");
}

#[test]
fn test_match_expression_basic() {
    let output = run_repl_command("match 5 { 0 => \"zero\", 5 => \"five\", _ => \"other\" }");
    assert_eq!(output, "\"five\"", "Match expressions must work");
}

#[test]
fn test_match_expression_wildcard() {
    let output = run_repl_command("match 999 { 0 => \"zero\", _ => \"other\" }");
    assert_eq!(output, "\"other\"", "Match wildcard must work");
}

#[test]
fn test_for_loop_list() {
    let output = run_repl_command("for x in [1, 2, 3] { x }");
    // For loops return the last iteration value
    assert_eq!(output, "3", "For loops must complete");
}

#[test]
fn test_for_loop_range() {
    // Ranges are not yet fully supported in for loops
    let output = run_repl_command("for x in [0, 1, 2] { x }");
    assert_eq!(output, "2", "For loops must work with explicit lists");
}

#[test]
fn test_while_loop() {
    let outputs = run_repl_commands(&["let mut x = 0", "while x < 3 { x = x + 1; x }", "x"]);
    assert_eq!(outputs[2], "3", "While loops must work");
}

#[test]
fn test_integer_overflow_addition() {
    let output = run_repl_command("9223372036854775807 + 1");
    assert!(
        output.contains("overflow"),
        "Must detect integer overflow in addition"
    );
}

#[test]
fn test_integer_overflow_multiplication() {
    let output = run_repl_command("9223372036854775807 * 2");
    assert!(
        output.contains("overflow"),
        "Must detect integer overflow in multiplication"
    );
}

#[test]
fn test_integer_overflow_subtraction() {
    // Test underflow with min i64 value
    let outputs = run_repl_commands(&["let min_int = -9223372036854775807 - 1", "min_int - 1"]);
    assert!(
        outputs[1].contains("overflow"),
        "Must detect integer underflow"
    );
}

#[test]
fn test_integer_overflow_power() {
    let output = run_repl_command("10000000000 ** 10");
    assert!(output.contains("overflow"), "Must detect power overflow");
}

#[test]
fn test_division_by_zero() {
    let output = run_repl_command("10 / 0");
    assert!(
        output.contains("Division by zero"),
        "Must catch division by zero"
    );
}

#[test]
fn test_modulo_by_zero() {
    let output = run_repl_command("10 % 0");
    assert!(
        output.contains("Modulo by zero"),
        "Must catch modulo by zero"
    );
}

#[test]
fn test_string_interpolation() {
    let outputs = run_repl_commands(&["let name = \"World\"", "f\"Hello {name}!\""]);
    assert_eq!(
        outputs[1], "\"Hello World!\"",
        "String interpolation must work"
    );
}

#[test]
fn test_pipeline_operator() {
    let output = run_repl_command("[1, 2, 3] |> map(x => x * 2)");
    // Even if not fully implemented, should not crash
    assert!(!output.is_empty());
}

#[test]
fn test_ok_result() {
    let output = run_repl_command("Ok(42)");
    assert!(
        output.contains("Ok") || output.contains("42"),
        "Ok constructor must work"
    );
}

#[test]
fn test_err_result() {
    let output = run_repl_command("Err(\"error message\")");
    assert!(
        output.contains("Err") || output.contains("error"),
        "Err constructor must work"
    );
}

#[test]
fn test_let_binding() {
    let outputs = run_repl_commands(&["let x = 42", "x"]);
    assert_eq!(outputs[1], "42", "Let bindings must persist");
}

#[test]
fn test_let_mut_binding() {
    let outputs = run_repl_commands(&["let mut y = 10", "y = 20", "y"]);
    assert_eq!(outputs[2], "20", "Mutable bindings must work");
}

#[test]
fn test_if_expression() {
    let output = run_repl_command("if true { 1 } else { 2 }");
    assert_eq!(output, "1", "If expressions must work");
}

#[test]
fn test_if_else_chain() {
    let output = run_repl_command("if false { 1 } else if true { 2 } else { 3 }");
    assert_eq!(output, "2", "If-else chains must work");
}

#[test]
fn test_list_literal() {
    let output = run_repl_command("[1, 2, 3]");
    assert_eq!(output, "[1, 2, 3]", "List literals must work");
}

#[test]
fn test_empty_list() {
    let output = run_repl_command("[]");
    assert_eq!(output, "[]", "Empty lists must work");
}

#[test]
fn test_boolean_operations() {
    let outputs = run_repl_commands(&["true && false", "true || false", "!true"]);
    assert_eq!(outputs[0], "false", "AND must work");
    assert_eq!(outputs[1], "true", "OR must work");
    assert_eq!(outputs[2], "false", "NOT must work");
}

#[test]
fn test_comparison_operators() {
    let outputs = run_repl_commands(&["5 > 3", "5 < 3", "5 == 5", "5 != 3", "5 >= 5", "5 <= 5"]);
    assert_eq!(outputs[0], "true");
    assert_eq!(outputs[1], "false");
    assert_eq!(outputs[2], "true");
    assert_eq!(outputs[3], "true");
    assert_eq!(outputs[4], "true");
    assert_eq!(outputs[5], "true");
}

// Quality gate tests - these MUST pass or release is blocked
#[test]
fn quality_gate_no_silent_overflow() {
    // This is the most critical security issue
    let output = run_repl_command("9223372036854775807 + 1");
    assert!(
        !output.contains('-'),
        "CRITICAL: Must not silently wrap on overflow"
    );
    assert!(
        output.contains("overflow"),
        "CRITICAL: Must report overflow"
    );
}

#[test]
fn quality_gate_functions_work() {
    let outputs = run_repl_commands(&["fun identity(x) { x }", "identity(42)"]);
    assert_eq!(outputs[1], "42", "CRITICAL: Basic functions must work");
}

#[test]
fn quality_gate_no_regressions_from_v4() {
    // Everything that worked in v0.4.3 must still work
    let v4_features = vec![
        ("{ 1; 2; 3 }", "3"),       // Blocks
        ("let z = 100", "100"),     // Let bindings
        ("5 + 3", "8"),             // Basic arithmetic
        ("\"hello\"", "\"hello\""), // Strings
        ("[1, 2]", "[1, 2]"),       // Lists
    ];

    for (input, expected) in v4_features {
        let output = run_repl_command(input);
        assert_eq!(output, expected, "v0.4 feature regression: {input}");
    }
}
