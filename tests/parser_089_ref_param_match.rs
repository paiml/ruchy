#![allow(missing_docs)]
//! PARSER-089: Fix parser handling of reference parameters with match expressions (GitHub Issue #73)
//!
//! ROOT CAUSE: When parsing a function with reference-type parameters (`&str`),
//! the parser incorrectly remains in "parameter parsing mode" after finishing the
//! parameter list. When it encounters a match expression's closing brace `}`, it
//! misinterprets it as an invalid parameter.
//!
//! FIX: Properly reset parser state after parsing reference-type parameters.
//!
//! DISCOVERED VIA: Binary search/bisection testing
//! - ✅ Works: Function WITHOUT parameter, WITH match + method chain
//! - ❌ Fails: Function WITH parameter (`&str`), WITH match + method chain
//! - ✅ Works: Function WITH parameter, WITHOUT match
//! - ROOT: Reference parameter parsing doesn't reset state before match expression

use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn write_and_check(code: &str) {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("check")
        .arg(temp_file.path())
        .assert()
        .success();
}

/// Test minimal reproduction case: reference parameter + match expression
#[test]
fn test_parser_089_ref_param_with_match() {
    let code = r#"
use std::process::Command;

pub fun check_command(command: &str) -> bool {
    let result = Command::new("which").arg(command).output();
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
"#;
    write_and_check(code);
}

/// Test that reference parameters work WITHOUT match (baseline)
#[test]
fn test_parser_089_ref_param_no_match_works() {
    let code = r#"
use std::process::Command;

pub fun test(command: &str) -> bool {
    let result = Command::new("which").arg(command).output();
    true
}
"#;
    write_and_check(code);
}

/// Test that match works WITHOUT reference parameters (baseline)
#[test]
fn test_parser_089_match_no_ref_param_works() {
    let code = r#"
use std::process::Command;

pub fun test() -> bool {
    let result = Command::new("which").arg("test").output();
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
"#;
    write_and_check(code);
}

/// Test multiple reference parameters with match
#[test]
fn test_parser_089_multiple_ref_params_with_match() {
    let code = r#"
pub fun check_two(first: &str, second: &str) -> bool {
    let result = match first {
        "ok" => true,
        _ => false,
    };
    match second {
        "valid" => result,
        _ => false,
    }
}
"#;
    write_and_check(code);
}

/// Test nested match expressions with reference parameter
#[test]
fn test_parser_089_nested_match_with_ref_param() {
    let code = r#"
pub fun nested_check(input: &str) -> i32 {
    match input {
        "a" => match "x" {
            "x" => 1,
            _ => 2,
        },
        _ => 3,
    }
}
"#;
    write_and_check(code);
}

/// Test original Issue #73 reproduction case (32 lines)
#[test]
fn test_parser_089_issue_73_full_reproduction() {
    let code = r#"
use std::process::Command;

pub fun check_command(command: &str) -> bool {
    let result = Command::new("which")
        .arg(command)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fun validate_dependencies(deps: Vec<String>) -> bool {
    let mut all_present = true;

    for dep in deps {
        let exists = check_command(&dep);
        if !exists {
            println!("Missing dependency: {}", dep);
            all_present = false;
        } else {
            println!("Found dependency: {}", dep);
        }
    }

    all_present
}
"#;
    write_and_check(code);
}

/// Test mutable reference parameter with match
#[test]
fn test_parser_089_mut_ref_param_with_match() {
    let code = r"
pub fun modify_and_check(value: &mut i32) -> bool {
    *value += 1;
    match *value {
        1 => true,
        _ => false,
    }
}
";
    write_and_check(code);
}

/// Test mixed value and reference parameters with match
#[test]
fn test_parser_089_mixed_params_with_match() {
    let code = r#"
pub fun mixed(owned: String, borrowed: &str, number: i32) -> bool {
    match borrowed {
        "test" => number > 0,
        _ => false,
    }
}
"#;
    write_and_check(code);
}
