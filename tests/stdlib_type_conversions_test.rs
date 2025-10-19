// STDLIB-001: Type Conversion Functions
// Test both interpreter (REPL) and transpiler modes

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_stdlib001_str_from_int() {
    let code = r#"
let x = str(42)
assert_eq(x, "42")
println(x)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_stdlib001_str_from_float() {
    let code = r#"
let x = str(3.14)
assert_eq(x, "3.14")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_str_from_bool() {
    let code = r#"
assert_eq(str(true), "true")
assert_eq(str(false), "false")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_int_from_string() {
    let code = r#"
let x = int("42")
assert_eq(x, 42)
println(x)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_stdlib001_int_from_float() {
    let code = r#"
let x = int(3.14)
assert_eq(x, 3)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_int_from_bool() {
    let code = r#"
assert_eq(int(true), 1)
assert_eq(int(false), 0)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_float_from_string() {
    let code = r#"
let x = float("3.14")
assert_eq(x, 3.14)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_float_from_int() {
    let code = r#"
let x = float(42)
assert_eq(x, 42.0)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_bool_from_int() {
    let code = r#"
assert_eq(bool(1), true)
assert_eq(bool(0), false)
assert_eq(bool(42), true)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_bool_from_string() {
    let code = r#"
assert_eq(bool(""), false)
assert_eq(bool("hello"), true)
assert_eq(bool("false"), true)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

// Test transpiler mode (compile to binary)
#[test]
fn test_stdlib001_transpiler_str() {
    let code = r#"
fn main() {
    let x = str(42)
    assert_eq(x, "42")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_transpiler_int() {
    let code = r#"
fn main() {
    let x = int("42")
    assert_eq(x, 42)
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_transpiler_float() {
    let code = r#"
fn main() {
    let x = float("3.14")
    assert_eq(x, 3.14)
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_transpiler_bool() {
    let code = r#"
fn main() {
    assert_eq(bool(1), true)
    assert_eq(bool(0), false)
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success();
}

// Edge cases
#[test]
fn test_stdlib001_str_empty_string() {
    let code = r#"
let x = str("")
assert_eq(x, "")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_int_negative() {
    let code = r#"
let x = int("-42")
assert_eq(x, -42)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib001_float_negative() {
    let code = r#"
let x = float("-3.14")
assert_eq(x, -3.14)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}
