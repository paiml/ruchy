//! Integration tests for enum runtime support
//!
//! Tests enum variant construction and execution:
//! - Unit variants (Status::Success)
//! - Tuple variants (Response::Error("msg"))
//! - Keyword variants (Maybe::Some, Maybe::None)

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_enum_unit_variants() {
    let code = r#"
enum Status {
    Success,
    Pending,
    Failed
}

let s1 = Status::Success
let s2 = Status::Pending
let s3 = Status::Failed
println("All unit variants work")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("All unit variants work"));
}

#[test]
fn test_enum_tuple_variants() {
    let code = r#"
enum Response {
    Ok,
    Error(String)
}

let ok = Response::Ok
let err = Response::Error("Something went wrong")
println("Tuple variants work")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Tuple variants work"));
}

#[test]
fn test_enum_keyword_variants() {
    let code = r#"
enum Maybe {
    Some,
    None
}

let some = Maybe::Some
let none = Maybe::None
println("Keyword variants work")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Keyword variants work"));
}

#[test]
fn test_enum_multiple_tuple_variants() {
    let code = r#"
enum Message {
    Quit,
    Move(String),
    Write(String)
}

let m1 = Message::Quit
let m2 = Message::Move("up")
let m3 = Message::Write("hello")
println("Multiple tuple variants work")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Multiple tuple variants work"));
}

#[test]
fn test_enum_ok_err_variants() {
    let code = r#"
enum Outcome {
    Ok,
    Err
}

let success = Outcome::Ok
let failure = Outcome::Err
println("Ok and Err keyword variants work")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Ok and Err keyword variants work"));
}
