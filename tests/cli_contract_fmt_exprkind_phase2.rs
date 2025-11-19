#![allow(missing_docs)]
// Sprint 2 Phase 2: ExprKind Coverage Tests
// RED phase: Failing tests for next 8 high-priority variants

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// Priority 1: Array literals and initialization
#[test]
fn test_fmt_array_literal() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array.ruchy");

    let original = "let nums = [1, 2, 3, 4, 5]";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("[1, 2, 3, 4, 5]"),
        "Array literal lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Array not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_array_init() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array_init.ruchy");

    let original = "let zeros = [0; 10]";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("[0; 10]"),
        "Array init lost! Got:\n{formatted}"
    );
}

// Priority 2: Result type (Ok/Err)
#[test]
fn test_fmt_ok_variant() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("ok.ruchy");

    let original = "let result = Ok(42)";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("Ok(42)"),
        "Ok variant lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Ok not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_err_variant() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("err.ruchy");

    let original = "let result = Err(\"failed\")";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("Err("),
        "Err variant lost! Got:\n{formatted}"
    );
}

// Priority 3: Option type (Some/None)
#[test]
fn test_fmt_some_variant() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("some.ruchy");

    let original = "let value = Some(100)";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("Some(100)"),
        "Some variant lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Some not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_none_variant() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("none.ruchy");

    let original = "let value = None";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("None"),
        "None variant lost! Got:\n{formatted}"
    );
}

// Priority 4: Try operator
#[test]
fn test_fmt_try_operator() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("try_op.ruchy");

    let original = "let result = risky()?";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains('?'),
        "Try operator lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Try not implemented! Got:\n{formatted}"
    );
}

// Priority 5: Async operations
#[test]
fn test_fmt_spawn() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("spawn.ruchy");

    let original = "spawn MyActor { count: 0 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("spawn"),
        "Spawn keyword lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Spawn not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_async_lambda() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("async_lambda.ruchy");

    let original = "let handler = async |x| { fetch(x) }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("async") && formatted.contains("|x|"),
        "Async lambda lost! Got:\n{formatted}"
    );
}

// Priority 6: Pattern matching extensions
#[test]
fn test_fmt_if_let() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("if_let.ruchy");

    let original = "if let Some(x) = maybe { x } else { 0 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("if let"),
        "if let lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "IfLet not implemented! Got:\n{formatted}"
    );
}

// Priority 7: Collection operations
#[test]
fn test_fmt_slice() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("slice.ruchy");

    let original = "let sub = arr[1..5]";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("[1..5]"),
        "Slice syntax lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Slice not implemented! Got:\n{formatted}"
    );
}

// Priority 8: Optional chaining
#[test]
fn test_fmt_optional_field_access() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("optional_field.ruchy");

    let original = "let value = obj?.field";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("?."),
        "Optional chaining lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "OptionalFieldAccess not implemented! Got:\n{formatted}"
    );
}
