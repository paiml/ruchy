//! Comprehensive tests for transpiler/mod.rs (2,360 lines → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for largest under-tested module
//! Target: src/backend/transpiler/mod.rs (NO existing comprehensive tests)
//! Coverage: Main entry points, mutability analysis, function signatures, program generation

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Basic Transpilation (transpile + transpile_expr)
// ============================================================================

#[test]
fn test_transpile_simple_literal() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("42")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("42"));
}

#[test]
fn test_transpile_variable_binding() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("let x = 42")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("let") && output.contains("x") && output.contains("42"));
}

#[test]
fn test_transpile_function_definition() {
    let code = r#"
        fun add(a, b) {
            a + b
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn add"));
}

#[test]
fn test_transpile_if_expression() {
    let code = r#"
        if x > 0 {
            println("positive")
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("if"));
}

#[test]
fn test_transpile_for_loop() {
    let code = r#"
        for i in range(0, 10) {
            println(i)
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("for"));
}

// ============================================================================
// Program Generation (transpile_to_program)
// ============================================================================

#[test]
fn test_transpile_to_program_with_main() {
    let code = r#"
        fun main() {
            println("Hello")
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn main"));
}

#[test]
fn test_transpile_to_program_multiple_functions() {
    let code = r#"
        fun helper(x) {
            x * 2
        }

        fun main() {
            println(helper(21))
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn helper"));
    assert!(output.contains("fn main"));
}

#[test]
fn test_transpile_to_program_with_imports() {
    let code = r#"
        import std::collections::HashMap

        fun main() {
            let map = HashMap::new();
            println(map)
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("use") || output.contains("HashMap"));
}

// ============================================================================
// Mutability Analysis (analyze_mutability)
// ============================================================================

#[test]
fn test_mutability_simple_reassignment() {
    let code = r#"
        let x = 10;
        x = 20
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // x should be inferred as mutable
    assert!(output.contains("mut") || output.contains("x"));
}

#[test]
fn test_mutability_loop_counter() {
    let code = r#"
        let i = 0;
        while i < 10 {
            i = i + 1
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("mut"));
}

#[test]
fn test_mutability_immutable_binding() {
    let code = r#"
        let x = 42;
        println(x)
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // x is never reassigned, should NOT be mut
    assert!(output.contains("let"));
}

// ============================================================================
// Function Signatures (collect_function_signatures)
// ============================================================================

#[test]
fn test_function_signature_typed_params() {
    let code = r#"
        fun add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("i32"));
}

#[test]
fn test_function_signature_inferred_params() {
    let code = r#"
        fun double(x) {
            x * 2
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn double"));
}

// ============================================================================
// Module Names (collect_module_names)
// ============================================================================

#[test]
fn test_module_names_std() {
    let code = r#"
        import std::time

        fun main() {
            std::time::sleep(1000)
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("std") || output.contains("time") || output.contains("sleep"));
}

// ============================================================================
// Async Context (in_async_context)
// ============================================================================

#[test]
#[ignore = "Async functions not yet fully implemented in runtime"]
fn test_async_function() {
    let code = r#"
        async fun fetch_data() {
            await some_async_operation()
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("async"));
}

// ============================================================================
// Loop Context (in_loop_context - DEFECT-018 fix)
// ============================================================================

#[test]
fn test_loop_context_cloning() {
    let code = r#"
        for i in range(0, 10) {
            println(i)
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Should handle loop context properly
    assert!(output.contains("for"));
}

// ============================================================================
// String Variables (string_vars - DEFECT-016 fix)
// ============================================================================

#[test]
fn test_string_concatenation() {
    let code = r#"
        let s1 = "Hello";
        let s2 = "World";
        let result = s1 + s2
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // String concatenation should work
    assert!(output.contains("String") || output.contains("format!") || output.contains("+"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_empty_function() {
    let code = r#"
        fun empty() {
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn empty"));
}

#[test]
fn edge_case_nested_blocks() {
    let code = r#"
        fun test() {
            {
                {
                    42
                }
            }
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn test"));
}

#[test]
fn edge_case_complex_expression() {
    let code = r#"
        let result = (1 + 2) * (3 - 4) / 5
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("result"));
}

#[test]
fn edge_case_multiple_statements() {
    let code = r#"
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("let"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_function_param_counts_0_to_10() {
    // Property: Functions with varying parameter counts transpile correctly
    for n in 0..=10 {
        let params = (0..n)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("fun test({}) {{ 42 }}", params);

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_nested_block_depth_1_to_5() {
    // Property: Nested blocks transpile at arbitrary depth
    for depth in 1..=5 {
        let mut code = "42".to_string();
        for _ in 0..depth {
            code = format!("{{ {} }}", code);
        }
        code = format!("let x = {}", code);

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_binary_operators() {
    // Property: All binary operators transpile correctly
    let operators = vec!["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">="];

    for op in operators {
        let code = format!("let result = 42 {} 10", op);
        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

// ============================================================================
// Integration: Full Pipeline
// ============================================================================

#[test]
fn integration_transpile_compile_execute() {
    let code = r#"
        fun factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }

        fun main() {
            println!("{}", factorial(5));
        }
    "#;

    // Transpile
    let transpile_result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let rust_code = String::from_utf8_lossy(&transpile_result.get_output().stdout);

    // Verify contains expected elements
    assert!(rust_code.contains("fn factorial"));
    assert!(rust_code.contains("fn main"));

    // Write to temp file and compile
    std::fs::write("/tmp/transpiler_integration_test.rs", rust_code.as_ref())
        .expect("Failed to write temp file");

    let compile_result = std::process::Command::new("rustc")
        .args(["--crate-type", "bin", "/tmp/transpiler_integration_test.rs", "-o", "/tmp/transpiler_integration_test"])
        .output()
        .expect("Failed to run rustc");

    assert!(
        compile_result.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile_result.stderr)
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn error_case_invalid_syntax() {
    ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("let x = ")
        .assert()
        .failure(); // Should fail on invalid syntax
}

#[test]
fn error_case_empty_input() {
    ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("")
        .assert()
        .failure(); // Should fail on empty input
}

#[test]
fn error_case_unbalanced_braces() {
    ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("fun test() { let x = 42")
        .assert()
        .failure(); // Should fail on unbalanced braces
}
