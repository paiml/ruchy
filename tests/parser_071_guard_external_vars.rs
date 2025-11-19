#![allow(missing_docs)]
// PARSER-071: Guard clauses with external variable references (GitHub Issue #56)
//
// Root Cause: When parsing guard expressions like `n < limit`, if an external variable
// like `limit` is followed by `=>`, the expression parser treats `variable =>` as a
// lambda expression and consumes the `=>` token, leaving no `=>` for the match arm.
//
// Example: `t if t < limit =>` - parser sees `limit =>` and thinks it's a lambda.
//
// Solution: Use a specialized guard expression parser that stops at `=>` and `->`
// tokens instead of treating them as lambda syntax.

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_parser_071_guard_with_external_variable() {
    // RED: This test currently FAILS with "Expected '=>' or '->' in match arm"
    let code = r#"
let limit = 10
let result = match 5 {
  n if n < limit => "less",
  _ => "greater or equal"
}
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("less"));
}

#[test]
fn test_parser_071_guard_with_compound_expression() {
    // RED: This test currently FAILS - compound guard with external variable
    let code = r#"
let temp = 85
let is_summer = true
let comfort = match temp {
  t if t < 90 && is_summer => "warm summer day",
  t if t < 32 => "freezing",
  _ => "very hot"
}
println("{}", comfort)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("warm summer day"));
}

#[test]
fn test_parser_071_guard_with_multiple_external_vars() {
    // RED: Multiple external variables in guard expression
    let code = r#"
let min = 0
let max = 100
let result = match 50 {
  n if n > min && n < max => "in range",
  _ => "out of range"
}
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("in range"));
}

#[test]
fn test_parser_071_guard_external_var_only() {
    // RED: Guard with only external variable reference (no bound variable used)
    let code = r#"
let flag = true
let result = match 42 {
  _ if flag => "flag is true",
  _ => "flag is false"
}
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("flag is true"));
}

#[test]
fn test_parser_071_guard_with_function_call() {
    // External function call in guard expression
    let code = r#"
fun is_even(n) {
  n % 2 == 0
}
let result = match 4 {
  n if is_even(n) => "even",
  _ => "odd"
}
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("even"));
}

#[test]
fn test_parser_071_guard_simple_still_works() {
    // This test should PASS (no regression) - simple guard with only literals
    let code = r#"
let result = match 5 {
  n if n > 0 => "positive",
  _ => "non-positive"
}
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("positive"));
}

#[test]
fn test_parser_071_guard_transpile_mode() {
    // Verify transpilation works correctly with external variables in guards
    let code = r#"
let limit = 10
let result = match 5 {
  n if n < limit => "less",
  _ => "greater"
}
result
"#;

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_071_transpile.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("match"))
        .stdout(predicate::str::contains("if"));
}

#[test]
fn test_parser_071_guard_check_mode() {
    // Verify syntax checking works
    let code = r#"
let limit = 10
let result = match 5 {
  n if n < limit => "less",
  _ => "greater"
}
"#;

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_071_check.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd().arg("check").arg(temp_file).assert().success();
}
