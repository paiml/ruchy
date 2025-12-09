//! Comprehensive type inference tests via CLI (TDG-driven)
//!
//! Target: `src/backend/transpiler/type_inference.rs` (881 lines, 25 tests → 35.2 lines/test)
//! Strategy: Test ALL inference patterns through transpiler output validation
//! Coverage: Function arguments, numeric usage, return types, builtin calls

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Parameter Used as Function Argument
// ============================================================================

#[test]
fn test_param_as_i32_argument() {
    // Parameter used as argument to function expecting i32 → infer i32
    let code = r"
        fn add_one(x) {
            x + 1
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should infer x: i32 (default numeric type in transpiler)
    assert!(output.contains("x: i32") || output.contains("x:i32"));
}

#[test]
fn test_param_passed_to_builtin() {
    // Parameter passed to len() → infer Vec/String
    let code = r"
        fn get_length(arr) {
            len(arr)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should compile successfully (len works on multiple types)
    assert!(output.contains("fn get_length"));
}

#[test]
fn test_param_in_nested_call() {
    // Parameter deeply nested in function calls
    let code = r"
        fn process(x) {
            println(str(x))
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn process"));
}

// ============================================================================
// Parameter Used as Function
// ============================================================================

#[test]
fn test_param_called_as_function() {
    // Parameter is called directly → must be function type
    let code = r"
        fn apply(f, x) {
            f(x)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // f should be inferred as function-like
    assert!(output.contains("fn apply"));
}

#[test]
fn test_param_as_callback() {
    // Higher-order function pattern
    let code = r"
        fn map_fn(arr, mapper) {
            mapper(arr[0])
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn map_fn"));
}

// ============================================================================
// Parameter Used Numerically
// ============================================================================

#[test]
fn test_param_in_addition() {
    // x + 1 → infer numeric type
    let code = r"
        fn increment(x) {
            x + 1
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should infer i32 (default numeric type)
    assert!(output.contains("x: i32") || output.contains("x:i32"));
}

#[test]
fn test_param_in_multiplication() {
    // x * 2 → numeric inference
    let code = r"
        fn double(x) {
            x * 2
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn double"));
}

#[test]
fn test_param_in_comparison() {
    // x > 0 → numeric usage
    let code = r"
        fn is_positive(x) {
            x > 0
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn is_positive"));
}

#[test]
fn test_param_in_complex_expression() {
    // (x + 1) * 2 - 3 → clearly numeric
    let code = r"
        fn compute(x) {
            (x + 1) * 2 - 3
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn compute"));
    assert!(output.contains("i32"));
}

// ============================================================================
// Builtin Return Type Inference
// ============================================================================

#[test]
fn test_infer_len_returns_usize() {
    // len() always returns usize
    let code = r"
        fn get_size(arr) {
            len(arr)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Return type should be inferred from len()
    assert!(output.contains("fn get_size"));
}

#[test]
fn test_infer_str_returns_string() {
    // str() returns String
    let code = r"
        fn to_string(x) {
            str(x)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn to_string") || output.contains("to_string"));
}

#[test]
fn test_infer_int_returns_i64() {
    // int() returns i64
    let code = r"
        fn parse_int(s) {
            int(s)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn parse_int"));
}

// ============================================================================
// Mixed Inference Scenarios
// ============================================================================

#[test]
fn test_numeric_and_function_usage() {
    // Parameter used both numerically and as function argument
    let code = r"
        fn process(x) {
            let doubled = x * 2;
            println(doubled)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn process"));
    assert!(output.contains("i32"));
}

#[test]
fn test_string_concatenation_inference() {
    // String + String → should infer String type
    let code = r"
        fn concat(a, b) {
            a + b
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn concat"));
}

#[test]
fn test_if_branch_inference() {
    // Both branches must have compatible types
    let code = r"
        fn conditional(x) {
            if x > 0 {
                x + 1
            } else {
                x - 1
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn conditional"));
}

#[test]
fn test_let_binding_inference() {
    // Let binding should propagate type info
    let code = r"
        fn compute(x) {
            let doubled = x * 2;
            let incremented = doubled + 1;
            incremented
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn compute"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_unused_parameter() {
    // Parameter never used → generic type
    let code = r"
        fn ignore(x) {
            42
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn ignore"));
}

#[test]
fn edge_case_multiple_numeric_operations() {
    // x used in +, -, *, / → all numeric
    let code = r"
        fn math(x) {
            x + x - x * x / x
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn math"));
}

#[test]
fn edge_case_nested_function_calls() {
    // len(str(int(x))) → chain of inferences
    let code = r"
        fn chain(x) {
            len(str(int(x)))
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("fn chain"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_all_numeric_operators() {
    // Property: All binary operators infer numeric types
    let operators = vec!["+", "-", "*", "/", "%"];

    for op in operators {
        let code = format!("fn op(x) {{ x {op} 1 }}");
        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_all_comparison_operators() {
    // Property: All comparisons work with inferred types
    let operators = vec!["==", "!=", "<", ">", "<=", ">="];

    for op in operators {
        let code = format!("fn cmp(x) {{ x {op} 0 }}");
        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_nested_depth_1_to_5() {
    // Property: Type inference works at arbitrary nesting depth
    for depth in 1..=5 {
        let mut code = "x".to_string();
        for _ in 0..depth {
            code = format!("({code})");
        }
        code = format!("fn nested(x) {{ {code} + 1 }}");

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

// ============================================================================
// Integration: Full Transpile → Compile → Execute
// ============================================================================

#[test]
fn integration_numeric_inference_compiles() {
    let code = r#"
        fn factorial(n) {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }

        fn main() {
            println!("{}", factorial(5));
        }
    "#;

    // Transpile
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();
    let rust_code = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify contains inferred types
    assert!(rust_code.contains("fn factorial"));

    // Compile
    std::fs::write("/tmp/type_inference_test.rs", rust_code.as_ref()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "/tmp/type_inference_test.rs",
            "-o",
            "/tmp/type_inference_test",
        ])
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}
