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
}
