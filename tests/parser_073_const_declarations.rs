// PARSER-073: Const variable declarations (GitHub Issue #57, Part 2/3)
//
// Root Cause: Parser only accepts `const fun` or `const fn`, not `const VARIABLE = value`
// Current behavior: `const PI = 3.14159` fails with "Expected 'fun' or 'fn' after 'const'"
// Expected: Const variables should parse and prevent reassignment attempts
//
// Solution: Extend parse_const_token() to handle variable declarations after const keyword

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_parser_073_const_basic_declaration() {
    // RED: This test currently FAILS - const variables not supported
    let code = r#"
const PI = 3.14159
println("{}", PI)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14159"));
}

#[test]
fn test_parser_073_const_integer() {
    // Const with integer value
    let code = r#"
const MAX_SIZE = 100
println("{}", MAX_SIZE)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_parser_073_const_string() {
    // Const with string value
    let code = r#"
const GREETING = "Hello, World!"
println("{}", GREETING)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn test_parser_073_const_expression() {
    // Const with expression value
    let code = r#"
const DOUBLE_PI = 3.14159 * 2
println("{}", DOUBLE_PI)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6.28318"));
}

#[test]
fn test_parser_073_const_multiple_declarations() {
    // Multiple const declarations
    let code = r#"
const PI = 3.14159
const E = 2.71828
println("{}", PI + E)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("5.85987"));
}

#[test]
fn test_parser_073_const_in_function() {
    // Const used within function
    let code = r#"
const MULTIPLIER = 10
fun multiply(x) {
  x * MULTIPLIER
}
println("{}", multiply(5))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("50"));
}

#[test]
fn test_parser_073_const_vs_let() {
    // Both const and let should work
    let code = r#"
const PI = 3.14159
let radius = 5
println("{}", PI * radius * radius)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("78.53975"));
}

#[test]
fn test_parser_073_const_transpile_mode() {
    // Verify transpilation works with const
    let code = r#"
const PI = 3.14159
println("{}", PI)
"#;

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_073_transpile.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("const"))
        .stdout(predicate::str::contains("PI"));
}

#[test]
fn test_parser_073_const_check_mode() {
    // Verify syntax checking works with const
    let code = r"
const MAX_VALUE = 1000
";

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_073_check.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();
}

#[test]
fn test_parser_073_const_function_still_works() {
    // Regression test: const functions should still work
    let code = r#"
const fun get_pi() {
  3.14159
}
println("{}", get_pi())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14159"));
}
