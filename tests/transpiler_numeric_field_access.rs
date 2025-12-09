//! Comprehensive tests for numeric field access (tuple fields)
//!
//! DEFECT-PROPERTY-001: Property test found panic on input "A.0"
//! Root Cause: `format_ident`!() called on pure numbers
//! Fix: Check numeric fields BEFORE calling `format_ident`!()
//! Coverage: All object types × all numeric field patterns

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Single Digit Tuple Access
// ============================================================================

#[test]
fn test_numeric_field_single_digit_0() {
    // Regression: "A.0" caused panic in format_ident!
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("A.0")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Should transpile to: A.0 (Rust tuple access)
    assert!(
        output.contains("A.0"),
        "Should generate tuple access: {output}"
    );
}

#[test]
fn test_numeric_field_single_digit_1() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("obj.1")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("obj.1"));
}

#[test]
fn test_numeric_field_single_digit_9() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("result.9")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("result.9"));
}

// ============================================================================
// Multi-Digit Tuple Access
// ============================================================================

#[test]
fn test_numeric_field_two_digits() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("tuple.10")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("tuple.10"));
}

#[test]
fn test_numeric_field_three_digits() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("large.100")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("large.100"));
}

// ============================================================================
// Nested Tuple Access
// ============================================================================

#[test]
fn test_numeric_field_nested_simple() {
    // (nested.0).1 - access field 1 of the result of nested.0
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("(nested.0).1")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Should generate nested tuple access
    assert!(output.contains(".0") && output.contains(".1"));
}

#[test]
fn test_numeric_field_deeply_nested() {
    // ((data.0).1).2 - deeply nested
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("((data.0).1).2")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains(".0"));
    assert!(output.contains(".1"));
    assert!(output.contains(".2"));
}

// ============================================================================
// Different Object Types + Numeric Fields
// ============================================================================

#[test]
fn test_numeric_field_uppercase_identifier() {
    // DEFECT-PROPERTY-001: Original failing case
    // "A" is uppercase → matches "Type name" heuristic
    // field "0" → was calling format_ident!("0") → PANIC
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("Result.0")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Result.0"));
}

#[test]
fn test_numeric_field_lowercase_identifier() {
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("obj.0")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("obj.0"));
}

#[test]
fn test_numeric_field_module_like_identifier() {
    // module_name.0 → should use tuple access (not :: syntax)
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("my_module.0")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("my_module.0"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_numeric_field_zero_leading() {
    // 01, 007 → all digits, should work
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin("obj.007")
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("obj.7")); // Should parse as 7, not "007"
}

#[test]
fn test_numeric_field_in_expression() {
    // Tuple access as part of larger expression
    let code = r"
        let x = data.0 + data.1
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("data.0"));
    assert!(output.contains("data.1"));
}

#[test]
fn test_numeric_field_in_function_call() {
    // Tuple field as function argument
    let code = r"
        println(result.0)
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("result.0"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_numeric_fields_0_to_20() {
    // Property: All single/double digit tuple indices should transpile
    for i in 0..=20 {
        let code = format!("obj.{i}");
        let result = ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();

        let output = String::from_utf8_lossy(&result.get_output().stdout);
        assert!(
            output.contains(&format!("obj.{i}")),
            "Failed for index {i}: {output:?}"
        );
    }
}

#[test]
fn property_numeric_fields_all_object_types() {
    // Property: Numeric fields work with all identifier types
    let identifiers = vec!["obj", "Result", "my_module", "X", "data123"];

    for ident in identifiers {
        let code = format!("{ident}.0");
        let result = ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();

        let output = String::from_utf8_lossy(&result.get_output().stdout);
        assert!(
            output.contains(&format!("{ident}.0")),
            "Failed for identifier '{ident}': {output:?}"
        );
    }
}

#[test]
fn property_nested_depth_1_to_5() {
    // Property: Nested tuple access works to arbitrary depth
    for depth in 1..=5 {
        let mut code = "obj.0".to_string();
        for i in 1..depth {
            code = format!("({code}).{i}");
        }

        let result = ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();

        let output = String::from_utf8_lossy(&result.get_output().stdout);
        // Should contain all indices 0 to depth-1
        for i in 0..depth {
            assert!(
                output.contains(&format!(".{i}")),
                "Missing index {i} at depth {depth}: {output:?}"
            );
        }
    }
}

// ============================================================================
// Integration: Compile and Execute
// ============================================================================

#[test]
fn integration_compile_tuple_access() {
    // Full pipeline: transpile → compile → execute
    let code = r#"
        fn main() {
            let tuple = (42, "hello", 3.14);
            println!("{}", tuple.0);
        }
    "#;

    // Step 1: Transpile
    let transpile_result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let rust_code = String::from_utf8_lossy(&transpile_result.get_output().stdout);

    // Step 2: Verify contains tuple access
    assert!(rust_code.contains(".0"), "Should generate .0 tuple access");

    // Step 3: Write to temp file and compile
    std::fs::write("/tmp/tuple_test.rs", rust_code.as_ref()).expect("Failed to write temp file");

    let compile_result = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "/tmp/tuple_test.rs",
            "-o",
            "/tmp/tuple_test",
        ])
        .output()
        .expect("Failed to run rustc");

    assert!(
        compile_result.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile_result.stderr)
    );
}
