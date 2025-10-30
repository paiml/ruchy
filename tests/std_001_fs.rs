//! STD-001: File I/O Module Tests (ruchy/std/fs)
//!
//! Test suite for file system operations module.
//! Thin wrappers around Rust's `std::fs` with Ruchy-friendly API.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use std::fs;
use tempfile::TempDir;

/// Helper to create a test file with content
fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

#[test]
fn test_std_001_read_to_string_success() {
    // STD-001: Test reading file contents as string

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_content = "Hello, Ruchy!";
    let file_path = create_test_file(&temp_dir, "test.txt", test_content);

    // Call ruchy::stdlib::fs::read_to_string
    let result = ruchy::stdlib::fs::read_to_string(file_path.to_str().unwrap());

    assert!(result.is_ok(), "read_to_string should succeed");
    let content = result.unwrap();
    assert_eq!(content, test_content, "Content must match exactly");
    assert_eq!(
        content.len(),
        test_content.len(),
        "Content length must match"
    );
    assert!(content.contains("Ruchy"), "Content must contain 'Ruchy'");
}

#[test]
fn test_std_001_read_to_string_nonexistent() {
    // STD-001: Test reading nonexistent file returns error

    let result = ruchy::stdlib::fs::read_to_string("/nonexistent/file.txt");

    assert!(result.is_err(), "Reading nonexistent file should fail");
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No such file") || error.to_string().contains("not found"));
}

#[test]
fn test_std_001_write_success() {
    // STD-001: Test writing content to file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("output.txt");
    let test_content = "Written by Ruchy";

    // Verify file doesn't exist before write
    assert!(!file_path.exists(), "File should not exist before write");

    // Call ruchy::stdlib::fs::write
    let result = ruchy::stdlib::fs::write(file_path.to_str().unwrap(), test_content);

    assert!(result.is_ok(), "write should succeed");

    // Verify file was created
    assert!(file_path.exists(), "File should exist after write");

    // Verify file was created with correct content
    let content = fs::read_to_string(&file_path).expect("Failed to read written file");
    assert_eq!(
        content, test_content,
        "File content should match what was written"
    );
    assert_eq!(
        content.len(),
        test_content.len(),
        "Content length must match"
    );

    // Verify file metadata
    let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
    assert!(metadata.is_file(), "Should be a file");
    assert_eq!(
        metadata.len(),
        test_content.len() as u64,
        "File size must match content length"
    );
}

#[test]
fn test_std_001_write_overwrites_existing() {
    // STD-001: Test that write overwrites existing file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "overwrite.txt", "Old content");

    let new_content = "New content";
    let result = ruchy::stdlib::fs::write(file_path.to_str().unwrap(), new_content);

    assert!(result.is_ok(), "write should succeed");

    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, new_content);
}

#[test]
fn test_std_001_read_bytes_success() {
    // STD-001: Test reading file as bytes

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_content = b"Binary\x00\xFF\xFEdata";
    let file_path = temp_dir.path().join("binary.dat");
    fs::write(&file_path, test_content).expect("Failed to write test file");

    // Call ruchy::stdlib::fs::read
    let result = ruchy::stdlib::fs::read(file_path.to_str().unwrap());

    assert!(result.is_ok(), "read should succeed");
    let bytes = result.unwrap();
    assert_eq!(bytes, test_content, "Bytes must match exactly");
    assert_eq!(bytes.len(), test_content.len(), "Byte length must match");
    assert_eq!(bytes[0], b'B', "First byte must be 'B'");
    assert_eq!(bytes[6], 0x00, "Null byte must be preserved");
    assert_eq!(bytes[7], 0xFF, "0xFF byte must be preserved");
    assert_eq!(bytes[8], 0xFE, "0xFE byte must be preserved");
}

