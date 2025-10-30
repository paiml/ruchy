//! Integration tests for nested enum pattern matching
//!
//! Tests matching enum variants that contain other enum variants:
//! - `Token::Char(ch`, `Position::Pos(line`, col, offset))
//! - `Result::Ok(Option::Some(value))`
//! - `Message::Data(Response::Error(msg))`

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_nested_enum_simple() {
    let code = r#"
enum Position {
    Pos(i32, i32)
}

enum Token {
    Char(String, Position)
}

let pos = Position::Pos(1, 10)
let token = Token::Char("a", pos)

let result = match token {
    Token::Char(ch, Position::Pos(line, col)) => {
        println(ch + " at line " + line.to_string())
        "success"
    }
}

println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("a at line 1"))
        .stdout(predicate::str::contains("success"));
}

#[test]
fn test_nested_enum_triple() {
    let code = r#"
enum Position {
    Pos(i32, i32, i32)
}

enum Token {
    Char(String, Position),
    EOF
}

let pos = Position::Pos(5, 20, 99)
let token = Token::Char("x", pos)

let result = match token {
    Token::Char(ch, Position::Pos(line, col, offset)) => {
        println("Char: " + ch)
        println("Line: " + line.to_string())
        println("Col: " + col.to_string())
        println("Offset: " + offset.to_string())
        "found"
    },
    Token::EOF => "eof"
}

println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Char: x"))
        .stdout(predicate::str::contains("Line: 5"))
        .stdout(predicate::str::contains("Col: 20"))
        .stdout(predicate::str::contains("Offset: 99"))
        .stdout(predicate::str::contains("found"));
}

#[test]
fn test_nested_enum_maybe_value() {
    let code = r#"
enum Outcome {
    Good(Maybe),
    Bad
}

enum Maybe {
    Value(String),
    Empty
}

let m = Maybe::Value("data")
let o = Outcome::Good(m)

let result = match o {
    Outcome::Good(Maybe::Value(val)) => val,
    Outcome::Good(Maybe::Empty) => "empty",
    Outcome::Bad => "bad"
}

println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("data"));
}

#[test]
fn test_nested_enum_multiple_levels() {
    let code = r"
enum Inner {
    Val(i32)
}

enum Middle {
    Data(Inner)
}

enum Outer {
    Wrap(Middle)
}

let inner = Inner::Val(42)
let middle = Middle::Data(inner)
let outer = Outer::Wrap(middle)

let result = match outer {
    Outer::Wrap(Middle::Data(Inner::Val(n))) => n.to_string()
}

println(result)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_nested_enum_wildcard_inner() {
    let code = r#"
enum Position {
    Pos(i32, i32)
}

enum Token {
    Char(String, Position),
    EOF
}

let pos = Position::Pos(3, 15)
let token = Token::Char("b", pos)

let result = match token {
    Token::Char(ch, Position::Pos(_, col)) => {
        println("Char: " + ch + " at column " + col.to_string())
        "matched"
    },
    Token::EOF => "eof"
}

println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Char: b at column 15"))
        .stdout(predicate::str::contains("matched"));
}
