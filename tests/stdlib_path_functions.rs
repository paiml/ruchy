#![allow(missing_docs)]
//! STDLIB Phase 3: Path Functions Tests
//!
//! **Task**: Implement 13 path functions
//! **Priority**: HIGH (Phase 3 of `STDLIB_ACCESS_PLAN`)
//! **Pattern**: Three-layer builtin function (proven from env/fs functions)
//!
//! Functions:
//! 1. `path_join(base`: String, component: String) -> String
//! 2. `path_join_many(components`: Vec<String>) -> String
//! 3. `path_parent(path`: String) -> Option<String>
//! 4. `path_file_name(path`: String) -> Option<String>
//! 5. `path_file_stem(path`: String) -> Option<String>
//! 6. `path_extension(path`: String) -> Option<String>
//! 7. `path_is_absolute(path`: String) -> Bool
//! 8. `path_is_relative(path`: String) -> Bool
//! 9. `path_canonicalize(path`: String) -> Result<String>
//! 10. `path_with_extension(path`: String, ext: String) -> String
//! 11. `path_with_file_name(path`: String, name: String) -> String
//! 12. `path_components(path`: String) -> Vec<String>
//! 13. `path_normalize(path`: String) -> String
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== path_join() Tests ====================

#[test]
fn test_path_join_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_join("/home/user", "documents");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_join_many() Tests ====================

#[test]
fn test_path_join_many_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_join_many(["/home", "user", "documents"]);
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_parent() Tests ====================

#[test]
fn test_path_parent_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_parent("/home/user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_file_name() Tests ====================

#[test]
fn test_path_file_name_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_file_name("/home/user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_file_stem() Tests ====================

#[test]
fn test_path_file_stem_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_file_stem("/home/user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_extension() Tests ====================

#[test]
fn test_path_extension_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_extension("/home/user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_is_absolute() Tests ====================

#[test]
fn test_path_is_absolute_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_is_absolute("/home/user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_is_relative() Tests ====================

#[test]
fn test_path_is_relative_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_is_relative("user/file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_canonicalize() Tests ====================

#[test]
fn test_path_canonicalize_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "content").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let result = path_canonicalize("{}");
    println(result);
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_with_extension() Tests ====================

#[test]
fn test_path_with_extension_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_with_extension("/home/user/file.txt", "md");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_with_file_name() Tests ====================

#[test]
fn test_path_with_file_name_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_with_file_name("/home/user/oldfile.txt", "newfile.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_components() Tests ====================

#[test]
fn test_path_components_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_components("/home/user/documents");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== path_normalize() Tests ====================

#[test]
fn test_path_normalize_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let result = path_normalize("/home/user/./documents/../file.txt");
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== Summary Test ====================

#[test]
fn test_path_functions_summary() {
    println!("STDLIB Phase 3: Path Functions");
    println!("1. path_join(base, component) - Join two path components");
    println!("2. path_join_many(components) - Join multiple path components");
    println!("3. path_parent(path) - Get parent directory");
    println!("4. path_file_name(path) - Get file name");
    println!("5. path_file_stem(path) - Get file name without extension");
    println!("6. path_extension(path) - Get file extension");
    println!("7. path_is_absolute(path) - Check if path is absolute");
    println!("8. path_is_relative(path) - Check if path is relative");
    println!("9. path_canonicalize(path) - Get absolute canonical path");
    println!("10. path_with_extension(path, ext) - Replace extension");
    println!("11. path_with_file_name(path, name) - Replace file name");
    println!("12. path_components(path) - Split path into components");
    println!("13. path_normalize(path) - Normalize path (./ and ../)");
    println!();
    println!("Three-Layer Implementation Required for each:");
    println!("1. Runtime: builtin_* in builtins.rs");
    println!("2. Transpiler: case in try_transpile_path_function()");
    println!("3. Environment: eval_* in eval_builtin.rs + builtin_init.rs");
}
