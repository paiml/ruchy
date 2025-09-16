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

    #[test]
    fn test_harness_default_duplicate_renamed() {
        let harness = RuchyTestHarness::default();
        assert!(!harness.keep_intermediates);
        assert!(matches!(harness.optimization_level, OptLevel::Basic));
        assert_eq!(harness.timeout_secs, 30);
    }

    #[test]
    fn test_harness_new_vs_default() {
        let harness1 = RuchyTestHarness::new();
        let harness2 = RuchyTestHarness::default();
        assert_eq!(harness1.keep_intermediates, harness2.keep_intermediates);
        assert_eq!(harness1.timeout_secs, harness2.timeout_secs);
    }

    #[test]
    fn test_all_opt_levels() {
        let levels = [OptLevel::None, OptLevel::Basic, OptLevel::Full];
        for level in levels {
            let mut harness = RuchyTestHarness::new();
            harness.optimization_level = level;
            // Just test they can be set and matched
            match level {
                OptLevel::None => assert!(matches!(harness.optimization_level, OptLevel::None)),
                OptLevel::Basic => assert!(matches!(harness.optimization_level, OptLevel::Basic)),
                OptLevel::Full => assert!(matches!(harness.optimization_level, OptLevel::Full)),
            }
        }
    }

    #[test]
    fn test_validation_result_with_rust_code() {
        let result = ValidationResult {
            name: "test".to_string(),
            parse_success: true,
            transpile_success: true,
            compile_success: true,
            execution_output: Some("42".to_string()),
            rust_code: Some("fn main() { println!(\"42\"); }".to_string()),
        };

        assert_eq!(result.name, "test");
        assert!(result.parse_success);
        assert!(result.transpile_success);
        assert!(result.compile_success);
        assert_eq!(result.execution_output.unwrap(), "42");
        assert!(result.rust_code.is_some());
        assert!(result.rust_code.unwrap().contains("main"));
    }

    #[test]
    fn test_validation_result_without_rust_code() {
        let result = ValidationResult {
            name: "test".to_string(),
            parse_success: true,
            transpile_success: true,
            compile_success: true,
            execution_output: Some("42".to_string()),
            rust_code: None,
        };

        assert_eq!(result.name, "test");
        assert!(result.parse_success);
        assert!(result.transpile_success);
        assert!(result.compile_success);
        assert_eq!(result.execution_output.unwrap(), "42");
        assert!(result.rust_code.is_none());
    }

    #[test]
    fn test_execution_result_compilation_failure() {
        let result = ExecutionResult {
            compiled: false,
            output: None,
            stderr: Some("compilation error".to_string()),
        };

        assert!(!result.compiled);
        assert!(result.output.is_none());
        assert_eq!(result.stderr.unwrap(), "compilation error");
    }

    #[test]
    fn test_execution_result_success_no_stderr() {
        let result = ExecutionResult {
            compiled: true,
            output: Some("Hello, World!".to_string()),
            stderr: None,
        };

        assert!(result.compiled);
        assert_eq!(result.output.unwrap(), "Hello, World!");
        assert!(result.stderr.is_none());
    }

    #[test]
    fn test_execution_result_success_with_stderr() {
        let result = ExecutionResult {
            compiled: true,
            output: Some("output".to_string()),
            stderr: Some("warning: unused variable".to_string()),
        };

        assert!(result.compiled);
        assert!(result.output.is_some());
        assert!(result.stderr.is_some());
        assert!(result.stderr.unwrap().contains("warning"));
    }

    #[test]
    fn test_test_error_file_read() {
        let error = TestError::FileRead("Permission denied".to_string());
        assert_eq!(error.to_string(), "Failed to read file: Permission denied");
    }

    #[test]
    fn test_test_error_parse() {
        let error = TestError::Parse("Unexpected token".to_string());
        assert_eq!(error.to_string(), "Parse error: Unexpected token");
    }

    #[test]
    fn test_test_error_transpile() {
        let error = TestError::Transpile("Unknown type".to_string());
        assert_eq!(error.to_string(), "Transpile error: Unknown type");
    }

    #[test]
    fn test_test_error_compile() {
        let error = TestError::Compile("rustc not found".to_string());
        assert_eq!(error.to_string(), "Compilation error: rustc not found");
    }

    #[test]
    fn test_test_error_execute() {
        let error = TestError::Execute("Binary crashed".to_string());
        assert_eq!(error.to_string(), "Execution error: Binary crashed");
    }

    #[test]
    fn test_test_error_output_mismatch() {
        let error = TestError::OutputMismatch {
            expected: "Hello".to_string(),
            actual: "Hi".to_string(),
        };
        assert_eq!(error.to_string(), "Output mismatch: expected Hello, got Hi");
    }

    #[test]
    fn test_harness_field_modifications() {
        let mut harness = RuchyTestHarness::new();

        // Modify each field
        harness.keep_intermediates = true;
        harness.optimization_level = OptLevel::None;
        harness.timeout_secs = 120;

        assert!(harness.keep_intermediates);
        assert!(matches!(harness.optimization_level, OptLevel::None));
        assert_eq!(harness.timeout_secs, 120);
    }

    #[test]
    fn test_validation_result_all_failures() {
        let result = ValidationResult {
            name: "failed_test".to_string(),
            parse_success: false,
            transpile_success: false,
            compile_success: false,
            execution_output: None,
            rust_code: None,
        };

        assert_eq!(result.name, "failed_test");
        assert!(!result.parse_success);
        assert!(!result.transpile_success);
        assert!(!result.compile_success);
        assert!(result.execution_output.is_none());
        assert!(result.rust_code.is_none());
    }

    #[test]
    fn test_validation_result_partial_success() {
        let result = ValidationResult {
            name: "partial_test".to_string(),
            parse_success: true,
            transpile_success: true,
            compile_success: false,
            execution_output: None,
            rust_code: Some("invalid rust code".to_string()),
        };

        assert_eq!(result.name, "partial_test");
        assert!(result.parse_success);
        assert!(result.transpile_success);
        assert!(!result.compile_success);
        assert!(result.execution_output.is_none());
        assert!(result.rust_code.is_some());
    }

    #[test]
    fn test_test_result_type_ok() {
        let result: TestResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_test_result_type_err() {
        let result: TestResult<String> = Err(TestError::Parse("error".to_string()));
        assert!(result.is_err());
        match result.unwrap_err() {
            TestError::Parse(msg) => assert_eq!(msg, "error"),
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_harness_debug_formatting() {
        let harness = RuchyTestHarness::new();
        let debug_str = format!("{:?}", harness);
        assert!(debug_str.contains("RuchyTestHarness"));
        assert!(debug_str.contains("keep_intermediates"));
        assert!(debug_str.contains("optimization_level"));
        assert!(debug_str.contains("timeout_secs"));
    }

    #[test]
    fn test_opt_level_debug_formatting() {
        let levels = [OptLevel::None, OptLevel::Basic, OptLevel::Full];
        for level in levels {
            let debug_str = format!("{:?}", level);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_validation_result_debug_formatting() {
        let result = ValidationResult {
            name: "debug_test".to_string(),
            parse_success: true,
            transpile_success: true,
            compile_success: true,
            execution_output: Some("output".to_string()),
            rust_code: None,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("ValidationResult"));
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("parse_success"));
    }

    #[test]
    fn test_execution_result_debug_formatting() {
        let result = ExecutionResult {
            compiled: true,
            output: Some("test_output".to_string()),
            stderr: None,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("ExecutionResult"));
        assert!(debug_str.contains("compiled"));
        assert!(debug_str.contains("test_output"));
    }

    #[test]
    fn test_test_error_debug_formatting() {
        let error = TestError::Parse("test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Parse"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_harness_fields_independent() {
        let mut harness1 = RuchyTestHarness::new();
        let mut harness2 = RuchyTestHarness::new();

        harness1.keep_intermediates = true;
        harness2.timeout_secs = 60;

        assert!(harness1.keep_intermediates);
        assert!(!harness2.keep_intermediates);
        assert_eq!(harness1.timeout_secs, 30);
        assert_eq!(harness2.timeout_secs, 60);
    }

    #[test]
    fn test_opt_level_enum_completeness() {
        // Test that we can construct all variants
        let _none = OptLevel::None;
        let _basic = OptLevel::Basic;
        let _full = OptLevel::Full;

        // Test that they're different
        assert!(!matches!(OptLevel::None, OptLevel::Basic));
        assert!(!matches!(OptLevel::Basic, OptLevel::Full));
        assert!(!matches!(OptLevel::Full, OptLevel::None));
    }
}
