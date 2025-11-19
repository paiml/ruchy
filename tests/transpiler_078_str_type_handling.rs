#![allow(missing_docs)]
//! TRANSPILER-078: Fix str type transpilation (str → &str)
//!
//! **Problem**: Transpiler emits `str` instead of `&str` for function parameters
//! **GitHub Issue**: #13
//! **Severity**: HIGH - Breaks Rust compilation for string functions
//!
//! **Root Cause**: Type conversion in transpiler doesn't handle str as &str
//! **Expected**: `fun greet(name: str)` → `fn greet(name: &str)`
//! **Actual**: `fun greet(name: str)` → `fn greet(name: str)` (BROKEN)
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Simple string parameter function should compile successfully
///
/// This is the PRIMARY test from GitHub Issue #13
#[test]
fn test_transpiler_078_red_01_simple_string_parameter() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun greet(name: str) {
    println("Hello, {}!", name)
}

fun main() {
    greet("World")
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // This should compile successfully (currently FAILS)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 2: Multiple string parameters
#[test]
fn test_transpiler_078_red_02_multiple_string_parameters() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun concat(a: str, b: str) -> String {
    format!("{}{}", a, b)
}

fun main() {
    let result = concat("Hello", "World");
    println("{}", result)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 3: String parameter with return type
#[test]
fn test_transpiler_078_red_03_string_return_type() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun echo(msg: str) -> str {
    msg
}

fun main() {
    let result = echo("test");
    println("{}", result)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 4: Verify transpiled output contains &str not str
#[test]
fn test_transpiler_078_red_04_transpile_output_has_ampersand_str() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun greet(name: str) {
    println("Hello, {}!", name)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .output()
        .expect("Failed to execute ruchy");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // CRITICAL ASSERTIONS:
    // 1. Must contain &str (correct)
    // 2. Must NOT contain ": str)" (incorrect - str as parameter)

    assert!(
        stdout.contains("&str"),
        "Expected &str in transpiled output, but got: {stdout}"
    );

    // This checks for ": str)" pattern which indicates unsized str parameter (WRONG)
    assert!(
        !stdout.contains(": str)"),
        "Found ': str)' pattern (unsized str parameter - BUG!), output: {stdout}"
    );
}

/// Test 5: Execute compiled binary and verify output
#[test]
fn test_transpiler_078_red_05_execute_compiled_binary() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let binary = temp.path().join("test_binary");

    let code = r#"
fun greet(name: str) {
    println("Hello, {}!", name)
}

fun main() {
    greet("World")
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Compile
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .assert()
        .success();

    // Execute and verify output
    Command::new(&binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

/// Test 6: Edge case - empty string
#[test]
fn test_transpiler_078_red_06_empty_string() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun process(input: str) -> i32 {
    input.len() as i32
}

fun main() {
    let result = process("");
    println("{}", result)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 7: String in struct field
#[test]
fn test_transpiler_078_red_07_string_in_struct() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Person {
    name: String,
    age: i32
}

fun main() {
    let p = Person { name: "Alice".to_string(), age: 30 };
    println("{}", p.name)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test documenting fix status
#[test]
fn test_transpiler_078_red_phase_summary() {
    println!("TRANSPILER-078 Status: ✅ PRIMARY FIX COMPLETE");
    println!();
    println!("Fix Applied: src/backend/transpiler/types.rs:83");
    println!("  Before: 'str' => quote! {{ str }}");
    println!("  After:  'str' => quote! {{ &str }}");
    println!();
    println!("Impact:");
    println!("✅ Function parameters: 'name: str' → 'name: &str' (sized, borrowing)");
    println!("ℹ️  Return types: Use 'String' for owned data (e.g., format! returns String)");
    println!("ℹ️  Struct fields: Use 'String' for owned data (avoids lifetime annotations)");
    println!();
    println!("Idiomatic Rust string types:");
    println!("  - Parameters: &str (borrowed, most flexible)");
    println!("  - Returns: String (owned) or &str with lifetime");
    println!("  - Struct fields: String (owned, simpler) or &str with lifetime");
    println!();
    println!("GitHub Issue #13: ✅ RESOLVED");
}

// ==================== PROPERTY TESTS (10K+ Random String Functions) ====================

/// Property test: Random function names with str parameters should compile
#[test]
#[ignore = "Property test with 10K cases - run explicitly with --ignored flag"]
fn test_transpiler_078_property_01_random_string_functions() {
    proptest!(ProptestConfig::with_cases(10000), |(
        fn_name in "[a-z][a-z0-9_]{0,15}",
        param_name in "[a-z][a-z0-9_]{0,15}",
        output_str in "[a-zA-Z0-9 ]{1,20}"
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");

        let code = format!(r#"
fun {fn_name}({param_name}: str) {{
    println("{{}}", {param_name})
}}

fun main() {{
    {fn_name}("{output_str}")
}}
"#);

        fs::write(&source, code).expect("Failed to write test file");

        // Should compile successfully
        let result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(result.status.success(),
            "Compilation failed for fn {}: {}",
            fn_name, String::from_utf8_lossy(&result.stderr)
        );

        // Transpiled output should contain &str, not unsized str
        let transpile_result = ruchy_cmd()
            .arg("transpile")
            .arg(&source)
            .output()
            .expect("Failed to transpile");

        let stdout = String::from_utf8_lossy(&transpile_result.stdout);
        prop_assert!(stdout.contains("&str"),
            "Expected &str in transpiled output for fn {}", fn_name
        );
        prop_assert!(!stdout.contains(": str)"),
            "Found unsized ': str)' pattern for fn {}", fn_name
        );
    });
}

/// Property test: Multiple string parameters should all be &str
#[test]
#[ignore = "Property test with 5K cases - run explicitly with --ignored flag"]
fn test_transpiler_078_property_02_multiple_params() {
    proptest!(ProptestConfig::with_cases(5000), |(
        param_count in 2usize..5,
        fn_name in "[a-z][a-z0-9_]{0,10}"
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");

        // Generate multiple parameters
        let params: Vec<String> = (0..param_count)
            .map(|i| format!("p{i}: str"))
            .collect();
        let param_str = params.join(", ");

        let param_uses: Vec<String> = (0..param_count)
            .map(|i| format!("p{i}"))
            .collect();

        let code = format!(r#"
fun {fn_name}({param_str}) {{
    println("{{}}", {})
}}

fun main() {{
    {fn_name}({})
}}
"#, param_uses[0], (0..param_count).map(|_| "\"test\"").collect::<Vec<_>>().join(", "));

        fs::write(&source, code).expect("Failed to write test file");

        let result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(result.status.success(),
            "Compilation failed for {} params: {}",
            param_count, String::from_utf8_lossy(&result.stderr)
        );
    });
}

/// Property test: str parameters with different body complexities compile
#[test]
#[ignore = "Property test with 3K cases - run explicitly with --ignored flag"]
fn test_transpiler_078_property_03_complex_bodies() {
    proptest!(ProptestConfig::with_cases(3000), |(
        use_if in prop::bool::ANY,
        use_match in prop::bool::ANY,
        fn_name in "[a-z][a-z0-9_]{0,10}"
    )| {
        let temp = temp_dir();
        let source = temp.path().join("test.ruchy");

        let body = if use_if {
            "if input.len() > 0 { println(\"non-empty\") } else { println(\"empty\") }"
        } else if use_match {
            "match input.len() { 0 => println(\"empty\"), _ => println(\"non-empty\") }"
        } else {
            "println(\"{}\", input)"
        };

        let code = format!(r#"
fun {fn_name}(input: str) {{
    {body}
}}

fun main() {{
    {fn_name}("test")
}}
"#);

        fs::write(&source, code).expect("Failed to write test file");

        let result = ruchy_cmd()
            .arg("compile")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("test_binary"))
            .output()
            .expect("Failed to execute ruchy");

        prop_assert!(result.status.success(),
            "Compilation failed for {}: {}",
            fn_name, String::from_utf8_lossy(&result.stderr)
        );
    });
}

/// Property test summary
#[test]
fn test_transpiler_078_property_summary() {
    println!("TRANSPILER-078 Property Tests:");
    println!("- Test 1: 10K random function names with str parameters");
    println!("- Test 2: 5K random multi-parameter functions (2-4 params)");
    println!("- Test 3: 3K random function bodies (if/match/simple)");
    println!();
    println!("Total: 18K property test cases");
    println!("Run with: cargo test --test transpiler_078_str_type_handling property -- --ignored --nocapture");
}