#[test]
fn test_std_001_create_dir_success() {
    // STD-001: Test creating directory

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let new_dir = temp_dir.path().join("new_directory");

    // Verify dir doesn't exist before creation
    assert!(
        !new_dir.exists(),
        "Directory should not exist before creation"
    );

    // Call ruchy::stdlib::fs::create_dir
    let result = ruchy::stdlib::fs::create_dir(new_dir.to_str().unwrap());

    assert!(result.is_ok(), "create_dir should succeed");
    assert!(new_dir.exists(), "Directory should be created");
    assert!(new_dir.is_dir(), "Path should be a directory");

    // Verify we can read the directory (it's actually a directory)
    let read_result = fs::read_dir(&new_dir);
    assert!(
        read_result.is_ok(),
        "Should be able to read created directory"
    );
}

#[test]
fn test_std_001_create_dir_all_nested() {
    // STD-001: Test creating nested directories

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nested_dir = temp_dir.path().join("a").join("b").join("c");

    // Verify nested path doesn't exist
    assert!(
        !nested_dir.exists(),
        "Nested path should not exist before creation"
    );

    // Call ruchy::stdlib::fs::create_dir_all
    let result = ruchy::stdlib::fs::create_dir_all(nested_dir.to_str().unwrap());

    assert!(result.is_ok(), "create_dir_all should succeed");
    assert!(nested_dir.exists(), "Nested directory should be created");
    assert!(nested_dir.is_dir(), "Path should be a directory");

    // Verify all intermediate directories exist
    let a_dir = temp_dir.path().join("a");
    let b_dir = a_dir.join("b");
    assert!(
        a_dir.exists() && a_dir.is_dir(),
        "Intermediate 'a' should exist"
    );
    assert!(
        b_dir.exists() && b_dir.is_dir(),
        "Intermediate 'b' should exist"
    );
}

#[test]
fn test_std_001_remove_file_success() {
    // STD-001: Test removing file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "to_delete.txt", "Delete me");

    assert!(file_path.exists(), "File should exist before deletion");
    assert!(file_path.is_file(), "Should be a file");

    // Call ruchy::stdlib::fs::remove_file
    let result = ruchy::stdlib::fs::remove_file(file_path.to_str().unwrap());

    assert!(result.is_ok(), "remove_file should succeed");
    assert!(!file_path.exists(), "File should be deleted");

    // Verify we cannot read the deleted file
    let read_result = fs::read_to_string(&file_path);
    assert!(
        read_result.is_err(),
        "Should not be able to read deleted file"
    );
}

#[test]
fn test_std_001_remove_dir_success() {
    // STD-001: Test removing empty directory

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir_path = temp_dir.path().join("to_remove");
    fs::create_dir(&dir_path).expect("Failed to create test dir");

    assert!(dir_path.exists(), "Directory should exist before removal");
    assert!(dir_path.is_dir(), "Should be a directory");

    // Call ruchy::stdlib::fs::remove_dir
    let result = ruchy::stdlib::fs::remove_dir(dir_path.to_str().unwrap());

    assert!(result.is_ok(), "remove_dir should succeed");
    assert!(!dir_path.exists(), "Directory should be removed");

    // Verify we cannot read the deleted directory
    let read_result = fs::read_dir(&dir_path);
    assert!(
        read_result.is_err(),
        "Should not be able to read deleted directory"
    );
}

#[test]
fn test_std_001_copy_file_success() {
    // STD-001: Test copying file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = create_test_file(&temp_dir, "source.txt", "Copy this");
    let dest = temp_dir.path().join("dest.txt");

    assert!(source.exists(), "Source file should exist");
    assert!(!dest.exists(), "Destination should not exist before copy");

    // Call ruchy::stdlib::fs::copy
    let result = ruchy::stdlib::fs::copy(source.to_str().unwrap(), dest.to_str().unwrap());

    assert!(result.is_ok(), "copy should succeed");
    let bytes_copied = result.unwrap();
    assert!(bytes_copied > 0, "Should have copied some bytes");

    assert!(dest.exists(), "Destination file should exist");
    assert!(source.exists(), "Source should still exist after copy");

    let dest_content = fs::read_to_string(&dest).expect("Failed to read dest file");
    assert_eq!(
        dest_content, "Copy this",
        "Destination content must match source"
    );
    assert_eq!(
        bytes_copied,
        "Copy this".len() as u64,
        "Bytes copied must match content length"
    );
}

