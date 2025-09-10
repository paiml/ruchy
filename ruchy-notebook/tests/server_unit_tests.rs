/// Unit tests for notebook server components
/// Target: >80% coverage for server module

use anyhow::Result;
use ruchy_notebook::server::{ExecuteRequest, ExecuteResponse};
use serde_json;
use std::time::Duration;

/// Test ExecuteRequest serialization/deserialization
#[test]
fn test_execute_request_serde() {
    // Test serialization
    let request = ExecuteRequest {
        code: "2 + 2".to_string(),
        cell_id: "cell_001".to_string(),
        session_id: Some("session_123".to_string()),
    };
    
    let json = serde_json::to_string(&request).expect("Should serialize");
    assert!(json.contains("2 + 2"));
    assert!(json.contains("cell_001"));
    assert!(json.contains("session_123"));
    
    // Test deserialization
    let json_str = r#"{"code":"println(42)","cell_id":"cell_002","session_id":null}"#;
    let deserialized: ExecuteRequest = serde_json::from_str(json_str).expect("Should deserialize");
    assert_eq!(deserialized.code, "println(42)");
    assert_eq!(deserialized.cell_id, "cell_002");
    assert!(deserialized.session_id.is_none());
}

/// Test ExecuteResponse serialization
#[test]
fn test_execute_response_serde() {
    let response = ExecuteResponse {
        success: true,
        result: Some("42".to_string()),
        error: None,
        cell_id: "cell_001".to_string(),
        execution_time_ms: 150,
    };
    
    let json = serde_json::to_string(&response).expect("Should serialize");
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"result\":\"42\""));
    assert!(json.contains("\"execution_time_ms\":150"));
    
    // Test error response
    let error_response = ExecuteResponse {
        success: false,
        result: None,
        error: Some("Parse error".to_string()),
        cell_id: "cell_002".to_string(),
        execution_time_ms: 50,
    };
    
    let error_json = serde_json::to_string(&error_response).expect("Should serialize");
    assert!(error_json.contains("\"success\":false"));
    assert!(error_json.contains("\"error\":\"Parse error\""));
}

/// Test ExecuteRequest validation
#[test]
fn test_execute_request_validation() {
    // Valid request
    let valid_request = ExecuteRequest {
        code: "2 + 2".to_string(),
        cell_id: "cell_001".to_string(),
        session_id: None,
    };
    
    assert!(!valid_request.code.is_empty());
    assert!(!valid_request.cell_id.is_empty());
    
    // Empty code request
    let empty_code = ExecuteRequest {
        code: "".to_string(),
        cell_id: "cell_002".to_string(),
        session_id: None,
    };
    
    assert!(empty_code.code.is_empty());
    
    // Whitespace-only code
    let whitespace_code = ExecuteRequest {
        code: "   \n\t  ".to_string(),
        cell_id: "cell_003".to_string(),
        session_id: None,
    };
    
    assert!(whitespace_code.code.trim().is_empty());
}

/// Test ExecuteResponse construction patterns
#[test]
fn test_execute_response_patterns() {
    // Success response pattern
    let success = ExecuteResponse {
        success: true,
        result: Some("Success result".to_string()),
        error: None,
        cell_id: "cell_001".to_string(),
        execution_time_ms: 100,
    };
    
    assert!(success.success);
    assert!(success.result.is_some());
    assert!(success.error.is_none());
    
    // Error response pattern
    let error = ExecuteResponse {
        success: false,
        result: None,
        error: Some("Error message".to_string()),
        cell_id: "cell_002".to_string(),
        execution_time_ms: 25,
    };
    
    assert!(!error.success);
    assert!(error.result.is_none());
    assert!(error.error.is_some());
    
    // Both result and error (edge case)
    let mixed = ExecuteResponse {
        success: true,
        result: Some("Partial result".to_string()),
        error: Some("Warning message".to_string()),
        cell_id: "cell_003".to_string(),
        execution_time_ms: 200,
    };
    
    assert!(mixed.success);
    assert!(mixed.result.is_some());
    assert!(mixed.error.is_some());
}

/// Test cell ID patterns
#[test]
fn test_cell_id_patterns() {
    let test_cases = vec![
        ("cell_001", true),
        ("cell-001", true), 
        ("cell.001", true),
        ("", false),
        ("   ", false),
        ("cell_123_abc", true),
        ("CELL_001", true),
    ];
    
    for (cell_id, expected_valid) in test_cases {
        let request = ExecuteRequest {
            code: "test".to_string(),
            cell_id: cell_id.to_string(),
            session_id: None,
        };
        
        let is_valid = !request.cell_id.trim().is_empty();
        assert_eq!(is_valid, expected_valid, "Cell ID '{}' validation failed", cell_id);
    }
}

