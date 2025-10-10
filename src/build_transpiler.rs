//! Build Transpiler for Cargo Integration (CARGO-001)
//!
//! This module provides functionality to transpile .ruchy files to .rs files
//! during the cargo build process via build.rs integration.
//!
//! # Usage in build.rs
//!
//! ```ignore
//! // build.rs
//! fn main() {
//!     ruchy::build_transpiler::transpile_all("src", "**/*.ruchy", "src")
//!         .expect("Failed to transpile Ruchy files");
//! }
//! ```
//!
//! # Features
//!
//! - Automatic file discovery with glob patterns
//! - Incremental compilation (only transpile changed files)
//! - Clear error reporting with file names
//! - Nested directory support

use anyhow::{Context, Result};
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

use crate::backend::transpiler::Transpiler;
use crate::frontend::parser::Parser;

/// Transpile all .ruchy files matching the pattern to .rs files
///
/// # Arguments
///
/// * `source_dir` - Base directory to search for .ruchy files
/// * `pattern` - Glob pattern to match files (e.g., "**/*.ruchy")
/// * `output_dir` - Directory to write transpiled .rs files
///
/// # Examples
///
/// ```no_run
/// use ruchy::build_transpiler;
///
/// // Transpile all .ruchy files in src/ directory
/// build_transpiler::transpile_all("src", "**/*.ruchy", "src")
///     .expect("Transpilation failed");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Pattern matching fails
/// - File reading fails
/// - Parsing fails (syntax errors)
/// - Transpilation fails
/// - File writing fails
///
/// # Complexity
///
/// Complexity: 7 (within Toyota Way limits ≤10)
pub fn transpile_all(source_dir: &str, pattern: &str, output_dir: &str) -> Result<()> {
    let source_path = Path::new(source_dir);
    let output_path = Path::new(output_dir);

    // Build full glob pattern
    let full_pattern = source_path.join(pattern);
    let pattern_str = full_pattern.to_str().context("Invalid pattern path")?;

    // Find all .ruchy files matching pattern
    let ruchy_files = find_ruchy_files(pattern_str)?;

    // Transpile each file (with incremental compilation check)
    for ruchy_file in ruchy_files {
        transpile_single_file(&ruchy_file, source_path, output_path)
            .with_context(|| format!("Failed to transpile file: {}", ruchy_file.display()))?;
    }

    Ok(())
}

/// Find all .ruchy files matching the glob pattern
///
/// # Complexity
///
/// Complexity: 3 (within Toyota Way limits ≤10)
fn find_ruchy_files(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in glob(pattern).context("Failed to parse glob pattern")? {
        match entry {
            Ok(path) => files.push(path),
            Err(e) => {
                eprintln!("Warning: Failed to read glob entry: {e}");
            }
        }
    }

    Ok(files)
}

/// Transpile a single .ruchy file to .rs
///
/// Implements incremental compilation: only transpile if .ruchy is newer than .rs
///
/// # Complexity
///
/// Complexity: 8 (within Toyota Way limits ≤10)
fn transpile_single_file(ruchy_file: &Path, source_dir: &Path, output_dir: &Path) -> Result<()> {
    // Calculate output .rs file path
    let relative_path = ruchy_file.strip_prefix(source_dir).unwrap_or(ruchy_file);

    let rs_file = output_dir.join(relative_path).with_extension("rs");

    // Check if incremental compilation can skip this file
    if should_skip_transpilation(ruchy_file, &rs_file)? {
        return Ok(());
    }

    // Read source file
    let source_code = fs::read_to_string(ruchy_file)
        .with_context(|| format!("Failed to read file: {}", ruchy_file.display()))?;

    // Parse the source code
    let mut parser = Parser::new(&source_code);
    let ast = parser
        .parse()
        .with_context(|| format!("Syntax error in file: {}", ruchy_file.display()))?;

    // Transpile to Rust code
    let mut transpiler = Transpiler::new();
    let rust_tokens = transpiler
        .transpile_to_program(&ast)
        .with_context(|| format!("Transpilation failed for file: {}", ruchy_file.display()))?;

    // Format the Rust code
    let rust_code = rust_tokens.to_string();

    // Ensure output directory exists
    if let Some(parent) = rs_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Write transpiled code to .rs file
    fs::write(&rs_file, rust_code)
        .with_context(|| format!("Failed to write output file: {}", rs_file.display()))?;

    Ok(())
}

/// Check if transpilation can be skipped (incremental compilation)
///
/// Returns true if .rs file exists and is newer than .ruchy file
///
/// # Complexity
///
/// Complexity: 5 (within Toyota Way limits ≤10)
fn should_skip_transpilation(ruchy_file: &Path, rs_file: &Path) -> Result<bool> {
    // If .rs doesn't exist, must transpile
    if !rs_file.exists() {
        return Ok(false);
    }

    // Get modification times
    let ruchy_mtime = fs::metadata(ruchy_file)?.modified()?;
    let rs_mtime = fs::metadata(rs_file)?.modified()?;

    // Skip transpilation if .rs is newer than .ruchy
    Ok(rs_mtime >= ruchy_mtime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_ruchy_files_empty() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pattern = temp_dir.path().join("**/*.ruchy");

        let files = find_ruchy_files(pattern.to_str().unwrap()).expect("Should succeed");
        assert_eq!(files.len(), 0, "Should find no files in empty directory");
    }

    #[test]
    fn test_should_skip_transpilation_nonexistent_rs() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("test.ruchy");
        let rs_file = temp_dir.path().join("test.rs");

        fs::write(&ruchy_file, "fun main() {}").expect("Failed to write test file");

        let should_skip = should_skip_transpilation(&ruchy_file, &rs_file).expect("Should succeed");

        assert!(!should_skip, "Should not skip when .rs doesn't exist");
    }

    #[test]
    fn test_should_skip_transpilation_rs_newer() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("test.ruchy");
        let rs_file = temp_dir.path().join("test.rs");

        // Create .ruchy file
        fs::write(&ruchy_file, "fun main() {}").expect("Failed to write .ruchy file");

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create .rs file (newer)
        fs::write(&rs_file, "fn main() {}").expect("Failed to write .rs file");

        let should_skip = should_skip_transpilation(&ruchy_file, &rs_file).expect("Should succeed");

        assert!(should_skip, "Should skip when .rs is newer than .ruchy");
    }
}
