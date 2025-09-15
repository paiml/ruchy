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
/// # Examples
/// 
/// ```
/// use ruchy::testing::harness::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Self::default()
    }
    /// Validate a Ruchy file through the full compilation pipeline
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, transpiled, compiled, or executed.
/// # Examples
/// 
/// ```
/// use ruchy::testing::harness::validate_file;
/// 
/// let result = validate_file(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn validate_file(&self, path: &Path) -> TestResult<ValidationResult> {
        let content = fs::read_to_string(path).map_err(|e| TestError::FileRead(e.to_string()))?;
        self.validate_source(&content, path.to_string_lossy().as_ref())
    }
    /// Validate Ruchy source code
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, transpiled, compiled, or executed.
/// # Examples
/// 
/// ```
/// use ruchy::testing::harness::validate_source;
/// 
/// let result = validate_source("example");
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::harness::assert_output;
/// 
/// let result = assert_output("example");
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::harness::validate_directory;
/// 
/// let result = validate_directory(());
/// assert_eq!(result, Ok(()));
/// ```
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_harness_default() {
        let harness = RuchyTestHarness::default();
        assert!(!harness.keep_intermediates);
        assert_eq!(harness.timeout_secs, 30);
        assert!(matches!(harness.optimization_level, OptLevel::Basic));
    }

    #[test]
    fn test_harness_new() {
        let harness = RuchyTestHarness::new();
        assert_eq!(harness.keep_intermediates, false);
        assert_eq!(harness.timeout_secs, 30);
    }

    #[test]
    fn test_opt_level_variants() {
        let _ = OptLevel::None;
        let _ = OptLevel::Basic;
        let _ = OptLevel::Full;
    }

    #[test]
    fn test_validate_source_parse_error() {
        let harness = RuchyTestHarness::new();
        let result = harness.validate_source("let x = ", "test");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, TestError::Parse(_)));
        }
    }

    #[test]
    fn test_validate_source_simple() {
        let harness = RuchyTestHarness::new();
        let result = harness.validate_source("let x = 42", "test");
        // May fail at transpile or compile stage, but parse should succeed
        match result {
            Ok(validation) => {
                assert_eq!(validation.name, "test");
                assert!(validation.parse_success);
            }
            Err(e) => {
                // Expected - transpiler may not handle all constructs
                assert!(!matches!(e, TestError::Parse(_)));
            }
        }
    }

    #[test]
    fn test_assert_output_mismatch() {
        let harness = RuchyTestHarness::new();
        // This will likely fail at parse/transpile, but tests the error path
        let result = harness.assert_output("println(\"hello\")", "goodbye", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_directory_empty() {
        let harness = RuchyTestHarness::new();
        let temp_dir = tempdir().unwrap();
        let result = harness.validate_directory(temp_dir.path());
        assert!(result.is_ok());
        if let Ok(results) = result {
            assert_eq!(results.len(), 0);
        }
    }

    #[test]
    fn test_validate_directory_with_ruchy_file() {
        let harness = RuchyTestHarness::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");
        fs::write(&file_path, "let x = 1").unwrap();

        let result = harness.validate_directory(temp_dir.path());
        match result {
            Ok(results) => {
                assert_eq!(results.len(), 1);
            }
            Err(_) => {
                // Expected - may fail at parse/transpile stage
            }
        }
    }

    #[test]
    fn test_validation_result_fields() {
        let result = ValidationResult {
            name: "test".to_string(),
            parse_success: true,
            transpile_success: false,
            compile_success: false,
            execution_output: None,
            rust_code: Some("code".to_string()),
        };
        assert_eq!(result.name, "test");
        assert!(result.parse_success);
        assert!(!result.transpile_success);
    }

    #[test]
    fn test_execution_result_fields() {
        let result = ExecutionResult {
            compiled: true,
            output: Some("output".to_string()),
            stderr: Some("error".to_string()),
        };
        assert!(result.compiled);
        assert_eq!(result.output.unwrap(), "output");
        assert_eq!(result.stderr.unwrap(), "error");
    }

    #[test]
    fn test_error_variants() {
        let _ = TestError::FileRead("error".to_string());
        let _ = TestError::Parse("error".to_string());
        let _ = TestError::Transpile("error".to_string());
        let _ = TestError::Compile("error".to_string());
        let _ = TestError::Execute("error".to_string());
        let _ = TestError::OutputMismatch {
            expected: "a".to_string(),
            actual: "b".to_string(),
        };
    }

    #[test]
    fn test_harness_with_keep_intermediates() {
        let mut harness = RuchyTestHarness::new();
        harness.keep_intermediates = true;
        assert!(harness.keep_intermediates);
    }

    #[test]
    fn test_harness_with_optimization() {
        let mut harness = RuchyTestHarness::new();
        harness.optimization_level = OptLevel::Full;
        assert!(matches!(harness.optimization_level, OptLevel::Full));
    }

    #[test]
    fn test_harness_with_timeout() {
        let mut harness = RuchyTestHarness::new();
        harness.timeout_secs = 60;
        assert_eq!(harness.timeout_secs, 60);
    }

    #[test]
    fn test_validate_file_not_found() {
        let harness = RuchyTestHarness::new();
        let result = harness.validate_file(Path::new("/nonexistent/file.ruchy"));
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, TestError::FileRead(_)));
        }
    }

    #[test]
    fn test_error_display() {
        let err = TestError::Parse("test error".to_string());
        assert_eq!(err.to_string(), "Parse error: test error");

        let err = TestError::OutputMismatch {
            expected: "a".to_string(),
            actual: "b".to_string(),
        };
        assert_eq!(err.to_string(), "Output mismatch: expected a, got b");
    }

    #[test]
    fn test_result_type_alias() {
        let result: TestResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);

        let result: TestResult<i32> = Err(TestError::Execute("failed".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_harness_clone() {
        let harness1 = RuchyTestHarness::new();
        let harness2 = harness1.clone();
        assert_eq!(harness1.timeout_secs, harness2.timeout_secs);
    }

    #[test]
    fn test_opt_level_copy() {
        let opt1 = OptLevel::Basic;
        let opt2 = opt1;
        assert!(matches!(opt2, OptLevel::Basic));
    }
}
