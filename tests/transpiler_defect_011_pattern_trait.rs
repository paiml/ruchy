#![allow(missing_docs)]
//! TRANSPILER-DEFECT-011: String arguments to contains() need &str coercion
//!
//! **Problem**: String field access / variables passed to contains() fail with E0277
//! **Discovered**: 2025-10-31 (Issue #111 - Reaper project 3 E0277 errors)
//! **Severity**: CRITICAL (blocks string pattern matching)
//!
//! Expected: `text.contains(string_var)` should work
//! Actual: E0277 - the trait `Pattern` is not implemented for `String`
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== GREEN PHASE: Tests Should Pass Now ====================

/// Test 1: Field access in contains() (main bug from reaper)
#[test]
fn test_defect_011_green_field_access_pattern() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Config {
    pattern: String,
}

fun test_pattern(cfg: Config) -> bool {
    let text = "hello world";
    text.contains(cfg.pattern)
}

fun main() {
    let config = Config { pattern: String::from("world") };
    let result = test_pattern(config);
    println!("{}", result);
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

/// Test 2: Variable identifier in contains()
#[test]
fn test_defect_011_green_variable_pattern() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun search_text(text: String, pattern: String) -> bool {
    text.contains(pattern)
}

fun main() {
    let haystack = String::from("hello world");
    let needle = String::from("world");
    let found = search_text(haystack, needle);
    println!("{}", found);
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

/// Test 3: Lowercase pattern (reaper real-world case)
#[test]
fn test_defect_011_green_lowercase_pattern() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun match_process(name: String, pattern: String) -> bool {
    let name_lower = name.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    name_lower.contains(pattern_lower)
}

fun main() {
    let result = match_process(String::from("Python"), String::from("python"));
    println!("{}", result);
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

/// Test 4: String literal should NOT get & (baseline)
#[test]
fn test_defect_011_baseline_string_literal() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "hello world";
    let result = text.contains("world");
    println!("{}", result);
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

/// Test 5: Multiple contains() calls
#[test]
fn test_defect_011_green_multiple_contains() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
struct Rule {
    name_pattern: String,
    cmd_pattern: String,
}

fun matches(proc_name: String, proc_cmd: String, rule: Rule) -> bool {
    if !proc_name.contains(rule.name_pattern) {
        return false;
    }
    if !proc_cmd.contains(rule.cmd_pattern) {
        return false;
    }
    true
}

fun main() {
    let rule = Rule {
        name_pattern: String::from("python"),
        cmd_pattern: String::from("script")
    };
    let result = matches(String::from("python3"), String::from("myscript.py"), rule);
    println!("{}", result);
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

// ==================== GREEN PHASE SUMMARY ====================

/// Summary test to document GREEN phase state
#[test]
fn test_defect_011_green_phase_summary() {
    println!("TRANSPILER-DEFECT-011 GREEN Phase:");
    println!("- 5 tests created that MUST PASS (fix implemented)");
    println!("- Covers: field access, variables, lowercase, literals, multiple calls");
    println!();
    println!("Fix location: src/backend/transpiler/statements.rs:1542-1554");
    println!("Fix approach: Wrap FieldAccess/Identifier args with & for contains()");
    println!();
    println!("Real-world impact: Reaper project 13 → 10 errors (3 E0277 eliminated, 23% reduction)");
}
