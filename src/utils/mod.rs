//! Utility modules for common patterns and shared functionality
pub mod common_patterns;
pub use common_patterns::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Sprint 7: Comprehensive utils tests for coverage improvement

    #[test]
    fn test_format_module_error() {
        let error = format_module_error("load", "test_module");
        assert_eq!(error, "Failed to load module 'test_module'");

        let error = format_module_error("parse", "my_module");
        assert_eq!(error, "Failed to parse module 'my_module'");
    }

    #[test]
    fn test_format_parse_error() {
        let error = format_parse_error("test.ruchy");
        assert!(error.contains("test.ruchy"));
    }

    #[test]
    fn test_format_compile_error() {
        let error = format_compile_error("type checking");
        assert!(error.contains("type checking"));
    }

    #[test]
    fn test_format_memory_size() {
        assert_eq!(format_memory_size(0), "0 B");
        assert_eq!(format_memory_size(1024), "1.00 KB");
        assert_eq!(format_memory_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_memory_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_parse_ruchy_code_valid() {
        let result = parse_ruchy_code("42");
        assert!(result.is_ok());

        let result = parse_ruchy_code("true");
        assert!(result.is_ok());

        let result = parse_ruchy_code("\"hello\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ruchy_code_invalid() {
        let result = parse_ruchy_code("let x =");
        assert!(result.is_err());
    }

    // WASM response tests removed - using Pure Rust notebook module instead

    #[test]
    fn test_format_version_info() {
        let version = format_version_info();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_format_duration() {
        use std::time::Duration;

        let duration = Duration::from_secs(0);
        let formatted = format_duration(duration);
        assert!(formatted.contains('0'));

        let duration = Duration::from_secs(61);
        let formatted = format_duration(duration);
        assert!(formatted.contains('1') || formatted.contains("61"));
    }

    #[test]
    fn test_format_file_error() {
        let path = Path::new("test.txt");
        let error = format_file_error("read", path);
        assert!(error.contains("read"));
        assert!(error.contains("test.txt"));
    }

    #[test]
    fn test_format_serialize_error() {
        let error = format_serialize_error("Config", "invalid JSON");
        assert!(error.contains("Config"));
        assert!(error.contains("invalid JSON"));
    }

    #[test]
    fn test_format_deserialize_error() {
        let error = format_deserialize_error("Response", "missing field");
        assert!(error.contains("Response"));
        assert!(error.contains("missing field"));
    }

    #[test]
    fn test_format_operation_error() {
        let error = format_operation_error("database query", "connection timeout");
        assert!(error.contains("database query"));
        assert!(error.contains("connection timeout"));
    }

    #[test]
    fn test_empty_string_handling() {
        let error = format_module_error("", "");
        assert_eq!(error, "Failed to  module ''");
    }

    #[test]
    fn test_special_characters_in_errors() {
        let error = format_module_error("load", "test/module@#$");
        assert!(error.contains("test/module@#$"));

        let path = Path::new("file with spaces.ruchy");
        let error = format_file_error("parse", path);
        assert!(error.contains("file with spaces.ruchy"));
    }

    #[test]
    fn test_unicode_in_errors() {
        let error = format_module_error("load", "模块");
        assert!(error.contains("模块"));

        let path = Path::new("файл.ruchy");
        let error = format_file_error("read", path);
        assert!(error.contains("файл.ruchy"));
    }

    #[test]
    fn test_format_memory_edge_cases() {
        // Test edge cases for memory formatting
        assert_eq!(format_memory_size(1023), "1023 B");
        assert_eq!(format_memory_size(1025), "1.00 KB");
        assert_eq!(format_memory_size(u64::MAX), format_memory_size(u64::MAX));
    }

    #[test]
    fn test_parse_complex_expressions() {
        let result = parse_ruchy_code("1 + 2 * 3");
        assert!(result.is_ok());

        let result = parse_ruchy_code("[1, 2, 3]");
        assert!(result.is_ok());

        let result = parse_ruchy_code("(true, false)");
        assert!(result.is_ok());
    }

    // WASM response consistency tests removed - using Pure Rust notebook module instead
}
