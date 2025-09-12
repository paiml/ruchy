//! Comprehensive test suite for Common Patterns utility module
//! Aims to improve code coverage from 11% to significant coverage

use ruchy::utils::common_patterns::*;
use std::path::Path;
use anyhow::Result;

#[test]
fn test_read_file_with_context_success() {
    // Create a temp file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_read.txt");
    std::fs::write(&test_file, "test content").unwrap();
    
    let result = read_file_with_context(&test_file);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test content");
    
    // Clean up
    std::fs::remove_file(test_file).ok();
}

#[test]
fn test_read_file_with_context_failure() {
    let non_existent = Path::new("/non/existent/file.txt");
    let result = read_file_with_context(non_existent);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read file"));
}

#[test]
fn test_write_file_with_context_success() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_write.txt");
    
    let result = write_file_with_context(&test_file, "test content");
    assert!(result.is_ok());
    
    // Verify content was written
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "test content");
    
    // Clean up
    std::fs::remove_file(test_file).ok();
}

#[test]
fn test_write_file_with_context_failure() {
    let invalid_path = Path::new("/root/cannot_write_here.txt");
    let result = write_file_with_context(invalid_path, "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to write file"));
}

#[test]
fn test_parse_ruchy_code_valid() {
    let code = "42";
    let result = parse_ruchy_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_ruchy_code_expression() {
    let code = "1 + 2 * 3";
    let result = parse_ruchy_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_parse_ruchy_code_invalid() {
    let code = "let x = {"; // Unclosed brace
    let result = parse_ruchy_code(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse error"));
}

#[test]
fn test_create_success_response() {
    let response = create_success_response(
        "test_value".to_string(),
        "cell_123".to_string(),
        100.5
    );
    
    assert!(response.success);
    assert_eq!(response.cell_id, "cell_123");
    assert_eq!(response.value, "test_value");
    assert_eq!(response.execution_time_ms, 100.5);
    assert!(response.error.is_none());
}

#[test]
fn test_create_error_response() {
    let response = create_error_response(
        "Error occurred".to_string(),
        "cell_456".to_string()
    );
    
    assert!(!response.success);
    assert_eq!(response.cell_id, "cell_456");
    assert_eq!(response.error, Some("Error occurred".to_string()));
    assert_eq!(response.value, String::new());
    assert_eq!(response.execution_time_ms, 0.0);
}

#[test]
fn test_format_file_error() {
    let path = Path::new("/some/file.txt");
    let error = format_file_error("read", path);
    assert_eq!(error, "Failed to read file: /some/file.txt");
    
    let error = format_file_error("write", path);
    assert_eq!(error, "Failed to write file: /some/file.txt");
}

#[test]
fn test_format_module_error() {
    let error = format_module_error("load", "my_module");
    assert_eq!(error, "Failed to load module 'my_module'");
    
    let error = format_module_error("import", "external_lib");
    assert_eq!(error, "Failed to import module 'external_lib'");
}

#[test]
fn test_format_parse_error() {
    let error = format_parse_error("expression");
    assert_eq!(error, "Failed to parse expression");
    
    let error = format_parse_error("function definition");
    assert_eq!(error, "Failed to parse function definition");
}

#[test]
fn test_format_compile_error() {
    let error = format_compile_error("transpile to Rust");
    assert_eq!(error, "Failed to transpile to Rust");
    
    let error = format_compile_error("generate WASM");
    assert_eq!(error, "Failed to generate WASM");
}

#[test]
fn test_result_context_ext_file_operations() {
    // Test with Ok result
    let ok_result: Result<String> = Ok("success".to_string());
    let path = Path::new("/test/file.txt");
    let with_context = ok_result.file_context("read", path);
    assert!(with_context.is_ok());
    assert_eq!(with_context.unwrap(), "success");
    
    // Test with Err result
    let err_result: Result<String> = Err(anyhow::anyhow!("original error"));
    let with_context = err_result.file_context("write", path);
    assert!(with_context.is_err());
    let error_msg = with_context.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to write file"));
    assert!(error_msg.contains("/test/file.txt"));
}

#[test]
fn test_result_context_ext_module_operations() {
    // Test with Ok result
    let ok_result: Result<i32> = Ok(42);
    let with_context = ok_result.module_context("load", "test_module");
    assert!(with_context.is_ok());
    assert_eq!(with_context.unwrap(), 42);
    
    // Test with Err result
    let err_result: Result<i32> = Err(anyhow::anyhow!("module not found"));
    let with_context = err_result.module_context("import", "missing_module");
    assert!(with_context.is_err());
    let error_msg = with_context.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to import module"));
    assert!(error_msg.contains("missing_module"));
}

#[test]
fn test_result_context_ext_parse_operations() {
    // Test with Ok result
    let ok_result: Result<bool> = Ok(true);
    let with_context = ok_result.parse_context("statement");
    assert!(with_context.is_ok());
    assert!(with_context.unwrap());
    
    // Test with Err result
    let err_result: Result<bool> = Err(anyhow::anyhow!("syntax error"));
    let with_context = err_result.parse_context("complex expression");
    assert!(with_context.is_err());
    let error_msg = with_context.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));
    assert!(error_msg.contains("complex expression"));
}

#[test]
fn test_result_context_ext_compile_operations() {
    // Test with Ok result
    let ok_result: Result<Vec<u8>> = Ok(vec![1, 2, 3]);
    let with_context = ok_result.compile_context("optimize");
    assert!(with_context.is_ok());
    assert_eq!(with_context.unwrap(), vec![1, 2, 3]);
    
    // Test with Err result
    let err_result: Result<Vec<u8>> = Err(anyhow::anyhow!("optimization failed"));
    let with_context = err_result.compile_context("generate bytecode");
    assert!(with_context.is_err());
    let error_msg = with_context.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to generate bytecode"));
}

// Property-based tests
use quickcheck::{quickcheck, Arbitrary, Gen};

#[derive(Clone, Debug)]
struct ValidPath(String);

impl Arbitrary for ValidPath {
    fn arbitrary(g: &mut Gen) -> Self {
        let segments = vec!["src", "tests", "lib", "tmp"];
        let files = vec!["file", "test", "data", "config"];
        let extensions = vec!["txt", "rs", "ruchy", "json"];
        
        let segment = segments[usize::arbitrary(g) % segments.len()];
        let file = files[usize::arbitrary(g) % files.len()];
        let ext = extensions[usize::arbitrary(g) % extensions.len()];
        
        ValidPath(format!("/{}/{}.{}", segment, file, ext))
    }
}

#[test]
fn prop_format_file_error_consistent() {
    fn prop(path: ValidPath, operation: bool) -> bool {
        let op = if operation { "read" } else { "write" };
        let path = Path::new(&path.0);
        let error = format_file_error(op, path);
        
        error.starts_with("Failed to ") &&
        error.contains(op) &&
        error.contains(path.to_str().unwrap())
    }
    
    quickcheck(prop as fn(ValidPath, bool) -> bool);
}

#[test]
fn prop_format_module_error_consistent() {
    fn prop(module_name: String, operation: u8) -> bool {
        if module_name.is_empty() {
            return true; // Skip empty names
        }
        
        let ops = vec!["load", "import", "compile", "link"];
        let op = ops[operation as usize % ops.len()];
        let error = format_module_error(op, &module_name);
        
        error.starts_with("Failed to ") &&
        error.contains(op) &&
        error.contains(&module_name)
    }
    
    quickcheck(prop as fn(String, u8) -> bool);
}

#[test]
fn prop_format_parse_error_consistent() {
    fn prop(target: String) -> bool {
        if target.is_empty() {
            return true; // Skip empty targets
        }
        
        let error = format_parse_error(&target);
        error.starts_with("Failed to parse ") &&
        error.contains(&target)
    }
    
    quickcheck(prop as fn(String) -> bool);
}

#[test]
fn prop_format_compile_error_consistent() {
    fn prop(stage: String) -> bool {
        if stage.is_empty() {
            return true; // Skip empty stages
        }
        
        let error = format_compile_error(&stage);
        error.starts_with("Failed to ") &&
        error.contains(&stage)
    }
    
    quickcheck(prop as fn(String) -> bool);
}

#[test]
fn prop_success_response_fields() {
    fn prop(value: String, cell_id: String, time: f32) -> bool {
        let time = time.abs(); // Ensure positive time
        let response = create_success_response(value.clone(), cell_id.clone(), time as f64);
        
        response.success &&
        response.value == value &&
        response.cell_id == cell_id &&
        response.execution_time_ms == time as f64 &&
        response.error.is_none()
    }
    
    quickcheck(prop as fn(String, String, f32) -> bool);
}

#[test]
fn prop_error_response_fields() {
    fn prop(error: String, cell_id: String) -> bool {
        let response = create_error_response(error.clone(), cell_id.clone());
        
        !response.success &&
        response.error == Some(error) &&
        response.cell_id == cell_id &&
        response.value.is_empty() &&
        response.execution_time_ms == 0.0
    }
    
    quickcheck(prop as fn(String, String) -> bool);
}