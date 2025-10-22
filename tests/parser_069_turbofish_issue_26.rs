// PARSER-069: Fix turbofish syntax parsing in method calls
// GitHub Issue: https://github.com/paiml/ruchy/issues/26
//
// BUG: Turbofish syntax (::<Type>) fails to parse in method calls
// Example: "42".parse::<i32>() causes "Expected identifier...after '::'...got Less"
//
// ROOT CAUSE: parse_method_or_field_access() checks for '(' immediately after method name
// With turbofish, next token is '::' not '(', so parser treats it as field access
//
// FIX: Check for '::' first, parse turbofish if present, then check for '('
// Modified: parse_method_or_field_access() in src/frontend/parser/functions.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp file with code and return path
fn write_temp_file(code: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).expect("Failed to write temp file");
    (temp_dir, file_path)
}

/// Test basic turbofish method call (original bug report - top level)
#[test]
fn test_parser_069_turbofish_basic_method_call() {
    let code = r#"
let x = "42".parse::<i32>()
println("Result: {}", x)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 42"));
}

/// Test turbofish in lambda without block (original bug report)
#[test]
fn test_parser_069_turbofish_lambda_no_block() {
    let code = r#"
let parser = || "42".parse::<i32>()
let result = parser()
println("Result: {}", result)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 42"));
}

/// Test turbofish in lambda with block (main bug from issue)
#[test]
fn test_parser_069_turbofish_lambda_with_block() {
    let code = r#"
let parser = || {
    "42".parse::<i32>()
}
let result = parser()
println("Result: {}", result)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 42"));
}

/// Test turbofish in higher-order function (exact original bug report)
#[test]
fn test_parser_069_turbofish_higher_order_function() {
    let code = r#"
fun test(name, f) {
    println(name)
    f()
}

test("demo", || {
    "42".parse::<i32>()
    true
})
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("demo"));
}

/// Test turbofish with multiple type parameters
/// NOTE: Requires PARSER-070 (path expression turbofish) + HashMap type implementation
#[test]
#[ignore = "PARSER-070: Path expression turbofish not yet implemented (HashMap::<T>::new)"]
fn test_parser_069_turbofish_multiple_type_params() {
    let code = r#"
let map = HashMap::<String, i32>::new()
println("Created map")
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created map"));
}

/// Test nested turbofish generics
/// NOTE: Requires PARSER-070 (path expression turbofish) + Vec type implementation
#[test]
#[ignore = "PARSER-070: Path expression turbofish not yet implemented (Vec::<T>::new)"]
fn test_parser_069_turbofish_nested_generics() {
    let code = r#"
let vec = Vec::<Vec::<i32>>::new()
println("Created nested vec")
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created nested vec"));
}

/// Test turbofish in method chain
#[test]
fn test_parser_069_turbofish_method_chain() {
    let code = r#"
let result = "42"
    .parse::<i32>()
    .to_string()
println("Result: {}", result)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 42"));
}

/// Test turbofish with function return type
#[test]
fn test_parser_069_turbofish_function_return() {
    let code = r#"
fun parse_number(s) -> i32 {
    s.parse::<i32>()
}

let result = parse_number("123")
println("Result: {}", result)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 123"));
}

/// Test turbofish in if condition
#[test]
fn test_parser_069_turbofish_in_condition() {
    let code = r#"
let s = "42"
if s.parse::<i32>() == 42 {
    println("Correct")
} else {
    println("Wrong")
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Correct"));
}

/// Test that AST is correct - should be MethodCall with turbofish
#[test]
fn test_parser_069_ast_structure() {
    let code = r#"
"42".parse::<i32>()
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("ast")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("MethodCall"))
        .stdout(predicate::str::contains("parse"))
        .stdout(predicate::str::contains("FieldAccess").not()); // Should NOT be field access
}
