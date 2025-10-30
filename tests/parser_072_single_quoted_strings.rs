// PARSER-072: Single-quoted strings (GitHub Issue #57, Part 1/3)
//
// Root Cause: Lexer only recognizes single quotes for character literals ('c'), not strings.
// Current behavior: `'hello'` fails with "Syntax error: Expected Colon, found Identifier"
// Expected: Both quote styles should work equivalently like `assert_eq("hello", 'hello')`
//
// Solution: Add single-quoted string regex pattern to lexer BEFORE char literal pattern

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_parser_072_single_quoted_string_basic() {
    // RED: This test currently FAILS - single quotes not supported for strings
    let code = r#"
let msg = 'hello world'
println("{}", msg)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_parser_072_single_vs_double_quotes_equivalent() {
    // Both quote styles should produce identical results
    let code = r#"
let double = "hello"
let single = 'hello'
println("{}", double == single)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parser_072_single_quoted_with_escapes() {
    // Single-quoted strings should support escape sequences
    let code = r#"
let escaped = 'hello\nworld'
println("{}", escaped)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello\nworld"));
}

#[test]
fn test_parser_072_single_quoted_empty_string() {
    // Empty single-quoted string
    let code = r#"
let empty = ''
println("{}", empty)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_parser_072_single_quoted_with_embedded_double_quotes() {
    // Single quotes should allow embedded double quotes without escaping
    let code = r#"
let msg = 'She said "hello"'
println("{}", msg)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("She said \"hello\""));
}

#[test]
fn test_parser_072_char_literal_still_works() {
    // Regression test: character literals should still work with single quotes
    let code = r#"
let ch = 'x'
println("{}", ch)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("x"));
}

#[test]
fn test_parser_072_transpile_mode() {
    // Verify transpilation works with single-quoted strings
    let code = r#"
let msg = 'hello'
println("{}", msg)
"#;

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_072_transpile.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("let"))
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_parser_072_check_mode() {
    // Verify syntax checking works with single-quoted strings
    let code = r"
let msg = 'hello world'
";

    // Write code to temporary file
    let temp_file = "/tmp/test_parser_072_check.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();
}

#[test]
fn test_parser_072_single_quoted_string_in_function() {
    // Single-quoted strings in function context
    let code = r#"
fun greet(name) {
  'Hello, ' + name + '!'
}
println("{}", greet('Alice'))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Alice!"));
}

#[test]
fn test_parser_072_single_quoted_string_concatenation() {
    // String concatenation with single quotes
    let code = r#"
let result = 'hello' + ' ' + 'world'
println("{}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}
