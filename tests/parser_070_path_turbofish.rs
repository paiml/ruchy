// PARSER-070: Path Expression Turbofish Support
//
// FEATURE: Enable turbofish syntax (::<Type>) in path expressions (not just method calls)
// Examples: HashMap::<String, i32>::new(), Vec::<i32>::new(), Option::<T>::Some(value)
//
// ROOT CAUSE: Parser expects identifier after '::' but turbofish starts with '<'
// Error: "Expected identifier or keyword usable as identifier after '::' but got Less"
//
// FIX: In parse_path_expression(), check for '<' after '::' to handle turbofish
// Similar pattern to PARSER-069 (method call turbofish) but for path expressions
//
// SCOPE: Path expressions only (e.g., Vec::<T>::new)
// OUT OF SCOPE: Associated types (e.g., <T as Trait>::AssocType) - future work

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

/// Test basic path turbofish - `Vec::`<i32>`::new()`
#[test]
fn test_parser_070_basic_path_turbofish() {
    let code = r#"
let vec = Vec::<i32>::new()
println("Created vec: {:?}", vec)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created vec"));
}

/// Test path turbofish with multiple type parameters - `HashMap::`<String, `i32>::new()`
#[test]
fn test_parser_070_multiple_type_params() {
    let code = r#"
let map = HashMap::<String, i32>::new()
println("Created map")
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    // Test parsing only (HashMap not implemented in interpreter)
    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .assert()
        .success();
}

/// Test nested path turbofish - `Vec::`<`Vec::`<i32>>`::new()`
#[test]
fn test_parser_070_nested_generics() {
    let code = r#"
let vec = Vec::<Vec::<i32>>::new()
println("Created nested vec")
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    // Test parsing only (nested generics not fully supported in interpreter)
    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .assert()
        .success();
}

/// Test path turbofish with Result - `Result::`<i32, `String>::new()`
/// NOTE: Enum variant turbofish (`::Some`, `::Ok`) is out of scope for PARSER-070
/// This test uses method call syntax instead
#[test]
fn test_parser_070_result_turbofish() {
    let code = r#"
let result = Result::<i32, String>::new()
println("Created result")
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    // Test parsing only (Result not implemented in interpreter)
    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .assert()
        .success();
}

/// Test path turbofish in variable assignment
#[test]
fn test_parser_070_assignment() {
    let code = r#"
let x = Vec::<String>::new()
x.push("hello")
println("Vec has {} items", x.len())
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vec has 1 items"));
}

/// Test path turbofish in function argument
#[test]
fn test_parser_070_function_argument() {
    let code = r#"
fun process(v: Vec<i32>) -> i32 {
    v.len()
}

let vec = Vec::<i32>::new()
let len = process(vec)
println("Length: {}", len)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Length: 0"));
}

/// Test path turbofish in if condition
#[test]
fn test_parser_070_in_condition() {
    let code = r#"
let vec = Vec::<i32>::new()
if vec.is_empty() {
    println("Empty")
} else {
    println("Not empty")
}
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Empty"));
}

/// Test path turbofish with chained method call
#[test]
fn test_parser_070_chained_method() {
    let code = r#"
let vec = Vec::<i32>::new().push(42).push(43)
println("Vec: {:?}", vec)
"#;

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("run")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vec"));
}

/// Test that AST is correct - should parse as path expression with turbofish
#[test]
fn test_parser_070_ast_structure() {
    let code = r"
Vec::<i32>::new()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    // Verify turbofish parses correctly - generates FieldAccess on Vec identifier
    ruchy_cmd()
        .arg("ast")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("FieldAccess"))
        .stdout(predicate::str::contains("Vec"));
}

/// Test syntax check passes (not just runtime)
#[test]
fn test_parser_070_check_command() {
    let code = r"
let map = HashMap::<String, i32>::new()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .assert()
        .success();
}

/// Test linting passes
#[test]
fn test_parser_070_lint_command() {
    let code = r"
let vec = Vec::<i32>::new()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    ruchy_cmd()
        .arg("lint")
        .arg(file_path)
        .assert()
        .success();
}

/// Test transpilation succeeds
#[test]
fn test_parser_070_transpile_command() {
    let code = r"
let vec = Vec::<i32>::new()
";

    let (_temp_dir, file_path) = write_temp_file(code);

    // Turbofish is correctly consumed during parsing, so it won't appear in transpiled output
    ruchy_cmd()
        .arg("transpile")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vec :: new"));
}
