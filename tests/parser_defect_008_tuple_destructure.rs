#![allow(missing_docs)]
// DEFECT-PARSER-008: Tuple Struct Destructuring in Let Patterns
// Bug: Parser failed with "Expected RightParen, found Comma" on tuple patterns
// Fix: Updated parse_variant_pattern_with_name() to handle multiple comma-separated patterns

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic tuple struct destructuring with 3 elements
#[test]
fn test_parser_008_tuple_three_elements() {
    let code = r"
let Color(r, g, b) = red
";
    std::fs::write("/tmp/test_parser_008_three.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_three.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Syntax is valid"));
}

// Test 2: Tuple struct with 2 elements
#[test]
fn test_parser_008_tuple_two_elements() {
    let code = r"
let Point(x, y) = origin
";
    std::fs::write("/tmp/test_parser_008_two.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_two.ruchy")
        .assert()
        .success();
}

// Test 3: Tuple struct with 4 elements
#[test]
fn test_parser_008_tuple_four_elements() {
    let code = r"
let Rgba(r, g, b, a) = color
";
    std::fs::write("/tmp/test_parser_008_four.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_four.ruchy")
        .assert()
        .success();
}

// Test 4: Single element tuple (regression test for Some/Ok/Err)
#[test]
fn test_parser_008_single_element_still_works() {
    let code = r"
let Some(value) = maybe
let Ok(result) = res
let Err(error) = failure
";
    std::fs::write("/tmp/test_parser_008_single.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_single.ruchy")
        .assert()
        .success();
}

// Test 5: Book example (appendix-b-syntax-reference_example_18)
#[test]
fn test_parser_008_book_example() {
    let code = r"
// Creating instances
let origin = Point { x: 0.0, y: 0.0 }
let red = Color(255, 0, 0)

// Field access
let x_coord = origin.x

// Struct update syntax
let point2 = Point { x: 1.0, ..origin }

// Destructuring
let Point { x, y } = origin
let Color(r, g, b) = red
";
    std::fs::write("/tmp/test_parser_008_book.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_book.ruchy")
        .assert()
        .success();
}

// Test 6: Empty tuple (edge case)
#[test]
fn test_parser_008_empty_tuple() {
    let code = r"
let Unit() = unit_value
";
    std::fs::write("/tmp/test_parser_008_empty.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_empty.ruchy")
        .assert()
        .success();
}

// Test 7: Trailing comma support
#[test]
fn test_parser_008_trailing_comma() {
    let code = r"
let Color(r, g, b,) = red
";
    std::fs::write("/tmp/test_parser_008_trailing.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_008_trailing.ruchy")
        .assert()
        .success();
}
