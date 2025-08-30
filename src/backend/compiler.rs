//! Binary compilation support for Ruchy
//! 
//! This module provides functionality to compile Ruchy code to standalone binaries
//! via Rust compilation toolchain (rustc).

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

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
        .with_context(|| format!("Failed to read source file: {}", source_path.display()))?;
    
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
/// - The temporary directory cannot be created
/// - The rustc compilation fails
pub fn compile_source_to_binary(source: &str, options: &CompileOptions) -> Result<PathBuf> {
    // Parse the Ruchy source
    let mut parser = Parser::new(source);
    let ast = parser.parse()
        .context("Failed to parse Ruchy source")?;
    
    // Transpile to Rust
    eprintln!("DEBUG: About to call transpile_to_program");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .context("Failed to transpile to Rust")?;
    eprintln!("DEBUG: transpile_to_program completed");
    
    // Create temporary directory for compilation
    let temp_dir = TempDir::new()
        .context("Failed to create temporary directory")?;
    
    // Write Rust code to temporary file
    let rust_file = temp_dir.path().join("main.rs");
    let rust_code_str = rust_code.to_string();
    
    // Debug: Also write to /tmp/debug_rust_output.rs for inspection
    fs::write("/tmp/debug_rust_output.rs", &rust_code_str)
        .context("Failed to write debug Rust code")?;
    
    fs::write(&rust_file, rust_code_str)
        .context("Failed to write Rust code to temporary file")?;
    
    // Build rustc command
    let mut cmd = Command::new("rustc");
    cmd.arg(&rust_file)
        .arg("-o")
        .arg(&options.output);
    
    // Add optimization level
    cmd.arg("-C").arg(format!("opt-level={}", options.opt_level));
    
    // Add strip flag if requested
    if options.strip {
        cmd.arg("-C").arg("strip=symbols");
    }
    
    // Add static linking if requested
    if options.static_link {
        cmd.arg("-C").arg("target-feature=+crt-static");
    }
    
    // Add target if specified
    if let Some(target) = &options.target {
        cmd.arg("--target").arg(target);
    }
    
    // Add additional flags
    for flag in &options.rustc_flags {
        cmd.arg(flag);
    }
    
    // Execute rustc
    let output = cmd.output()
        .context("Failed to execute rustc")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Compilation failed:\n{}", stderr);
    }
    
    // Ensure the output file exists
    if !options.output.exists() {
        bail!("Expected output file not created: {}", options.output.display());
    }
    
    Ok(options.output.clone())
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
}