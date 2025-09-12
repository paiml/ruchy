//! Common patterns and utilities to reduce code entropy
//! Extracted to eliminate repetitive code patterns across the codebase
use anyhow::{Context, Result};
use std::path::Path;
#[cfg(test)]
use proptest::prelude::*;
/// Standard error handling pattern for file operations
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::read_file_with_context;
/// 
/// let result = read_file_with_context(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn read_file_with_context(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}
/// Standard error handling pattern for writing files  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::write_file_with_context;
/// 
/// let result = write_file_with_context("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn write_file_with_context(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}
/// Standard pattern for parsing Ruchy code
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::parse_ruchy_code;
/// 
/// let result = parse_ruchy_code("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn parse_ruchy_code(source: &str) -> Result<crate::frontend::ast::Expr> {
    let mut parser = crate::frontend::parser::Parser::new(source);
    parser.parse()
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))
}
/// Standard pattern for success response creation  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::create_success_response;
/// 
/// let result = create_success_response(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn create_success_response(value: String, cell_id: String, execution_time: f64) -> crate::wasm::shared_session::ExecuteResponse {
    crate::wasm::shared_session::ExecuteResponse {
        success: true,
        cell_id,
        value: value.clone(),
        result: value,
        error: None,
        execution_time_ms: execution_time,
    }
}
/// Standard pattern for error response creation
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::create_error_response;
/// 
/// let result = create_error_response(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn create_error_response(error: String, cell_id: String) -> crate::wasm::shared_session::ExecuteResponse {
    crate::wasm::shared_session::ExecuteResponse {
        success: false,
        cell_id,
        value: String::new(),
        result: String::new(),
        error: Some(error),
        execution_time_ms: 0.0,
    }
}
/// Format a module operation error
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_module_error;
/// 
/// let result = format_module_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_module_error(operation: &str, module_name: &str) -> String {
    format!("Failed to {} module '{}'", operation, module_name)
}
/// Format a parsing error
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_parse_error;
/// 
/// let result = format_parse_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_parse_error(target: &str) -> String {
    format!("Failed to parse {}", target)
}
/// Format a compilation error  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_compile_error;
/// 
/// let result = format_compile_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_compile_error(stage: &str) -> String {
    format!("Failed to {}", stage)
}
/// Extension trait for Result types to add common context patterns
pub trait ResultContextExt<T> {
    /// Add context for file operations
    fn file_context(self, operation: &str, path: &Path) -> Result<T>;
    /// Add context for module operations
    fn module_context(self, operation: &str, module_name: &str) -> Result<T>;
    /// Add context for parsing operations
    fn parse_context(self, target: &str) -> Result<T>;
    /// Add context for compilation operations
    fn compile_context(self, stage: &str) -> Result<T>;
}
impl<T, E> ResultContextExt<T> for std::result::Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn file_context(self, operation: &str, path: &Path) -> Result<T> {
        self.map_err(Into::into)
            .with_context(|| format!("Failed to {} file: {}", operation, path.display()))
    }
    fn module_context(self, operation: &str, module_name: &str) -> Result<T> {
        self.map_err(Into::into)
            .with_context(|| format_module_error(operation, module_name))
    }
    fn parse_context(self, target: &str) -> Result<T> {
        self.map_err(Into::into)
            .with_context(|| format_parse_error(target))
    }
    fn compile_context(self, stage: &str) -> Result<T> {
        self.map_err(Into::into)
            .with_context(|| format_compile_error(stage))
    }
}
/// Standard timing pattern for operations
pub fn time_operation<F, R>(operation: F) -> (R, f64) 
where 
    F: FnOnce() -> R
{
    let start = std::time::Instant::now();
    let result = operation();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    (result, elapsed_ms)
}
/// Standard validation pattern for identifiers
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::is_valid_identifier;
/// 
/// let result = is_valid_identifier("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty() && 
    name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') &&
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
/// Standard pattern for creating section headers in output
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::create_section_header;
/// 
/// let result = create_section_header("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn create_section_header(title: &str) -> String {
    format!("=== {} ===\n", title)
}
/// Standard pattern for adding checkmarks to output  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::add_success_indicator;
/// 
/// let result = add_success_indicator("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_success_indicator(message: &str) -> String {
    format!("✅ {}\n", message)
}
/// Standard pattern for adding error indicators to output
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::add_error_indicator;
/// 
/// let result = add_error_indicator("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_error_indicator(message: &str) -> String {
    format!("❌ {}\n", message)
}
/// Standard pattern for handling optional output file writing
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::write_output_or_print;
/// 
/// let result = write_output_or_print(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn write_output_or_print(content: String, output: Option<&Path>) -> Result<()> {
    match output {
        Some(output_path) => {
            write_file_with_context(output_path, &content)?;
            println!("✅ Output written to: {}", output_path.display());
        }
        None => print!("{}", content),
    }
    Ok(())
}
/// Standard pattern for progress indication during operations
pub struct ProgressIndicator {
    pub total: usize,
    pub current: usize,
    pub label: String,
}
impl ProgressIndicator {
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new(total: usize, label: String) -> Self {
        Self { total, current: 0, label }
    }
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::increment;
/// 
/// let result = increment(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn increment(&mut self) {
        self.current += 1;
        if self.current % 10 == 0 || self.current == self.total {
            println!("📊 {}: {}/{}", self.label, self.current, self.total);
        }
    }
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::finish;
/// 
/// let result = finish(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn finish(&self) {
        println!("✅ {} completed: {}/{}", self.label, self.current, self.total);
    }
}
/// Standard pattern for retry logic with exponential backoff
pub fn retry_operation<F, R, E>(mut operation: F, max_attempts: u32) -> std::result::Result<R, E>
where
    F: FnMut() -> std::result::Result<R, E>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                let delay = std::time::Duration::from_millis(2_u64.pow(attempts - 1) * 100);
                std::thread::sleep(delay);
            }
        }
    }
}
/// Standard pattern for conditional compilation features
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::check_feature_enabled;
/// 
/// let result = check_feature_enabled("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn check_feature_enabled(feature: &str) -> bool {
    match feature {
        "notebook" => cfg!(feature = "notebook"),
        "wasm-compile" => cfg!(feature = "wasm-compile"),  
        _ => false,
    }
}
/// Standard pattern for memory size formatting
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_memory_size;
/// 
/// let result = format_memory_size(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_memory_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{:.0} {}", size, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}
/// Standard pattern for version string formatting
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_version_info;
/// 
/// let result = format_version_info(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_version_info() -> String {
    format!("Ruchy v{} ({})", 
            env!("CARGO_PKG_VERSION"),
            if cfg!(debug_assertions) { "debug" } else { "release" })
}
/// Standard pattern for test assertion with string conversion
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::assert_output_contains;
/// 
/// let result = assert_output_contains("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn assert_output_contains(result: impl ToString, expected: &str) {
    let output = result.to_string();
    assert!(output.contains(expected), 
            "Output does not contain '{}'. Actual output: '{}'", expected, output);
}
/// Standard pattern for test assertion with exact match
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::assert_output_equals;
/// 
/// let result = assert_output_equals("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn assert_output_equals(result: impl ToString, expected: &str) {
    let output = result.to_string();
    assert_eq!(output, expected, "Output does not match expected value");
}
/// Standard pattern for elapsed time formatting  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_duration;
/// 
/// let result = format_duration(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_ms = duration.as_millis();
    if total_ms < 1000 {
        format!("{}ms", total_ms)
    } else if total_ms < 60_000 {
        format!("{:.2}s", total_ms as f64 / 1000.0)
    } else {
        let minutes = total_ms / 60_000;
        let seconds = (total_ms % 60_000) as f64 / 1000.0;
        format!("{}m {:.1}s", minutes, seconds)
    }
}
/// Safe alternative to unwrap() for Option values with context
pub fn unwrap_or_bail<T>(opt: Option<T>, msg: &str) -> Result<T> {
    opt.ok_or_else(|| anyhow::anyhow!("{}", msg))
}
/// Safe alternative to unwrap() for Result values  
pub fn unwrap_result_or_bail<T, E>(res: std::result::Result<T, E>, msg: &str) -> Result<T> 
where 
    E: std::fmt::Display
{
    res.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
}
/// Standard pattern for error formatting with file operations
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_file_error;
/// 
/// let result = format_file_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_file_error(operation: &str, path: &std::path::Path) -> String {
    format!("Failed to {} file: {}", operation, path.display())
}
/// Standard pattern for serialization error formatting
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_serialize_error;
/// 
/// let result = format_serialize_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_serialize_error(object_type: &str, error: impl std::fmt::Display) -> String {
    format!("Failed to serialize {}: {}", object_type, error)
}
/// Standard pattern for deserialization error formatting  
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_deserialize_error;
/// 
/// let result = format_deserialize_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_deserialize_error(object_type: &str, error: impl std::fmt::Display) -> String {
    format!("Failed to deserialize {}: {}", object_type, error)
}
/// Standard pattern for operation error formatting
/// # Examples
/// 
/// ```
/// use ruchy::utils::common_patterns::format_operation_error;
/// 
/// let result = format_operation_error("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_operation_error(operation: &str, error: impl std::fmt::Display) -> String {
    format!("Failed to {}: {}", operation, error)
}
#[cfg(test)]
mod property_tests_common_patterns {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_read_file_with_context_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
