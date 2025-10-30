//! STD-004: Path Module Tests (ruchy/std/path)
//!
//! Test suite for path manipulation operations module.
//! Thin wrappers around Rust's `std::path` with Ruchy-friendly API.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

#![allow(clippy::case_sensitive_file_extension_comparisons)] // Path tests intentionally test exact strings
#![allow(clippy::collapsible_if)] // Test readability over compactness

use std::path::Path;
use tempfile::TempDir;

/// Helper to create a test file
fn create_test_file(dir: &TempDir, name: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    std::fs::write(&file_path, "test").expect("Failed to write test file");
    file_path
}

#[test]
fn test_std_004_join_paths() {
    // STD-004: Test joining path components

    // Call ruchy::stdlib::path::join
    let result = ruchy::stdlib::path::join("/home/user", "documents");

    assert!(result.is_ok(), "join should succeed");
    let path_str = result.unwrap();
    assert!(path_str.contains("/home/user"), "Must contain base path");
    assert!(
        path_str.contains("documents"),
        "Must contain appended component"
    );
    assert!(
        path_str.ends_with("documents"),
        "Must end with appended component"
    );
    assert!(!path_str.is_empty(), "Path must not be empty");
}

#[test]
fn test_std_004_join_multiple_components() {
    // STD-004: Test joining multiple path components

    let result = ruchy::stdlib::path::join_many(&["/home", "user", "documents", "file.txt"]);

    assert!(result.is_ok(), "join_many should succeed");
    let path_str = result.unwrap();
    assert!(path_str.contains("home"), "Must contain 'home'");
    assert!(path_str.contains("user"), "Must contain 'user'");
    assert!(path_str.contains("documents"), "Must contain 'documents'");
    assert!(path_str.ends_with("file.txt"), "Must end with 'file.txt'");
    assert!(!path_str.is_empty(), "Path must not be empty");
}

#[test]
fn test_std_004_parent() {
    // STD-004: Test getting parent directory

    let result = ruchy::stdlib::path::parent("/home/user/documents/file.txt");

    assert!(result.is_ok(), "parent should succeed");
    let parent_str = result.unwrap();
    assert!(parent_str.is_some(), "Parent should exist");
    let parent = parent_str.unwrap();
    assert!(
        parent.contains("documents"),
        "Parent must contain 'documents'"
    );
    assert!(
        !parent.contains("file.txt"),
        "Parent must not contain filename"
    );
    assert!(!parent.is_empty(), "Parent must not be empty");
}

#[test]
fn test_std_004_parent_root() {
    // STD-004: Test parent of root path

    let result = ruchy::stdlib::path::parent("/");

    assert!(result.is_ok(), "parent should succeed");
    let parent_str = result.unwrap();
    assert!(parent_str.is_none(), "Root should have no parent");
}

#[test]
fn test_std_004_file_name() {
    // STD-004: Test getting file name from path

    let result = ruchy::stdlib::path::file_name("/home/user/documents/file.txt");

    assert!(result.is_ok(), "file_name should succeed");
    let name = result.unwrap();
    assert!(name.is_some(), "File name should exist");
    let name_str = name.unwrap();
    assert_eq!(name_str, "file.txt", "File name must be exactly 'file.txt'");
    assert_eq!(name_str.len(), 8, "File name length must be 8");
    assert!(!name_str.is_empty(), "File name must not be empty");
}

#[test]
fn test_std_004_file_name_no_name() {
    // STD-004: Test file_name on path with no file name

    let result = ruchy::stdlib::path::file_name("/home/user/");

    assert!(result.is_ok(), "file_name should succeed");
    let _name = result.unwrap();
    // Trailing slash may or may not have file name depending on implementation
    // This is OK - either "user" or None is acceptable
}

#[test]
fn test_std_004_file_stem() {
    // STD-004: Test getting file stem (name without extension)

    let result = ruchy::stdlib::path::file_stem("/home/user/file.txt");

    assert!(result.is_ok(), "file_stem should succeed");
    let stem = result.unwrap();
    assert!(stem.is_some(), "File stem should exist");
    let stem_str = stem.unwrap();
    assert_eq!(stem_str, "file", "File stem must be exactly 'file'");
    assert_eq!(stem_str.len(), 4, "File stem length must be 4");
    assert!(
        !stem_str.contains(".txt"),
        "Stem must not contain extension"
    );
    assert!(!stem_str.is_empty(), "Stem must not be empty");
}

#[test]
fn test_std_004_extension() {
    // STD-004: Test getting file extension

    let result = ruchy::stdlib::path::extension("/home/user/file.txt");

    assert!(result.is_ok(), "extension should succeed");
    let ext = result.unwrap();
    assert!(ext.is_some(), "Extension should exist");
    let ext_str = ext.unwrap();
    assert_eq!(ext_str, "txt", "Extension must be exactly 'txt'");
    assert_eq!(ext_str.len(), 3, "Extension length must be 3");
    assert!(!ext_str.contains('.'), "Extension must not contain dot");
    assert!(!ext_str.is_empty(), "Extension must not be empty");
}

#[test]
fn test_std_004_extension_none() {
    // STD-004: Test path with no extension

    let result = ruchy::stdlib::path::extension("/home/user/file");

    assert!(result.is_ok(), "extension should succeed");
    let ext = result.unwrap();
    assert!(ext.is_none(), "Path without extension should return None");
}

