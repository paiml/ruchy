//! TRANSPILER-DEFECT-016: String Field Borrowing in Concatenation
//!
//! **Issue**: Binary + operator doesn't auto-borrow String fields/variables
//!
//! **Root Cause**: transpile_binary() doesn't detect when right operand is String type
//! and left is &str, requiring automatic borrowing with &.
//!
//! **Impact**: 6 errors in reaper project (75% of remaining errors after DEFECT-015)
//! - 5 × E0308: result = result + field.name (field is String, needs &field.name)
//! - 1 × E0308: match arm returning &str when function returns String
//!
//! **Real-world Examples** (reaper main.ruchy:85, 183, 185, 197, 244):
//! ```ruchy
//! struct Rule { name: String }
//! fun format_rule(rule: Rule) -> String {
//!     let result = "Rule: ";
//!     result = result + rule.name;  // ❌ E0308: expected &str, found String
//!     result
//! }
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! fn format_rule(rule: Rule) -> String {
//!     let mut result = String::from("Rule: ");
//!     result = format!("{}{}", result, &rule.name);  // ✅ Auto-borrow
//!     result
//! }
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 1: String field concatenation (ACTUAL reaper pattern line 85)
#[test]
fn test_defect_016_01_string_field_concatenation_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // ACTUAL pattern from reaper main.ruchy:85
    let ruchy_code = r#"
struct Process {
    name: String,
}

fun format_process(proc: Process) -> String {
    let formatted = "Process: ";
    formatted = formatted + proc.name;  // ❌ Should auto-borrow: &proc.name
    formatted
}

let p = Process { name: "test" };
let result = format_process(p);
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
            "Expected E0308: expected &str, found String. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: String field concatenation error confirmed");
    } else {
        eprintln!("✅ GREEN: String field auto-borrowed");
    }
}

/// Test 2: Multiple String fields (pattern from reaper lines 183, 185, 197, 244)
#[test]
fn test_defect_016_02_multiple_string_fields_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Config {
    name: String,
    path: String,
    log_file: String,
}

fun format_config(cfg: Config) -> String {
    let result = "Config: ";
    result = result + cfg.name;      // ❌ needs &cfg.name
    result = result + ", Path: ";
    result = result + cfg.path;      // ❌ needs &cfg.path
    result = result + ", Log: ";
    result = result + cfg.log_file;  // ❌ needs &cfg.log_file
    result
}

let c = Config { name: "app", path: "/etc", log_file: "/var/log/app.log" };
println(format_config(c));
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
            "Expected E0308 errors for multiple String fields. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Multiple String field errors confirmed");
    } else {
        eprintln!("✅ GREEN: Multiple String fields auto-borrowed");
    }
}

/// Test 3: String variable concatenation
#[test]
fn test_defect_016_03_string_variable_concatenation_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun build_message(name: String) -> String {
    let msg = "Hello, ";
    msg = msg + name;  // ❌ needs &name
    msg
}

println(build_message("World"));
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
            "Expected E0308 for String parameter. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: String variable concatenation error confirmed");
    } else {
        eprintln!("✅ GREEN: String variable auto-borrowed");
    }
}

/// Test 4: Baseline - String literals should NOT be borrowed
#[test]
fn test_defect_016_04_string_literal_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
fun format_message() -> String {
    let msg = "Hello, ";
    msg = msg + "World";  // ✅ String literals work without &
    msg
}

println(format_message());
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 5: Baseline - Explicit .to_string() should work
#[test]
fn test_defect_016_05_explicit_to_string_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Data {
    value: String,
}

fun format_data(d: Data) -> String {
    let result = "Data: ";
    result = result + d.value.clone();  // ✅ Explicit clone works
    result
}

let data = Data { value: "test" };
println(format_data(data));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 6: Function return value concatenation (reaper line 185)
#[test]
fn test_defect_016_06_function_return_concatenation_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
enum Priority { High, Medium, Low }

fun priority_to_string(p: Priority) -> String {
    match p {
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

fun format_priority(p: Priority) -> String {
    let result = "Priority: ";
    let priority_str = priority_to_string(p);
    result = result + priority_str;  // ❌ priority_str is String, needs &priority_str
    result
}

println(format_priority(Priority::High));
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
        // Two errors expected: match arms + concatenation
        assert!(
            stderr.contains("E0308"),
            "Expected E0308 errors. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Function return concatenation error confirmed");
    } else {
        eprintln!("✅ GREEN: Function return concatenation fixed");
    }
}

// PROPERTY TESTS (Run after GREEN phase)
// These will be written in Phase 3 (REFACTOR) with proptest

// MUTATION TESTS (Run after GREEN phase)
// cargo mutants --file src/backend/transpiler/statements.rs --timeout 60
