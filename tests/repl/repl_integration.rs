//! REPL Integration Tests
//! 
//! Tests interactive REPL features including metacommands, multiline input, and state persistence

use ruchy::runtime::repl::Repl;
use std::{env, time::Duration;

#[test]
fn repl_basic_evaluation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval("2 + 2").unwrap();
    assert_eq!(result.to_string(), "4");
    
    let result = repl.eval("6 * 7").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn repl_variable_persistence() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Define a variable
    repl.eval("let x = 10").unwrap();
    
    // Use it in another expression
    let result = repl.eval("x * 2").unwrap();
    assert_eq!(result.to_string(), "20");
    
    // Redefine it
    repl.eval("let x = 100").unwrap();
    let result = repl.eval("x").unwrap();
    assert_eq!(result.to_string(), "100");
}

#[test]
fn repl_function_definition_and_call() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Define a function
    repl.eval("fun square(x: i32) -> i32 { x * x }").unwrap();
    
    // Call it
    let result = repl.eval("square(5)").unwrap();
    assert_eq!(result.to_string(), "25");
    
    // Call with different argument
    let result = repl.eval("square(10)").unwrap();
    assert_eq!(result.to_string(), "100");
}

#[test]
fn repl_multiline_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Multi-line let binding
    let code = "let result = 
        10 + 20";
    repl.eval(code).unwrap();
    
    let result = repl.eval("result").unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn repl_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    repl.eval(r#"let name = "Ruchy""#).unwrap();
    let result = repl.eval(r#"f"Hello, {name}!""#).unwrap();
    assert_eq!(result.to_string(), "\"Hello, Ruchy!\"");
}

#[test]
fn repl_list_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Map
    let result = repl.eval("[1, 2, 3].map(|x| x * 2)").unwrap();
    assert_eq!(result.to_string(), "[2, 4, 6]");
    
    // Filter
    let result = repl.eval("[1, 2, 3, 4, 5].filter(|x| x > 3)").unwrap();
    assert_eq!(result.to_string(), "[4, 5]");
    
    // Reduce
    let result = repl.eval("[1, 2, 3, 4].reduce(0, |acc, x| acc + x)").unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn repl_match_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval(r#"
        match 5 {
            0 => "zero",
            5 => "five",
            _ => "other"
        }
    "#).unwrap();
    assert_eq!(result.to_string(), "\"five\"");
}

#[test]
fn repl_lambda_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Define a lambda
    repl.eval("let double = |x| x * 2").unwrap();
    
    // Use it
    let result = repl.eval("double(21)").unwrap();
    assert_eq!(result.to_string(), "42");
    
    // Lambda with fat arrow
    repl.eval("let add_one = |x| => x + 1").unwrap();
    let result = repl.eval("add_one(41)").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn repl_boolean_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval("true && false").unwrap();
    assert_eq!(result.to_string(), "false");
    
    let result = repl.eval("true || false").unwrap();
    assert_eq!(result.to_string(), "true");
    
    let result = repl.eval("!true").unwrap();
    assert_eq!(result.to_string(), "false");
}

#[test]
fn repl_comparison_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval("10 > 5").unwrap();
    assert_eq!(result.to_string(), "true");
    
    let result = repl.eval("10 < 5").unwrap();
    assert_eq!(result.to_string(), "false");
    
    let result = repl.eval("42 == 42").unwrap();
    assert_eq!(result.to_string(), "true");
}

#[test]
fn repl_if_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval(r#"if 10 > 5 { "yes" } else { "no" }"#).unwrap();
    assert_eq!(result.to_string(), "\"yes\"");
    
    let result = repl.eval(r#"if 2 > 5 { "yes" } else { "no" }"#).unwrap();
    assert_eq!(result.to_string(), "\"no\"");
}

#[test]
fn repl_for_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // For loop with range
    repl.eval("let mut sum = 0").unwrap();
    repl.eval("for i in 1..5 { sum = sum + i }").unwrap();
    let result = repl.eval("sum").unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn repl_while_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    repl.eval("let mut counter = 0").unwrap();
    repl.eval("while counter < 5 { counter = counter + 1 }").unwrap();
    let result = repl.eval("counter").unwrap();
    assert_eq!(result.to_string(), "5");
}

#[test]
fn repl_string_methods() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.eval(r#""hello".len()"#).unwrap();
    assert_eq!(result.to_string(), "5");
    
    let result = repl.eval(r#""hello".to_upper()"#).unwrap();
    assert_eq!(result.to_string(), "\"HELLO\"");
    
    let result = repl.eval(r#""  spaces  ".trim()"#).unwrap();
    assert_eq!(result.to_string(), "\"spaces\"");
}

#[test]
fn repl_memory_limits() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Should respect memory limits (10MB arena)
    // This should work within limits
    let result = repl.eval("[1; 1000]"); // List of 1000 ones
    assert!(result.is_ok());
}

#[test]
fn repl_timeout_limits() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Set a deadline for evaluation
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    
    // Simple expression should complete quickly
    let result = repl.evaluate_expr_str("2 + 2", Some(deadline));
    assert!(result.is_ok());
}

#[test]
fn repl_error_recovery() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Invalid syntax should error
    let result = repl.eval("2 + + 3");
    assert!(result.is_err());
    
    // But REPL should still work after error
    let result = repl.eval("2 + 3").unwrap();
    assert_eq!(result.to_string(), "5");
}