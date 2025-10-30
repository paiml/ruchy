#![allow(missing_docs)]
//! Integration tests for match expression with enum variants
//!
//! Tests pattern matching on custom enum types:
//! - Unit variant patterns (`Status::Success`)
//! - Tuple variant patterns with destructuring (`Response::Error(msg)`)
//! - Match exhaustiveness
//! - Guards with enum patterns

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_match_unit_variant() {
    let code = r#"
enum Status {
    Success,
    Failed
}

let s = Status::Success
let result = match s {
    Status::Success => "good",
    Status::Failed => "bad"
}
println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("good"));
}

#[test]
fn test_match_tuple_variant_destructure() {
    let code = r#"
enum Response {
    Ok,
    Error(String)
}

let r = Response::Error("failed")
let result = match r {
    Response::Ok => "success",
    Response::Error(msg) => msg
}
println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("failed"));
}

#[test]
fn test_match_enum_with_guard() {
    let code = r#"
enum Message {
    Quit,
    Move(String)
}

let m = Message::Move("up")
let result = match m {
    Message::Quit => "quit",
    Message::Move(dir) if dir == "up" => "moving up",
    Message::Move(_) => "moving other"
}
println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("moving up"));
}

#[test]
fn test_match_multiple_variants() {
    let code = r"
enum Action {
    Start,
    Stop,
    Pause
}

let a1 = Action::Start
let a2 = Action::Pause

let r1 = match a1 {
    Action::Start => 1,
    Action::Stop => 2,
    Action::Pause => 3
}

let r2 = match a2 {
    Action::Start => 1,
    Action::Stop => 2,
    Action::Pause => 3
}

println(r1)
println(r2)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1\n3"));
}

#[test]
fn test_match_keyword_variants() {
    let code = r#"
enum Outcome {
    Ok,
    Err
}

let o = Outcome::Ok
let result = match o {
    Outcome::Ok => "ok",
    Outcome::Err => "error"
}
println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}
