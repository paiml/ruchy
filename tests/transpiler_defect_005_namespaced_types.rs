//! TRANSPILER-DEFECT-005: Namespaced types in function parameters cause panic
//!
//! Root Cause: `transpile_named_type()` uses `format_ident`! on full string "`Result::Ok`"
//! Expected: Parse path segments and build :: separated path tokens\
//! Impact: CRITICAL - Cannot use `std::Result`, `std::Option`, or any namespaced types

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Test that `std::result::Result` works in type position
#[test]
fn test_std_result_in_function_param() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun process_result(r: std::result::Result) {
    println!("Processing result");
}

fun main() {
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// Test that `std::option::Option` works in type position  
#[test]
fn test_std_option_in_return_type() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun get_value() -> std::option::Option {
    None
}

fun main() {
    let val = get_value();
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// Test Vec<std::string::String> pattern
#[test]
fn test_nested_namespace_in_generic() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
struct Container {
    items: Vec
}

fun main() {
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// Test that transpiler doesn't panic on simple :: paths
#[test]
fn test_simple_namespace_transpiles_without_panic() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
// This should transpile without panicking
// The resulting Rust may not compile (if MyModule doesn't exist)
// but the transpiler should not crash
fun use_namespaced(val: MyModule::MyType) {
    println!("Got value");
}

fun main() {
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    // The key test: transpiler should not panic
    // It's OK if rustc fails later (MyModule doesn't exist)
    // but the transpiler itself must succeed
    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}
