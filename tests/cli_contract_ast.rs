//! CLI Contract Tests: `ruchy parse` (AST Pretty-Printer)
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, AST output)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Valid syntax, AST printed
//! - Exit code 1: Invalid syntax OR file not found
//! - stdout: AST representation (Rust Debug format)
//! - stderr: Error messages (parse errors, missing files)
//! - AST structure: Contains "Expr", "kind", "span", etc.
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (ast: 0.5 → target 0.67)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_parse_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_parse_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_parse_missing_file_exits_nonzero() {
    ruchy_cmd()
        .arg("parse")
        .arg("nonexistent_xyz.ruchy")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (AST output)
// ============================================================================

#[test]
fn cli_parse_outputs_ast_structure() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "ast_test.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr"))
        .stdout(predicate::str::contains("kind"))
        .stdout(predicate::str::contains("span"));
}

#[test]
fn cli_parse_shows_literal_value() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "literal.ruchy", "let x = 42\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"))
        .stdout(predicate::str::contains("Literal")
            .or(predicate::str::contains("Integer")));
}

#[test]
fn cli_parse_shows_variable_name() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "variable.ruchy", "let foo = 100\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("foo"))
        .stdout(predicate::str::contains("Let"));
}

#[test]
fn cli_parse_shows_function_definition() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "function.ruchy",
        "fun add(a, b) {\n  a + b\n}\n",
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("Fun")
            .or(predicate::str::contains("Function")));
}

#[test]
fn cli_parse_shows_binary_operation() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "binary.ruchy", "let result = 2 + 3\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Binary")
            .or(predicate::str::contains("Add"))
            .or(predicate::str::contains("+")));
}

// ============================================================================
// CLI CONTRACT TESTS: STDERR (error messages)
// ============================================================================

#[test]
fn cli_parse_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

#[test]
fn cli_parse_syntax_error_mentions_parse() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "error.ruchy", "let x = \n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Parse")
            .or(predicate::str::contains("parse"))
            .or(predicate::str::contains("error")));
}

#[test]
fn cli_parse_missing_file_writes_stderr() {
    ruchy_cmd()
        .arg("parse")
        .arg("missing.ruchy")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: COMPLEX AST STRUCTURES
// ============================================================================

#[test]
fn cli_parse_if_expression() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "if_expr.ruchy",
        "if x > 0 {\n  println(\"positive\")\n}\n",
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("If")
            .or(predicate::str::contains("Conditional")));
}

#[test]
fn cli_parse_for_loop() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "for_loop.ruchy",
        "for i in range(0, 10) {\n  println(i)\n}\n",
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("For")
            .or(predicate::str::contains("range")));
}

#[test]
fn cli_parse_match_expression() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "match_expr.ruchy",
        "match x {\n  1 => \"one\"\n  _ => \"other\"\n}\n",
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Match"));
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_parse_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .failure() // Empty file is error
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn cli_parse_comment_only_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .failure() // Comment-only is error
        .stderr(predicate::str::contains("Unexpected end of input"));
}

#[test]
fn cli_parse_complex_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "complex.ruchy",
        r#"
fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

let result = fibonacci(10)
println(result)
"#,
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fibonacci"))
        .stdout(predicate::str::contains("Fun"));
}

#[test]
fn cli_parse_string_literal() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "string.ruchy", "let msg = \"Hello, World!\"\n");

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"))
        .stdout(predicate::str::contains("String")
            .or(predicate::str::contains("Literal")));
}

#[test]
fn cli_parse_multiline_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "multiline.ruchy",
        "let x = 1\nlet y = 2\nlet z = x + y\n",
    );

    ruchy_cmd()
        .arg("parse")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Block")
            .or(predicate::str::contains("Expr")));
}
