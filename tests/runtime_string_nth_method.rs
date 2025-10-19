//! BOOTSTRAP-002 Character Stream Processing - Runtime Test
//! Tests that String.chars().nth() method works in runtime evaluation
//!
//! Bug Report: String.chars().nth() method not implemented
//! Version: Ruchy 3.93.0
//! Blocker: BOOTSTRAP-002 Character Stream Processing

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_string_chars_nth_basic() {
    let code = r#"
let input = "hello"
let chars = input.chars()
let c = chars.nth(0)
match c {
    Some(ch) => println(ch.to_string()),
    None => println("No char")
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("h"));
}

#[test]
fn test_string_chars_nth_middle() {
    let code = r#"
let input = "hello"
let c = input.chars().nth(2)
match c {
    Some(ch) => println(ch.to_string()),
    None => println("No char")
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("l"));
}

#[test]
fn test_string_chars_nth_out_of_bounds() {
    let code = r#"
let input = "hi"
let c = input.chars().nth(10)
match c {
    Some(ch) => println("Found: " + ch.to_string()),
    None => println("None")
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("None"));
}

#[test]
fn test_string_chars_nth_bootstrap_002_scenario() {
    // BOOTSTRAP-002: Character stream processing for lexer
    // Token struct: Token::Char(ch, Position::Pos(line, col, offset))
    let code = r#"
let input = "x"
let c = input.chars().nth(0)
match c {
    Some(ch) => {
        println("Char: " + ch.to_string())
        ch.to_string()
    },
    None => "EOF"
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Char: x"));
}
