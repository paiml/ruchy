#![allow(missing_docs)]
//! Test inline comments in enum variant definitions
//!
//! Bug: Parser fails with "Expected variant name in enum" when comments
//! follow variant definitions like: Pos(i32, i32) // comment

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_inline_comment_after_tuple_variant() {
    let code = r#"
enum Position {
    Pos(i32, i32, i32)  // line, column, offset
}

let p = Position::Pos(1, 2, 3)
println("success")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
}

#[test]
fn test_inline_comment_after_unit_variant() {
    let code = r#"
enum Status {
    Success,  // all good
    Failed    // something wrong
}

let s = Status::Success
println("ok")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn test_multiple_variants_with_comments() {
    let code = r#"
enum Token {
    Char(String, i32),  // character with position
    EOF,                // end of file
    Number(i32)         // numeric literal
}

let t = Token::Number(42)
println("done")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}
