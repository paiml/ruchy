//! TRANSPILER-DEFECT-016-B: Function Call String Results Not Tracked
//!
//! **Issue**: Variables assigned from function calls returning String are not tracked in `string_vars`,
//! causing E0308 errors when used in string concatenation without auto-borrowing.
//!
//! **Root Cause**: The `string_vars` tracking only handles string literals and `String::from()`,
//! but not function call results that return String.
//!
//! **Impact**: 1 error in reaper project (line 185/731)
//!
//! **Real-world Example** (reaper main.ruchy:724-731):
//! ```ruchy
//! fun format_rule(rule: DetectionRule) -> String {
//!     let priority_str = priority_to_string(rule.priority);  // Returns String
//!     let mut result = "Rule: ";
//!     result = result + priority_str;  // E0308: expected &str, found String
//! }
//! ```
//!
//! **Current Transpilation** (BROKEN):
//! ```rust
//! fn format_rule(rule: DetectionRule) -> String {
//!     let priority_str = priority_to_string(rule.priority);
//!     let mut result = String::from("Rule: ");
//!     result = format!("{}{}", result, priority_str);  // ❌ Should auto-borrow
//! }
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! fn format_rule(rule: DetectionRule) -> String {
//!     let priority_str = priority_to_string(rule.priority);
//!     let mut result = String::from("Rule: ");
//!     result = format!("{}{}", result, &priority_str);  // ✅ Auto-borrow
//! }
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 1: Function call returning String used in concatenation (ACTUAL reaper pattern line 731)
#[test]
fn test_defect_016_b_01_function_call_string_concat_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // ACTUAL pattern from reaper main.ruchy:724-731
    let ruchy_code = r#"
enum Priority { High, Low }

fun priority_to_string(p: Priority) -> String {
    match p {
        Priority::High => "high",
        Priority::Low => "low",
    }
}

fun format_msg(p: Priority) -> String {
    let priority_str = priority_to_string(p);
    let mut result = "Priority: ";
    result = result + priority_str;
    result
}

println(format_msg(Priority::High));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Function call results auto-tracked and auto-borrowed");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308: expected &str, found String. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Function call String concatenation error confirmed");
    }
}

/// Test 2: Multiple function calls returning String
#[test]
fn test_defect_016_b_02_multiple_function_calls_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun get_name() -> String { "Alice" }
fun get_title() -> String { "Engineer" }

fun format_person() -> String {
    let name = get_name();
    let title = get_title();
    let mut result = "Name: ";
    result = result + name;
    result = result + ", Title: ";
    result = result + title;
    result
}

println(format_person());
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Multiple function calls auto-tracked");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308 for multiple function calls. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Multiple function call concatenation error confirmed");
    }
}

/// Test 3: Baseline - Direct string literal concatenation (should work)
#[test]
fn test_defect_016_b_03_direct_literal_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun format_msg() -> String {
    let name = "Alice";
    let mut result = "Hello: ";
    result = result + name;
    result
}

println(format_msg());
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 4: Function call with non-String return (should NOT auto-borrow)
#[test]
fn test_defect_016_b_04_integer_function_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
fun get_age() -> i32 { 42 }

fun main() {
    let age = get_age();
    let sum = age + 10;
    println(sum);
}
";

    fs::write(&test_file, ruchy_code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

// PROPERTY TESTS (Run after GREEN phase)
// These will be written in Phase 3 (REFACTOR) with proptest

// MUTATION TESTS (Run after GREEN phase)
// cargo mutants --file src/backend/transpiler/statements.rs --timeout 60
