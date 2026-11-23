//! Comprehensive tests for `ruchy -e` flag combinations
//! Ensures 100% coverage of all eval mode flags
//!
//! Flags tested:
//! - `-e/--eval` (base flag, tested in all)
//! - `--format` (text, json)
//! - `-v/--verbose`
//! - `--trace`
//! - `--vm-mode` (ast, bytecode)

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Basic -e flag (baseline)
// ============================================================================

#[test]
fn test_eval_basic() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_long_form() {
    ruchy_cmd()
        .arg("--eval")
        .arg("println(21 + 21)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// --format flag tests
// ============================================================================

#[test]
fn test_eval_format_text_default() {
    // [CLI-EVAL-001] Default format is text, eval results ARE printed (REPL behavior)
    // This changed from CLI-UNIFY-003 which suppressed output
    ruchy_cmd()
        .arg("-e")
        .arg("2 + 2")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_eval_format_text_explicit() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(2 + 2)")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_eval_format_json_success() {
    ruchy_cmd()
        .arg("-e")
        .arg("2 + 2")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    // JSON format doesn't output for successful evals without println
}

#[test]
fn test_eval_format_json_error() {
    ruchy_cmd()
        .arg("-e")
        .arg("undefined_var")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("success").and(predicate::str::contains("false")));
}

// ============================================================================
// -v/--verbose flag tests
// ============================================================================

#[test]
fn test_eval_verbose_short() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(42)")
        .arg("-v")
        .assert()
        .success()
        .stderr(predicate::str::contains("Parsing expression"))
        .stderr(predicate::str::contains("Evaluation successful"));
}

#[test]
fn test_eval_verbose_long() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(21 + 21)")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Parsing expression: println(21 + 21)",
        ))
        .stderr(predicate::str::contains("Evaluation successful"));
}

#[test]
fn test_eval_verbose_error() {
    ruchy_cmd()
        .arg("-e")
        .arg("parse_int(\"invalid\")")
        .arg("-v")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Parsing expression"))
        .stderr(predicate::str::contains("Evaluation failed"));
}

// ============================================================================
// --trace flag tests (DEBUGGER-014)
// ============================================================================

#[test]
fn test_eval_trace_basic() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; println(x)")
        .arg("--trace")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_trace_function_call() {
    ruchy_cmd()
        .arg("-e")
        .arg("fun add(a, b) { a + b }; println(add(10, 32))")
        .arg("--trace")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// --vm-mode flag tests (**NEW - 0% coverage before**)
// ============================================================================

#[test]
fn test_eval_vm_mode_ast_default() {
    // Default vm-mode is ast
    ruchy_cmd()
        .arg("-e")
        .arg("println(2 + 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_eval_vm_mode_ast_explicit() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(10 * 4 + 2)")
        .arg("--vm-mode")
        .arg("ast")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_vm_mode_bytecode() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(21 * 2)")
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_vm_mode_bytecode_loop() {
    // Bytecode mode should handle loops
    ruchy_cmd()
        .arg("-e")
        .arg("let mut sum = 0; for i in 0..10 { sum = sum + i }; println(sum)")
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .stdout(predicate::str::contains("45")); // 0+1+2+...+9 = 45
}

#[test]
fn test_eval_vm_mode_bytecode_function() {
    ruchy_cmd()
        .arg("-e")
        .arg("fun double(x) { x * 2 }; println(double(21))")
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Flag combinations (stress testing)
// ============================================================================

#[test]
fn test_eval_all_flags_combined_ast() {
    // All flags together with ast mode
    ruchy_cmd()
        .arg("-e")
        .arg("println(42)")
        .arg("--format")
        .arg("text")
        .arg("-v")
        .arg("--trace")
        .arg("--vm-mode")
        .arg("ast")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"))
        .stderr(predicate::str::contains("Parsing expression"))
        .stderr(predicate::str::contains("Evaluation successful"));
}

#[test]
fn test_eval_all_flags_combined_bytecode() {
    // All flags together with bytecode mode
    ruchy_cmd()
        .arg("-e")
        .arg("println(21 + 21)")
        .arg("--format")
        .arg("text")
        .arg("--verbose")
        .arg("--trace")
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"))
        .stderr(predicate::str::contains("Parsing expression"))
        .stderr(predicate::str::contains("Evaluation successful"));
}

#[test]
fn test_eval_json_verbose_ast() {
    ruchy_cmd()
        .arg("-e")
        .arg("undefined_var")
        .arg("--format")
        .arg("json")
        .arg("-v")
        .arg("--vm-mode")
        .arg("ast")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"success\":false"))
        .stderr(predicate::str::contains("Evaluation failed"));
}

#[test]
fn test_eval_json_trace_bytecode() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(100 - 58)")
        .arg("--format")
        .arg("text")
        .arg("--trace")
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Edge cases and error conditions
// ============================================================================

#[test]
fn test_eval_empty_expression() {
    ruchy_cmd()
        .arg("-e")
        .arg("")
        .assert()
        .failure() // Empty expression fails with "Empty program" error
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn test_eval_whitespace_only() {
    ruchy_cmd()
        .arg("-e")
        .arg("   \n\t   ")
        .assert()
        .failure() // Whitespace-only fails with "Empty program" error
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn test_eval_syntax_error() {
    ruchy_cmd().arg("-e").arg("let x = ").assert().failure();
}

#[test]
fn test_eval_runtime_error_verbose() {
    ruchy_cmd()
        .arg("-e")
        .arg("parse_float(\"not_a_float\")")
        .arg("-v")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Evaluation failed"));
}

#[test]
fn test_eval_multiline_expression() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 21;\nlet y = 21;\nprintln(x + y)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_main_function() {
    // When main() is defined, it should be called automatically
    ruchy_cmd()
        .arg("-e")
        .arg("fun main() { println(42) }")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Property: All VM modes produce same output
// ============================================================================

#[test]
fn test_property_vm_mode_equivalence() {
    let code = "let x = 10; let y = 32; println(x + y)";

    // Run with AST mode
    let ast_output = ruchy_cmd()
        .arg("-e")
        .arg(code)
        .arg("--vm-mode")
        .arg("ast")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Run with bytecode mode
    let bytecode_output = ruchy_cmd()
        .arg("-e")
        .arg(code)
        .arg("--vm-mode")
        .arg("bytecode")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Both should produce identical output
    assert_eq!(
        ast_output,
        bytecode_output,
        "VM modes produced different output: ast={:?}, bytecode={:?}",
        String::from_utf8_lossy(&ast_output),
        String::from_utf8_lossy(&bytecode_output)
    );
}