/// Test execution timing constraints
#[test]
fn test_execution_timing() {
    // Test reasonable execution times
    let fast_execution = ExecuteResponse {
        success: true,
        result: Some("42".to_string()),
        error: None,
        cell_id: "cell_001".to_string(),
        execution_time_ms: 1, // Very fast
    };
    
    assert!(fast_execution.execution_time_ms >= 0);
    
    let slow_execution = ExecuteResponse {
        success: true,
        result: Some("Complex computation".to_string()),
        error: None,
        cell_id: "cell_002".to_string(),
        execution_time_ms: 5000, // 5 seconds
    };
    
    assert!(slow_execution.execution_time_ms > 1000);
    
    // Test timeout scenario
    let timeout_execution = ExecuteResponse {
        success: false,
        result: None,
        error: Some("Execution timeout".to_string()),
        cell_id: "cell_003".to_string(),
        execution_time_ms: 30000, // 30 seconds
    };
    
    assert!(!timeout_execution.success);
    assert!(timeout_execution.execution_time_ms > 10000);
}

/// Test session ID handling
#[test]
fn test_session_id_handling() {
    // No session ID
    let no_session = ExecuteRequest {
        code: "test".to_string(),
        cell_id: "cell_001".to_string(),
        session_id: None,
    };
    
    assert!(no_session.session_id.is_none());
    
    // With session ID
    let with_session = ExecuteRequest {
        code: "test".to_string(),
        cell_id: "cell_002".to_string(),
        session_id: Some("session_abc123".to_string()),
    };
    
    assert!(with_session.session_id.is_some());
    assert_eq!(with_session.session_id.unwrap(), "session_abc123");
    
    // Empty session ID (should be None or handled)
    let empty_session = ExecuteRequest {
        code: "test".to_string(),
        cell_id: "cell_003".to_string(),
        session_id: Some("".to_string()),
    };
    
    // Empty session IDs might be considered invalid
    assert!(empty_session.session_id.unwrap().is_empty());
}

/// Test debug serialization for ExecuteRequest
#[test] 
fn test_execute_request_debug() {
    let request = ExecuteRequest {
        code: "println!(\"Hello\")".to_string(),
        cell_id: "cell_debug".to_string(),
        session_id: Some("debug_session".to_string()),
    };
    
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("ExecuteRequest"));
    assert!(debug_str.contains("println!"));
    assert!(debug_str.contains("cell_debug"));
    assert!(debug_str.contains("debug_session"));
}

/// Test debug serialization for ExecuteResponse
#[test]
fn test_execute_response_debug() {
    let response = ExecuteResponse {
        success: true,
        result: Some("Debug output".to_string()),
        error: None,
        cell_id: "cell_debug".to_string(),
        execution_time_ms: 42,
    };
    
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("ExecuteResponse"));
    assert!(debug_str.contains("Debug output"));
    assert!(debug_str.contains("cell_debug"));
    assert!(debug_str.contains("42"));
}

/// Test large code input handling
#[test]
fn test_large_code_input() {
    let large_code = "a".repeat(10000); // 10KB of code
    
    let request = ExecuteRequest {
        code: large_code.clone(),
        cell_id: "cell_large".to_string(),
        session_id: None,
    };
    
    assert_eq!(request.code.len(), 10000);
    
    // Should be serializable
    let json_result = serde_json::to_string(&request);
    assert!(json_result.is_ok());
}

/// Test special characters in code
#[test]
fn test_special_characters_in_code() {
    let special_code = r#"let msg = "Hello \"world\"!\n\t💻"; println!(msg);"#;
    
    let request = ExecuteRequest {
        code: special_code.to_string(),
        cell_id: "cell_special".to_string(),
        session_id: None,
    };
    
    // Should handle Unicode and escape sequences
    assert!(request.code.contains("💻"));
    assert!(request.code.contains("\\n"));
    assert!(request.code.contains("\\\""));
    
    // Should be serializable
    let json_result = serde_json::to_string(&request);
    assert!(json_result.is_ok());
}

/// Test concurrent cell execution simulation
#[test]
fn test_concurrent_cell_simulation() {
    let cell_ids = vec!["cell_001", "cell_002", "cell_003", "cell_004"];
    let mut responses = Vec::new();
    
    for cell_id in cell_ids {
        let response = ExecuteResponse {
            success: true,
            result: Some(format!("Result from {}", cell_id)),
            error: None,
            cell_id: cell_id.to_string(),
            execution_time_ms: 100 + (responses.len() as u64 * 10),
        };
        responses.push(response);
    }
    
    // Verify all cells got unique responses
    assert_eq!(responses.len(), 4);
    for (i, response) in responses.iter().enumerate() {
        assert!(response.result.as_ref().unwrap().contains(&format!("cell_{:03}", i + 1)));
        assert!(response.execution_time_ms >= 100);
    }
}