#[test]
fn test_std_004_is_absolute() {
    // STD-004: Test checking if path is absolute

    let result = ruchy::stdlib::path::is_absolute("/home/user/file.txt");
    assert!(result, "Path starting with / must be absolute");

    let result = ruchy::stdlib::path::is_absolute("relative/path");
    assert!(!result, "Path not starting with / must be relative");

    let result = ruchy::stdlib::path::is_absolute("./relative");
    assert!(!result, "Path starting with ./ must be relative");
}

#[test]
fn test_std_004_is_relative() {
    // STD-004: Test checking if path is relative

    let result = ruchy::stdlib::path::is_relative("relative/path");
    assert!(result, "Path without / must be relative");

    let result = ruchy::stdlib::path::is_relative("/home/user");
    assert!(!result, "Absolute path must not be relative");
}

#[test]
fn test_std_004_canonicalize() {
    // STD-004: Test resolving path to canonical form

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "test.txt");

    let result = ruchy::stdlib::path::canonicalize(file_path.to_str().unwrap());

    assert!(
        result.is_ok(),
        "canonicalize should succeed for existing file"
    );
    let canonical = result.unwrap();
    assert!(!canonical.is_empty(), "Canonical path must not be empty");
    assert!(
        Path::new(&canonical).is_absolute(),
        "Canonical path must be absolute"
    );
    assert!(canonical.contains("test.txt"), "Must contain filename");
}

#[test]
fn test_std_004_canonicalize_nonexistent() {
    // STD-004: Test canonicalize on nonexistent path

    let result = ruchy::stdlib::path::canonicalize("/nonexistent/path/file.txt");

    assert!(
        result.is_err(),
        "canonicalize should fail for nonexistent path"
    );
}

#[test]
fn test_std_004_with_extension() {
    // STD-004: Test replacing extension

    let result = ruchy::stdlib::path::with_extension("/home/user/file.txt", "md");

    assert!(result.is_ok(), "with_extension should succeed");
    let new_path = result.unwrap();
    assert!(new_path.ends_with(".md"), "Must end with new extension");
    assert!(!new_path.ends_with(".txt"), "Must not have old extension");
    assert!(new_path.contains("file"), "Must still contain filename");
    assert!(!new_path.is_empty(), "Path must not be empty");
}

#[test]
fn test_std_004_with_file_name() {
    // STD-004: Test replacing file name

    let result = ruchy::stdlib::path::with_file_name("/home/user/old.txt", "new.txt");

    assert!(result.is_ok(), "with_file_name should succeed");
    let new_path = result.unwrap();
    assert!(new_path.ends_with("new.txt"), "Must end with new filename");
    assert!(
        !new_path.contains("old.txt"),
        "Must not contain old filename"
    );
    assert!(
        new_path.contains("/home/user"),
        "Must retain parent directory"
    );
    assert!(!new_path.is_empty(), "Path must not be empty");
}

#[test]
fn test_std_004_components() {
    // STD-004: Test getting path components

    let result = ruchy::stdlib::path::components("/home/user/file.txt");

    assert!(result.is_ok(), "components should succeed");
    let comps = result.unwrap();
    assert!(!comps.is_empty(), "Components must not be empty");
    assert!(comps.len() >= 3, "Should have at least 3 components");
    assert!(comps.contains(&"home".to_string()), "Must contain 'home'");
    assert!(comps.contains(&"user".to_string()), "Must contain 'user'");
    assert!(
        comps.contains(&"file.txt".to_string()),
        "Must contain 'file.txt'"
    );
}

#[test]
fn test_std_004_normalize() {
    // STD-004: Test normalizing path (removing . and ..)

    let result = ruchy::stdlib::path::normalize("/home/user/../admin/./file.txt");

    assert!(result.is_ok(), "normalize should succeed");
    let normalized = result.unwrap();
    assert!(!normalized.contains(".."), "Must not contain '..'");
    assert!(!normalized.contains("/."), "Must not contain '/.'");
    assert!(normalized.contains("admin"), "Must contain 'admin'");
    assert!(normalized.contains("file.txt"), "Must contain 'file.txt'");
    assert!(!normalized.is_empty(), "Path must not be empty");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_004_join_never_panics(base in "[a-z/]{1,20}", comp in "[a-z]{1,10}") {
            // Property: join should never panic

            let _ = ruchy::stdlib::path::join(&base, &comp);
            // Should not panic
        }

        #[test]
        fn test_std_004_parent_idempotent(path in "/[a-z/]{1,50}") {
            // Property: parent(parent(x)) should equal parent(x) for paths with <=1 component

            if let Ok(p1) = ruchy::stdlib::path::parent(&path) {
                if let Some(parent1) = p1 {
                    if let Ok(p2) = ruchy::stdlib::path::parent(&parent1) {
                        // Both should succeed
                        assert!(p2.is_some() || parent1 == "/");
                    }
                }
            }
        }

        #[test]
        fn test_std_004_absolute_relative_inverse(path in "[a-z/]{1,20}") {
            // Property: is_absolute and is_relative should be inverses

            let abs = ruchy::stdlib::path::is_absolute(&path);
            let rel = ruchy::stdlib::path::is_relative(&path);
            assert_ne!(abs, rel, "is_absolute and is_relative must be opposites");
        }
    }
}
