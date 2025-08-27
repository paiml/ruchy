// STDLIB-007: Process/Environment Functions Test Suite
// Following Toyota Way TDD - RED phase first

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;
use std::env;

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {:?}", e))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {:?}", e))?;
    
    // Remove quotes if present (REPL string formatting)
    if result.starts_with('"') && result.ends_with('"') && result.len() >= 2 {
        Ok(result[1..result.len()-1].to_string())
    } else {
        Ok(result)
    }
}

// Helper to test transpiled code with unique filenames
fn eval_transpiled(code: &str) -> Result<String, String> {
    let test_file = format!("/tmp/process_env_test_{}.ruchy", 
        std::process::id());
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {}", e))?;
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    
    if !output.status.success() {
        return Err(format!("Execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[test]
fn test_current_dir() {
    // Test current_dir returns a string path
    let result = eval_in_repl("current_dir()");
    assert!(result.is_ok(), "current_dir should work in REPL: {:?}", result);
    
    let path = result.unwrap();
    assert!(path.starts_with('/'), "current_dir should return absolute path: {}", path);
    assert!(path.contains("ruchy"), "Should be in ruchy project directory: {}", path);
    
    // Test transpiled version
    let code = "println(current_dir())";
    let result = eval_transpiled(code);
    assert!(result.is_ok(), "current_dir should work in transpiler: {:?}", result);
    
    let path = result.unwrap();
    assert!(path.contains("ruchy"), "Should be in ruchy project directory: {}", path);
}

#[test]
fn test_env() {
    // Set up test environment variable
    env::set_var("RUCHY_TEST_VAR", "test_value_123");
    
    // Test env function
    let code = r#"env("RUCHY_TEST_VAR")"#;
    let result = eval_in_repl(code).unwrap();
    assert_eq!(result, "test_value_123");
    
    // Test transpiled version
    let code = r#"println(env("RUCHY_TEST_VAR"))"#;
    let result = eval_transpiled(code).unwrap();
    assert_eq!(result, "test_value_123");
    
    // Test non-existent environment variable
    let code = r#"env("RUCHY_NON_EXISTENT_VAR_12345")"#;
    let result = eval_in_repl(code);
    // Should return empty string or None - depending on implementation
    assert!(result.is_ok(), "env should handle non-existent vars gracefully: {:?}", result);
    
    // Clean up
    env::remove_var("RUCHY_TEST_VAR");
}

#[test]
fn test_set_env() {
    // Test setting environment variable
    let code = r#"set_env("RUCHY_SET_TEST", "new_value_456")"#;
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "set_env should work in REPL: {:?}", result);
    
    // Verify it was set
    let value = env::var("RUCHY_SET_TEST");
    assert!(value.is_ok(), "Environment variable should be set");
    assert_eq!(value.unwrap(), "new_value_456");
    
    // Test transpiled version
    let code = r#"set_env("RUCHY_SET_TEST_2", "transpiled_value")"#;
    let result = eval_transpiled(code);
    assert!(result.is_ok(), "set_env should work in transpiler: {:?}", result);
    
    // Clean up
    env::remove_var("RUCHY_SET_TEST");
    env::remove_var("RUCHY_SET_TEST_2");
}

#[test]
fn test_args() {
    // Test args function - should return command line arguments
    let result = eval_in_repl("args()");
    assert!(result.is_ok(), "args should work in REPL: {:?}", result);
    
    // In REPL, args might be empty or contain repl-specific args
    let args_str = result.unwrap();
    assert!(args_str.starts_with('[') && args_str.ends_with(']'), 
        "args should return array format: {}", args_str);
    
    // Test transpiled version with a script
    let code = "println(args())";
    let result = eval_transpiled(code);
    assert!(result.is_ok(), "args should work in transpiler: {:?}", result);
    
    let args_str = result.unwrap();
    assert!(args_str.starts_with('[') && args_str.ends_with(']'), 
        "args should return array format: {}", args_str);
}

#[test]
fn test_environment_operations_integration() {
    // Test combined environment operations
    let code = r#"
        set_env("RUCHY_INTEGRATION_TEST", "integration_value")
        env("RUCHY_INTEGRATION_TEST")
    "#;
    
    let result = eval_in_repl(code).unwrap();
    assert_eq!(result, "integration_value");
    
    // Verify it exists in actual environment
    let env_value = env::var("RUCHY_INTEGRATION_TEST").unwrap();
    assert_eq!(env_value, "integration_value");
    
    // Clean up
    env::remove_var("RUCHY_INTEGRATION_TEST");
}

#[test]
fn test_process_functions_exist() {
    // Test that all process functions exist and don't error
    
    let functions = ["current_dir()", "args()", r#"env("PATH")"#];
    
    for func in &functions {
        let result = eval_in_repl(func);
        assert!(result.is_ok(), "Function {} should exist and not error: {:?}", func, result);
    }
}