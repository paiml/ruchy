// TDD tests for code refactoring - ensure behavior remains identical
// This verifies that our refactoring doesn't break existing functionality

use ruchy::runtime::repl::Repl;

#[test]
fn test_option_none_creation() {
    let mut repl = Repl::new().unwrap();
    
    // Test that empty list operations return Option::None
    let result = repl.eval("[].min()").unwrap();
    assert_eq!(result, "Option::None");
    
    let result = repl.eval("[].max()").unwrap();
    assert_eq!(result, "Option::None");
    
    let result = repl.eval("[].find(|x| x > 5)").unwrap();
    assert_eq!(result, "Option::None");
}

#[test]
fn test_option_some_creation() {
    let mut repl = Repl::new().unwrap();
    
    // Test that non-empty list operations return Option::Some
    let result = repl.eval("[1, 2, 3].min()").unwrap();
    assert_eq!(result, "Option::Some(1)");
    
    let result = repl.eval("[1, 2, 3].max()").unwrap();
    assert_eq!(result, "Option::Some(3)");
    
    let result = repl.eval("[1, 2, 3].find(|x| x > 2)").unwrap();
    assert_eq!(result, "Option::Some(3)");
}

#[test]
fn test_math_functions_refactored() {
    let mut repl = Repl::new().unwrap();
    
    // Test sin function
    let result = repl.eval("sin(0)").unwrap();
    assert_eq!(result, "0");
    
    // Test cos function
    let result = repl.eval("cos(0)").unwrap();
    assert_eq!(result, "1");
    
    // Test tan function
    let result = repl.eval("tan(0)").unwrap();
    assert_eq!(result, "0");
    
    // Test sqrt function
    let result = repl.eval("sqrt(4)").unwrap();
    assert_eq!(result, "2");
    
    // Test log function
    let result = repl.eval("log(1)").unwrap();
    assert_eq!(result, "0");
    
    // Test log10 function
    let result = repl.eval("log10(10)").unwrap();
    assert_eq!(result, "1");
    
    // Test abs function
    let result = repl.eval("abs(-5)").unwrap();
    assert_eq!(result, "5");
    
    // Test with float input
    let result = repl.eval("sin(3.14159 / 2.0)").unwrap();
    assert!(result.starts_with("0.9999")); // Close to 1
}

#[test]
fn test_math_functions_error_handling() {
    let mut repl = Repl::new().unwrap();
    
    // Test wrong number of arguments
    assert!(repl.eval("sin()").is_err());
    assert!(repl.eval("sin(1, 2)").is_err());
    assert!(repl.eval("cos()").is_err());
    assert!(repl.eval("sqrt()").is_err());
    
    // Test non-numeric arguments
    assert!(repl.eval(r#"sin("hello")"#).is_err());
    assert!(repl.eval(r#"sqrt("test")"#).is_err());
}

#[test]
fn test_type_conversion_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Test str conversion
    let result = repl.eval("str(42)").unwrap();
    assert_eq!(result, r#""42""#);
    
    let result = repl.eval("str(3.14)").unwrap();
    assert_eq!(result, r#""3.14""#);
    
    let result = repl.eval("str(true)").unwrap();
    assert_eq!(result, r#""true""#);
    
    // Test int conversion
    let result = repl.eval(r#"int("42")"#).unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval("int(3.14)").unwrap();
    assert_eq!(result, "3");
    
    let result = repl.eval("int(true)").unwrap();
    assert_eq!(result, "1");
    
    // Test float conversion
    let result = repl.eval(r#"float("3.14")"#).unwrap();
    assert_eq!(result, "3.14");
    
    let result = repl.eval("float(42)").unwrap();
    assert_eq!(result, "42");
    
    // Test bool conversion
    let result = repl.eval("bool(1)").unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval("bool(0)").unwrap();
    assert_eq!(result, "false");
    
    let result = repl.eval(r#"bool("true")"#).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_list_method_argument_validation() {
    let mut repl = Repl::new().unwrap();
    
    // Test methods that require no arguments
    assert!(repl.eval("[1, 2].flatten(1)").is_err()); // Should fail - no args expected
    assert!(repl.eval("[1, 2].unique(1)").is_err()); // Should fail - no args expected
    assert!(repl.eval("[1, 2].reverse(1)").is_err()); // Should fail - no args expected
    
    // Test methods that require exactly 1 argument
    assert!(repl.eval("[1, 2].push()").is_err()); // Should fail - 1 arg expected
    assert!(repl.eval("[1, 2].push(1, 2)").is_err()); // Should fail - 1 arg expected
    assert!(repl.eval("[1, 2].append()").is_err()); // Should fail - 1 arg expected
    
    // Test methods that require exactly 2 arguments
    assert!(repl.eval("[1, 2].insert(0)").is_err()); // Should fail - 2 args expected
    assert!(repl.eval("[1, 2].insert(0, 1, 2)").is_err()); // Should fail - 2 args expected
}

#[test]
fn test_list_min_max_with_mixed_types() {
    let mut repl = Repl::new().unwrap();
    
    // Test min with mixed int/float
    let result = repl.eval("[1, 2.5, 3].min()").unwrap();
    assert_eq!(result, "Option::Some(1)");
    
    let result = repl.eval("[3.5, 1, 2.5].min()").unwrap();
    assert_eq!(result, "Option::Some(1)");
    
    // Test max with mixed int/float
    let result = repl.eval("[1, 2.5, 3].max()").unwrap();
    assert_eq!(result, "Option::Some(3)");
    
    let result = repl.eval("[1.5, 3, 2.5].max()").unwrap();
    assert_eq!(result, "Option::Some(3)");
}

#[test]
fn test_string_method_errors() {
    let mut repl = Repl::new().unwrap();
    
    // Test wrong number of arguments
    assert!(repl.eval(r#""test".to_int(1)"#).is_err());
    assert!(repl.eval(r#""test".to_float(1)"#).is_err());
    assert!(repl.eval(r#""test".is_numeric(1)"#).is_err());
    assert!(repl.eval(r#""test".repeat()"#).is_err());
    assert!(repl.eval(r#""test".pad_left()"#).is_err());
}

#[test]
fn test_reduce_functionality() {
    let mut repl = Repl::new().unwrap();
    
    // Test basic reduce operations
    let result = repl.eval("[1, 2, 3].reduce(|acc, x| acc + x, 0)").unwrap();
    assert_eq!(result, "6");
    
    let result = repl.eval("[1, 2, 3].reduce(|acc, x| acc * x, 1)").unwrap();
    assert_eq!(result, "6");
    
    // Test empty list
    let result = repl.eval("[].reduce(|acc, x| acc + x, 42)").unwrap();
    assert_eq!(result, "42");
}