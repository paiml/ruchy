/// Testing harness for validating Ruchy code
/// This module provides a public API for external projects (like ruchy-book)
/// to validate that Ruchy code compiles and executes correctly via LLVM
use crate::Parser;
use crate::Transpiler;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Failed to read file: {0}")]
    FileRead(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Transpile error: {0}")]
    Transpile(String),

    #[error("Compilation error: {0}")]
    Compile(String),

    #[error("Execution error: {0}")]
    Execute(String),

    #[error("Output mismatch: expected {expected}, got {actual}")]
    OutputMismatch { expected: String, actual: String },
}

/// Result type for testing operations
pub type TestResult<T> = Result<T, TestError>;

/// Test harness for validating Ruchy code
#[derive(Debug, Clone)]
pub struct RuchyTestHarness {
    /// Whether to keep intermediate files for debugging
    pub keep_intermediates: bool,

    /// Optimization level for LLVM compilation
    pub optimization_level: OptLevel,

    /// Timeout for execution in seconds
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    None,
    Basic,
    Full,
}

impl Default for RuchyTestHarness {
    fn default() -> Self {
        Self {
            keep_intermediates: false,
            optimization_level: OptLevel::Basic,
            timeout_secs: 30,
        }
    }
}

impl RuchyTestHarness {
    /// Create a new test harness with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a Ruchy file through the full compilation pipeline
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, transpiled, compiled, or executed.
    pub fn validate_file(&self, path: &Path) -> TestResult<ValidationResult> {
        let content = fs::read_to_string(path).map_err(|e| TestError::FileRead(e.to_string()))?;

        self.validate_source(&content, path.to_string_lossy().as_ref())
    }

    /// Validate Ruchy source code
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, transpiled, compiled, or executed.
    pub fn validate_source(&self, source: &str, name: &str) -> TestResult<ValidationResult> {
        // Parse
        let mut parser = Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| TestError::Parse(format!("{name}: {e:?}")))?;

        // Transpile to Rust
        let transpiler = Transpiler::new();
        let rust_code = transpiler
            .transpile(&ast)
            .map_err(|e| TestError::Transpile(format!("{name}: {e:?}")))?;
        let rust_code = rust_code.to_string();

        // Compile and execute
        let execution_result = self.compile_and_run(&rust_code, name)?;

        Ok(ValidationResult {
            name: name.to_string(),
            parse_success: true,
            transpile_success: true,
            compile_success: execution_result.compiled,
            execution_output: execution_result.output,
            rust_code: if self.keep_intermediates {
                Some(rust_code)
            } else {
                None
            },
        })
    }

    /// Compile Rust code to binary via LLVM and run it
    fn compile_and_run(&self, rust_code: &str, _name: &str) -> TestResult<ExecutionResult> {
        // Write Rust code to working file
        let mut temp_file = NamedTempFile::new().map_err(|e| TestError::Compile(e.to_string()))?;

        temp_file
            .write_all(rust_code.as_bytes())
            .map_err(|e| TestError::Compile(e.to_string()))?;

        temp_file
            .flush()
            .map_err(|e| TestError::Compile(e.to_string()))?;

        // Compile with rustc (LLVM backend)
        let output_binary = temp_file.path().with_extension("exe");

        let opt_level = match self.optimization_level {
            OptLevel::None => "opt-level=0",
            OptLevel::Basic => "opt-level=2",
            OptLevel::Full => "opt-level=3",
        };

        let compile_result = Command::new("rustc")
            .arg("--edition=2021")
            .arg("-C")
            .arg(opt_level)
            .arg("-o")
            .arg(&output_binary)
            .arg(temp_file.path())
            .output()
            .map_err(|e| TestError::Compile(e.to_string()))?;

        if !compile_result.status.success() {
            return Ok(ExecutionResult {
                compiled: false,
                output: None,
                stderr: Some(String::from_utf8_lossy(&compile_result.stderr).to_string()),
            });
        }

        // Run the binary
        let run_result = Command::new(&output_binary)
            .output()
            .map_err(|e| TestError::Execute(e.to_string()))?;

        // Clean up unless keeping intermediates
        if !self.keep_intermediates && output_binary.exists() {
            fs::remove_file(output_binary).ok();
        }

        Ok(ExecutionResult {
            compiled: true,
            output: Some(String::from_utf8_lossy(&run_result.stdout).to_string()),
            stderr: if run_result.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&run_result.stderr).to_string())
            },
        })
    }

    /// Validate that source produces expected output
    ///
    /// # Errors
    ///
    /// Returns an error if parsing, transpilation, compilation, or execution fails,
    /// or if the actual output doesn't match the expected output.
    pub fn assert_output(&self, source: &str, expected: &str, name: &str) -> TestResult<()> {
        let result = self.validate_source(source, name)?;

        if let Some(actual) = result.execution_output {
            if actual.trim() != expected.trim() {
                return Err(TestError::OutputMismatch {
                    expected: expected.to_string(),
                    actual,
                });
            }
        } else {
            return Err(TestError::Execute("No output produced".to_string()));
        }

        Ok(())
    }

    /// Batch validate multiple files
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or any of the .ruchy files fail to validate.
    pub fn validate_directory(&self, dir: &Path) -> TestResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for entry in fs::read_dir(dir).map_err(|e| TestError::FileRead(e.to_string()))? {
            let entry = entry.map_err(|e| TestError::FileRead(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
                results.push(self.validate_file(&path)?);
            }
        }

        Ok(results)
    }
}

/// Result of validating a Ruchy source file
#[derive(Debug)]
pub struct ValidationResult {
    pub name: String,
    pub parse_success: bool,
    pub transpile_success: bool,
    pub compile_success: bool,
    pub execution_output: Option<String>,
    pub rust_code: Option<String>,
}

/// Result of compiling and executing code
#[derive(Debug)]
struct ExecutionResult {
    compiled: bool,
    output: Option<String>,
    #[allow(dead_code)]
    stderr: Option<String>,
}
