#![allow(deprecated)]
// LANG-COMP-015: Enums - Validation Tests with Traceability
// Links to: examples/lang_comp/15-enums/*.ruchy
// Validates: LANG-COMP-015 Enums (definition, variants, matching, data)
// EXTREME TDD Protocol: Tests use assert_cmd + mandatory naming convention

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to get example file path
fn example_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/lang_comp/15-enums")
        .join(relative_path)
}

// ==================== LANG-COMP-015-01: Basic Enums ====================

#[test]
fn test_langcomp_015_01_basic_enums_example_file() {
    let example = example_path("01_basic_enums.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_015_01_enum_definition_and_creation() {
    ruchy_cmd()
        .write_stdin("fn main() { enum Color { Red, Green } let c = Color::Red; println(c) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Red"));
}

#[test]
fn test_langcomp_015_01_enum_printing() {
    ruchy_cmd()
        .write_stdin("fn main() { enum Status { Ready } let s = Status::Ready; println(s) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ready"));
}

// ==================== LANG-COMP-015-02: Enum Pattern Matching ====================

#[test]
fn test_langcomp_015_02_enum_matching_example_file() {
    let example = example_path("02_enum_matching.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_015_02_enum_match_simple() {
    ruchy_cmd()
        .write_stdin("fn main() { enum Color { Red, Blue } let c = Color::Red; match c { Color::Red => println(\"red\"), Color::Blue => println(\"blue\") } }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("red"));
}

// ==================== LANG-COMP-015-03: Enums with Data ====================

#[test]
fn test_langcomp_015_03_enum_with_data_example_file() {
    let example = example_path("03_enum_with_data.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_015_03_tuple_variant_creation() {
    ruchy_cmd()
        .write_stdin(
            "fn main() { enum Message { Text(i32) } let msg = Message::Text(42); println(msg) }",
        )
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Text"));
}

#[test]
fn test_langcomp_015_03_tuple_variant_matching() {
    ruchy_cmd()
        .write_stdin(
            "fn main() { enum Message { Text(i32) } let msg = Message::Text(42); match msg { Message::Text(n) => println(n) } }",
        )
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ==================== LANG-COMP-015-04: Mixed Enum Variants ====================

#[test]
fn test_langcomp_015_04_enum_mixed_example_file() {
    let example = example_path("04_enum_mixed.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_015_04_mixed_variants_matching() {
    ruchy_cmd()
        .write_stdin("fn main() { enum Msg { Quit, Move(i32) } let m = Msg::Move(10); match m { Msg::Quit => println(0), Msg::Move(x) => println(x) } }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}
