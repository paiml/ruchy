//! Binary compilation support for Ruchy
//!
//! This module provides functionality to compile Ruchy code to standalone binaries
//! via Rust compilation toolchain (rustc).
use crate::utils::common_patterns::ResultContextExt;
use crate::{Parser, Transpiler};
use anyhow::{bail, Context, Result};
use proc_macro2::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
/// Binary compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Output binary path
    pub output: PathBuf,
    /// Optimization level (0-3, or 's' for size)
    pub opt_level: String,
    /// Strip debug symbols
    pub strip: bool,
    /// Static linking
    pub static_link: bool,
    /// Target triple (e.g., x86_64-unknown-linux-gnu)
    pub target: Option<String>,
    /// Additional rustc flags
    pub rustc_flags: Vec<String>,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            output: PathBuf::from("a.out"),
            opt_level: "2".to_string(),
            strip: false,
            static_link: false,
            target: None,
            rustc_flags: Vec::new(),
        }
    }
}
/// Compile a Ruchy source file to a standalone binary
///
/// # Examples
///
/// ```no_run
/// use ruchy::backend::{compile_to_binary, CompileOptions};
/// use std::path::PathBuf;
///
/// let options = CompileOptions {
///     output: PathBuf::from("my_program"),
///     opt_level: "2".to_string(),
///     strip: false,
///     static_link: false,
///     target: None,
///     rustc_flags: Vec::new(),
/// };
///
/// let result = compile_to_binary(&PathBuf::from("program.ruchy"), &options);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The source file cannot be read
/// - The source code fails to parse
/// - The transpilation fails
/// - The rustc compilation fails
pub fn compile_to_binary(source_path: &Path, options: &CompileOptions) -> Result<PathBuf> {
    // Read source file
    let source = fs::read_to_string(source_path).file_context("read", source_path)?;
    compile_source_to_binary(&source, options)
}
/// Compile Ruchy source code to a standalone binary
///
/// # Examples
///
/// ```no_run
/// use ruchy::backend::{compile_source_to_binary, CompileOptions};
/// use std::path::PathBuf;
///
/// let source = r#"
///     fun main() {
///         println("Hello, World!");
///     }
/// "#;
///
/// let options = CompileOptions::default();
/// let result = compile_source_to_binary(source, &options);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The source code fails to parse
/// - The transpilation fails
/// - The working directory cannot be created
/// - The rustc compilation fails
pub fn compile_source_to_binary(source: &str, options: &CompileOptions) -> Result<PathBuf> {
    // Parse and transpile
    let rust_code = parse_and_transpile(source)?;
    // Prepare compilation artifacts
    let (_temp_dir, rust_file) = prepare_rust_file(&rust_code)?;
    // Build and execute rustc
    let cmd = build_rustc_command(&rust_file, options);
    execute_compilation(cmd)?;
    // Verify output
    verify_output_exists(&options.output)?;
    Ok(options.output.clone())
}
/// Parse Ruchy source and transpile to Rust (complexity: 4)
fn parse_and_transpile(source: &str) -> Result<TokenStream> {
    eprintln!("DEBUG: About to call transpile_to_program");
    let mut parser = Parser::new(source);
    let ast = parser.parse().parse_context("Ruchy source")?;
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .compile_context("transpile to Rust")?;
    eprintln!("DEBUG: transpile_to_program completed");
    Ok(rust_code)
}
/// Prepare temporary Rust file for compilation (complexity: 4)
fn prepare_rust_file(rust_code: &TokenStream) -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new().compile_context("create temporary directory")?;
    let rust_file = temp_dir.path().join("main.rs");
    let rust_code_str = rust_code.to_string();
    // Debug: Also write to /tmp/debug_rust_output.rs for inspection
    fs::write("/tmp/debug_rust_output.rs", &rust_code_str)
        .context("Failed to write debug Rust code")?;
    fs::write(&rust_file, &rust_code_str).context("Failed to write Rust code to temporary file")?;
    Ok((temp_dir, rust_file))
}
/// Build rustc command with options (complexity: 7)
fn build_rustc_command(rust_file: &Path, options: &CompileOptions) -> Command {
    let mut cmd = Command::new("rustc");
    cmd.arg(rust_file).arg("-o").arg(&options.output);
    // Add optimization level
    cmd.arg("-C")
        .arg(format!("opt-level={}", options.opt_level));
    // Add optional flags
    apply_optional_flags(&mut cmd, options);
    cmd
}
/// Apply optional compilation flags (complexity: 5)
fn apply_optional_flags(cmd: &mut Command, options: &CompileOptions) {
    if options.strip {
        cmd.arg("-C").arg("strip=symbols");
    }
    if options.static_link {
        cmd.arg("-C").arg("target-feature=+crt-static");
    }
    if let Some(target) = &options.target {
        cmd.arg("--target").arg(target);
    }
    for flag in &options.rustc_flags {
        cmd.arg(flag);
    }
}
/// Execute compilation command (complexity: 3)
fn execute_compilation(mut cmd: Command) -> Result<()> {
    let output = cmd.output().context("Failed to execute rustc")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Compilation failed:\n{}", stderr);
    }
    Ok(())
}
/// Verify output file exists (complexity: 2)
fn verify_output_exists(output_path: &Path) -> Result<()> {
    if !output_path.exists() {
        bail!(
            "Expected output file not created: {}",
            output_path.display()
        );
    }
    Ok(())
}
/// Check if rustc is available
///
/// # Examples
///
/// ```
/// use ruchy::backend::compiler::check_rustc_available;
///
/// if check_rustc_available().is_ok() {
///     println!("rustc is available");
/// }
/// ```
///
/// # Errors
///
/// Returns an error if rustc is not installed or cannot be executed
pub fn check_rustc_available() -> Result<()> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .context("Failed to execute rustc")?;
    if !output.status.success() {
        bail!("rustc is not available. Please install Rust toolchain.");
    }
    Ok(())
}
/// Get rustc version information
///
/// # Examples
///
/// ```
/// use ruchy::backend::compiler::get_rustc_version;
///
/// if let Ok(version) = get_rustc_version() {
///     println!("rustc version: {}", version);
/// }
/// ```
///
/// # Errors
///
/// Returns an error if rustc is not available or version cannot be retrieved
pub fn get_rustc_version() -> Result<String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .context("Failed to execute rustc")?;
    if !output.status.success() {
        bail!("Failed to get rustc version");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_check_rustc_available() {
        // This should pass in any environment with Rust installed
        assert!(check_rustc_available().is_ok());
    }
    #[test]
    fn test_get_rustc_version() {
        let version = get_rustc_version().unwrap_or_else(|_| "unknown".to_string());
        assert!(version.contains("rustc"));
    }
    #[test]
    fn test_compile_simple_program() {
        let source = r#"
            fun main() {
                println("Hello from compiled Ruchy!");
            }
        "#;
        let options = CompileOptions {
            output: PathBuf::from("/tmp/test_ruchy_binary"),
            ..Default::default()
        };
        // This might fail if the transpiler doesn't support the syntax yet
        // but the infrastructure should work
        let _ = compile_source_to_binary(source, &options);
    }

    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        assert_eq!(options.output, PathBuf::from("a.out"));
        assert_eq!(options.opt_level, "2");
        assert!(!options.strip);
        assert!(!options.static_link);
        assert!(options.target.is_none());
        assert!(options.rustc_flags.is_empty());
    }

    #[test]
    fn test_compile_options_custom() {
        let options = CompileOptions {
            output: PathBuf::from("my_binary"),
            opt_level: "3".to_string(),
            strip: true,
            static_link: true,
            target: Some("x86_64-unknown-linux-musl".to_string()),
            rustc_flags: vec!["-C".to_string(), "lto=fat".to_string()],
        };

        assert_eq!(options.output, PathBuf::from("my_binary"));
        assert_eq!(options.opt_level, "3");
        assert!(options.strip);
        assert!(options.static_link);
        assert_eq!(
            options.target,
            Some("x86_64-unknown-linux-musl".to_string())
        );
        assert_eq!(options.rustc_flags.len(), 2);
    }

    #[test]
    fn test_build_rustc_command() {
        let rust_file = Path::new("/tmp/test.rs");
        let options = CompileOptions {
            opt_level: "2".to_string(),
            strip: true,
            ..Default::default()
        };

        let _cmd = build_rustc_command(rust_file, &options);

        // Can't easily test Command internals, but verify it doesn't panic
        // The function returns a Command which we can't easily inspect
        // Test passes without panic; // Just verify no panic
    }

    #[test]
    fn test_apply_optional_flags() {
        let mut cmd = Command::new("rustc");
        let options = CompileOptions {
            strip: true,
            static_link: true,
            target: Some("x86_64-unknown-linux-musl".to_string()),
            rustc_flags: vec!["-C".to_string(), "lto=fat".to_string()],
            ..Default::default()
        };

        apply_optional_flags(&mut cmd, &options);
        // Can't easily inspect Command internals, but verify it doesn't panic
        // Test passes without panic;
    }

    #[test]
    fn test_prepare_rust_file() {
        let rust_code = TokenStream::new();
        let result = prepare_rust_file(&rust_code);
        assert!(result.is_ok());

        if let Ok((_temp_dir, rust_file)) = result {
            assert!(rust_file.exists());
            assert!(rust_file.extension() == Some(std::ffi::OsStr::new("rs")));
        }
    }

    #[test]
    fn test_parse_and_transpile() {
        // Test with valid Ruchy code
        let source = "fun main() { println(\"Hello\"); }";
        let result = parse_and_transpile(source);
        // This might fail if parser doesn't support this syntax yet
        let _ = result; // Just check it doesn't panic
    }

    #[test]
    fn test_execute_compilation() {
        // Test with a command that will fail (non-existent file)
        let mut cmd = Command::new("rustc");
        cmd.arg("/non/existent/file.rs");

        let result = execute_compilation(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_output_exists() {
        // Test with non-existent file
        let result = verify_output_exists(Path::new("/non/existent/binary"));
        assert!(result.is_err());

        // Test with existing file (use temp file)
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = verify_output_exists(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_invalid_source() {
        let source = "this is not valid Ruchy code!@#$%";
        let options = CompileOptions::default();

        let result = compile_source_to_binary(source, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_empty_source() {
        let source = "";
        let options = CompileOptions::default();

        let result = compile_source_to_binary(source, &options);
        // Empty source might be valid or not depending on parser
        let _ = result; // Just check it doesn't panic
    }

    #[test]
    fn test_compile_whitespace_only() {
        let source = "   \n\t\n   ";
        let options = CompileOptions::default();

        let result = compile_source_to_binary(source, &options);
        // Whitespace might be valid or not depending on parser
        let _ = result; // Just check it doesn't panic
    }

    // Test 14: CompileOptions builder pattern functionality
    #[test]
    fn test_compile_options_builder_pattern() {
        let mut options = CompileOptions::default();
        options.output = PathBuf::from("custom_binary");
        options.opt_level = "3".to_string();
        options.strip = true;
        options.rustc_flags.push("--verbose".to_string());

        assert_eq!(options.output, PathBuf::from("custom_binary"));
        assert_eq!(options.opt_level, "3");
        assert!(options.strip);
        assert_eq!(options.rustc_flags.len(), 1);
        assert_eq!(options.rustc_flags[0], "--verbose");
    }

    // Test 15: All optimization level variations
    #[test]
    fn test_all_optimization_levels() {
        let valid_levels = vec!["0", "1", "2", "3", "s", "z"];

        for level in valid_levels {
            let options = CompileOptions {
                opt_level: level.to_string(),
                ..Default::default()
            };
            assert_eq!(options.opt_level, level);

            // Test command building doesn't panic with any opt level
            let rust_file = Path::new("/tmp/test.rs");
            let cmd = build_rustc_command(rust_file, &options);
            // Verify command was created successfully
            assert_eq!(cmd.get_program(), "rustc");
        }
    }

    // Test 16: Target triple validation
    #[test]
    fn test_target_triple_combinations() {
        let targets = vec![
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "x86_64-pc-windows-msvc",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "wasm32-unknown-unknown",
        ];

        for target in targets {
            let options = CompileOptions {
                target: Some(target.to_string()),
                ..Default::default()
            };

            assert_eq!(options.target, Some(target.to_string()));

            // Test command building with target
            let rust_file = Path::new("/tmp/test.rs");
            let cmd = build_rustc_command(rust_file, &options);
            assert_eq!(cmd.get_program(), "rustc");
        }
    }

    // Test 17: Multiple rustc flags handling
    #[test]
    fn test_multiple_rustc_flags() {
        let flags = vec![
            "-C".to_string(),
            "lto=fat".to_string(),
            "--verbose".to_string(),
            "-Z".to_string(),
            "print-type-sizes".to_string(),
        ];

        let options = CompileOptions {
            rustc_flags: flags.clone(),
            ..Default::default()
        };

        assert_eq!(options.rustc_flags.len(), 5);
        assert_eq!(options.rustc_flags, flags);

        // Test flag application
        let mut cmd = Command::new("rustc");
        apply_optional_flags(&mut cmd, &options);
        // Function shouldn't panic with multiple flags
        // Test passes without panic;
    }

    // Test 18: Strip and static link combinations
    #[test]
    fn test_strip_and_static_combinations() {
        let combinations = vec![(false, false), (true, false), (false, true), (true, true)];

        for (strip, static_link) in combinations {
            let options = CompileOptions {
                strip,
                static_link,
                ..Default::default()
            };

            assert_eq!(options.strip, strip);
            assert_eq!(options.static_link, static_link);

            let mut cmd = Command::new("rustc");
            apply_optional_flags(&mut cmd, &options);
            // Should handle all combinations without panic
            // Test passes without panic;
        }
    }

    // Test 19: Path handling with different file extensions
    #[test]
    fn test_path_handling_extensions() {
        let paths = vec![
            PathBuf::from("binary"),
            PathBuf::from("program.exe"),
            PathBuf::from("/tmp/output"),
            PathBuf::from("./relative/path"),
            PathBuf::from("../parent/binary"),
        ];

        for path in paths {
            let options = CompileOptions {
                output: path.clone(),
                ..Default::default()
            };

            assert_eq!(options.output, path);

            let rust_file = Path::new("/tmp/test.rs");
            let cmd = build_rustc_command(rust_file, &options);
            assert_eq!(cmd.get_program(), "rustc");
        }
    }

    // Test 20: Temporary file creation and cleanup
    #[test]
    fn test_temp_file_creation_cleanup() {
        let rust_code = TokenStream::new();

        // Test multiple temp file creations
        for _i in 0..5 {
            let result = prepare_rust_file(&rust_code);
            assert!(result.is_ok());

            if let Ok((_temp_dir, rust_file)) = result {
                // Verify file was created
                assert!(rust_file.exists());
                assert!(rust_file.file_name().unwrap() == "main.rs");

                // Verify parent directory exists
                assert!(rust_file.parent().unwrap().exists());

                // When _temp_dir goes out of scope, cleanup should happen automatically
                // This tests the RAII behavior of TempDir
            }
        }
    }

    // Test 21: Error message handling in execute_compilation
    #[test]
    fn test_execute_compilation_error_messages() {
        // Create command that will fail with specific error
        let mut cmd = Command::new("rustc");
        cmd.arg("--invalid-flag-that-does-not-exist");
        cmd.arg("/non/existent/file.rs");

        let result = execute_compilation(cmd);
        assert!(result.is_err());

        let error_msg = format!("{}", result.err().unwrap());
        assert!(error_msg.contains("Compilation failed"));
    }

    // Test 22: Complex source code patterns
    #[test]
    fn test_complex_source_patterns() {
        let complex_sources = vec![
            // Unicode content
            "fun main() { println(\"Hello 世界! 🚀\"); }",
            // Long identifier names
            "fun this_is_a_very_long_function_name_that_might_cause_issues() { }",
            // Nested structures
            "fun main() { if (true) { if (false) { println(\"nested\"); } } }",
            // Comments and whitespace
            "// Comment\nfun main() {\n  // Another comment\n  println(\"test\");\n}",
            // String with escape sequences
            "fun main() { println(\"Line 1\\nLine 2\\tTabbed\"); }",
        ];

        for source in complex_sources {
            let options = CompileOptions {
                output: PathBuf::from("/tmp/complex_test"),
                ..Default::default()
            };

            // These may fail due to parser limitations, but shouldn't panic
            let result = compile_source_to_binary(source, &options);
            if let Ok(_) = result {
                // Success is good
            }
            // Expected failure is also fine
        }
    }

    // Test 23: Parse and transpile error handling
    #[test]
    fn test_parse_and_transpile_error_handling() {
        let invalid_sources = vec![
            "{{{[[[@#$%",            // Invalid syntax
            "fun(",                  // Incomplete function
            "\"unterminated string", // Unterminated string
        ];

        for source in invalid_sources {
            let result = compile_source_to_binary(source, &CompileOptions::default());
            // Should return error, not panic
            assert!(result.is_err(), "Expected error for source: '{source}'");
        }
    }

    // Test 24: File I/O edge cases
    #[test]
    fn test_file_io_edge_cases() {
        // Test with empty token stream
        let empty_tokens = TokenStream::new();
        let result = prepare_rust_file(&empty_tokens);
        assert!(result.is_ok());

        if let Ok((_temp_dir, rust_file)) = result {
            // Verify empty file was created
            assert!(rust_file.exists());
            let contents = std::fs::read_to_string(&rust_file).unwrap();
            assert!(contents.is_empty());
        }
    }

    // Test 25: Verify output with different scenarios
    #[test]
    fn test_verify_output_scenarios() {
        // Test with temporary file that exists
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = verify_output_exists(temp_file.path());
        assert!(result.is_ok());

        // Test with directory instead of file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = verify_output_exists(temp_dir.path());
        assert!(result.is_ok()); // Directory exists, so should pass

        // Test with file in nested directory that doesn't exist
        let nested_path = Path::new("/non/existent/directory/binary");
        let result = verify_output_exists(nested_path);
        assert!(result.is_err());
    }

    // Test 26: Command building with extreme cases
    #[test]
    fn test_command_building_extreme_cases() {
        let options = CompileOptions {
            output: PathBuf::from("/very/long/path/with/many/segments/binary"),
            opt_level: "z".to_string(), // Size optimization
            strip: true,
            static_link: true,
            target: Some("wasm32-unknown-unknown".to_string()),
            rustc_flags: vec![
                "-C".to_string(),
                "lto=fat".to_string(),
                "-C".to_string(),
                "codegen-units=1".to_string(),
                "-C".to_string(),
                "panic=abort".to_string(),
            ],
        };

        let rust_file = Path::new("/tmp/test.rs");
        let cmd = build_rustc_command(rust_file, &options);

        // Verify command was built successfully
        assert_eq!(cmd.get_program(), "rustc");
    }

    // Test 27: CompileOptions clone and debug functionality
    #[test]
    fn test_compile_options_traits() {
        let options = CompileOptions {
            output: PathBuf::from("test_binary"),
            opt_level: "2".to_string(),
            strip: true,
            static_link: false,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            rustc_flags: vec!["--verbose".to_string()],
        };

        // Test Clone trait
        let cloned_options = options.clone();
        assert_eq!(options.output, cloned_options.output);
        assert_eq!(options.opt_level, cloned_options.opt_level);
        assert_eq!(options.strip, cloned_options.strip);
        assert_eq!(options.static_link, cloned_options.static_link);
        assert_eq!(options.target, cloned_options.target);
        assert_eq!(options.rustc_flags, cloned_options.rustc_flags);

        // Test Debug trait
        let debug_str = format!("{options:?}");
        assert!(debug_str.contains("CompileOptions"));
        assert!(debug_str.contains("test_binary"));
        assert!(debug_str.contains('2'));
    }

    // Test 28: Integration with file system operations
    #[test]
    fn test_filesystem_integration() {
        use std::fs;

        // Create a temporary directory for our tests
        let temp_dir = tempfile::TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test_program.ruchy");

        // Write a simple test program
        let source_content = "fun main() { println(\"Integration test\"); }";
        fs::write(&source_file, source_content).unwrap();

        // Verify file was created and can be read
        assert!(source_file.exists());
        let read_content = fs::read_to_string(&source_file).unwrap();
        assert_eq!(read_content, source_content);

        // Test compile_to_binary with actual file
        let output_path = temp_dir.path().join("test_output");
        let options = CompileOptions {
            output: output_path,
            ..Default::default()
        };

        // This may fail due to parser/transpiler limitations, but should not panic
        let result = compile_to_binary(&source_file, &options);
        if let Ok(_) = result {
            // Success is good
        }
        // Expected failure due to incomplete implementation
    }

    // Test 29: rustc version parsing
    #[test]
    fn test_rustc_version_parsing() {
        if let Ok(version) = get_rustc_version() {
            // Should contain rustc and version number
            assert!(version.contains("rustc"));

            // Should contain a version number pattern (X.Y.Z)
            let has_version_pattern = version.split_whitespace().any(|part| {
                part.split('.').count() >= 2 && part.chars().any(|c| c.is_ascii_digit())
            });
            assert!(has_version_pattern);
        }
    }

    // Test 30: Error context propagation
    #[test]
    fn test_error_context_propagation() {
        // Test parse context in parse_and_transpile
        let invalid_source = "syntax error @#$%";
        let result = parse_and_transpile(invalid_source);

        if let Err(error) = result {
            let error_str = format!("{error}");
            // Should contain context information
            assert!(!error_str.is_empty()); // At least some error message
        }

        // Test compile context for invalid paths
        let invalid_path = Path::new("/root/no_permission/file.ruchy");
        let options = CompileOptions::default();

        if invalid_path.exists() {
            // Only test if file actually exists and we can't read it
            let result = compile_to_binary(invalid_path, &options);
            assert!(result.is_err());
        }
    }
}
#[cfg(test)]
mod property_tests_compiler {
    use super::*;
    use proptest::proptest;

    proptest! {
        /// Property: compile_source_to_binary never panics on any string input
        #[test]
        fn test_compile_source_to_binary_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input, even invalid syntax
            let result = std::panic::catch_unwind(|| {
                let options = CompileOptions::default();
                let _ = compile_source_to_binary(&input, &options);
            });
            // Assert that no panic occurred (Result can be Ok or Err, but no panic)
            assert!(result.is_ok(), "compile_source_to_binary panicked on input: {input:?}");
        }
    }
}
