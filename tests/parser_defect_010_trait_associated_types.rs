// DEFECT-PARSER-010: Trait Associated Types
// Bug: Parser failed with "Expected 'fun' or 'fn' keyword" on `type Item` declarations
// Also: Parser failed on trait generic parameters and default implementations
// Fix: Added associated_types to Trait variant, parse_trait_associated_type() helper,
//      trait generic parameter parsing, and proper method body depth tracking

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic trait with associated type
#[test]
fn test_parser_010_basic_associated_type() {
    let code = r"
trait Iterator {
    type Item
}
";
    std::fs::write("/tmp/test_parser_010_basic.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_basic.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Syntax is valid"));
}

// Test 2: Trait with associated type and method
#[test]
fn test_parser_010_associated_type_with_method() {
    let code = r"
trait Iterator {
    type Item
    fn next(&mut self) -> Option<Self::Item>
}
";
    std::fs::write("/tmp/test_parser_010_with_method.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_with_method.ruchy")
        .assert()
        .success();
}

// Test 3: Trait with generic parameter
#[test]
fn test_parser_010_trait_generic() {
    let code = r"
trait From<T> {
    fn from(value: T) -> Self
}
";
    std::fs::write("/tmp/test_parser_010_generic.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_generic.ruchy")
        .assert()
        .success();
}

// Test 4: Trait with default implementation
#[test]
fn test_parser_010_default_implementation() {
    let code = r#"
trait Summary {
    fn summarize_author(&self) -> String

    fn summarize(&self) -> String {
        f"(Read more from {}...)", self.summarize_author())
    }
}
"#;
    std::fs::write("/tmp/test_parser_010_default.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_default.ruchy")
        .assert()
        .success();
}

// Test 5: Multiple associated types (including reserved keywords)
#[test]
fn test_parser_010_multiple_associated_types() {
    let code = r"
trait Container {
    type Item
    type Err
    type Result
    type Output
}
";
    std::fs::write("/tmp/test_parser_010_multiple.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_multiple.ruchy")
        .assert()
        .success();
}

// Test 6: Book example (appendix-b-syntax-reference_example_21)
#[test]
fn test_parser_010_book_example() {
    let code = r#"
// Basic trait
trait Draw {
    fn draw(&self)
}

// Trait with default implementation
trait Summary {
    fn summarize_author(&self) -> String

    fn summarize(&self) -> String {
        f"(Read more from {}...)", self.summarize_author())
    }
}

// Trait with associated types
trait Iterator {
    type Item

    fn next(&mut self) -> Option<Self::Item>
}

// Trait with generic parameters
trait From<T> {
    fn from(value: T) -> Self
}
"#;
    std::fs::write("/tmp/test_parser_010_book.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_book.ruchy")
        .assert()
        .success();
}

// Test 7: Transpile trait with associated type
#[test]
fn test_parser_010_transpile() {
    let code = r"
trait Iterator {
    type Item
    fn next(&mut self) -> Option<Self::Item>
}
";
    std::fs::write("/tmp/test_parser_010_transpile.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("transpile")
        .arg("/tmp/test_parser_010_transpile.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("trait Iterator"))
        .stdout(predicate::str::contains("type Item"));
}

// Test 8: Complex trait with all features
#[test]
fn test_parser_010_complex() {
    let code = r"
trait ComplexTrait<T, U> {
    type Output
    type Error

    fn process(&self, input: T) -> Result<Self::Output, Self::Error>

    fn default_process(&self, input: T) -> U {
        // Default implementation
        input
    }
}
";
    std::fs::write("/tmp/test_parser_010_complex.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_010_complex.ruchy")
        .assert()
        .success();
}
