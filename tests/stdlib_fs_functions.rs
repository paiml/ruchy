#![allow(missing_docs)]
//! STDLIB Phase 2: File System Functions Tests
//!
//! **Task**: Implement 12 file system functions
//! **Priority**: HIGH (Phase 2 of `STDLIB_ACCESS_PLAN`)
//! **Pattern**: Three-layer builtin function (proven from env functions)
//!
//! Functions:
//! 1. `fs_read(path`: String) -> Result<String>
//! 2. `fs_write(path`: String, content: String) -> Result<()>
//! 3. `fs_exists(path`: String) -> Bool
//! 4. `fs_create_dir(path`: String) -> Result<()>
//! 5. `fs_remove_file(path`: String) -> Result<()>
//! 6. `fs_remove_dir(path`: String) -> Result<()>
//! 7. `fs_copy(from`: String, to: String) -> Result<()>
//! 8. `fs_rename(from`: String, to: String) -> Result<()>
//! 9. `fs_metadata(path`: String) -> Result<Metadata>
//! 10. `fs_read_dir(path`: String) -> Result<Vec<String>>
//! 11. `fs_canonicalize(path`: String) -> Result<String>
//! 12. `fs_is_file(path`: String) -> Bool
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

// ==================== fs_read() Tests ====================

#[test]
fn test_fs_read_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let content = fs_read("{}");
    println(content);
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_write() Tests ====================

#[test]
fn test_fs_write_basic() {
    let temp = temp_dir();
    let output_file = temp.path().join("output.txt");

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_write("{}", "Test content");
    println("File written");
}}
"#,
        output_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_exists() Tests ====================

#[test]
fn test_fs_exists_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("exists.txt");
    fs::write(&test_file, "exists").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let exists = fs_exists("{}");
    println(exists);
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_create_dir() Tests ====================

#[test]
fn test_fs_create_dir_basic() {
    let temp = temp_dir();
    let new_dir = temp.path().join("new_directory");

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_create_dir("{}");
    println("Directory created");
}}
"#,
        new_dir.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_remove_file() Tests ====================

#[test]
fn test_fs_remove_file_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("remove_me.txt");
    fs::write(&test_file, "delete this").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_remove_file("{}");
    println("File removed");
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_remove_dir() Tests ====================

#[test]
fn test_fs_remove_dir_basic() {
    let temp = temp_dir();
    let test_dir = temp.path().join("remove_dir");
    fs::create_dir(&test_dir).unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_remove_dir("{}");
    println("Directory removed");
}}
"#,
        test_dir.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_copy() Tests ====================

#[test]
fn test_fs_copy_basic() {
    let temp = temp_dir();
    let source_file = temp.path().join("source.txt");
    let dest_file = temp.path().join("dest.txt");
    fs::write(&source_file, "copy me").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_copy("{}", "{}");
    println("File copied");
}}
"#,
        source_file.display(),
        dest_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_rename() Tests ====================

#[test]
fn test_fs_rename_basic() {
    let temp = temp_dir();
    let old_name = temp.path().join("old.txt");
    let new_name = temp.path().join("new.txt");
    fs::write(&old_name, "rename me").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    fs_rename("{}", "{}");
    println("File renamed");
}}
"#,
        old_name.display(),
        new_name.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_metadata() Tests ====================

#[test]
fn test_fs_metadata_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("metadata.txt");
    fs::write(&test_file, "some content").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let meta = fs_metadata("{}");
    println("Metadata retrieved");
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_read_dir() Tests ====================

#[test]
fn test_fs_read_dir_basic() {
    let temp = temp_dir();
    fs::write(temp.path().join("file1.txt"), "1").unwrap();
    fs::write(temp.path().join("file2.txt"), "2").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let entries = fs_read_dir("{}");
    println("Directory read");
}}
"#,
        temp.path().display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_canonicalize() Tests ====================

#[test]
fn test_fs_canonicalize_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("canonical.txt");
    fs::write(&test_file, "canonical").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let canonical = fs_canonicalize("{}");
    println(canonical);
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== fs_is_file() Tests ====================

#[test]
fn test_fs_is_file_basic() {
    let temp = temp_dir();
    let test_file = temp.path().join("is_file.txt");
    fs::write(&test_file, "file").unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let is_file = fs_is_file("{}");
    println(is_file);
}}
"#,
        test_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== Summary Test ====================

#[test]
fn test_fs_functions_summary() {
    println!("STDLIB Phase 2: File System Functions");
    println!("1. fs_read(path) - Read file contents");
    println!("2. fs_write(path, content) - Write to file");
    println!("3. fs_exists(path) - Check if path exists");
    println!("4. fs_create_dir(path) - Create directory");
    println!("5. fs_remove_file(path) - Delete file");
    println!("6. fs_remove_dir(path) - Delete directory");
    println!("7. fs_copy(from, to) - Copy file");
    println!("8. fs_rename(from, to) - Rename/move file");
    println!("9. fs_metadata(path) - Get file metadata");
    println!("10. fs_read_dir(path) - List directory contents");
    println!("11. fs_canonicalize(path) - Get absolute path");
    println!("12. fs_is_file(path) - Check if path is file");
    println!();
    println!("Three-Layer Implementation Required for each:");
    println!("1. Runtime: builtin_* in builtins.rs");
    println!("2. Transpiler: case in try_transpile_fs_function()");
    println!("3. Environment: eval_* in eval_builtin.rs + builtin_init.rs");
}
