//! EXTREME TDD Tests for Issue #147: Duplicate pub keyword
//!
//! Bug: `pub fun foo()` transpiles to `pub pub fn foo()` instead of `pub fn foo()`
//! Root cause: Visibility modifier handling adds extra `pub`

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

/// Helper to create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: These tests should FAIL before the fix
// ============================================================================

#[test]
fn test_issue_147_01_pub_fun_basic() {
    // Basic pub fun should not produce duplicate pub
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
pub fun hello() {
    println!("Hello");
}
fun main() {
    hello();
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have "pub fn", NOT "pub pub fn"
    assert!(
        output.contains("pub fn hello"),
        "Expected 'pub fn hello', got: {}",
        output
    );
    assert!(
        !output.contains("pub pub fn"),
        "Should NOT contain 'pub pub fn': {}",
        output
    );
}

#[test]
fn test_issue_147_02_pub_fun_in_impl() {
    // pub fun in impl block
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fun new() -> Counter {
        Counter { value: 0 }
    }

    pub fun increment(&mut self) {
        self.value = self.value + 1;
    }
}

fun main() {
    let mut c = Counter::new();
    c.increment();
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Should have "pub fn new" and "pub fn increment", NOT "pub pub fn"
    assert!(
        !output.contains("pub pub fn"),
        "Should NOT contain 'pub pub fn': {}",
        output
    );
}

#[test]
fn test_issue_147_03_pub_fun_compiles() {
    // The transpiled code must actually compile with rustc
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
pub fun greet(name: String) -> String {
    format!("Hello, {}", name)
}

fun main() {
    let msg = greet(String::from("World"));
    println!("{}", msg);
}
"#,
    )
    .unwrap();

    // Should compile successfully
    ruchy_cmd()
        .arg("compile")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_issue_147_04_non_pub_fun_unchanged() {
    // Non-pub fun should just be "fn", not "pub fn"
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun helper() -> i32 {
    42
}

fun main() {
    let x = helper();
    println!("{}", x);
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // helper should be "fn helper", not "pub fn helper"
    assert!(
        output.contains("fn helper") && !output.contains("pub fn helper"),
        "helper should be private, got: {}",
        output
    );
}

#[test]
fn test_issue_147_05_mixed_visibility() {
    // Mix of pub and non-pub functions
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
pub fun public_api() -> i32 {
    private_helper()
}

fun private_helper() -> i32 {
    42
}

fun main() {
    let x = public_api();
    println!("{}", x);
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // public_api should be "pub fn", private_helper should be just "fn"
    assert!(
        output.contains("pub fn public_api"),
        "Expected 'pub fn public_api': {}",
        output
    );
    assert!(
        !output.contains("pub pub fn"),
        "Should NOT contain 'pub pub fn': {}",
        output
    );
}

#[test]
fn test_issue_147_06_pub_struct_with_pub_methods() {
    // pub struct with pub methods in impl
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
pub struct Runtime {
    api_endpoint: String,
}

impl Runtime {
    pub fun new() -> Runtime {
        let endpoint = String::from("127.0.0.1:9001");
        Runtime { api_endpoint: endpoint }
    }

    pub fun get_endpoint(&self) -> String {
        self.api_endpoint.clone()
    }
}

fun main() {
    let rt = Runtime::new();
    println!("{}", rt.get_endpoint());
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Verify no duplicate pub
    assert!(
        !output.contains("pub pub"),
        "Should NOT contain 'pub pub': {}",
        output
    );
}
