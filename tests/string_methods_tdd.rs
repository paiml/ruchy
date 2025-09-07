// TDD tests for additional string methods
// This captures the requirement for string manipulation and conversion

use ruchy::runtime::repl::Repl;

#[test]
fn test_string_to_int() {
    let mut repl = Repl::new().unwrap();
    
    // Convert string to integer
    let result = repl.eval(r#""42".to_int()"#).unwrap();
    assert_eq!(result, "42");
    
    // Negative numbers
    let result = repl.eval(r#""-123".to_int()"#).unwrap();
    assert_eq!(result, "-123");
}

#[test]
fn test_string_to_float() {
    let mut repl = Repl::new().unwrap();
    
    // Convert string to float
    let result = repl.eval(r#""3.14".to_float()"#).unwrap();
    assert_eq!(result, "3.14");
    
    // Integer string to float
    let result = repl.eval(r#""42".to_float()"#).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_string_parse() {
    let mut repl = Repl::new().unwrap();
    
    // Parse as integer
    let result = repl.eval(r#""123".parse()"#).unwrap();
    assert_eq!(result, "123");
}

#[test]
fn test_string_repeat() {
    let mut repl = Repl::new().unwrap();
    
    // Repeat string n times
    let result = repl.eval(r#""ab".repeat(3)"#).unwrap();
    assert_eq!(result, r#""ababab""#);
    
    // Repeat zero times
    let result = repl.eval(r#""test".repeat(0)"#).unwrap();
    assert_eq!(result, r#""""#);
}

#[test]
fn test_string_pad() {
    let mut repl = Repl::new().unwrap();
    
    // Pad left
    let result = repl.eval(r#""5".pad_left(3, "0")"#).unwrap();
    assert_eq!(result, r#""005""#);
    
    // Pad right
    let result = repl.eval(r#""hello".pad_right(10, " ")"#).unwrap();
    assert_eq!(result, r#""hello     ""#);
}

#[test]
fn test_string_chars() {
    let mut repl = Repl::new().unwrap();
    
    // Get list of characters
    let result = repl.eval(r#""abc".chars()"#).unwrap();
    assert_eq!(result, r#"["a", "b", "c"]"#);
    
    // Empty string
    let result = repl.eval(r#""".chars()"#).unwrap();
    assert_eq!(result, "[]");
}

#[test]
fn test_string_bytes() {
    let mut repl = Repl::new().unwrap();
    
    // Get list of byte values
    let result = repl.eval(r#""ABC".bytes()"#).unwrap();
    assert_eq!(result, "[65, 66, 67]");
}

#[test]
fn test_string_is_numeric() {
    let mut repl = Repl::new().unwrap();
    
    // Check if string is numeric
    let result = repl.eval(r#""123".is_numeric()"#).unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval(r#""12.34".is_numeric()"#).unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval(r#""abc".is_numeric()"#).unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_string_is_alpha() {
    let mut repl = Repl::new().unwrap();
    
    // Check if string is alphabetic
    let result = repl.eval(r#""hello".is_alpha()"#).unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval(r#""hello123".is_alpha()"#).unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_string_is_alphanumeric() {
    let mut repl = Repl::new().unwrap();
    
    // Check if string is alphanumeric
    let result = repl.eval(r#""hello123".is_alphanumeric()"#).unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval(r#""hello 123".is_alphanumeric()"#).unwrap();
    assert_eq!(result, "false");
}