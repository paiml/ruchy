//! Common patterns and utilities to reduce code entropy
//! Extracted to eliminate repetitive code patterns across the codebase
use anyhow::{Context, Result};
use std::path::Path;
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
    format!("Failed to {operation} module '{module_name}'")
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
    format!("Failed to parse {target}")
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
    format!("Failed to {stage}")
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
    name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') &&
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
    format!("=== {title} ===\n")
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
    format!("✅ {message}\n")
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
    format!("❌ {message}\n")
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
        None => print!("{content}"),
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
#[allow(clippy::match_like_matches_macro)]
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
            "Output does not contain '{expected}'. Actual output: '{output}'");
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
        format!("{total_ms}ms")
    } else if total_ms < 60_000 {
        format!("{:.2}s", total_ms as f64 / 1000.0)
    } else {
        let minutes = total_ms / 60_000;
        let seconds = (total_ms % 60_000) as f64 / 1000.0;
        format!("{minutes}m {seconds:.1}s")
    }
}
/// Safe alternative to `unwrap()` for Option values with context
pub fn unwrap_or_bail<T>(opt: Option<T>, msg: &str) -> Result<T> {
    opt.ok_or_else(|| anyhow::anyhow!("{}", msg))
}
/// Safe alternative to `unwrap()` for Result values  
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
    format!("Failed to serialize {object_type}: {error}")
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
    format!("Failed to deserialize {object_type}: {error}")
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
    format!("Failed to {operation}: {error}")
}

// String manipulation utilities
pub fn is_keyword(word: &str) -> bool {
    matches!(word, "let" | "if" | "else" | "while" | "for" | "return" | "fun" | "match" | "true" | "false" | "struct" | "enum" | "impl" | "trait" | "pub" | "mod" | "use" | "type" | "const" | "static" | "async" | "await" | "break" | "continue" | "loop" | "in" | "ref" | "mut" | "self" | "super" | "crate" | "where" | "as" | "fn")
}

pub fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\'' => result.push_str("\\'"),
            _ => result.push(ch),
        }
    }
    result
}

pub fn unescape_string(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(c) => return Err(anyhow::anyhow!("Invalid escape sequence: \\{}", c)),
                None => return Err(anyhow::anyhow!("Incomplete escape sequence")),
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}

pub fn capitalize(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            if first.is_alphabetic() {
                first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect()
            } else {
                s.to_string()
            }
        }
    }
}

pub fn snake_to_camel(s: &str) -> String {
    if s.starts_with('_') || s.is_empty() {
        return s.to_string();
    }
    
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push_str(&ch.to_uppercase().to_string());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    
    // Don't lose trailing underscores
    if s.ends_with('_') {
        result.push('_');
    }
    
    result
}

pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push_str(&ch.to_lowercase().to_string());
            prev_was_upper = true;
        } else {
            result.push(ch);
            prev_was_upper = false;
        }
    }
    
    result
}

pub fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

pub fn is_float(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let (before, after) = (parts[0], parts[1]);
    
    (before.is_empty() || before.chars().all(|c| c.is_ascii_digit())) &&
    (after.is_empty() || after.chars().all(|c| c.is_ascii_digit()))
}

pub fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                // Don't strip if it's part of a URL
                if pos > 0 && line[..pos].contains("http") {
                    line.to_string()
                } else {
                    line[..pos].to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        0
    } else {
        s.lines().count() + s.chars().filter(|&c| c == '\n').count().saturating_sub(s.lines().count() - 1)
    }
}

