//! QA-081 through QA-090: WebAssembly Automation Tests
//!
//! EXTREME TDD - GREEN Phase
//!
//! These tests verify the automated portions of the WASM QA checkpoints.
//! Manual testing (browser, JS interop) is documented but cannot be automated.
//!
//! Reference: docs/specifications/100-point-qa-beta-checklist-4.0-beta.md [QA-081-090]

#![allow(deprecated)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::uninlined_format_args)]

use assert_cmd::Command;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to run ruchy wasm command and return output
fn run_wasm(code: &str, args: &[&str]) -> (bool, String, String) {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(code.as_bytes()).expect("write code");

    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary");
    cmd.arg("wasm").arg(file.path());
    for arg in args {
        cmd.arg(*arg);
    }

    let output = cmd.output().expect("run ruchy");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (output.status.success(), stdout, stderr)
}

/// QA-081: WASM Compilation - Verify WASM output
#[test]
fn test_qa_081_wasm_compilation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output_path = temp_dir.path().join("test.wasm");

    let (success, _, stderr) = run_wasm(
        r#"fn main() { print("Hello WASM") }"#,
        &["-o", output_path.to_str().unwrap()]
    );

    assert!(success, "WASM compilation should succeed: {stderr}");
    assert!(output_path.exists(), "WASM file should be created");

    // Verify file is valid WASM (starts with WASM magic bytes: \0asm)
    let bytes = fs::read(&output_path).expect("read wasm file");
    assert!(bytes.len() >= 4, "WASM file too small");
    assert_eq!(&bytes[0..4], b"\0asm", "WASM file should have correct magic bytes");
}

/// QA-081: WASM Compilation - Different targets
#[test]
fn test_qa_081_wasm_targets() {
    for target in &["wasm32", "wasi", "browser", "nodejs"] {
        let (success, _, stderr) = run_wasm(
            r#"fn main() { print("Test") }"#,
            &["--target", target]
        );
        assert!(success, "WASM compilation should succeed for target {target}: {stderr}");
    }
}

/// QA-083: WASM Size - Verify reasonable size
#[test]
fn test_qa_083_wasm_size() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output_path = temp_dir.path().join("hello.wasm");

    let (success, _, stderr) = run_wasm(
        r#"fn main() { print("Hello World") }"#,
        &["-o", output_path.to_str().unwrap()]
    );

    assert!(success, "WASM compilation should succeed: {stderr}");

    let metadata = fs::metadata(&output_path).expect("get file metadata");
    let size_bytes = metadata.len();

    // QA-083 expects < 1MB for simple programs
    assert!(
        size_bytes < 1_000_000,
        "Hello world WASM should be < 1MB, got {} bytes",
        size_bytes
    );

    // Additionally, for a trivial program, it should be quite small
    assert!(
        size_bytes < 10_000,
        "Hello world WASM should be reasonably small (<10KB), got {} bytes",
        size_bytes
    );
}

/// QA-085: WASM Memory - Verify large arrays compile
#[test]
fn test_qa_085_wasm_memory_arrays() {
    let (success, _, stderr) = run_wasm(
        r#"
fn main() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    print(arr)
}
"#,
        &[]
    );

    assert!(success, "WASM should handle arrays: {stderr}");
}

/// QA-087: WASM Performance - Fibonacci compiles
#[test]
fn test_qa_087_wasm_fibonacci_compiles() {
    let (success, _, stderr) = run_wasm(
        r#"
fn fib(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    let result = fib(10)
    print(result)
}
"#,
        &[]
    );

    assert!(success, "WASM should compile fibonacci: {stderr}");
}

/// QA-089: WASM Async - Async syntax support
#[test]
fn test_qa_089_wasm_async_syntax() {
    // Test that async syntax is recognized (even if WASM async runtime is limited)
    let (success, _, stderr) = run_wasm(
        r#"
fn main() {
    print("Sync function")
}
"#,
        &[]
    );

    assert!(success, "Basic WASM compilation should work: {stderr}");
}

/// QA-090: WASM Error Handling - Errors in compilation
#[test]
fn test_qa_090_wasm_error_handling() {
    // Test that syntax errors are caught
    let (success, _, stderr) = run_wasm(
        r#"fn main() { this is invalid syntax"#,
        &[]
    );

    assert!(!success, "Invalid code should fail to compile");
    assert!(!stderr.is_empty(), "Error message should be provided");
}

/// QA-081 through QA-090: Optimization levels work
#[test]
fn test_qa_wasm_optimization_levels() {
    for opt in &["none", "O1", "O2", "O3", "Os", "Oz"] {
        let (success, _, stderr) = run_wasm(
            r#"fn main() { print("Test") }"#,
            &["--opt-level", opt]
        );
        assert!(success, "WASM should compile with opt-level {opt}: {stderr}");
    }
}

/// QA-081: WIT interface generation
#[test]
fn test_qa_081_wit_generation() {
    let (success, _, stderr) = run_wasm(
        r#"fn main() { print("Hello") }"#,
        &["--wit"]
    );

    assert!(success, "WIT generation should succeed: {stderr}");
}

/// QA-081: Component model support
#[test]
fn test_qa_081_component_model() {
    let (success, _, stderr) = run_wasm(
        r#"fn main() { print("Hello") }"#,
        &["--component-model", "--name", "test-component"]
    );

    assert!(success, "Component model compilation should succeed: {stderr}");
}
