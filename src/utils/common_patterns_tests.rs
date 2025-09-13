//! Comprehensive tests for Common Patterns utilities
//! Target: Increase coverage for utils/common_patterns
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod common_patterns_tests {
    use crate::utils::common_patterns::*;
    use std::path::Path;
    use tempfile::TempDir;
    use std::fs;
    
    // ========== File Operation Tests ==========
    
    #[test]
    fn test_read_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        
        let result = read_file_with_context(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }
    
    #[test]
    fn test_read_file_with_context_failure() {
        let non_existent = Path::new("/nonexistent/file.txt");
        let result = read_file_with_context(non_existent);
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read file"));
    }
    
    #[test]
    fn test_write_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");
        
        let result = write_file_with_context(&file_path, "test output");
        assert!(result.is_ok());
        
        // Verify file was written
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test output");
    }
    
    #[test]
    fn test_write_file_with_context_failure() {
        let invalid_path = Path::new("/invalid/directory/file.txt");
        let result = write_file_with_context(invalid_path, "content");
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to write file"));
    }
    
    // ========== Parse Tests ==========
    
    #[test]
    fn test_parse_ruchy_code_success() {
        let code = "42";
        let result = parse_ruchy_code(code);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_ruchy_code_failure() {
        let invalid_code = "let x = ]invalid[";
        let result = parse_ruchy_code(invalid_code);
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Parse error"));
    }
    
    // ========== Response Creation Tests ==========
    
    #[test]
    fn test_create_success_response() {
        let response = create_success_response(
            "result_value".to_string(),
            "cell_123".to_string(),
            42.5
        );
        
        assert!(response.success);
        assert_eq!(response.cell_id, "cell_123");
        assert_eq!(response.value, "result_value");
        assert_eq!(response.result, "result_value");
        assert!(response.error.is_none());
        assert_eq!(response.execution_time_ms, 42.5);
    }
    
    #[test]
    fn test_create_error_response() {
        let response = create_error_response(
            "error message".to_string(),
            "cell_456".to_string()
        );
        
        assert!(!response.success);
        assert_eq!(response.cell_id, "cell_456");
        assert_eq!(response.value, "");
        assert_eq!(response.result, "");
        assert_eq!(response.error, Some("error message".to_string()));
        assert_eq!(response.execution_time_ms, 0.0);
    }
    
    // ========== Format Error Tests ==========
    
    #[test]
    fn test_format_module_error() {
        let error = format_module_error("load", "my_module");
        assert_eq!(error, "Failed to load module 'my_module'");
        
        let error2 = format_module_error("compile", "test_mod");
        assert_eq!(error2, "Failed to compile module 'test_mod'");
    }
    
    #[test]
    fn test_format_parse_error() {
        let error = format_parse_error("expression");
        assert_eq!(error, "Failed to parse expression");
        
        let error2 = format_parse_error("statement");
        assert_eq!(error2, "Failed to parse statement");
    }
    
    #[test]
    fn test_format_compile_error() {
        let error = format_compile_error("type checking");
        assert_eq!(error, "Failed to type checking");
        
        let error2 = format_compile_error("code generation");
        assert_eq!(error2, "Failed to code generation");
    }
    
    // ========== ResultContextExt Tests ==========
    
    #[test]
    fn test_result_context_file() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test error"
        ));
        
        let with_context = result.file_context("read", Path::new("test.txt"));
        assert!(with_context.is_err());
        
        let err_msg = with_context.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read file"));
        assert!(err_msg.contains("test.txt"));
    }
    
    #[test]
    fn test_result_context_module() {
        let result: Result<(), anyhow::Error> = Err(anyhow::anyhow!("module error"));
        
        let with_context = result.module_context("import", "my_module");
        assert!(with_context.is_err());
        
        let err_msg = with_context.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to import module 'my_module'"));
    }
    
    #[test]
    fn test_result_context_parse() {
        let result: Result<(), anyhow::Error> = Err(anyhow::anyhow!("parse error"));
        
        let with_context = result.parse_context("function body");
        assert!(with_context.is_err());
        
        let err_msg = with_context.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse function body"));
    }
    
    #[test]
    fn test_result_context_compile() {
        let result: Result<(), anyhow::Error> = Err(anyhow::anyhow!("compile error"));
        
        let with_context = result.compile_context("optimization");
        assert!(with_context.is_err());
        
        let err_msg = with_context.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to optimization"));
    }
    
    // ========== Time Operation Tests ==========
    
    #[test]
    fn test_time_operation() {
        let (result, elapsed) = time_operation(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(elapsed >= 10.0); // At least 10ms
        assert!(elapsed < 100.0); // But not too long
    }
    
    #[test]
    fn test_time_operation_fast() {
        let (result, elapsed) = time_operation(|| {
            2 + 2
        });
        
        assert_eq!(result, 4);
        assert!(elapsed >= 0.0);
        assert!(elapsed < 10.0); // Should be very fast
    }
    
    // ========== Identifier Validation Tests ==========
    
    #[test]
    fn test_is_valid_identifier() {
        // Valid identifiers
        assert!(is_valid_identifier("valid"));
        assert!(is_valid_identifier("_underscore"));
        assert!(is_valid_identifier("with123numbers"));
        assert!(is_valid_identifier("camelCase"));
        assert!(is_valid_identifier("snake_case"));
        assert!(is_valid_identifier("CONSTANT"));
        
        // Invalid identifiers
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123start"));
        assert!(!is_valid_identifier("with-dash"));
        assert!(!is_valid_identifier("with space"));
        assert!(!is_valid_identifier("with.dot"));
        assert!(!is_valid_identifier("emoji😀"));
    }
    
    // ========== Output Formatting Tests ==========
    
    #[test]
    fn test_create_section_header() {
        let header = create_section_header("Test Section");
        assert_eq!(header, "=== Test Section ===\n");
        
        let header2 = create_section_header("Another");
        assert_eq!(header2, "=== Another ===\n");
    }
    
    #[test]
    fn test_add_success_indicator() {
        let message = add_success_indicator("Operation completed");
        assert_eq!(message, "✅ Operation completed\n");
        
        let message2 = add_success_indicator("All tests passed");
        assert_eq!(message2, "✅ All tests passed\n");
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_format_errors_never_panic(
            operation in "[a-z]+",
            name in "[a-zA-Z0-9_]+",
            stage in ".*"
        ) {
            let _ = format_module_error(&operation, &name);
            let _ = format_parse_error(&name);
            let _ = format_compile_error(&stage);
        }
        
        #[test]
        fn test_response_creation_never_panics(
            value in ".*",
            cell_id in "[a-z0-9-]+",
            time in 0.0f64..10000.0,
            error in ".*"
        ) {
            let _ = create_success_response(value.clone(), cell_id.clone(), time);
            let _ = create_error_response(error, cell_id);
        }
        
        #[test]
        fn test_identifier_validation_properties(s in ".*") {
            let valid = is_valid_identifier(&s);
            
            if valid {
                // If valid, must start with letter or underscore
                assert!(s.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_'));
                // All chars must be alphanumeric or underscore
                assert!(s.chars().all(|c| c.is_alphanumeric() || c == '_'));
            }
        }
        
        #[test]
        fn test_section_header_properties(title in "[a-zA-Z0-9 ]+") {
            let header = create_section_header(&title);
            assert!(header.starts_with("==="));
            assert!(header.ends_with("===\n"));
            assert!(header.contains(&title));
        }
    }
}