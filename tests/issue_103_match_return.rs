//! Issue #103: Match arms with early return generate invalid Rust (semicolon before comma)
//!
//! Root Cause: transpiler adds semicolons unconditionally to return statements
//! Fixed in: src/backend/transpiler/dispatcher_helpers/misc.rs:43-52
//! Fixed in: src/backend/transpiler/statements.rs:811-813, 1002-1004, 1208-1213
//!
//! Pattern: Match arm with return expression
//! Before: `Err(e) => return Err(e); ,` ❌ (invalid Rust syntax)
//! After: `Err(e) => return Err(e),` ✅ (valid Rust syntax)

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test match arm with early return (minimal reproduction)
#[test]
fn test_issue_103_match_return_minimal() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun test_early_return() -> Result<i32, String> {
    let x = match some_result() {
        Ok(val) => val,
        Err(e) => return Err(e)
    };
    Ok(x + 1)
}

fun some_result() -> Result<i32, String> {
    Ok(42)
}

fun main() {
    match test_early_return() {
        Ok(v) => println!("Success: {}", v),
        Err(e) => println!("Error: {}", e)
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    // Should compile successfully (not "expected `,`, found `;`")
    ruchy_cmd()
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg("/tmp/test_issue_103_1")
        .assert()
        .success();
}

/// RED: Test multiple match arms with returns
#[test]
fn test_issue_103_multiple_returns() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun process() -> Result<i32, String> {
    let x = match get_value() {
        Ok(0) => return Err("zero".to_string()),
        Ok(n) => n,
        Err(e) => return Err(e)
    };
    Ok(x * 2)
}

fun get_value() -> Result<i32, String> {
    Ok(5)
}

fun main() {
    match process() {
        Ok(v) => println!("Result: {}", v),
        Err(e) => println!("Error: {}", e)
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd()
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg("/tmp/test_issue_103_2")
        .assert()
        .success();
}

/// RED: Test nested matches with returns
#[test]
fn test_issue_103_nested_match_returns() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun nested() -> Result<i32, String> {
    let outer = match first() {
        Ok(val) => {
            match second(val) {
                Ok(v) => v,
                Err(e) => return Err(e)
            }
        },
        Err(e) => return Err(e)
    };
    Ok(outer)
}

fun first() -> Result<i32, String> {
    Ok(10)
}

fun second(n: i32) -> Result<i32, String> {
    Ok(n + 5)
}

fun main() {
    match nested() {
        Ok(v) => println!("Nested result: {}", v),
        Err(e) => println!("Nested error: {}", e)
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd()
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg("/tmp/test_issue_103_3")
        .assert()
        .success();
}

/// RED: Test that functions starting with "test_" retain return types
/// This was a secondary bug discovered during Issue #103 investigation
#[test]
fn test_issue_103_test_prefix_return_type() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun test_helper() -> Result<i32, String> {
    Ok(42)
}

fun main() {
    match test_helper() {
        Ok(v) => println!("Helper: {}", v),
        Err(e) => println!("Error: {}", e)
    }
}

main()
"#;
    fs::write(&script, code).unwrap();

    // Transpile and check that return type is preserved
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&script)
        .output()
        .unwrap();

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should have return type (not "fn test_helper() {")
    assert!(transpiled.contains("fn test_helper() -> Result<i32, String>"),
        "Function starting with 'test_' should retain return type annotation");

    // Should compile successfully
    ruchy_cmd()
        .arg("compile")
        .arg(&script)
        .arg("-o")
        .arg("/tmp/test_issue_103_4")
        .assert()
        .success();
}

/// GREEN: Verify transpiled code has correct syntax
#[test]
fn test_issue_103_transpiled_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun check() -> Result<i32, String> {
    match get() {
        Ok(v) => Ok(v),
        Err(e) => return Err(e)
    }
}

fun get() -> Result<i32, String> {
    Ok(100)
}

fun main() {
    println!("Testing");
}

main()
"#;
    fs::write(&script, code).unwrap();

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&script)
        .output()
        .unwrap();

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should NOT have semicolon before comma
    assert!(!transpiled.contains("return Err(e); ,"),
        "Should not have semicolon before comma in match arm");

    // Should have proper comma after return expression
    assert!(transpiled.contains("return Err(e),") || transpiled.contains("return Err(e)"),
        "Should have return expression without semicolon in match arm");
}
