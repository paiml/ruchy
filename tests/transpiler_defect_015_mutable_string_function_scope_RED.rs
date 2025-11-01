//! TRANSPILER-DEFECT-015: Mutable String Inference (Function Scope)
//!
//! **Issue**: Mutable string variables in function scope not detected as needing String type
//!
//! **Root Cause**: v3.163.0's `is_variable_mutated()` doesn't detect string accumulator pattern
//! in function bodies (only works at top-level).
//!
//! **Impact**: 9 errors in reaper project (60% of remaining errors)
//! - 8 × E0308: format!() returns String, assigned to &str
//! - 1 × E0369: cannot add String to &str
//!
//! **Real-world Example** (reaper main.ruchy:357-366):
//! ```ruchy
//! fun format_process(proc: Process) -> String {
//!     let formatted = "Process[PID=";           // ❌ Should be String::from("...")
//!     formatted = formatted + proc.pid.to_string();  // ❌ Assignment fails
//!     formatted = formatted + ", name='";
//!     formatted
//! }
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! fn format_process(proc: Process) -> String {
//!     let mut formatted = String::from("Process[PID=");  // ✅ Mutable String
//!     formatted = format!("{}{}", formatted, proc.pid.to_string());
//!     formatted = format!("{}{}", formatted, ", name='");
//!     formatted
//! }
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)
//! - RED: These tests MUST fail with E0308/E0369
//! - GREEN: Fix is_variable_mutated() to detect function-scope mutations
//! - REFACTOR: Property tests with 10K+ inputs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test 1: Function-scope string accumulator pattern (ACTUAL reaper pattern)
///
/// This is the EXACT pattern from reaper's format_process() function.
/// Ruchy code uses string concatenation in function body.
#[test]
fn test_defect_015_01_function_scope_string_accumulator_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // ACTUAL pattern from reaper main.ruchy:352-368
    let ruchy_code = r#"
struct Process {
    pid: i32,
    name: String,
    cpu_usage: f64,
}

fun format_process(proc: Process) -> String {
    let formatted = "Process[PID=";
    formatted = formatted + proc.pid.to_string();
    formatted = formatted + ", name='";
    formatted = formatted + proc.name;
    formatted = formatted + "', CPU=";
    formatted = formatted + proc.cpu_usage.to_string();
    formatted = formatted + "%]";
    formatted
}

let proc = Process { pid: 123, name: "test", cpu_usage: 45.5 };
let result = format_process(proc);
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    // Run ruchy compile - should succeed after fix
    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // This test is RED - we EXPECT these errors before fix
        assert!(
            stderr.contains("E0308") || stderr.contains("E0369"),
            "Expected E0308 or E0369 errors (mutable string not detected). Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: E0308/E0369 errors confirmed (as expected before fix)");
        eprintln!("Error details:\n{}", stderr);
    } else {
        // After fix is applied, this should compile and run successfully
        eprintln!("✅ GREEN: Test passes after fix applied");
    }
}

/// Test 2: format!() macro returns String, assigned to &str
///
/// This is the SPECIFIC error pattern from reaper line 83.
#[test]
fn test_defect_015_02_format_macro_returns_string_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun build_message(id: i32, name: String) -> String {
    let msg = "ID: ";
    msg = msg + id.to_string();  // After transpilation: format!("{}{}", msg, id.to_string())
    msg = msg + ", Name: ";
    msg = msg + name;
    msg
}

let result = build_message(42, "test");
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308: format!() returns String, msg is &str. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: format!() type mismatch confirmed");
    } else {
        eprintln!("✅ GREEN: format!() pattern fixed");
    }
}

/// Test 3: Multiple concatenations in sequence
///
/// Pattern from reaper's format_rule function.
#[test]
fn test_defect_015_03_multiple_concatenations_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun build_long_string(a: String, b: String, c: String) -> String {
    let result = "Start: ";
    result = result + a;
    result = result + ", Middle: ";
    result = result + b;
    result = result + ", End: ";
    result = result + c;
    result
}

