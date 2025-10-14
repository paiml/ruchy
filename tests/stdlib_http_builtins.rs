//! STDLIB-PHASE-5: HTTP Builtin Functions Tests (RED Phase)
//!
//! Following EXTREME TDD: RED → GREEN → REFACTOR
//! Pattern: Three-layer builtin architecture (proven from path/json modules)
//!
//! Functions to implement:
//! 1. http_get(url) - Send GET request, return response body
//! 2. http_post(url, body) - Send POST request with body
//! 3. http_put(url, body) - Send PUT request with body
//! 4. http_delete(url) - Send DELETE request

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_http_get_success() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let code = r#"
fn main() {
    // Using httpbin.org for testing
    let response = http_get("https://httpbin.org/get");
    println(response);
}
"#;
    fs::write(&source, code).expect("Failed to write test file");
    
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_http_post_with_json_body() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let code = r#"
fn main() {
    let body = "{\"name\": \"Alice\", \"age\": 30}";
    let response = http_post("https://httpbin.org/post", body);
    println(response);
}
"#;
    fs::write(&source, code).expect("Failed to write test file");
    
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_http_put_with_json_body() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let code = r#"
fn main() {
    let body = "{\"name\": \"Bob\", \"age\": 31}";
    let response = http_put("https://httpbin.org/put", body);
    println(response);
}
"#;
    fs::write(&source, code).expect("Failed to write test file");
    
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_http_delete() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let code = r#"
fn main() {
    let response = http_delete("https://httpbin.org/delete");
    println(response);
}
"#;
    fs::write(&source, code).expect("Failed to write test file");
    
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

#[test]
fn test_http_get_invalid_url() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let code = r#"
fn main() {
    let response = http_get("not-a-valid-url");
    println(response);
}
"#;
    fs::write(&source, code).expect("Failed to write test file");
    
    // Should fail with runtime error
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .failure();
}

#[test]
fn test_stdlib_http_builtin_summary() {
    println!("\n=== STDLIB-PHASE-5: HTTP Builtin Functions Summary ===");
    println!("✓ test_http_get_success");
    println!("✓ test_http_post_with_json_body");
    println!("✓ test_http_put_with_json_body");
    println!("✓ test_http_delete");
    println!("✓ test_http_get_invalid_url (error handling)");
    println!("\nImplementation Plan:");
    println!("Layer 1 (Runtime): 4 builtin_http_* functions wrapping stdlib::http");
    println!("Layer 2 (Transpiler): try_transpile_http_function() with 4 cases");
    println!("Layer 3 (Environment): eval_http_* dispatcher + registration");
    println!("Compiler: Add uses_http() detection for smart compilation");
    println!("\nTotal: 4 HTTP functions following proven three-layer pattern");
}
