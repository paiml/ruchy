#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! REPL function definition and calling tests

#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unwrap_used)]

use ruchy::runtime::Repl;

#[test]
fn test_function_definition() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval("fun add(a: i32, b: i32) -> i32 { a + b }");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "fn add(a, b)");
}

#[test]
fn test_function_call() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define function
    let result = repl.eval("fun add(a: i32, b: i32) -> i32 { a + b }");
    assert!(result.is_ok());

    // Call function
    let result = repl.eval("add(5, 3)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "8");
}

#[test]
fn test_function_with_local_vars() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define function that uses local variables
    let result =
        repl.eval("fun multiply_and_add(x: i32, y: i32) -> i32 { let prod = x * y; prod + 10 }");
    assert!(result.is_ok());

    // Call function
    let result = repl.eval("multiply_and_add(3, 4)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "22"); // 3 * 4 + 10 = 22
}

#[test]
fn test_function_wrong_arg_count() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define function
    let result = repl.eval("fun add(a: i32, b: i32) -> i32 { a + b }");
    assert!(result.is_ok());

    // Call with wrong number of arguments
    let result = repl.eval("add(5)");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expects 2 arguments, got 1"));
}

#[test]
fn test_recursive_function() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define factorial function
    let result = repl.eval("fun fact(n: i32) -> i32 { if n <= 1 { 1 } else { n * fact(n - 1) } }");
    assert!(result.is_ok());

    // Test factorial of 5
    let result = repl.eval("fact(5)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "120"); // 5! = 120
}

#[test]
fn test_function_with_string_params() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define function with strings
    let result = repl.eval(r#"fun greet(name: String) -> String { "Hello, " + name }"#);
    assert!(result.is_ok());

    // Call function
    let result = repl.eval(r#"greet("Alice")"#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, r#""Hello, Alice""#);
}

#[test]
fn test_multiple_functions() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define multiple functions
    assert!(repl
        .eval("fun add(a: i32, b: i32) -> i32 { a + b }")
        .is_ok());
    assert!(repl
        .eval("fun sub(a: i32, b: i32) -> i32 { a - b }")
        .is_ok());
    assert!(repl
        .eval("fun mul(a: i32, b: i32) -> i32 { a * b }")
        .is_ok());

    // Call them
    assert_eq!(repl.eval("add(10, 5)").unwrap(), "15");
    assert_eq!(repl.eval("sub(10, 5)").unwrap(), "5");
    assert_eq!(repl.eval("mul(10, 5)").unwrap(), "50");
}

#[test]
fn test_function_using_global_variable() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define global variable
    assert!(repl.eval("let global = 100").is_ok());

    // Define function that uses global
    assert!(repl
        .eval("fun use_global(x: i32) -> i32 { x + global }")
        .is_ok());

    // Call function
    let result = repl.eval("use_global(50)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, "150");
}
