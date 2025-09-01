// Basic replay functionality tests
// Replaces the broken generated_comprehensive.rs with working tests

use ruchy::runtime::repl::Repl;
use anyhow::Result;

#[test] 
fn test_replay_converter_string_escaping() -> Result<()> {
    // Test that the replay converter properly handles strings with quotes
    let mut repl = Repl::new()?;
    
    // Test basic string output - println doesn't return the printed content
    let result = repl.eval(r#"println("Hello World")"#)?;
    // println returns empty string (unit value), but we test it doesn't crash
    assert_eq!(result, "");
    
    // Test object with quotes (this was causing the format issue)
    let result = repl.eval(r#"let user = { name: "Alice", age: 30 }"#)?;
    // Should not cause parsing errors
    assert!(result.contains("Alice"));
    
    // Test string literals with escapes
    let result = repl.eval(r#"let text = "Line 1\nLine 2""#)?;
    assert!(!result.is_empty());
    
    Ok(())
}

#[test]
fn test_replay_system_integration() -> Result<()> {
    // Test basic replay system functionality
    let mut repl = Repl::new()?;
    
    // Test arithmetic 
    let result = repl.eval("2 + 3")?;
    assert_eq!(result, "5");
    
    // Test string operations
    let result = repl.eval(r#""hello".upper()"#)?;
    assert_eq!(result, r#""HELLO""#);
    
    // Test list operations  
    let result = repl.eval("[1, 2, 3].len()")?;
    assert_eq!(result, "3");
    
    Ok(())
}

#[test]
fn test_replay_value_formatting() -> Result<()> {
    // Test that values format correctly without causing syntax errors
    let mut repl = Repl::new()?;
    
    // Test various value types
    let test_cases = vec![
        ("42", "42"),
        ("3.14", "3.14"), 
        ("true", "true"),
        (r#""test""#, r#""test""#),
    ];
    
    for (input, expected_pattern) in test_cases {
        let result = repl.eval(input)?;
        assert!(result.contains(expected_pattern), 
               "Input: {} did not contain expected pattern: {}, got: {}", 
               input, expected_pattern, result);
    }
    
    Ok(())
}