#[test]
fn test_std_001_rename_file_success() {
    // STD-001: Test renaming file

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let old_path = create_test_file(&temp_dir, "old_name.txt", "Rename me");
    let new_path = temp_dir.path().join("new_name.txt");

    assert!(old_path.exists(), "Old path should exist before rename");
    assert!(
        !new_path.exists(),
        "New path should not exist before rename"
    );

    // Call ruchy::stdlib::fs::rename
    let result = ruchy::stdlib::fs::rename(old_path.to_str().unwrap(), new_path.to_str().unwrap());

    assert!(result.is_ok(), "rename should succeed");
    assert!(!old_path.exists(), "Old path should not exist");
    assert!(new_path.exists(), "New path should exist");

    // Verify content preserved after rename
    let content = fs::read_to_string(&new_path).expect("Failed to read renamed file");
    assert_eq!(
        content, "Rename me",
        "Content should be preserved after rename"
    );

    // Verify old path cannot be read
    let old_read = fs::read_to_string(&old_path);
    assert!(old_read.is_err(), "Should not be able to read old path");
}

#[test]
fn test_std_001_read_dir_success() {
    // STD-001: Test reading directory contents

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_test_file(&temp_dir, "file1.txt", "1");
    create_test_file(&temp_dir, "file2.txt", "2");
    create_test_file(&temp_dir, "file3.txt", "3");

    // Call ruchy::stdlib::fs::read_dir
    let result = ruchy::stdlib::fs::read_dir(temp_dir.path().to_str().unwrap());

    assert!(result.is_ok(), "read_dir should succeed");

    let entries = result.unwrap();
    assert_eq!(entries.len(), 3, "Should have 3 entries");

    // Verify we got actual entries, not empty list
    assert!(!entries.is_empty(), "Entries should not be empty");

    // Verify entry names
    let mut names: Vec<String> = entries
        .iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["file1.txt", "file2.txt", "file3.txt"],
        "Entry names must match"
    );
}

#[test]
fn test_std_001_metadata_file() {
    // STD-001: Test getting file metadata

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "meta.txt", "metadata");

    // Call ruchy::stdlib::fs::metadata
    let result = ruchy::stdlib::fs::metadata(file_path.to_str().unwrap());

    assert!(result.is_ok(), "metadata should succeed");

    let meta = result.unwrap();
    assert!(meta.is_file(), "Should be a file");
    assert!(!meta.is_dir(), "Should not be a directory");
}

#[test]
fn test_std_001_metadata_directory() {
    // STD-001: Test getting directory metadata

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Call ruchy::stdlib::fs::metadata
    let result = ruchy::stdlib::fs::metadata(temp_dir.path().to_str().unwrap());

    assert!(result.is_ok(), "metadata should succeed");

    let meta = result.unwrap();
    assert!(!meta.is_file(), "Should not be a file");
    assert!(meta.is_dir(), "Should be a directory");
}

#[test]
fn test_std_001_exists_helper() {
    // STD-001: Test exists() helper function

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "exists.txt", "I exist");
    let nonexistent = temp_dir.path().join("nonexistent.txt");

    // Call ruchy::stdlib::fs::exists
    assert!(
        ruchy::stdlib::fs::exists(file_path.to_str().unwrap()),
        "File should exist"
    );
    assert!(
        !ruchy::stdlib::fs::exists(nonexistent.to_str().unwrap()),
        "Nonexistent file should not exist"
    );
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_001_read_write_roundtrip(content in "\\PC{0,1000}") {
            // Property: Writing then reading should return same content

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let file_path = temp_dir.path().join("roundtrip.txt");

            // Write content
            let write_result = ruchy::stdlib::fs::write(
                file_path.to_str().unwrap(),
                &content
            );
            prop_assert!(write_result.is_ok(), "Write should succeed");

            // Read content back
            let read_result = ruchy::stdlib::fs::read_to_string(
                file_path.to_str().unwrap()
            );
            prop_assert!(read_result.is_ok(), "Read should succeed");
            prop_assert_eq!(read_result.unwrap(), content);
        }
    }
}
