//! File System Module (STD-001)
//!
//! Thin wrappers around Rust's `std::fs` for Ruchy-friendly API.
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::stdlib::fs;
//!
//! // Read file as string
//! let content = fs::read_to_string("file.txt")?;
//!
//! // Write to file
//! fs::write("output.txt", "Hello, Ruchy!")?;
//!
//! // Create directory
//! fs::create_dir("my_directory")?;
//! # Ok::<(), std::io::Error>(())
//! ```

use anyhow::Result;
use std::path::Path;

/// Read the entire contents of a file into a string
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// let content = fs::read_to_string("file.txt")?;
/// println!("{}", content);
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if file doesn't exist or cannot be read
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn read_to_string(path: &str) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

/// Write a string to a file, creating it if it doesn't exist
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::write("output.txt", "Hello, World!")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if file cannot be written
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn write(path: &str, contents: &str) -> Result<()> {
    Ok(std::fs::write(path, contents)?)
}

/// Read the entire contents of a file into a byte vector
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// let bytes = fs::read("binary.dat")?;
/// println!("Read {} bytes", bytes.len());
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if file doesn't exist or cannot be read
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn read(path: &str) -> Result<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

/// Create a new, empty directory
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::create_dir("new_directory")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if directory already exists or cannot be created
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn create_dir(path: &str) -> Result<()> {
    Ok(std::fs::create_dir(path)?)
}

/// Recursively create a directory and all of its parent components if they don't exist
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::create_dir_all("a/b/c/d")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if directories cannot be created
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn create_dir_all(path: &str) -> Result<()> {
    Ok(std::fs::create_dir_all(path)?)
}

/// Remove a file from the filesystem
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::remove_file("old_file.txt")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if file doesn't exist or cannot be removed
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn remove_file(path: &str) -> Result<()> {
    Ok(std::fs::remove_file(path)?)
}

/// Remove an empty directory
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::remove_dir("empty_directory")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if directory doesn't exist, is not empty, or cannot be removed
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn remove_dir(path: &str) -> Result<()> {
    Ok(std::fs::remove_dir(path)?)
}

/// Copy a file to a new location
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// let bytes_copied = fs::copy("source.txt", "dest.txt")?;
/// println!("Copied {} bytes", bytes_copied);
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if source doesn't exist or destination cannot be written
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn copy(from: &str, to: &str) -> Result<u64> {
    Ok(std::fs::copy(from, to)?)
}

/// Rename a file or directory to a new name
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// fs::rename("old_name.txt", "new_name.txt")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if source doesn't exist or cannot be renamed
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn rename(from: &str, to: &str) -> Result<()> {
    Ok(std::fs::rename(from, to)?)
}

/// Read the entries in a directory
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// for entry in fs::read_dir(".")? {
///     println!("{:?}", entry);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if directory doesn't exist or cannot be read
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn read_dir(path: &str) -> Result<Vec<std::fs::DirEntry>> {
    let entries: Result<Vec<_>, _> = std::fs::read_dir(path)?.collect();
    Ok(entries?)
}

/// Get metadata for a file or directory
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// let meta = fs::metadata("file.txt")?;
/// println!("Is file: {}", meta.is_file());
/// println!("Size: {} bytes", meta.len());
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if path doesn't exist or metadata cannot be read
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn metadata(path: &str) -> Result<std::fs::Metadata> {
    Ok(std::fs::metadata(path)?)
}

