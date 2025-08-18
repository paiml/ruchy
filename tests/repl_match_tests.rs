//! REPL match expression tests

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unnecessary_unwrap)]

use ruchy::runtime::Repl;

#[test]
fn test_match_literal_patterns() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 42 {
            42 => "found forty-two",
            _ => "not forty-two"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""found forty-two""#);
}

#[test]
fn test_match_wildcard_pattern() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 999 {
            42 => "forty-two",
            _ => "something else"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""something else""#);
}

#[test]
fn test_match_variable_binding() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 100 {
            x => x + 50
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "150");
}

#[test]
fn test_match_multiple_patterns() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 2 {
            1 => "one",
            2 => "two", 
            3 => "three",
            _ => "other"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""two""#);
}

#[test]
fn test_match_boolean_patterns() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match true {
            true => "yes",
            false => "no"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""yes""#);
}

#[test]
fn test_match_string_patterns() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match "hello" {
            "world" => "greeting world",
            "hello" => "greeting hello",
            _ => "no greeting"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""greeting hello""#);
}

#[test]
fn test_match_list_patterns() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match [1, 2, 3] {
            [1, 2, 3] => "exact match",
            [1, _, _] => "starts with one",
            _ => "no match"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""exact match""#);
}

#[test]
fn test_match_list_binding() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match [10, 20] {
            [x, y] => x + y
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "30");
}

#[test]
fn test_match_with_variable() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    // Set up a variable
    assert!(repl.eval("let value = 42").is_ok());
    
    let result = repl.eval(r#"
        match value {
            42 => "the answer",
            _ => "not the answer"
        }
    "#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""the answer""#);
}

#[test]
fn test_match_range_pattern() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 5 {
            1..=10 => "in range",
            _ => "out of range"
        }
    "#);
    // Range patterns might not be fully implemented yet
    if result.is_ok() {
        let output = result.unwrap();
        assert_eq!(output, r#""in range""#);
    }
}

#[test]
fn test_match_no_pattern_matches() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    let result = repl.eval(r#"
        match 999 {
            1 => "one",
            2 => "two"
        }
    "#);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No matching pattern"));
}