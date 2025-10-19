//! TRANSPILER-DEFECT-004: Let-Else Transpilation Not Implemented
//!
//! **Problem**: Let-else statements fail with "not yet implemented" error
//! **Discovered**: 2025-10-19 (QUALITY-008 SATD remediation)
//! **Severity**: MEDIUM (5 TODO markers blocking language feature)
//!
//! **SATD Violations (5 TODOs)**:
//! - src/backend/transpiler/dispatcher.rs:390 (ExprKind::Let)
//! - src/backend/transpiler/dispatcher.rs:411 (ExprKind::LetPattern)
//! - src/backend/transpiler/statements.rs (1 instance)
//! - src/backend/transpiler/dispatcher_helpers/error_handling.rs (2 instances)
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR → MUTATION → PROPERTY)

use assert_cmd::Command;
use predicates::prelude::*;
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

// ==================== RED PHASE: Failing Tests ====================
// These tests MUST FAIL initially with "let-else transpilation not yet implemented"

/// Test 1: Basic let-else with Option::Some pattern
///
/// Ruchy code:
/// ```ruchy
/// let Some(x) = Some(42) else {
///     return 1
/// }
/// x
/// ```
///
/// Should transpile to Rust:
/// ```rust
/// let x = if let Some(x) = Some(42) { x } else {
///     return 1;
/// };
/// x
/// ```
#[test]
fn test_let_else_red_option_some_pattern() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() -> i64 {
    let Some(x) = Some(42) else {
        return 1
    }
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // GREEN: Now transpilation should SUCCEED
    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 2: Let-else with Option::None fallback
///
/// Ruchy code:
/// ```ruchy
/// let Some(value) = parse_int(input) else {
///     println("Invalid input")
///     return -1
/// }
/// value
/// ```
#[test]
fn test_let_else_red_option_none_fallback() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun parse_int(s: String) -> Option<i64> {
    Some(42)
}

fun main() -> i64 {
    let input = "test"
    let Some(value) = parse_int(input) else {
        println("Invalid input")
        return -1
    }
    value
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 3: Let-else with Result::Ok pattern
///
/// Ruchy code:
/// ```ruchy
/// let Ok(config) = read_config() else {
///     eprintln("Failed to read config")
///     exit(1)
/// }
/// config
/// ```
#[test]
fn test_let_else_red_result_ok_pattern() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun read_config() -> Result<String, String> {
    Ok("config data")
}

fun main() -> String {
    let Ok(config) = read_config() else {
        eprintln("Failed to read config")
        return "default"
    }
    config
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 4: Let-else with enum destructuring
///
/// Ruchy code:
/// ```ruchy
/// enum Response {
///     Success(i64),
///     Error(String)
/// }
///
/// let Success(code) = get_response() else {
///     return -1
/// }
/// code
/// ```
#[test]
fn test_let_else_red_enum_destructuring() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
enum Response {
    Success(i64),
    Error(String)
}

fun get_response() -> Response {
    Response::Success(200)
}

fun main() -> i64 {
    let Success(code) = get_response() else {
        return -1
    }
    code
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 5: Let-else with tuple destructuring
///
/// Ruchy code:
/// ```ruchy
/// let (x, y) = get_coordinates() else {
///     return (0, 0)
/// }
/// (x, y)
/// ```
#[test]
fn test_let_else_red_tuple_destructuring() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_coordinates() -> Option<(i64, i64)> {
    Some((10, 20))
}

fun main() -> (i64, i64) {
    let Some((x, y)) = get_coordinates() else {
        return (0, 0)
    }
    (x, y)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 6: Let-else with nested pattern matching
///
/// Ruchy code:
/// ```ruchy
/// let Some(Ok(value)) = nested_result() else {
///     return -1
/// }
/// value
/// ```
#[test]
fn test_let_else_red_nested_patterns() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun nested_result() -> Option<Result<i64, String>> {
    Some(Ok(42))
}

fun main() -> i64 {
    let Some(Ok(value)) = nested_result() else {
        return -1
    }
    value
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Test 7: Let-else with multiple statements in else block
///
/// Ruchy code:
/// ```ruchy
/// let Some(x) = get_value() else {
///     println("Error: value not found")
///     log("Failed to retrieve value")
///     return -1
/// }
/// x
/// ```
#[test]
fn test_let_else_red_multiple_else_statements() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun get_value() -> Option<i64> {
    Some(42)
}

fun log(msg: String) {
    println(msg)
}

fun main() -> i64 {
    let Some(x) = get_value() else {
        println("Error: value not found")
        log("Failed to retrieve value")
        return -1
    }
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

// ==================== BASELINE TESTS (Currently Pass) ====================
// These tests verify existing functionality that should continue working

/// Baseline: Regular let binding without else should work
#[test]
fn test_let_else_baseline_regular_let() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() -> i64 {
    let x = 42
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Regular let should work NOW
    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

/// Baseline: Match expression should work (alternative to let-else)
#[test]
fn test_let_else_baseline_match_alternative() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() -> i64 {
    let opt = Some(42)
    match opt {
        Some(x) => x,
        None => -1
    }
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Match expression should work NOW
    ruchy_cmd()
        .arg("transpile")
        .arg(&source)
        .assert()
        .success();
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test documenting RED phase state
#[test]
fn test_let_else_red_phase_summary() {
    println!("\n=== LET-ELSE TRANSPILATION RED PHASE ===");
    println!("\nSATD Violations: 5 TODOs");
    println!("Files: dispatcher.rs (2), statements.rs (1), error_handling.rs (2)");
    println!("\nRED Tests Created: 7 failing tests");
    println!("1. Option::Some pattern");
    println!("2. Option::None fallback");
    println!("3. Result::Ok pattern");
    println!("4. Enum destructuring");
    println!("5. Tuple destructuring");
    println!("6. Nested patterns");
    println!("7. Multiple else statements");
    println!("\nBaseline Tests: 2 passing tests");
    println!("- Regular let binding (works now)");
    println!("- Match expression alternative (works now)");
    println!("\nNext: GREEN phase - implement let-else transpilation");
    println!("Target: Complexity ≤10, TDG A-, remove all 5 TODOs\n");
}
