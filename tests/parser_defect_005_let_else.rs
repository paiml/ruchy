//! DEFECT-PARSER-005: Let-else Pattern Syntax Not Supported
//!
//! ROOT CAUSE: Parser doesn't recognize `let pattern = expr else { diverging_block }` syntax
//! IMPACT: Many book examples failing (appendix-b-syntax-reference_example_10.ruchy and others)
//!
//! EXTREME TDD APPROACH:
//! - RED: Write tests that demonstrate expected behavior (will fail initially)
//! - GREEN: Implement minimal fix to make tests pass
//! - REFACTOR: Apply quality gates (complexity ≤10, zero SATD)

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn create_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

#[test]
fn test_parser_005_let_else_minimal() {
    let code = r"
fn main() {
    let x = Some(42);
    let Some(value) = x else {
        return;
    };
    println(value);
}
";
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_parser_005_let_else_with_return_error() {
    let code = r#"
fn get_value(opt: Option<i32>) -> Result<i32, String> {
    let Some(value) = opt else {
        return Err("No value");
    };
    Ok(value)
}
"#;
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_parser_005_let_else_with_panic() {
    let code = r#"
fn main() {
    let x = None;
    let Some(value) = x else {
        panic("Expected Some value");
    };
    println(value);
}
"#;
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_parser_005_let_else_complex_pattern() {
    let code = r"
fn main() {
    let x = Some((1, 2));
    let Some((a, b)) = x else {
        return;
    };
    println(a + b);
}
";
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_parser_005_let_else_book_example() {
    // Exact example from appendix-b-syntax-reference_example_10.ruchy line 18-20
    let code = r#"
fn main() {
    let optional = Some(42);
    let Some(value) = optional else {
        return Err("No value");
    };
    println(value);
}
"#;
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_parser_005_regular_let_still_works() {
    // Ensure regular let statements still work (control test)
    let code = r"
fn main() {
    let x = 42;
    let Some(y) = Some(10);
    println(x + y);
}
";
    let file = create_temp_file(code);
    ruchy_cmd()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}
