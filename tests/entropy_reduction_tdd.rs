// TDD tests for entropy reduction refactoring
// Validates that consolidation of validation, math methods, and error handling works correctly

use ruchy::runtime::repl::Repl;

#[test]
fn test_argument_validation_helpers() {
    let mut repl = Repl::new().unwrap();
    
    // Test that map still validates correctly
    let result = repl.eval("[1, 2, 3].map()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("map takes exactly 1 argument"));
    
    // Test that filter still validates correctly  
    let result = repl.eval("[1, 2, 3].filter()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("filter takes exactly 1 argument"));
    
    // Test that functions with specific arg counts still validate
    let result = repl.eval("sqrt()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("takes exactly 1 argument"));
}

#[test]
fn test_consolidated_numeric_methods() {
    let mut repl = Repl::new().unwrap();
    
    // Test integer methods still work
    let result = repl.eval("42.abs()").unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval("(-42).abs()").unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval("16.sqrt()").unwrap();
    assert_eq!(result, "4");
    
    let result = repl.eval("42.to_string()").unwrap();
    assert_eq!(result, r#""42""#);
    
    // Test float methods still work
    let result = repl.eval("3.14.abs()").unwrap();
    assert_eq!(result, "3.14");
    
    let result = repl.eval("(-3.14).abs()").unwrap();
    assert_eq!(result, "3.14");
    
    let result = repl.eval("3.7.floor()").unwrap();
    assert_eq!(result, "3");
    
    let result = repl.eval("3.2.ceil()").unwrap();
    assert_eq!(result, "4");
    
    let result = repl.eval("3.5.round()").unwrap();
    assert_eq!(result, "4");
}

#[test]
fn test_math_operations_on_both_types() {
    let mut repl = Repl::new().unwrap();
    
    // Test that math operations work on both int and float
    let result = repl.eval("0.sin()").unwrap();
    assert_eq!(result, "0");
    
    let result = repl.eval("0.0.sin()").unwrap();
    assert_eq!(result, "0");
    
    let result = repl.eval("1.cos()").unwrap();
    assert!(result.starts_with("0.540"));
    
    let result = repl.eval("1.0.cos()").unwrap();
    assert!(result.starts_with("0.540"));
}

#[test]
fn test_unknown_method_errors() {
    let mut repl = Repl::new().unwrap();
    
    // Test list unknown method
    let result = repl.eval("[1, 2, 3].nonexistent()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown list method: nonexistent"));
    
    // Test integer unknown method
    let result = repl.eval("42.nonexistent()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown integer method: nonexistent"));
    
    // Test float unknown method  
    let result = repl.eval("3.14.nonexistent()");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown float method: nonexistent"));
}

#[test]
fn test_error_factory_consistency() {
    let mut repl = Repl::new().unwrap();
    
    // All unknown method errors should follow the same pattern
    let errors = vec![
        repl.eval("[].unknown()").unwrap_err().to_string(),
        repl.eval("42.unknown()").unwrap_err().to_string(),
        repl.eval("3.14.unknown()").unwrap_err().to_string(),
    ];
    
    // All should contain "Unknown" and "method:"
    for error in &errors {
        assert!(error.contains("Unknown"));
        assert!(error.contains("method:"));
    }
}

#[test]
fn test_optional_chaining_with_numeric_methods() {
    let mut repl = Repl::new().unwrap();
    
    // Test that optional chaining still works with consolidated methods
    let result = repl.eval("42?.abs()").unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval("Option::None?.abs()").unwrap();
    assert_eq!(result, "null");
    
    let result = repl.eval("3.14?.round()").unwrap();
    assert_eq!(result, "3");
}