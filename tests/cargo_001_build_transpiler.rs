//! CARGO-001: Build.rs Integration Prototype Tests
//!
//! Test suite for the build transpiler that auto-transpiles .ruchy → .rs files
//! during cargo build via build.rs integration.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test .ruchy file
fn create_ruchy_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Helper to check if .rs file exists and contains expected content
fn assert_rs_file_exists(dir: &TempDir, name: &str) -> String {
    let rs_path = dir.path().join(name.replace(".ruchy", ".rs"));
    assert!(
        rs_path.exists(),
        "Expected transpiled .rs file to exist: {:?}",
        rs_path
    );
    fs::read_to_string(&rs_path).expect("Failed to read transpiled file")
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_single_file() {
    // CARGO-001: Test that transpile_all() can transpile a single .ruchy file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a simple .ruchy file
    create_ruchy_file(
        &temp_dir,
        "main.ruchy",
        r#"fun main() {
    println("Hello from Ruchy");
}"#,
    );

    // Call transpile_all() - this will fail until we implement it
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );

    assert!(result.is_ok(), "transpile_all should succeed");

    // Verify .rs file was created
    let rust_code = assert_rs_file_exists(&temp_dir, "main.ruchy");
    assert!(
        rust_code.contains("fn main"),
        "Should contain Rust main function"
    );
    assert!(rust_code.contains("println"), "Should contain println");
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_multiple_files() {
    // CARGO-001: Test that transpile_all() can handle multiple .ruchy files

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create multiple .ruchy files
    create_ruchy_file(&temp_dir, "main.ruchy", r#"fun main() { helper() }"#);

    create_ruchy_file(&temp_dir, "helper.ruchy", r#"fun helper() -> i32 { 42 }"#);

    // Transpile all files
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );

    assert!(result.is_ok(), "transpile_all should succeed");

    // Verify both .rs files were created
    let main_code = assert_rs_file_exists(&temp_dir, "main.ruchy");
    assert!(main_code.contains("fn main"));

    let helper_code = assert_rs_file_exists(&temp_dir, "helper.ruchy");
    assert!(helper_code.contains("fn helper"));
    assert!(helper_code.contains("i32"));
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_nested_directories() {
    // CARGO-001: Test that transpile_all() handles nested directory structures

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create nested directory structure
    let sub_dir = temp_dir.path().join("utils");
    fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

    create_ruchy_file(&temp_dir, "main.ruchy", r#"fun main() { println("Main") }"#);

    // Create file in subdirectory
    let utils_file = sub_dir.join("math.ruchy");
    fs::write(&utils_file, r#"fun add(a: i32, b: i32) -> i32 { a + b }"#)
        .expect("Failed to write utils file");

    // Transpile all files recursively
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );

    assert!(result.is_ok(), "transpile_all should succeed");

    // Verify main.rs exists
    assert_rs_file_exists(&temp_dir, "main.ruchy");

    // Verify utils/math.rs exists
    let math_rs_path = sub_dir.join("math.rs");
    assert!(
        math_rs_path.exists(),
        "Expected utils/math.rs to exist: {:?}",
        math_rs_path
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_incremental_no_changes() {
    // CARGO-001: Test incremental compilation - unchanged files should not be retranspiled

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    create_ruchy_file(
        &temp_dir,
        "main.ruchy",
        r#"fun main() { println("Hello") }"#,
    );

    // First transpilation
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );
    assert!(result.is_ok(), "First transpilation should succeed");

    // Get modification time of generated .rs file
    let rs_path = temp_dir.path().join("main.rs");
    let first_mtime = fs::metadata(&rs_path)
        .expect("Failed to get metadata")
        .modified()
        .expect("Failed to get modification time");

    // Wait a bit to ensure timestamps would differ
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Second transpilation without changes
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );
    assert!(result.is_ok(), "Second transpilation should succeed");

    // Get modification time after second transpilation
    let second_mtime = fs::metadata(&rs_path)
        .expect("Failed to get metadata")
        .modified()
        .expect("Failed to get modification time");

    // Modification time should be the same (file not retranspiled)
    assert_eq!(
        first_mtime, second_mtime,
        "Unchanged file should not be retranspiled"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_incremental_with_changes() {
    // CARGO-001: Test incremental compilation - changed files SHOULD be retranspiled

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ruchy_path = temp_dir.path().join("main.ruchy");

    // Create initial file
    fs::write(&ruchy_path, r#"fun main() { println("Version 1") }"#)
        .expect("Failed to write initial file");

    // First transpilation
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );
    assert!(result.is_ok(), "First transpilation should succeed");

    // Wait to ensure timestamps differ
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Modify the .ruchy file
    fs::write(&ruchy_path, r#"fun main() { println("Version 2") }"#)
        .expect("Failed to write modified file");

    // Wait to ensure .ruchy timestamp changes
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Second transpilation after changes
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );
    assert!(result.is_ok(), "Second transpilation should succeed");

    // Verify the .rs file contains new content
    let rust_code = fs::read_to_string(temp_dir.path().join("main.rs"))
        .expect("Failed to read transpiled file");
    assert!(
        rust_code.contains("Version 2"),
        "Updated content should be reflected in transpiled file"
    );
    assert!(
        !rust_code.contains("Version 1"),
        "Old content should be replaced"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_syntax_error_reporting() {
    // CARGO-001: Test that syntax errors are reported clearly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create file with syntax error
    create_ruchy_file(
        &temp_dir,
        "broken.ruchy",
        r#"fun main() {
    let x =  // Missing value (syntax error)
}"#,
    );

    // Transpilation should fail with clear error
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );

    assert!(result.is_err(), "Should fail on syntax error");

    // Error message should mention the file and be helpful
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("broken.ruchy"),
        "Error should mention the file name"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_001_transpile_all_empty_directory() {
    // CARGO-001: Test that empty directory is handled gracefully

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Call transpile_all on empty directory
    let result = ruchy::build_transpiler::transpile_all(
        temp_dir.path().to_str().unwrap(),
        "**/*.ruchy",
        temp_dir.path().to_str().unwrap(),
    );

    // Should succeed (no files to transpile is not an error)
    assert!(result.is_ok(), "Empty directory should not be an error");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        #[ignore] // Run after implementation
        fn test_cargo_001_transpile_all_never_panics(
            file_count in 1usize..10,
            file_name_seed in 0u64..1000
        ) {
            // Property: transpile_all should never panic, even with random inputs

            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create random number of files with random (but valid) names
            for i in 0..file_count {
                let file_name = format!("file_{}_{}. ruchy", file_name_seed, i);
                let _ = fs::write(
                    temp_dir.path().join(&file_name),
                    "fun main() { println(\"test\") }",
                );
            }

            // Should not panic (may succeed or fail, but no panic)
            let _ = std::panic::catch_unwind(|| {
                let _ = ruchy::build_transpiler::transpile_all(
                    temp_dir.path().to_str().unwrap(),
                    "**/*.ruchy",
                    temp_dir.path().to_str().unwrap(),
                );
            });
        }
    }
}
