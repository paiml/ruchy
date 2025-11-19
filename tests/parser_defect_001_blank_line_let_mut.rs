#![allow(missing_docs)]
//! DEFECT-PARSER-001: Parse Error with Blank Line Before Function with `let mut`
//!
//! ROOT CAUSE: Parser fails to properly close function block scope when:
//! 1. First function ends with complex if/else chain
//! 2. Followed by blank line
//! 3. Next function starts with `let mut`
//!
//! Error: "Expected `RightBrace`, found Let"
//! Location: Parser block closing logic (TBD)
//!
//! RED Phase Tests - Following EXTREME TDD

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_parser_001_complex_function_blank_line_let_mut() {
    // DEFECT-PARSER-001: This should parse successfully
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun process(x: &str) -> &str {
    if x == "a" {
        if true {
            return "a1";
        } else {
            return "a2";
        }
    } else if x == "b" {
        return "b";
    } else {
        return "other";
    }
}

fun second() -> &str {
    let mut state = "pending";
    state
}

fun main() {
    let result = process("a");
    println(result)
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should parse successfully (RED phase - will fail)
    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_parser_001_minimal_reproduction() {
    // TRUE minimal: Requires 3+ nested else-if chains
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun first(state: &str, action: &str) -> &str {
    if state == "a" {
        if action == "x" {
            return "ax";
        } else if action == "y" {
            return "ay";
        } else {
            return state;
        }
    } else if state == "b" {
        if action == "x" {
            return "bx";
        } else if action == "y" {
            return "by";
        } else {
            return state;
        }
    } else {
        return "error";
    }
}

fun second() -> &str {
    let mut x = "test";
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_parser_001_works_without_blank_line() {
    // Control test: Same code WITHOUT blank line should work
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun first() -> &str {
    if true {
        return "a";
    } else {
        return "b";
    }
}
fun second() -> &str {
    let mut x = "test";
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_parser_001_works_without_mut() {
    // Control test: Blank line + let (without mut) should work
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun first() -> &str {
    if true {
        return "a";
    } else {
        return "b";
    }
}

fun second() -> &str {
    let x = "test";
    x
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}
