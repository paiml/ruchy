// STDLIB-001: Type Conversion Functions Test Suite
// Following Toyota Way TDD - RED phase first

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {:?}", e))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {:?}", e))?;
    
    // The REPL returns string values with quotes, so we need to handle that
    if result.starts_with('"') && result.ends_with('"') && result.len() >= 2 {
        Ok(result[1..result.len()-1].to_string())
    } else {
        Ok(result)
    }
}

// Helper to test transpiled code
fn eval_transpiled(code: &str) -> Result<String, String> {
    // Use unique filename to avoid test interference
    let test_file = format!("/tmp/type_conv_test_{}.ruchy", 
        std::process::id());
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {}", e))?;
    
    // Clean up test file
    let _ = fs::remove_file(&test_file);
    
    if !output.status.success() {
        return Err(format!("Execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[test]
fn test_str_conversion() {
    // Test integer to string
    assert_eq!(eval_in_repl("str(42)").unwrap(), "42");
    assert_eq!(eval_transpiled("println(str(42))").unwrap(), "42");
    
    // Test float to string
    assert_eq!(eval_in_repl("str(3.14)").unwrap(), "3.14");
    assert_eq!(eval_transpiled("println(str(3.14))").unwrap(), "3.14");
    
    // Test boolean to string
    assert_eq!(eval_in_repl("str(true)").unwrap(), "true");
    assert_eq!(eval_transpiled("println(str(true))").unwrap(), "true");
    assert_eq!(eval_in_repl("str(false)").unwrap(), "false");
    assert_eq!(eval_transpiled("println(str(false))").unwrap(), "false");
}

#[test]
fn test_int_conversion() {
    // Test string to integer
    assert_eq!(eval_in_repl(r#"int("42")"#).unwrap(), "42");
    assert_eq!(eval_transpiled(r#"println(int("42"))"#).unwrap(), "42");
    
    // Test float to integer (truncation)
    assert_eq!(eval_in_repl("int(3.14)").unwrap(), "3");
    assert_eq!(eval_transpiled("println(int(3.14))").unwrap(), "3");
    assert_eq!(eval_in_repl("int(3.99)").unwrap(), "3");
    assert_eq!(eval_transpiled("println(int(3.99))").unwrap(), "3");
    
    // Test boolean to integer
    assert_eq!(eval_in_repl("int(true)").unwrap(), "1");
    assert_eq!(eval_transpiled("println(int(true))").unwrap(), "1");
    assert_eq!(eval_in_repl("int(false)").unwrap(), "0");
    assert_eq!(eval_transpiled("println(int(false))").unwrap(), "0");
}

#[test]
fn test_float_conversion() {
    // Test string to float
    assert_eq!(eval_in_repl(r#"float("3.14")"#).unwrap(), "3.14");
    assert_eq!(eval_transpiled(r#"println(float("3.14"))"#).unwrap(), "3.14");
    
    // Test integer to float - just verify it doesn't error
    assert!(eval_in_repl("float(42)").is_ok());
    assert!(eval_transpiled("println(float(42))").is_ok());
}

#[test]
fn test_bool_conversion() {
    // Test integer to boolean
    assert_eq!(eval_in_repl("bool(1)").unwrap(), "true");
    assert_eq!(eval_transpiled("println(bool(1))").unwrap(), "true");
    assert_eq!(eval_in_repl("bool(0)").unwrap(), "false");
    assert_eq!(eval_transpiled("println(bool(0))").unwrap(), "false");
    assert_eq!(eval_in_repl("bool(42)").unwrap(), "true");
    assert_eq!(eval_transpiled("println(bool(42))").unwrap(), "true");
    
    // Test string to boolean
    assert_eq!(eval_in_repl(r#"bool("")"#).unwrap(), "false");
    assert_eq!(eval_transpiled(r#"println(bool(""))"#).unwrap(), "false");
    assert_eq!(eval_in_repl(r#"bool("hello")"#).unwrap(), "true");
    assert_eq!(eval_transpiled(r#"println(bool("hello"))"#).unwrap(), "true");
}

#[test]
fn test_conversion_chain() {
    // Test chaining conversions
    assert_eq!(eval_in_repl(r#"str(int(float("3.14")))"#).unwrap(), "3");
    assert_eq!(eval_transpiled(r#"println(str(int(float("3.14"))))"#).unwrap(), "3");
    
    // Test in expressions
    assert_eq!(eval_in_repl(r#"int("40") + 2"#).unwrap(), "42");
    assert_eq!(eval_transpiled(r#"println(int("40") + 2)"#).unwrap(), "42");
}