/// Check if a path exists
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::fs;
///
/// if fs::exists("file.txt") {
///     println!("File exists!");
/// }
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ============================================================================
    // EXTREME TDD: Comprehensive File System Testing
    // Coverage Target: 27.59% → 80%+
    // Property Target: File operations idempotent/reversible where applicable
    // ============================================================================

    // --------------------------------------------------------------------------
    // read_to_string() + write() tests (round-trip verification)
    // --------------------------------------------------------------------------

    #[test]
    fn test_write_and_read_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let path_str = file_path.to_str().unwrap();

        let content = "Hello, Ruchy!";
        write(path_str, content).unwrap();
        let read_content = read_to_string(path_str).unwrap();

        assert_eq!(content, read_content, "Round-trip should preserve content");
    }

    #[test]
    fn test_write_empty_string() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        let path_str = file_path.to_str().unwrap();

        write(path_str, "").unwrap();
        let content = read_to_string(path_str).unwrap();

        assert_eq!(content, "", "Empty file should read as empty string");
    }

    #[test]
    fn test_write_multiline_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("multiline.txt");
        let path_str = file_path.to_str().unwrap();

        let content = "Line 1\nLine 2\nLine 3";
        write(path_str, content).unwrap();
        let read_content = read_to_string(path_str).unwrap();

        assert_eq!(content, read_content);
    }

    #[test]
    fn test_write_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.txt");
        let path_str = file_path.to_str().unwrap();

        let content = "Hello 世界 🦀";
        write(path_str, content).unwrap();
        let read_content = read_to_string(path_str).unwrap();

        assert_eq!(content, read_content, "Unicode should round-trip");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_to_string("/nonexistent/file.txt");
        assert!(result.is_err(), "Reading nonexistent file should fail");
    }

    #[test]
    fn test_write_to_invalid_path() {
        let result = write("/invalid/\0/path.txt", "content");
        assert!(result.is_err(), "Writing to invalid path should fail");
    }

    // --------------------------------------------------------------------------
    // read() + write() tests (binary data)
    // --------------------------------------------------------------------------

    #[test]
    fn test_read_binary_data() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.dat");
        let path_str = file_path.to_str().unwrap();

        let binary_data = vec![0u8, 1, 2, 255, 128];
        std::fs::write(&file_path, &binary_data).unwrap();

        let read_data = read(path_str).unwrap();
        assert_eq!(binary_data, read_data, "Binary data should round-trip");
    }

    // --------------------------------------------------------------------------
    // create_dir() + remove_dir() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_create_and_remove_dir() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("new_dir");
        let path_str = dir_path.to_str().unwrap();

        create_dir(path_str).unwrap();
        assert!(exists(path_str), "Directory should exist after creation");

        remove_dir(path_str).unwrap();
        assert!(!exists(path_str), "Directory should not exist after removal");
    }

    #[test]
    fn test_create_dir_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("existing_dir");
        let path_str = dir_path.to_str().unwrap();

        create_dir(path_str).unwrap();
        let result = create_dir(path_str);

        assert!(result.is_err(), "Creating existing directory should fail");
    }

    #[test]
    fn test_remove_nonexistent_dir() {
        let result = remove_dir("/nonexistent/directory");
        assert!(result.is_err(), "Removing nonexistent directory should fail");
    }

    // --------------------------------------------------------------------------
    // create_dir_all() tests (recursive creation)
    // --------------------------------------------------------------------------

    #[test]
    fn test_create_dir_all_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a/b/c/d");
        let path_str = nested_path.to_str().unwrap();

        create_dir_all(path_str).unwrap();
        assert!(exists(path_str), "Nested directories should be created");
    }

    #[test]
    fn test_create_dir_all_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("existing");
        let path_str = dir_path.to_str().unwrap();

        create_dir_all(path_str).unwrap();
        // Should not error if directory already exists
        let result = create_dir_all(path_str);
        assert!(result.is_ok(), "create_dir_all should be idempotent");
    }

    // --------------------------------------------------------------------------
    // remove_file() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_remove_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_remove.txt");
        let path_str = file_path.to_str().unwrap();

        write(path_str, "content").unwrap();
        assert!(exists(path_str), "File should exist");

        remove_file(path_str).unwrap();
        assert!(!exists(path_str), "File should not exist after removal");
    }

    #[test]
    fn test_remove_nonexistent_file() {
        let result = remove_file("/nonexistent/file.txt");
        assert!(result.is_err(), "Removing nonexistent file should fail");
    }

    // --------------------------------------------------------------------------
    // copy() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let source_str = source.to_str().unwrap();
        let dest_str = dest.to_str().unwrap();

        let content = "Copy me!";
        write(source_str, content).unwrap();

        let bytes_copied = copy(source_str, dest_str).unwrap();
        assert_eq!(bytes_copied as usize, content.len());

        let dest_content = read_to_string(dest_str).unwrap();
        assert_eq!(content, dest_content, "Copied content should match");
    }

    #[test]
    fn test_copy_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("dest.txt");

        let result = copy("/nonexistent/source.txt", dest.to_str().unwrap());
        assert!(result.is_err(), "Copying nonexistent file should fail");
    }

    // --------------------------------------------------------------------------
    // rename() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old_name.txt");
        let new_path = temp_dir.path().join("new_name.txt");
        let old_str = old_path.to_str().unwrap();
        let new_str = new_path.to_str().unwrap();

        write(old_str, "content").unwrap();
        assert!(exists(old_str));

        rename(old_str, new_str).unwrap();
        assert!(!exists(old_str), "Old path should not exist");
        assert!(exists(new_str), "New path should exist");
    }

    #[test]
    fn test_rename_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let new_path = temp_dir.path().join("new.txt");

        let result = rename("/nonexistent/file.txt", new_path.to_str().unwrap());
        assert!(result.is_err(), "Renaming nonexistent file should fail");
    }

    // --------------------------------------------------------------------------
    // read_dir() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_read_dir_empty() {
        let temp_dir = TempDir::new().unwrap();
        let entries = read_dir(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 0, "Empty directory should have no entries");
    }

    #[test]
    fn test_read_dir_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let path_str = temp_dir.path().to_str().unwrap();

        // Create some files
        write(&format!("{path_str}/file1.txt"), "content1").unwrap();
        write(&format!("{path_str}/file2.txt"), "content2").unwrap();

        let entries = read_dir(path_str).unwrap();
        assert_eq!(entries.len(), 2, "Directory should have 2 entries");
    }

    #[test]
    fn test_read_nonexistent_dir() {
        let result = read_dir("/nonexistent/directory");
        assert!(result.is_err(), "Reading nonexistent directory should fail");
    }

    // --------------------------------------------------------------------------
    // metadata() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_metadata_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("meta.txt");
        let path_str = file_path.to_str().unwrap();

        let content = "test content";
        write(path_str, content).unwrap();

        let meta = metadata(path_str).unwrap();
        assert!(meta.is_file(), "Metadata should indicate file");
        assert_eq!(meta.len(), content.len() as u64, "Size should match");
    }

    #[test]
    fn test_metadata_dir() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("meta_dir");
        let path_str = dir_path.to_str().unwrap();

        create_dir(path_str).unwrap();

        let meta = metadata(path_str).unwrap();
        assert!(meta.is_dir(), "Metadata should indicate directory");
    }

    #[test]
    fn test_metadata_nonexistent() {
        let result = metadata("/nonexistent/path");
        assert!(result.is_err(), "Metadata of nonexistent path should fail");
    }

    // --------------------------------------------------------------------------
    // exists() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_exists_true() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").expect("Failed to write file");

        assert!(exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_exists_false() {
        assert!(!exists("/nonexistent/path/file.txt"));
    }

    #[test]
    fn test_exists_directory() {
        let temp_dir = TempDir::new().unwrap();
        assert!(exists(temp_dir.path().to_str().unwrap()),
                "exists() should return true for directories");
    }
}

