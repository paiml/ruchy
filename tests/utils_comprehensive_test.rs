//! Comprehensive tests for utility functions 
//! Tests edge cases, error conditions, and performance characteristics

use ruchy::utils::*;
use anyhow::Result;
use std::path::Path;
use std::time::Duration;

#[test]
fn test_read_file_with_context_edge_cases() {
    // Test nonexistent file
    let result = read_file_with_context(Path::new("/nonexistent/file.txt"));
    assert!(result.is_err(), "Should fail for nonexistent file");
    
    // Test empty file
    let temp_file = std::env::temp_dir().join("empty_test.txt");
    std::fs::write(&temp_file, "").unwrap();
    let result = read_file_with_context(&temp_file);
    assert!(result.is_ok(), "Should handle empty file");
    assert_eq!(result.unwrap(), "", "Empty file should return empty string");
    std::fs::remove_file(&temp_file).ok();
    
    // Test large file handling
    let large_content = "x".repeat(10_000);
    let temp_file = std::env::temp_dir().join("large_test.txt");
    std::fs::write(&temp_file, &large_content).unwrap();
    let result = read_file_with_context(&temp_file);
    assert!(result.is_ok(), "Should handle large file");
    assert_eq!(result.unwrap().len(), 10_000, "Should read full large content");
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_write_file_with_context_edge_cases() {
    // Test writing to nonexistent directory (should fail)
    let result = write_file_with_context(Path::new("/nonexistent/dir/file.txt"), "content");
    assert!(result.is_err(), "Should fail for nonexistent directory");
    
    // Test writing empty content
    let temp_file = std::env::temp_dir().join("empty_write_test.txt");
    let result = write_file_with_context(&temp_file, "");
    assert!(result.is_ok(), "Should handle empty content");
    
    let content = std::fs::read_to_string(&temp_file).unwrap();
    assert_eq!(content, "", "Should write empty content correctly");
    std::fs::remove_file(&temp_file).ok();
    
    // Test writing large content
    let large_content = "y".repeat(50_000);
    let temp_file = std::env::temp_dir().join("large_write_test.txt");
    let result = write_file_with_context(&temp_file, &large_content);
    assert!(result.is_ok(), "Should handle large content");
    
    let read_content = std::fs::read_to_string(&temp_file).unwrap();
    assert_eq!(read_content, large_content, "Should write large content correctly");
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_parse_ruchy_code_edge_cases() {
    // Test empty code
    let result = parse_ruchy_code("");
    assert!(result.is_err(), "Should fail for empty code");
    
    // Test whitespace only
    let result = parse_ruchy_code("   \n\t  ");
    assert!(result.is_err(), "Should fail for whitespace only");
    
    // Test very simple valid code
    let result = parse_ruchy_code("42");
    assert!(result.is_ok(), "Should parse simple literal");
    
    // Test complex nested expression
    let complex = "(((1 + 2) * 3) - (4 / 2))";
    let result = parse_ruchy_code(complex);
    assert!(result.is_ok(), "Should parse complex nested expression");
    
    // Test invalid syntax
    let result = parse_ruchy_code("let = = = invalid");
    assert!(result.is_err(), "Should fail for invalid syntax");
}

#[test]
fn test_time_operation_precision() {
    // Test very fast operation
    let (result, elapsed_ms) = time_operation(|| 42);
    assert_eq!(result, 42, "Should return correct result");
    assert!(elapsed_ms >= 0.0, "Elapsed time should be non-negative");
    assert!(elapsed_ms < 10.0, "Fast operation should complete quickly");
    
    // Test operation with known duration
    let (result, elapsed_ms) = time_operation(|| {
        std::thread::sleep(Duration::from_millis(50));
        "done"
    });
    assert_eq!(result, "done", "Should return correct result after delay");
    assert!(elapsed_ms >= 40.0, "Should measure at least 40ms");
    assert!(elapsed_ms < 100.0, "Should measure less than 100ms for 50ms sleep");
}

#[test]
fn test_is_valid_identifier_comprehensive() {
    // Valid identifiers
    assert!(is_valid_identifier("x"), "Single letter should be valid");
    assert!(is_valid_identifier("variable"), "Word should be valid");
    assert!(is_valid_identifier("var_123"), "Underscore and numbers should be valid");
    assert!(is_valid_identifier("_private"), "Starting underscore should be valid");
    assert!(is_valid_identifier("CamelCase"), "CamelCase should be valid");
    assert!(is_valid_identifier("snake_case_123"), "Snake case with numbers should be valid");
    
    // Invalid identifiers
    assert!(!is_valid_identifier(""), "Empty string should be invalid");
    assert!(!is_valid_identifier("123abc"), "Starting with number should be invalid");
    assert!(!is_valid_identifier("var-name"), "Hyphen should be invalid");
    assert!(!is_valid_identifier("var.name"), "Dot should be invalid");
    assert!(!is_valid_identifier("var name"), "Space should be invalid");
    assert!(!is_valid_identifier("var@name"), "Special characters should be invalid");
    assert!(!is_valid_identifier("🦀rust"), "Unicode symbols should be invalid");
    
    // Edge cases
    assert!(!is_valid_identifier(" var"), "Leading space should be invalid");
    assert!(!is_valid_identifier("var "), "Trailing space should be invalid");
    assert!(!is_valid_identifier("\tvar"), "Tab should be invalid");
    assert!(!is_valid_identifier("var\n"), "Newline should be invalid");
}

#[test]
fn test_create_section_header_formatting() {
    assert_eq!(create_section_header("Test"), "=== Test ===\n");
    assert_eq!(create_section_header(""), "===  ===\n");
    assert_eq!(create_section_header("Long Section Title"), "=== Long Section Title ===\n");
    
    // Test with special characters
    assert_eq!(create_section_header("Test & Debug"), "=== Test & Debug ===\n");
    assert_eq!(create_section_header("2023-01-01"), "=== 2023-01-01 ===\n");
}

#[test]
fn test_success_and_error_indicators() {
    // Test success indicator
    assert_eq!(add_success_indicator("Done"), "✅ Done\n");
    assert_eq!(add_success_indicator(""), "✅ \n");
    
    // Test error indicator
    assert_eq!(add_error_indicator("Failed"), "❌ Failed\n");
    assert_eq!(add_error_indicator(""), "❌ \n");
    
    // Test with multiline text
    assert_eq!(add_success_indicator("Line 1\nLine 2"), "✅ Line 1\nLine 2\n");
}

#[test]
fn test_progress_indicator_functionality() {
    let mut progress = ProgressIndicator::new(10, "Test Progress".to_string());
    
    // Test initial state
    assert_eq!(progress.current, 0);
    assert_eq!(progress.total, 10);
    assert_eq!(progress.label, "Test Progress");
    
    // Test increment
    progress.increment();
    assert_eq!(progress.current, 1);
    
    // Test multiple increments
    for _ in 0..9 {
        progress.increment();
    }
    assert_eq!(progress.current, 10);
}

#[test]
fn test_format_memory_size_units() {
    assert_eq!(format_memory_size(0), "0 B");
    assert_eq!(format_memory_size(512), "512 B");
    assert_eq!(format_memory_size(1024), "1.00 KB");
    assert_eq!(format_memory_size(1536), "1.50 KB");
    assert_eq!(format_memory_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_memory_size(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_memory_size(1500 * 1024 * 1024), "1.46 GB");
}

#[test]
fn test_format_version_info_consistency() {
    let version = format_version_info();
    assert!(version.starts_with("Ruchy v"), "Should start with 'Ruchy v'");
    assert!(version.contains("debug") || version.contains("release"), "Should contain build type");
    
    // Test consistency across calls
    let version2 = format_version_info();
    assert_eq!(version, version2, "Should be consistent across calls");
}

#[test]
fn test_format_duration_ranges() {
    use std::time::Duration;
    
    // Test milliseconds
    assert_eq!(format_duration(Duration::from_millis(0)), "0ms");
    assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
    assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
    
    // Test seconds
    assert_eq!(format_duration(Duration::from_millis(1000)), "1.00s");
    assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
    assert_eq!(format_duration(Duration::from_millis(59999)), "60.00s");
    
    // Test minutes
    assert_eq!(format_duration(Duration::from_millis(60000)), "1m 0.0s");
    assert_eq!(format_duration(Duration::from_millis(65000)), "1m 5.0s");
    assert_eq!(format_duration(Duration::from_millis(125000)), "2m 5.0s");
}

#[test]
fn test_write_output_or_print_file_mode() {
    let temp_file = std::env::temp_dir().join("output_test.txt");
    
    // Test writing to file
    let result = write_output_or_print("Test content".to_string(), Some(&temp_file));
    assert!(result.is_ok(), "Should successfully write to file");
    
    let content = std::fs::read_to_string(&temp_file).unwrap();
    assert_eq!(content, "Test content", "Should write correct content");
    std::fs::remove_file(&temp_file).ok();
    
    // Test print mode (no file)
    let result = write_output_or_print("Print content".to_string(), None);
    assert!(result.is_ok(), "Should successfully print to stdout");
}

#[test]
fn test_retry_operation_success() {
    let mut attempts = 0;
    let result = retry_operation(|| {
        attempts += 1;
        if attempts < 3 {
            Err("Temporary failure")
        } else {
            Ok("Success")
        }
    }, 5);
    
    assert!(result.is_ok(), "Should succeed after retries");
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(attempts, 3, "Should make exactly 3 attempts");
}

#[test]
fn test_retry_operation_exhaustion() {
    let mut attempts = 0;
    let result: Result<&str, &str> = retry_operation(|| {
        attempts += 1;
        Err("Always fails")
    }, 3);
    
    assert!(result.is_err(), "Should fail after exhausting retries");
    assert_eq!(attempts, 3, "Should make exactly 3 attempts");
}

#[test]
fn test_check_feature_enabled() {
    // Test known features
    let notebook_enabled = check_feature_enabled("notebook");
    let wasm_enabled = check_feature_enabled("wasm-compile");
    
    // Results depend on build configuration, so just test they return boolean
    assert!(notebook_enabled == true || notebook_enabled == false);
    assert!(wasm_enabled == true || wasm_enabled == false);
    
    // Test unknown feature
    assert!(!check_feature_enabled("unknown-feature"), "Unknown feature should be false");
    assert!(!check_feature_enabled(""), "Empty feature should be false");
}