pub fn indent_string(s: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    s.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn trim_indent(s: &str) -> String {
    s.lines()
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn split_at_delimiter(s: &str, delimiter: char) -> Vec<String> {
    s.split(delimiter).map(std::string::ToString::to_string).collect()
}

pub fn common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    
    let mut prefix = String::new();
    let first = strings[0];
    
    for (i, ch) in first.chars().enumerate() {
        if strings.iter().all(|s| s.chars().nth(i) == Some(ch)) {
            prefix.push(ch);
        } else {
            break;
        }
    }
    
    prefix
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    #[allow(clippy::needless_range_loop)]
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, ch1) in s1.chars().enumerate() {
        for (j, ch2) in s2.chars().enumerate() {
            let cost = usize::from(ch1 != ch2);
            matrix[i + 1][j + 1] = std::cmp::min(
                matrix[i][j + 1] + 1,
                std::cmp::min(
                    matrix[i + 1][j] + 1,
                    matrix[i][j] + cost
                )
            );
        }
    }
    
    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_read_file_with_context() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let result = read_file_with_context(&file_path).unwrap();
        assert_eq!(result, "test content");

        // Test non-existent file
        let result = read_file_with_context(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_file_with_context() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        write_file_with_context(&file_path, "test content").unwrap();
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_parse_ruchy_code() {
        let result = parse_ruchy_code("42");
        assert!(result.is_ok());

        let result = parse_ruchy_code("let x = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_success_response() {
        let response = create_success_response(
            "value".to_string(),
            "cell_1".to_string(),
            100.5,
        );
        assert!(response.success);
        assert_eq!(response.cell_id, "cell_1");
        assert_eq!(response.value, "value");
        assert_eq!(response.result, "value");
        assert!(response.error.is_none());
        assert_eq!(response.execution_time_ms, 100.5);
    }

    #[test]
    fn test_create_error_response() {
        let response = create_error_response(
            "error message".to_string(),
            "cell_2".to_string(),
        );
        assert!(!response.success);
        assert_eq!(response.cell_id, "cell_2");
        assert!(response.value.is_empty());
        assert!(response.result.is_empty());
        assert_eq!(response.error, Some("error message".to_string()));
        assert_eq!(response.execution_time_ms, 0.0);
    }

    #[test]
    fn test_format_module_error() {
        let msg = format_module_error("load", "example");
        assert_eq!(msg, "Failed to load module 'example'");
    }

    #[test]
    fn test_format_parse_error() {
        let msg = format_parse_error("expression");
        assert_eq!(msg, "Failed to parse expression");
    }

    #[test]
    fn test_format_compile_error() {
        let msg = format_compile_error("transpile to Rust");
        assert_eq!(msg, "Failed to transpile to Rust");
    }

    #[test]
    fn test_time_operation() {
        let (result, elapsed) = time_operation(|| 42);
        assert_eq!(result, 42);
        assert!(elapsed >= 0.0);
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("valid_name"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("name123"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123name"));
        assert!(!is_valid_identifier("name-with-dash"));
    }

    #[test]
    fn test_create_section_header() {
        let header = create_section_header("Test Section");
        assert_eq!(header, "=== Test Section ===\n");
    }

    #[test]
    fn test_add_success_indicator() {
        let msg = add_success_indicator("Operation completed");
        assert_eq!(msg, "✅ Operation completed\n");
    }

    #[test]
    fn test_add_error_indicator() {
        let msg = add_error_indicator("Operation failed");
        assert_eq!(msg, "❌ Operation failed\n");
    }

    #[test]
    fn test_write_output_or_print() {
        // Test with output file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output.txt");
        write_output_or_print("test output".to_string(), Some(&file_path)).unwrap();
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test output");

        // Test without output file (just shouldn't panic)
        write_output_or_print("test output".to_string(), None).unwrap();
    }

    #[test]
    fn test_progress_indicator() {
        let mut indicator = ProgressIndicator::new(100, "Processing".to_string());
        assert_eq!(indicator.total, 100);
        assert_eq!(indicator.current, 0);
        assert_eq!(indicator.label, "Processing");

        indicator.increment();
        assert_eq!(indicator.current, 1);

        indicator.finish();
    }

    #[test]
    fn test_retry_operation() {
        let mut count = 0;
        let result = retry_operation(
            || {
                count += 1;
                if count < 3 {
                    Err("not yet")
                } else {
                    Ok(42)
                }
            },
            5,
        );
        assert_eq!(result, Ok(42));

        let result = retry_operation(|| Err::<i32, &str>("always fails"), 3);
        assert_eq!(result, Err("always fails"));
    }

    #[test]
    fn test_check_feature_enabled() {
        // These depend on compile-time features
        let _ = check_feature_enabled("notebook");
        let _ = check_feature_enabled("wasm-compile");
        assert!(!check_feature_enabled("unknown_feature"));
    }

    #[test]
    fn test_format_memory_size() {
        assert_eq!(format_memory_size(0), "0 B");
        assert_eq!(format_memory_size(512), "512 B");
        assert_eq!(format_memory_size(1024), "1.00 KB");
        assert_eq!(format_memory_size(1536), "1.50 KB");
        assert_eq!(format_memory_size(1048576), "1.00 MB");
        assert_eq!(format_memory_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_version_info() {
        let version = format_version_info();
        assert!(version.contains("Ruchy"));
        assert!(version.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_format_duration() {
        use std::time::Duration;

        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
        assert_eq!(format_duration(Duration::from_millis(65000)), "1m 5.0s");
    }

    #[test]
    fn test_unwrap_or_bail() {
        let result = unwrap_or_bail(Some(42), "error");
        assert_eq!(result.unwrap(), 42);

        let result = unwrap_or_bail::<i32>(None, "value not found");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("value not found"));
    }

    #[test]
    fn test_unwrap_result_or_bail() {
        let result = unwrap_result_or_bail(Ok::<i32, &str>(42), "error");
        assert_eq!(result.unwrap(), 42);

        let result = unwrap_result_or_bail(Err::<i32, _>("failure"), "operation failed");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("operation failed"));
    }

    #[test]
    fn test_format_file_error() {
        let msg = format_file_error("read", Path::new("/path/to/file.txt"));
        assert!(msg.contains("Failed to read file"));
        assert!(msg.contains("/path/to/file.txt"));
    }

    #[test]
    fn test_format_serialize_error() {
        let msg = format_serialize_error("user object", "invalid JSON");
        assert_eq!(msg, "Failed to serialize user object: invalid JSON");
    }

    #[test]
    fn test_format_deserialize_error() {
        let msg = format_deserialize_error("config", "missing field");
        assert_eq!(msg, "Failed to deserialize config: missing field");
    }

    #[test]
    fn test_format_operation_error() {
        let msg = format_operation_error("connect to server", "timeout");
        assert_eq!(msg, "Failed to connect to server: timeout");
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("let"));
        assert!(is_keyword("if"));
        assert!(is_keyword("async"));
        assert!(!is_keyword("variable"));
        assert!(!is_keyword(""));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("tab\there"), "tab\\there");
        assert_eq!(escape_string("quote\"test"), "quote\\\"test");
        assert_eq!(escape_string("normal text"), "normal text");
    }

    #[test]
    fn test_unescape_string() {
        assert_eq!(unescape_string("hello\\nworld").unwrap(), "hello\nworld");
        assert_eq!(unescape_string("tab\\there").unwrap(), "tab\there");
        assert_eq!(unescape_string("quote\\\"").unwrap(), "quote\"");
        assert_eq!(unescape_string("normal").unwrap(), "normal");

        // Test invalid escape
        assert!(unescape_string("\\x").is_err());
        assert!(unescape_string("incomplete\\").is_err());
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("WORLD"), "World");
        assert_eq!(capitalize("123abc"), "123abc");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(snake_to_camel("foo_bar_baz"), "fooBarBaz");
        assert_eq!(snake_to_camel("single"), "single");
        assert_eq!(snake_to_camel("_private"), "_private");
        assert_eq!(snake_to_camel(""), "");
        assert_eq!(snake_to_camel("trailing_"), "trailing_");
    }

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("helloWorld"), "hello_world");
        assert_eq!(camel_to_snake("fooBarBaz"), "foo_bar_baz");
        assert_eq!(camel_to_snake("single"), "single");
        assert_eq!(camel_to_snake("HTTPServer"), "httpserver");
    }

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric("123"));
        assert!(is_numeric("0"));
        assert!(!is_numeric("12.3"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric(""));
    }

    #[test]
    fn test_is_float() {
        assert!(is_float("12.34"));
        assert!(is_float("0.0"));
        assert!(is_float(".5"));
        assert!(is_float("5."));
        assert!(!is_float("123"));
        assert!(!is_float("12.34.56"));
        assert!(!is_float(""));
    }

    #[test]
    fn test_strip_comments() {
        assert_eq!(strip_comments("code // comment"), "code ");
        assert_eq!(strip_comments("line1\n// comment\nline2"), "line1\n\nline2");
        assert_eq!(strip_comments("http://example.com"), "http://example.com");
        assert_eq!(strip_comments("no comments"), "no comments");
    }

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("single line"), 1);
        assert_eq!(count_lines("line1\nline2\nline3"), 3);
        assert_eq!(count_lines("line1\nline2\n"), 3);
    }

    #[test]
    fn test_indent_string() {
        assert_eq!(indent_string("hello", 2), "  hello");
        assert_eq!(indent_string("line1\nline2", 4), "    line1\n    line2");
        assert_eq!(indent_string("\n", 2), "");
    }

    #[test]
    fn test_trim_indent() {
        assert_eq!(trim_indent("  hello"), "hello");
        assert_eq!(trim_indent("    line1\n    line2"), "line1\nline2");
        assert_eq!(trim_indent("mixed  \n  indent"), "mixed  \nindent");
    }

    #[test]
    fn test_split_at_delimiter() {
        assert_eq!(
            split_at_delimiter("a,b,c", ','),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            split_at_delimiter("single", ','),
            vec!["single"]
        );
    }

    #[test]
    fn test_common_prefix() {
        assert_eq!(
            common_prefix(&["prefix_a", "prefix_b", "prefix_c"]),
            "prefix_"
        );
        assert_eq!(
            common_prefix(&["hello", "help", "hero"]),
            "he"
        );
        assert_eq!(
            common_prefix(&["abc", "xyz"]),
            ""
        );
        assert_eq!(common_prefix(&[]), "");
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(levenshtein_distance("same", "same"), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_result_context_ext() {
        let result: Result<i32, std::io::Error> = Ok(42);
        let with_context = result.file_context("read", Path::new("/test.txt"));
        assert_eq!(with_context.unwrap(), 42);

        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let result: Result<i32, std::io::Error> = Err(error);
        let with_context = result.module_context("load", "example");
        assert!(with_context.is_err());
    }

    #[test]
    fn test_assert_output_contains() {
        assert_output_contains("hello world", "hello");
        assert_output_contains("hello world", "world");
    }

    #[test]
    fn test_assert_output_equals() {
        assert_output_equals("exact match", "exact match");
        assert_output_equals(42, "42");
    }
}