// ============================================================================
// Property Tests Module (High-Confidence Verification)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use tempfile::TempDir;

    // Property: Write then read should preserve content (idempotent)
    #[test]
    fn prop_write_read_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let test_cases = ["",
            "Hello",
            "Multi\nLine\nContent",
            "Unicode: 世界🦀",
            "Special: \t\r\n"];

        for (i, content) in test_cases.iter().enumerate() {
            let file_path = temp_dir.path().join(format!("test_{i}.txt"));
            let path_str = file_path.to_str().unwrap();

            write(path_str, content).unwrap();
            let read_back = read_to_string(path_str).unwrap();

            assert_eq!(*content, read_back,
                       "Round-trip should preserve content: {content:?}");
        }
    }

    // Property: Copy creates identical file
    #[test]
    fn prop_copy_creates_identical_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let source_str = source.to_str().unwrap();
        let dest_str = dest.to_str().unwrap();

        let content = "Original content";
        write(source_str, content).unwrap();

        copy(source_str, dest_str).unwrap();

        let source_content = read_to_string(source_str).unwrap();
        let dest_content = read_to_string(dest_str).unwrap();

        assert_eq!(source_content, dest_content,
                   "Copied file should have identical content");
    }

    // Property: rename is a move (source disappears, dest appears)
    #[test]
    fn prop_rename_is_move_operation() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.txt");
        let new_path = temp_dir.path().join("new.txt");
        let old_str = old_path.to_str().unwrap();
        let new_str = new_path.to_str().unwrap();

        write(old_str, "content").unwrap();
        assert!(exists(old_str), "Source should exist before rename");

        rename(old_str, new_str).unwrap();

        assert!(!exists(old_str), "Source should not exist after rename");
        assert!(exists(new_str), "Destination should exist after rename");
    }

    // Property: create_dir_all is idempotent
    #[test]
    fn prop_create_dir_all_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("a/b/c");
        let path_str = dir_path.to_str().unwrap();

        create_dir_all(path_str).unwrap();
        assert!(exists(path_str));

        // Second call should succeed (idempotent)
        let result = create_dir_all(path_str);
        assert!(result.is_ok(), "create_dir_all should be idempotent");
        assert!(exists(path_str));
    }

    // Property: File operations never panic on invalid paths
    #[test]
    fn prop_file_ops_never_panic_on_invalid_paths() {
        let invalid_paths = vec![
            "",
            "\0",
            "/invalid/\0/path",
            "/nonexistent/deep/nested/path/file.txt",
        ];

        for path in invalid_paths {
            // All operations should return Err, not panic
            let _ = read_to_string(path);
            let _ = read(path);
            let _ = write(path, "content");
            let _ = create_dir(path);
            let _ = create_dir_all(path);
            let _ = remove_file(path);
            let _ = remove_dir(path);
            let _ = metadata(path);
            let _ = read_dir(path);
            // exists() returns bool, not Result
            let _ = exists(path);
        }
        // If we reach here, no panic occurred
    }

    // Property: exists() consistency with metadata()
    #[test]
    fn prop_exists_consistent_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("consistency.txt");
        let path_str = file_path.to_str().unwrap();

        // Non-existent path
        assert_eq!(exists(path_str), metadata(path_str).is_ok(),
                   "exists() should match metadata() success");

        // Create file
        write(path_str, "content").unwrap();
        assert_eq!(exists(path_str), metadata(path_str).is_ok(),
                   "exists() should match metadata() success after creation");
    }
}
