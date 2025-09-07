// TDD tests for value constructor helper functions
// Validates comprehensive entropy reduction through helper functions

use ruchy::runtime::repl::Repl;

#[test]
fn test_string_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test string methods that use ok_string helper
    let result = repl.eval("\"hello\".to_upper()").unwrap();
    assert_eq!(result, "\"HELLO\"");
    
    let result = repl.eval("\"WORLD\".to_lower()").unwrap();
    assert_eq!(result, "\"world\"");
    
    let result = repl.eval("\" test \".trim()").unwrap();
    assert_eq!(result, "\"test\"");
}

#[test]
fn test_bool_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test bool methods that use ok_bool helper
    let result = repl.eval("'a'.is_alphabetic()").unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval("'1'.is_numeric()").unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval("[1, 2, 3].any(|x| x > 2)").unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_int_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test int methods that use ok_int helper
    let result = repl.eval("[1, 2, 3].len()").unwrap();
    assert_eq!(result, "3");
    
    let result = repl.eval("(-42).abs()").unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval("'a'.to_int()").unwrap();
    assert_eq!(result, "97");
}

#[test]
fn test_float_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test float methods that use ok_float helper
    let result = repl.eval("3.7.floor()").unwrap();
    assert_eq!(result, "3");
    
    let result = repl.eval("3.2.ceil()").unwrap();
    assert_eq!(result, "4");
    
    let result = repl.eval("3.5.round()").unwrap();
    assert_eq!(result, "4");
}

#[test]
fn test_list_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test list methods that use ok_list helper
    let result = repl.eval("[1, 2, 3].map(|x| x * 2)").unwrap();
    assert_eq!(result, "[2, 4, 6]");
    
    let result = repl.eval("[1, 2, 3, 4].filter(|x| x > 2)").unwrap();
    assert_eq!(result, "[3, 4]");
    
    let result = repl.eval("[1, 2].push(3)").unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_char_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test char methods that use ok_char helper
    let result = repl.eval("'a'.to_uppercase()").unwrap();
    assert_eq!(result, "\"A\"");
    
    let result = repl.eval("'Z'.to_lowercase()").unwrap();
    assert_eq!(result, "\"z\"");
    
    let result = repl.eval("'x'.to_string()").unwrap();
    assert_eq!(result, "\"x\"");
}

#[test]
fn test_tuple_constructor_helper() {
    let mut repl = Repl::new().unwrap();
    
    // Test tuple constructor with ok_tuple helper  
    let result = repl.eval("(1, 2, 3)").unwrap();
    assert_eq!(result, "(1, 2, 3)");
}

// Unit constructor helper is used internally in ok_unit() function

#[test]
fn test_comprehensive_value_construction() {
    let mut repl = Repl::new().unwrap();
    
    // Complex expression using multiple value constructors
    let result = repl.eval(r#"
        let numbers = [1, 2, 3, 4, 5];
        let evens = numbers.filter(|x| x % 2 == 0);
        let doubled = evens.map(|x| x * 2);
        let result = doubled.len() > 0;
        result
    "#).unwrap();
    assert_eq!(result, "true");
}