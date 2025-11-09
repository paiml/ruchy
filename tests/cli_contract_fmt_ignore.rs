#![allow(missing_docs)]
//! Tests for formatter ignore directives
//!
//! Sprint 3 Phase 2: Ignore Directives
//! Ticket: [FMT-PERFECT-022]
//!
//! Tests verify:
//! - // ruchy-fmt-ignore preserves exact formatting for next expression
//! - // ruchy-fmt-ignore-next alias works identically
//! - Ignore directives work with various expression types
//! - Multiple ignore directives in same file
//! - Formatted code interspersed with ignored code

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_preserves_single_line() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignore directive and intentionally bad formatting
    fs::write(&test_file, r"// ruchy-fmt-ignore
let x    =    1  +  2

let y = 3 + 4").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // First line should preserve bad formatting (ignored)
    assert!(formatted.contains("let x    =    1  +  2"), "Ignored line should preserve exact formatting");

    // Second line should be formatted normally
    assert!(formatted.contains("let y = 3 + 4"), "Non-ignored line should be formatted");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_next_alias() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignore-next directive
    fs::write(&test_file, r"// ruchy-fmt-ignore-next
let x    =    1  +  2").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Should preserve bad formatting
    assert!(formatted.contains("let x    =    1  +  2"), "ruchy-fmt-ignore-next should work like ruchy-fmt-ignore");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_multiple_expressions() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with multiple ignore directives
    fs::write(&test_file, r"// ruchy-fmt-ignore
let a    =    1

let b = 2

// ruchy-fmt-ignore
let c    =    3

let d = 4").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Ignored lines preserve formatting
    assert!(formatted.contains("let a    =    1"), "First ignored line should preserve formatting");
    assert!(formatted.contains("let c    =    3"), "Second ignored line should preserve formatting");

    // Non-ignored lines are formatted
    assert!(formatted.contains("let b = 2"), "Non-ignored line should be formatted");
    assert!(formatted.contains("let d = 4"), "Non-ignored line should be formatted");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_with_complex_expression() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignored function
    fs::write(&test_file, r"// ruchy-fmt-ignore
fn foo ( x,y ) { x+y }

fn bar(a, b) { a + b }").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Ignored function preserves bad formatting
    assert!(formatted.contains("fn foo ( x,y ) { x+y }"), "Ignored function should preserve exact formatting");

    // Non-ignored function is formatted
    assert!(formatted.contains("fn bar(a, b)"), "Non-ignored function should be formatted");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_with_check_mode() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignore directive
    fs::write(&test_file, r"// ruchy-fmt-ignore
let x    =    1  +  2").expect("Failed to write test file");

    // First format it
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Then check it - should pass because ignored line is preserved
    ruchy_cmd()
        .arg("fmt")
        .arg("--check")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("is properly formatted"));
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_preserves_comments_and_whitespace() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignored code containing comments and extra whitespace
    fs::write(&test_file, r"// ruchy-fmt-ignore
let x  =  1  +  2   // trailing comment
    // inner comment
    +   3").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Should preserve all whitespace and comments exactly
    assert!(formatted.contains("let x  =  1  +  2   // trailing comment"), "Should preserve exact formatting including comments");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_does_not_affect_other_files() {
    let temp_dir = setup_test_dir();
    let ignored_file = temp_dir.path().join("ignored.ruchy");
    let normal_file = temp_dir.path().join("normal.ruchy");

    // Create file with ignore directive
    fs::write(&ignored_file, r"// ruchy-fmt-ignore
let x    =    1  +  2").expect("Failed to write ignored file");

    // Create normal file with bad formatting
    fs::write(&normal_file, "let y    =    3  +  4").expect("Failed to write normal file");

    // Format both files
    ruchy_cmd()
        .arg("fmt")
        .arg(&ignored_file)
        .assert()
        .success();

    ruchy_cmd()
        .arg("fmt")
        .arg(&normal_file)
        .assert()
        .success();

    // Read formatted files
    let ignored_content = fs::read_to_string(&ignored_file).expect("Failed to read ignored file");
    let normal_content = fs::read_to_string(&normal_file).expect("Failed to read normal file");

    // Ignored file preserves formatting
    assert!(ignored_content.contains("let x    =    1  +  2"), "Ignored file should preserve formatting");

    // Normal file is formatted
    assert!(normal_content.contains("let y = 3 + 4"), "Normal file should be formatted");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_with_nested_expressions() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignored block containing nested expressions
    fs::write(&test_file, r"// ruchy-fmt-ignore
{
    let a=1
    let b=2
    a+b
}").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Entire block should be preserved
    assert!(formatted.contains("let a=1"), "Nested expressions in ignored block should preserve formatting");
    assert!(formatted.contains("let b=2"), "Nested expressions in ignored block should preserve formatting");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_case_sensitivity() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with uppercase ignore directive (should NOT work)
    fs::write(&test_file, r"// RUCHY-FMT-IGNORE
let x    =    1  +  2").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Should be formatted (uppercase directive not recognized)
    assert!(formatted.contains("let x = 1 + 2"), "Uppercase directive should not be recognized");
}

#[ignore = "RED phase TDD - formatter ignore directives not implemented yet. Sprint FORMATTER-004"]
#[test]
fn test_fmt_ignore_with_extra_whitespace() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("test.ruchy");

    // Write file with ignore directive with extra whitespace
    fs::write(&test_file, r"//   ruchy-fmt-ignore
let x    =    1  +  2").expect("Failed to write test file");

    // Format file
    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    // Read formatted file
    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");

    // Should preserve formatting (whitespace trimmed in directive check)
    assert!(formatted.contains("let x    =    1  +  2"), "Directive with whitespace should be recognized");
}
