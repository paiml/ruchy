// DEFECT-PARSER-012: F-string empty placeholders
// Tests for f"..." with {} positional arguments (Python-style)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn test_code(code: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::thread;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let thread_id = thread::current().id();
    let temp_file = PathBuf::from(format!("/tmp/test_fstring_defect_{}_{:?}.ruchy", timestamp, thread_id));
    fs::write(&temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    let _ = fs::remove_file(&temp_file); // Cleanup
}

#[test]
fn test_fstring_single_empty_placeholder() {
    test_code(r#"
fn test() {
    println(f"test {}", 42)
}
"#);
}

#[test]
fn test_fstring_multiple_empty_placeholders() {
    test_code(r#"
fn test() {
    println(f"Point at ({}, {})", x, y)
}
"#);
}

#[test]
fn test_fstring_in_impl_block() {
    test_code(r#"
impl Draw for Point {
    fn draw(&self) {
        println(f"Drawing point at ({}, {})", self.x, self.y)
    }
}
"#);
}

#[test]
fn test_fstring_with_expression_still_works() {
    test_code(r#"
fn test() {
    println(f"test {self.x}", self.x)
}
"#);
}

#[test]
fn test_fstring_mixed_placeholders() {
    test_code(r#"
fn test() {
    println(f"Value: {}, Name: {name}", 42, name)
}
"#);
}

#[test]
fn test_fstring_in_nested_blocks() {
    test_code(r#"
fn outer() {
    if true {
        for i in list {
            println(f"Item {}: {}", i, value)
        }
    }
}
"#);
}

#[test]
fn test_fstring_in_lambda() {
    test_code(r#"
let logger = |msg| {
    println(f"Log: {}", msg)
}
"#);
}

#[test]
fn test_fstring_only_empty_placeholders() {
    test_code(r#"
fn test() {
    println(f"{} {} {}", a, b, c)
}
"#);
}

#[test]
fn test_fstring_empty_placeholder_at_start() {
    test_code(r#"
fn test() {
    println(f"{} is the value", x)
}
"#);
}

#[test]
fn test_fstring_empty_placeholder_at_end() {
    test_code(r#"
fn test() {
    println(f"The value is {}", x)
}
"#);
}

#[test]
fn test_fstring_without_placeholders_still_works() {
    test_code(r#"
fn test() {
    println(f"Hello, World!")
}
"#);
}
