#![allow(missing_docs)]
// Sprint 2: ExprKind Coverage Tests
// RED phase: Failing tests for critical ExprKind variants

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// Priority 1: Lambda expressions (functional programming core)
#[test]
fn test_fmt_lambda_simple() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("lambda.ruchy");

    let original = "let add = |x, y| x + y";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("|x, y|"),
        "Lambda parameters lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Lambda not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_lambda_no_params() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("lambda_no_params.ruchy");

    let original = "let f = || 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("|| 42"),
        "Lambda format broken! Got:\n{formatted}"
    );
}

// Priority 2: ObjectLiteral (JavaScript-style objects)
#[test]
fn test_fmt_object_literal_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("object_empty.ruchy");

    let original = "let obj = {}";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("{}"),
        "Empty object lost! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_object_literal_with_fields() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("object_fields.ruchy");

    let original = "let obj = { name: \"Alice\", age: 30 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("name:") && formatted.contains("age:"),
        "Object fields lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "ObjectLiteral not implemented! Got:\n{formatted}"
    );
}

// Priority 3: StructLiteral (Rust-style structs)
#[test]
fn test_fmt_struct_literal() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("struct_literal.ruchy");

    let original = "let p = Point { x: 10, y: 20 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("Point") && formatted.contains("x:") && formatted.contains("y:"),
        "Struct literal broken! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "StructLiteral not implemented! Got:\n{formatted}"
    );
}

// Priority 4: Ternary operator
#[test]
fn test_fmt_ternary() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("ternary.ruchy");

    let original = "let result = x > 0 ? \"positive\" : \"negative\"";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains('?') && formatted.contains(':'),
        "Ternary operator lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Ternary not implemented! Got:\n{formatted}"
    );
}

// Priority 5: Throw/TryCatch (error handling)
#[test]
fn test_fmt_throw() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("throw.ruchy");

    let original = "throw \"Error occurred\"";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("throw"),
        "Throw keyword lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Throw not implemented! Got:\n{formatted}"
    );
}

#[test]
#[ignore = "RED phase: try/catch syntax not yet implemented in parser"]
fn test_fmt_try_catch() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("try_catch.ruchy");

    let original = "try { risky() } catch (e) { handle(e) }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("try") && formatted.contains("catch"),
        "Try/catch structure lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "TryCatch not implemented! Got:\n{formatted}"
    );
}

// Priority 6: Async/Await
#[test]
fn test_fmt_await() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("await.ruchy");

    let original = "let result = await fetch(url)";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("await"),
        "Await keyword lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Await not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_async_block() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("async_block.ruchy");

    let original = "let task = async { fetch(url) }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("async"),
        "Async keyword lost! Got:\n{formatted}"
    );
}

// Priority 7: TypeCast
#[test]
fn test_fmt_type_cast() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("type_cast.ruchy");

    let original = "let x = value as i32";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("as i32"),
        "Type cast lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "TypeCast not implemented! Got:\n{formatted}"
    );
}
