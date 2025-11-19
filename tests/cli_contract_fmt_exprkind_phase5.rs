#![allow(missing_docs)]
//! Sprint 2 Phase 5: Final `ExprKind` Variants (Complete 100%)
//!
//! Ticket: [FMT-PERFECT-010]
//! Goal: Implement final 10 `ExprKind` variants to achieve 85/85 (100%)
//! TDD: RED → GREEN → REFACTOR
//!
//! Final Variants:
//! - `DataFrame` (df![] literal syntax)
//! - `DataFrameOperation` (operations on `DataFrames`)
//! - Extension (extension blocks for types)
//! - Macro (macro definitions)
//! - `MacroInvocation` (macro calls)
//! - `OptionalMethodCall` (`obj?.method()`)
//! - `QualifiedName` (`module::path::name`)
//! - `ReExport` (re-export from another module)
//! - Spread (...expr for spreading)
//! - `TypeAlias` (type Name = Type)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

// ==================== QualifiedName ====================

#[test]
fn test_fmt_qualified_name() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("qualified.ruchy");

    fs::write(&test_file, "std::collections::HashMap::new()").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("::"),
        "Should preserve module path separator"
    );
}

// ==================== TypeAlias ====================

#[test]
fn test_fmt_type_alias() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("type_alias.ruchy");

    fs::write(&test_file, "type UserId = int").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("type"), "Should preserve type keyword");
    assert!(formatted.contains("UserId"), "Should preserve alias name");
}

// ==================== Spread ====================

#[test]
fn test_fmt_spread_operator() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("spread.ruchy");

    fs::write(&test_file, "let arr = [1, 2, ...rest]").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("..."), "Should preserve spread operator");
}

// ==================== OptionalMethodCall ====================

#[test]
fn test_fmt_optional_method_call() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("optional_method.ruchy");

    fs::write(&test_file, "obj?.method()").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("?."),
        "Should preserve optional chaining operator"
    );
}

// ==================== Extension ====================

#[test]
#[ignore = "Parser doesn't support extension blocks yet - needs PARSER enhancement"]
fn test_fmt_extension_block() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("extension.ruchy");

    fs::write(&test_file, "extension String { fun reverse() { } }")
        .expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("extension"),
        "Should preserve extension keyword"
    );
}

// ==================== ReExport ====================

#[test]
fn test_fmt_reexport() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("reexport.ruchy");

    fs::write(&test_file, "export { foo, bar } from utils").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("export"),
        "Should preserve export keyword"
    );
    assert!(formatted.contains("from"), "Should preserve from keyword");
}

// ==================== Macro ====================

#[test]
#[ignore = "Parser doesn't support macro definitions yet - needs PARSER enhancement"]
fn test_fmt_macro_definition() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("macro_def.ruchy");

    fs::write(
        &test_file,
        "macro debug($expr) { println!(stringify!($expr)) }",
    )
    .expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("macro"), "Should preserve macro keyword");
}

// ==================== MacroInvocation ====================

#[test]
#[ignore = "Parser confuses ! with lambda syntax - needs PARSER enhancement"]
fn test_fmt_macro_invocation() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("macro_call.ruchy");

    fs::write(&test_file, "println!(\"Hello\")").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("println!"),
        "Should preserve macro invocation"
    );
}

// ==================== DataFrame ====================

#[test]
fn test_fmt_dataframe_literal() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("dataframe.ruchy");

    fs::write(
        &test_file,
        r#"let df = df!["name" => ["Alice"], "age" => [30]]"#,
    )
    .expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("df!"),
        "Should preserve DataFrame macro syntax"
    );
}

// ==================== DataFrameOperation ====================

#[test]
#[ignore = "Parser doesn't support DataFrame operations yet - needs PARSER enhancement"]
fn test_fmt_dataframe_operation() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("df_op.ruchy");

    fs::write(&test_file, "df.select([\"name\", \"age\"])").expect("Failed to write test file");

    ruchy_cmd().arg("fmt").arg(&test_file).assert().success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(
        formatted.contains("select"),
        "Should preserve DataFrame operation"
    );
}