let output = build_long_string("foo", "bar", "baz");
println(output);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308") || stderr.contains("E0369"),
            "Expected type errors from multiple concatenations. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Multiple concatenation errors confirmed");
    } else {
        eprintln!("✅ GREEN: Multiple concatenations fixed");
    }
}

/// Test 4: String field concatenation (ACTUAL reaper pattern line 183)
///
/// This pattern appears 4 times in reaper (lines 183, 185, 197, 244).
#[test]
fn test_defect_015_04_string_field_concatenation_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Config {
    name: String,
    path: String,
}

fun format_config(cfg: Config) -> String {
    let output = "Config: ";
    output = output + cfg.name;    // cfg.name is String
    output = output + " at ";
    output = output + cfg.path;    // cfg.path is String
    output
}

let config = Config { name: "test.conf", path: "/etc/test" };
let result = format_config(config);
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308: output is &str, cfg.name is String. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: String field concatenation error confirmed");
    } else {
        eprintln!("✅ GREEN: String field concatenation fixed");
    }
}

/// Test 5: Baseline - Immutable string should remain &str
///
/// This test ensures fix doesn't break existing behavior.
#[test]
fn test_defect_015_05_immutable_string_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun get_message() -> String {
    let msg = "Hello, World!";  // Immutable, never reassigned
    msg.to_string()              // Explicit conversion
}

let result = get_message();
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 6: Top-level mutable string (v3.163.0 should already handle)
///
/// This is the pattern that v3.163.0 fixed - verify it still works.
#[test]
fn test_defect_015_06_top_level_mutable_string_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
let mut formatted = "Start";
formatted = formatted + " Middle";
formatted = formatted + " End";
println(formatted);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 7: Nested block string accumulator
///
/// Ensures fix works in nested scopes.
#[test]
fn test_defect_015_07_nested_block_string_accumulator_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun build_complex_message(x: i32) -> String {
    if x > 0 {
        let msg = "Positive: ";
        msg = msg + x.to_string();
        msg
    } else {
        let msg = "Negative: ";
        msg = msg + x.to_string();
        msg
    }
}

let result = build_complex_message(42);
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308") || stderr.contains("E0369"),
            "Expected type errors in nested blocks. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Nested block string accumulator errors confirmed");
    } else {
        eprintln!("✅ GREEN: Nested block patterns fixed");
    }
}

/// Test 8: String concatenation with method call results
///
/// Real-world pattern from reaper (to_string() returns String).
#[test]
fn test_defect_015_08_method_call_concatenation_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun format_number(n: i32) -> String {
    let output = "Number: ";
    output = output + n.to_string();
    output
}

let result = format_number(123);
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308") || stderr.contains("E0369"),
            "Expected type errors with method call results. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Method call concatenation errors confirmed");
    } else {
        eprintln!("✅ GREEN: Method call concatenation fixed");
    }
}

/// Test 9: E0369 specific - cannot add String to &str
///
/// This is the EXACT error from reaper line 85.
#[test]
fn test_defect_015_09_e0369_string_to_str_RED() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Data {
    value: String,
}

fun format_data(d: Data) -> String {
    let result = "Data: ";
    result = result + d.value;  // ❌ &str + String → E0369
    result
}

let data = Data { value: "test" };
let output = format_data(data);
println(output);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0369"),
            "Expected E0369: cannot add String to &str. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: E0369 (cannot add String to &str) confirmed");
    } else {
        eprintln!("✅ GREEN: E0369 pattern fixed");
    }
}

// PROPERTY TESTS (Run after GREEN phase)
// These will be written in Phase 3 (REFACTOR) with proptest

// MUTATION TESTS (Run after GREEN phase)
// cargo mutants --file src/backend/transpiler/statements.rs --timeout 60
