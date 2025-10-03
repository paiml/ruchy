// TDD Tests for REPL :debug mode (REPL-004)
//
// Requirements:
// 1. `:mode debug` should enable debug output for evaluations
// 2. Debug mode shows: AST + Transpiled Rust + Result
// 3. `:mode normal` should disable debug output
// 4. Debug mode persists across multiple evaluations
// 5. Mode changes should be reflected in REPL state

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_debug_mode_activation() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Activate debug mode
    let result = repl.eval(":mode debug").unwrap();
    assert!(
        result.contains("Debug") || result.contains("debug"),
        "Expected debug mode confirmation but got: {result}"
    );
}

#[test]
fn test_debug_mode_shows_ast() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Enable debug mode
    repl.eval(":mode debug").unwrap();

    // Evaluate simple expression
    let result = repl.eval("2 + 3").unwrap();

    // Should contain AST information
    assert!(
        result.contains("Binary") || result.contains("Add") || result.contains("AST"),
        "Expected AST output in debug mode but got: {result}"
    );
}

#[test]
fn test_debug_mode_shows_transpiled_code() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Enable debug mode
    repl.eval(":mode debug").unwrap();

    // Evaluate expression
    let result = repl.eval("2 + 3").unwrap();

    // Should contain transpiled Rust code markers
    assert!(
        result.contains("Rust") || result.contains("transpiled") || result.contains("fn "),
        "Expected transpiled code in debug mode but got: {result}"
    );
}

#[test]
fn test_debug_mode_shows_result() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Enable debug mode
    repl.eval(":mode debug").unwrap();

    // Evaluate expression
    let result = repl.eval("2 + 3").unwrap();

    // Should still contain the actual result
    assert!(
        result.contains('5') || result.contains("Result"),
        "Expected result in debug mode but got: {result}"
    );
}

#[test]
fn test_normal_mode_no_debug_output() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Start in normal mode (default)
    let result = repl.eval("2 + 3").unwrap();

    // Should NOT contain debug information
    assert!(
        !result.contains("Binary") && !result.contains("AST") && !result.contains("transpiled"),
        "Expected no debug output in normal mode but got: {result}"
    );

    // Should just show result
    assert!(
        result.contains('5'),
        "Expected simple result but got: {result}"
    );
}

#[test]
fn test_debug_mode_persistence() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Enable debug mode
    repl.eval(":mode debug").unwrap();

    // First evaluation
    let result1 = repl.eval("2 + 3").unwrap();
    assert!(
        result1.contains("Binary") || result1.contains("AST"),
        "Expected debug output in first eval"
    );

    // Second evaluation (debug mode should persist)
    let result2 = repl.eval("10 * 5").unwrap();
    assert!(
        result2.contains("Binary") || result2.contains("AST") || result2.contains("Multiply"),
        "Expected debug output to persist in second eval but got: {result2}"
    );
}

#[test]
fn test_mode_switch_back_to_normal() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Enable debug mode
    repl.eval(":mode debug").unwrap();

    // Verify debug mode works
    let debug_result = repl.eval("1 + 1").unwrap();
    assert!(debug_result.contains("Binary") || debug_result.contains("AST"));

    // Switch back to normal
    repl.eval(":mode normal").unwrap();

    // Verify normal mode restored
    let normal_result = repl.eval("1 + 1").unwrap();
    assert!(
        !normal_result.contains("Binary") && !normal_result.contains("AST"),
        "Expected normal output after mode switch but got: {normal_result}"
    );
}
