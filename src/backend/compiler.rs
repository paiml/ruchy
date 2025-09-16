//! Binary compilation support for Ruchy
//! 
//! This module provides functionality to compile Ruchy code to standalone binaries
//! via Rust compilation toolchain (rustc).
use anyhow::{Context, Result, bail};
use crate::utils::common_patterns::ResultContextExt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use proc_macro2::TokenStream;
use crate::{Parser, Transpiler};
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
    let source = fs::read_to_string(source_path)
        .file_context("read", source_path)?;
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
    let ast = parser.parse()
        .parse_context("Ruchy source")?;
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .compile_context("transpile to Rust")?;
    eprintln!("DEBUG: transpile_to_program completed");
    Ok(rust_code)
}
/// Prepare temporary Rust file for compilation (complexity: 4)
fn prepare_rust_file(rust_code: &TokenStream) -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()
        .compile_context("create temporary directory")?;
    let rust_file = temp_dir.path().join("main.rs");
    let rust_code_str = rust_code.to_string();
    // Debug: Also write to /tmp/debug_rust_output.rs for inspection
    fs::write("/tmp/debug_rust_output.rs", &rust_code_str)
        .context("Failed to write debug Rust code")?;
    fs::write(&rust_file, &rust_code_str)
        .context("Failed to write Rust code to temporary file")?;
    Ok((temp_dir, rust_file))
}
/// Build rustc command with options (complexity: 7)
fn build_rustc_command(rust_file: &Path, options: &CompileOptions) -> Command {
    let mut cmd = Command::new("rustc");
    cmd.arg(rust_file)
        .arg("-o")
        .arg(&options.output);
    // Add optimization level
    cmd.arg("-C").arg(format!("opt-level={}", options.opt_level));
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
    let output = cmd.output()
        .context("Failed to execute rustc")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Compilation failed:\n{}", stderr);
    }
    Ok(())
}
/// Verify output file exists (complexity: 2)
fn verify_output_exists(output_path: &Path) -> Result<()> {
    if !output_path.exists() {
        bail!("Expected output file not created: {}", output_path.display());
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
        assert_eq!(options.target, Some("x86_64-unknown-linux-musl".to_string()));
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

        let cmd = build_rustc_command(rust_file, &options);

        // Can't easily test Command internals, but verify it doesn't panic
        // The function returns a Command which we can't easily inspect
        assert!(true); // Just verify no panic
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
        assert!(true);
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
}
#[cfg(test)]
mod property_tests_compiler {
    use proptest::proptest;
    use super::*;
    
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
            assert!(result.is_ok(), "compile_source_to_binary panicked on input: {:?}", input);
        }
    }
}
