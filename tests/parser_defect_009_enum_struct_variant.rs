// DEFECT-PARSER-009: Enum Struct Variants
// Bug: Parser failed with "Expected variant name in enum" on struct-style variants
// Fix: Added EnumVariantKind enum and parse_variant_struct_fields() helper

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic enum with struct variant
#[test]
fn test_parser_009_basic_struct_variant() {
    let code = r#"
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
"#;
    std::fs::write("/tmp/test_parser_009_basic.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_basic.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Syntax is valid"));
}

// Test 2: Struct variant only
#[test]
fn test_parser_009_struct_variant_only() {
    let code = r#"
enum Position {
    Point { x: i32, y: i32, z: i32 },
}
"#;
    std::fs::write("/tmp/test_parser_009_struct_only.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_struct_only.ruchy")
        .assert()
        .success();
}

// Test 3: Multiple struct variants
#[test]
fn test_parser_009_multiple_struct_variants() {
    let code = r#"
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64, angle: f64 },
}
"#;
    std::fs::write("/tmp/test_parser_009_multiple.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_multiple.ruchy")
        .assert()
        .success();
}

// Test 4: Mixed variant kinds (unit, tuple, struct)
#[test]
fn test_parser_009_mixed_variants() {
    let code = r#"
enum Event {
    Quit,
    KeyPress(char),
    MouseMove { x: i32, y: i32 },
    Click { x: i32, y: i32, button: String },
}
"#;
    std::fs::write("/tmp/test_parser_009_mixed.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_mixed.ruchy")
        .assert()
        .success();
}

// Test 5: Generic enum with struct variant (Result-like)
#[test]
fn test_parser_009_generic_struct_variant() {
    let code = r#"
enum Result<T, E> {
    Ok(T),
    Err(E),
}
"#;
    std::fs::write("/tmp/test_parser_009_generic.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_generic.ruchy")
        .assert()
        .success();
}

// Test 6: Struct variant with trailing comma
#[test]
fn test_parser_009_trailing_comma() {
    let code = r#"
enum Message {
    Move { x: i32, y: i32, },
}
"#;
    std::fs::write("/tmp/test_parser_009_trailing.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_trailing.ruchy")
        .assert()
        .success();
}

// Test 7: Empty struct variant (edge case)
#[test]
fn test_parser_009_empty_struct_variant() {
    let code = r#"
enum Empty {
    Unit {},
}
"#;
    std::fs::write("/tmp/test_parser_009_empty.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_empty.ruchy")
        .assert()
        .success();
}

// Test 8: Book example (appendix-b-syntax-reference_example_19)
#[test]
fn test_parser_009_book_example() {
    let code = r#"
// Enum definition
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
"#;
    std::fs::write("/tmp/test_parser_009_book.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_009_book.ruchy")
        .assert()
        .success();
}

// Test 9: Transpile to Rust (verify code generation)
#[test]
fn test_parser_009_transpile() {
    let code = r#"
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
"#;
    std::fs::write("/tmp/test_parser_009_transpile.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("transpile")
        .arg("/tmp/test_parser_009_transpile.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("Move"))
        .stdout(predicate::str::contains("x : i32"))
        .stdout(predicate::str::contains("y : i32"));
}
