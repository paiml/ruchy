//! TDD Tests for F-String Interpolation
//! Critical regression prevention - f-strings must work!

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn run_ruchy_code(code: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("ruchy")
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to run ruchy");
    
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_basic_fstring_variable_interpolation() {
    let code = r#"
        let x = 42
        println(f"The answer is {x}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "The answer is 42");
}

#[test]
fn test_fstring_multiple_variables() {
    let code = r#"
        let name = "Alice"
        let age = 30
        println(f"Hello {name}, you are {age} years old")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Hello Alice, you are 30 years old");
}

#[test]
fn test_fstring_with_expressions() {
    let code = r#"
        let x = 10
        let y = 20
        println(f"Sum: {x + y}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Sum: 30");
}

#[test]
fn test_fstring_with_method_calls() {
    let code = r#"
        let nums = [1, 2, 3]
        println(f"Length: {nums.len()}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Length: 3");
}

#[test]
fn test_fstring_with_nested_expressions() {
    let code = r#"
        let x = 5
        println(f"Double: {x * 2}, Triple: {x * 3}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Double: 10, Triple: 15");
}

#[test]
fn test_fstring_with_boolean() {
    let code = r#"
        let is_ready = true
        println(f"Ready: {is_ready}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Ready: true");
}

#[test]
fn test_fstring_with_float() {
    let code = r#"
        let pi = 3.14159
        println(f"Pi is approximately {pi}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Pi is approximately 3.14159");
}

#[test]
fn test_fstring_empty_interpolation() {
    let code = r#"
        println(f"No variables here")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "No variables here");
}

#[test]
fn test_fstring_with_escape_sequences() {
    let code = r#"
        let name = "Bob"
        println(f"Hello\n{name}\tWelcome!")
    "#;
    
    let output = run_ruchy_code(code);
    assert!(output.contains("Hello\nBob\tWelcome!"));
}

#[test]
fn test_fstring_array_indexing() {
    let code = r#"
        let arr = ["first", "second", "third"]
        println(f"Item: {arr[1]}")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Item: second");
}

// Regression test for the exact book example that was failing
#[test]
fn test_book_example_chapter2() {
    let code = r#"
        let name = "Alice"
        let age = 25
        let is_learning = true
        
        println(f"Hi {name}, you're {age} years old!")
        println(f"Currently learning Ruchy: {is_learning}")
    "#;
    
    let output = run_ruchy_code(code);
    assert!(output.contains("Hi Alice, you're 25 years old!"));
    assert!(output.contains("Currently learning Ruchy: true"));
}

// Test the exact pattern from book examples
#[test]
fn test_temperature_formatting() {
    let code = r#"
        let temperature = -10
        println(f"Temperature: {temperature}°C")
    "#;
    
    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Temperature: -10°C");
}