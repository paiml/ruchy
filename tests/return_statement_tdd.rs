// TDD Test Suite for RETURN-STMT-001
// Bug: Explicit return statements return () instead of value
// Target: Fix 6+ examples in Ch17, Ch03, Ch04

use ruchy::runtime::repl::Repl;
use std::env;

fn eval(code: &str) -> String {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.eval(code).unwrap_or_else(|e| format!("Error: {}", e))
}

#[test]
fn test_explicit_return_integer() {
    let code = r#"
        fun add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        add(3, 4)
    "#;
    assert_eq!(eval(code), "7");
}

#[test]
fn test_explicit_return_float() {
    let code = r#"
        fun multiply(a: f64, b: f64) -> f64 {
            return a * b;
        }
        multiply(2.5, 4.0)
    "#;
    assert_eq!(eval(code), "10");
}

#[test]
fn test_explicit_return_string() {
    let code = r#"
        fun greet(name: &str) -> String {
            return "Hello, " + name;
        }
        greet("World")
    "#;
    assert_eq!(eval(code), "\"Hello, World\"");
}

#[test]
fn test_explicit_return_bool() {
    let code = r#"
        fun is_positive(n: i32) -> bool {
            return n > 0;
        }
        is_positive(5)
    "#;
    assert_eq!(eval(code), "true");
}

#[test]
fn test_early_return_in_if() {
    let code = r#"
        fun check_value(n: i32) -> &str {
            if n < 0 {
                return "negative";
            }
            if n == 0 {
                return "zero";
            }
            return "positive";
        }
        check_value(-5)
    "#;
    assert_eq!(eval(code), "\"negative\"");
}

#[test]
fn test_early_return_in_loop() {
    let code = r#"
        fun find_first_positive(arr: [i32]) -> i32 {
            let mut i = 0;
            while i < arr.len() {
                if arr[i] > 0 {
                    return arr[i];
                }
                i = i + 1;
            }
            return -1;
        }
        find_first_positive([-2, -1, 0, 3, 4])
    "#;
    assert_eq!(eval(code), "3");
}

#[test]
fn test_nested_function_returns() {
    let code = r#"
        fun outer(x: i32) -> i32 {
            fun inner(y: i32) -> i32 {
                return y * 2;
            }
            return inner(x) + 1;
        }
        outer(5)
    "#;
    assert_eq!(eval(code), "11");
}

#[test]
fn test_return_in_match() {
    let code = r#"
        fun classify(n: i32) -> &str {
            match n {
                0 => return "zero",
                n if n > 0 => return "positive",
                _ => return "negative"
            }
        }
        classify(42)
    "#;
    assert_eq!(eval(code), "\"positive\"");
}

#[test]
fn test_return_with_expression() {
    let code = r#"
        fun calculate(a: i32, b: i32) -> i32 {
            let sum = a + b;
            let product = a * b;
            return sum + product;
        }
        calculate(3, 4)
    "#;
    assert_eq!(eval(code), "19"); // 3+4=7, 3*4=12, 7+12=19
}

#[test]
fn test_void_function_explicit_return() {
    let code = r#"
        fun process(n: i32) -> i32 {
            if n < 0 {
                return n;  // Early return for negative
            }
            n * 2
        }
        
        process(-5)
    "#;
    assert_eq!(eval(code), "-5");
}

// Ch17 specific test - error handling with explicit returns
#[test]
fn test_ch17_safe_array_access() {
    let code = r#"
        fun safe_array_access(arr: [i32], index: i32) -> i32 {
            if index < 0 {
                println("Error: Negative index");
                return arr[0];  // Return first element as default
            }
            if index >= arr.len() {
                println("Error: Index out of bounds");
                return arr[arr.len() - 1];  // Return last element
            }
            return arr[index];
        }
        
        let data = [10, 20, 30, 40, 50];
        safe_array_access(data, 2)
    "#;
    assert_eq!(eval(code), "30");
}

// Ch03 specific test - function with multiple returns
#[test]
fn test_ch03_factorial_with_returns() {
    let code = r#"
        fun factorial(n: i32) -> i64 {
            if n < 0 {
                return 0;  // Error case
            }
            if n <= 1 {
                return 1;  // Base case
            }
            return (n as i64) * factorial(n - 1);
        }
        
        factorial(5)
    "#;
    assert_eq!(eval(code), "120");
}

// Ch04 specific test - practical pattern with early returns
#[test]
fn test_ch04_validation_pattern() {
    let code = r#"
        fun validate_age(age: i32) -> bool {
            if age < 0 {
                println("Error: Age cannot be negative");
                return false;
            }
            if age > 150 {
                println("Error: Age too high");
                return false;
            }
            return true;
        }
        
        validate_age(25)
    "#;
    assert_eq!(eval(code), "true");
}