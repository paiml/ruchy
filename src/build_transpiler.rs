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

    // Format the Rust code with prettyplease for proper multi-line output
    // TRANSPILER-DEFECT-009: build_transpiler was outputting single-line code
    // Solution: Use prettyplease like the CLI does
    let syntax_tree: syn::File = syn::parse2(rust_tokens).with_context(|| {
        format!(
            "Failed to parse generated tokens as Rust syntax for file: {}",
            ruchy_file.display()
        )
    })?;
    let rust_code = prettyplease::unparse(&syntax_tree);

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
    fn test_find_ruchy_files_with_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("test.ruchy");
        fs::write(&ruchy_file, "fun main() {}").expect("Failed to write file");

        let pattern = temp_dir.path().join("*.ruchy");
        let files = find_ruchy_files(pattern.to_str().unwrap()).expect("Should succeed");
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_find_ruchy_files_nested() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let nested = temp_dir.path().join("nested");
        fs::create_dir(&nested).expect("Failed to create dir");
        fs::write(nested.join("test.ruchy"), "42").expect("Failed to write file");

        let pattern = temp_dir.path().join("**/*.ruchy");
        let files = find_ruchy_files(pattern.to_str().unwrap()).expect("Should succeed");
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_transpile_all_empty_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = transpile_all(
            temp_dir.path().to_str().unwrap(),
            "**/*.ruchy",
            temp_dir.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_single_file_creates_rs() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("test.ruchy");
        fs::write(&ruchy_file, "fun main() { println!(\"Hello\"); }").expect("write failed");

        transpile_single_file(&ruchy_file, temp_dir.path(), temp_dir.path())
            .expect("transpile should succeed");

        let rs_file = temp_dir.path().join("test.rs");
        assert!(rs_file.exists(), ".rs file should be created");
    }

    #[test]
    fn test_transpile_single_file_creates_nested_dirs() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let src_dir = temp_dir.path().join("src");
        let nested = src_dir.join("a").join("b");
        fs::create_dir_all(&nested).expect("Failed to create nested dirs");
        let ruchy_file = nested.join("deep.ruchy");
        fs::write(&ruchy_file, "42").expect("write failed");

        let out_dir = temp_dir.path().join("out");
        transpile_single_file(&ruchy_file, &src_dir, &out_dir).expect("transpile should succeed");

        let rs_file = out_dir.join("a").join("b").join("deep.rs");
        assert!(rs_file.exists(), ".rs file should be created in nested dir");
    }

    #[test]
    fn test_should_skip_transpilation_ruchy_newer() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let rs_file = temp_dir.path().join("test.rs");
        fs::write(&rs_file, "fn main() {}").expect("write failed");

        std::thread::sleep(std::time::Duration::from_millis(10));

        let ruchy_file = temp_dir.path().join("test.ruchy");
        fs::write(&ruchy_file, "fun main() {}").expect("write failed");

        let should_skip = should_skip_transpilation(&ruchy_file, &rs_file).expect("should succeed");
        assert!(!should_skip, "Should NOT skip when .ruchy is newer");
    }

    #[test]
    fn test_transpile_all_with_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("hello.ruchy");
        fs::write(&ruchy_file, "42").expect("write failed");

        let result = transpile_all(
            temp_dir.path().to_str().unwrap(),
            "*.ruchy",
            temp_dir.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(temp_dir.path().join("hello.rs").exists());
    }

    #[test]
    fn test_find_ruchy_files_multiple() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(temp_dir.path().join("a.ruchy"), "1").expect("write failed");
        fs::write(temp_dir.path().join("b.ruchy"), "2").expect("write failed");
        fs::write(temp_dir.path().join("c.ruchy"), "3").expect("write failed");

        let pattern = temp_dir.path().join("*.ruchy");
        let files = find_ruchy_files(pattern.to_str().unwrap()).expect("Should succeed");
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_transpile_single_file_simple_expression() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("expr.ruchy");
        fs::write(&ruchy_file, "1 + 2 * 3").expect("write failed");

        transpile_single_file(&ruchy_file, temp_dir.path(), temp_dir.path())
            .expect("transpile should succeed");

        let rs_file = temp_dir.path().join("expr.rs");
        assert!(rs_file.exists());
        let content = fs::read_to_string(&rs_file).expect("read failed");
        assert!(!content.is_empty());
    }

    #[test]
    fn test_find_ruchy_files_ignores_other_extensions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(temp_dir.path().join("test.ruchy"), "1").expect("write failed");
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").expect("write failed");
        fs::write(temp_dir.path().join("test.txt"), "text").expect("write failed");

        let pattern = temp_dir.path().join("*.ruchy");
        let files = find_ruchy_files(pattern.to_str().unwrap()).expect("Should succeed");
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_transpile_all_multiple_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(temp_dir.path().join("a.ruchy"), "1").expect("write failed");
        fs::write(temp_dir.path().join("b.ruchy"), "2").expect("write failed");

        let result = transpile_all(
            temp_dir.path().to_str().unwrap(),
            "*.ruchy",
            temp_dir.path().to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(temp_dir.path().join("a.rs").exists());
        assert!(temp_dir.path().join("b.rs").exists());
    }

    #[test]
    fn test_transpile_single_file_overwrites_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("test.ruchy");
        let rs_file = temp_dir.path().join("test.rs");

        // Create initial .rs file
        fs::write(&rs_file, "old content").expect("write failed");
        // Create .ruchy file (newer)
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&ruchy_file, "42").expect("write failed");

        transpile_single_file(&ruchy_file, temp_dir.path(), temp_dir.path())
            .expect("transpile should succeed");

        let content = fs::read_to_string(&rs_file).expect("read failed");
        assert_ne!(content, "old content");
    }

    #[test]
    fn test_transpile_single_file_with_function() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("func.ruchy");
        fs::write(&ruchy_file, "fun add(a: i32, b: i32) -> i32 { a + b }").expect("write failed");

        transpile_single_file(&ruchy_file, temp_dir.path(), temp_dir.path())
            .expect("transpile should succeed");

        let rs_file = temp_dir.path().join("func.rs");
        assert!(rs_file.exists());
    }

    #[test]
    fn test_transpile_single_file_with_struct() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ruchy_file = temp_dir.path().join("point.ruchy");
        fs::write(&ruchy_file, "struct Point { x: i32, y: i32 }").expect("write failed");

        transpile_single_file(&ruchy_file, temp_dir.path(), temp_dir.path())
            .expect("transpile should succeed");

        assert!(temp_dir.path().join("point.rs").exists());
    }

    #[test]
    fn test_find_ruchy_files_empty() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pattern = temp_dir.path().join("**/*.ruchy");

        let files = find_ruchy_files(pattern.to_str().expect("operation should succeed in test"))
            .expect("Should succeed");
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

    #[test]
    fn test_transpiler_defect_009_formatted_output() {
        // TRANSPILER-DEFECT-009: build_transpiler should output formatted multi-line code
        // RED phase test - this will fail until we add prettyplease formatting

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src dir");

        // Create test file with enum and struct
        let ruchy_file = src_dir.join("test.ruchy");
        fs::write(
            &ruchy_file,
            r#"
enum Priority {
    High,
    Medium,
    Low,
}

struct Task {
    name: String,
    priority: Priority,
}

fun main() {
    println!("Test");
}
"#,
        )
        .expect("Failed to write test file");

        // Transpile
        transpile_all(
            src_dir.to_str().expect("operation should succeed in test"),
            "**/*.ruchy",
            src_dir.to_str().expect("operation should succeed in test"),
        )
        .expect("Transpilation should succeed");

        // Read generated .rs file
        let rs_file = src_dir.join("test.rs");
        let generated_code = fs::read_to_string(&rs_file).expect("Failed to read generated file");

        // Verify: Should NOT be single-line
        let line_count = generated_code.lines().count();
        assert!(
            line_count > 5,
            "Generated code should be multi-line (got {line_count} lines), not single-line"
        );

        // Verify: Enum should appear at top
        let first_100_chars = &generated_code[..100.min(generated_code.len())];
        assert!(
            first_100_chars.contains("enum") || generated_code.lines().next().unwrap_or("").contains("#[derive"),
            "Enum declaration should appear near the top of file (first 100 chars: '{first_100_chars}')"
        );

        // Verify: Should be properly formatted (check for newlines after braces)
        assert!(
            generated_code.contains("}\n") || generated_code.contains("}\r\n"),
            "Code should have newlines after closing braces (proper formatting)"
        );
    }

    // Property-based tests for EXTREME TDD
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn property_transpiled_code_always_multiline(
            enum_count in 1usize..5,
            struct_count in 0usize..5
        ) {
            // Property: ALL transpiled code must be multi-line (not single-line)
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let src_dir = temp_dir.path().join("src");
            fs::create_dir(&src_dir).expect("Failed to create src dir");

            // Generate test file with varying enums/structs
            let mut code = String::new();
            for i in 0..enum_count {
                code.push_str(&format!("enum Enum{i} {{ Variant1, Variant2 }}\n"));
            }
            for i in 0..struct_count {
                code.push_str(&format!("struct Struct{i} {{ field: i32 }}\n"));
            }
            code.push_str("fun main() { println!(\"Test\"); }");

            let ruchy_file = src_dir.join("test.ruchy");
            fs::write(&ruchy_file, &code).expect("Failed to write test file");

            // Transpile
            transpile_all(
                src_dir.to_str().expect("operation should succeed in test"),
                "**/*.ruchy",
                src_dir.to_str().expect("operation should succeed in test"),
            )
            .expect("Transpilation should succeed");

            // Verify property: MUST be multi-line
            let rs_file = src_dir.join("test.rs");
            let generated_code = fs::read_to_string(&rs_file).expect("Failed to read generated file");
            let line_count = generated_code.lines().count();

            prop_assert!(
                line_count > 5,
                "Generated code MUST be multi-line (got {} lines)",
                line_count
            );

            // Property: MUST have newlines (not single-line)
            prop_assert!(
                generated_code.contains('\n'),
                "Generated code MUST contain newlines"
            );
        }

        #[test]
        fn property_enums_always_at_top(
            enum_name in "[A-Z][a-z]{2,8}",
            variant_count in 1usize..5
        ) {
            // Property: Enum declarations ALWAYS appear before main()
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let src_dir = temp_dir.path().join("src");
            fs::create_dir(&src_dir).expect("Failed to create src dir");

            // Generate enum
            let mut code = format!("enum {enum_name} {{\n");
            for i in 0..variant_count {
                code.push_str(&format!("    Variant{i},\n"));
            }
            code.push_str("}\n\nfun main() { println!(\"Test\"); }");

            let ruchy_file = src_dir.join("test.ruchy");
            fs::write(&ruchy_file, &code).expect("Failed to write test file");

            // Transpile
            transpile_all(
                src_dir.to_str().expect("operation should succeed in test"),
                "**/*.ruchy",
                src_dir.to_str().expect("operation should succeed in test"),
            )
            .expect("Transpilation should succeed");

            // Verify property: enum BEFORE main()
            let rs_file = src_dir.join("test.rs");
            let generated_code = fs::read_to_string(&rs_file).expect("Failed to read generated file");

            let enum_pos = generated_code.find(&format!("enum {enum_name}"));
            let main_pos = generated_code.find("fn main()");

            prop_assert!(
                enum_pos.is_some() && main_pos.is_some(),
                "Both enum and main() must exist in generated code"
            );

            let enum_idx = enum_pos.expect("operation should succeed in test");
            let main_idx = main_pos.expect("operation should succeed in test");

            prop_assert!(
                enum_idx < main_idx,
                "Enum declaration MUST appear before main() (enum at {}, main at {})",
                enum_idx,
                main_idx
            );
        }
    }
}
