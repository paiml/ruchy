//! TDD tests for Error Handling functionality
//! 
//! These tests drive the implementation of error handling constructs
//! needed for Ch17 Error Handling examples in the book

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_transpile_assert_macro() {
    let code = r#"
        fun factorial(n: i32) -> i32 {
            assert!(n >= 0, "Factorial undefined for negative numbers");
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse assert macro");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile assert! macro");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("assert"), "Should contain assert macro in output");
    assert!(rust_code.contains("n >= 0"), "Should preserve condition");
    assert!(rust_code.contains("Factorial undefined"), "Should preserve error message");
}

#[test]
fn test_transpile_result_type_signature() {
    let code = r#"
        fun create_user(user: Json<User>) -> Result<Json<User>, Error> {
            // Implementation  
            Ok(user)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse Result type");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile Result<T,E> type");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("Result"), "Should preserve Result type");
    assert!(rust_code.contains("Json") && rust_code.contains("User"), "Should preserve generic types");
    assert!(rust_code.contains("Error"), "Should preserve Error type");
}

#[test]
fn test_transpile_ok_result() {
    let code = r#"
        fun safe_divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 {
                return Err("Division by zero");
            }
            Ok(a / b)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse Ok/Err results");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile Ok/Err expressions");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("Ok (") || rust_code.contains("Ok("), "Should contain Ok expression");
    assert!(rust_code.contains("Err (") || rust_code.contains("Err("), "Should contain Err expression");
    assert!(rust_code.contains("Division by zero"), "Should preserve error message");
}

#[test]
fn test_transpile_panic_macro() {
    let code = r#"
        fun validate_input(x: i32) {
            if x < 0 {
                panic!("Input must be non-negative, got: {}", x);
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse panic macro");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile panic! macro");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("panic"), "Should contain panic macro in output");
    assert!(rust_code.contains("Input must be non-negative"), "Should preserve panic message");
}

#[test]
fn test_transpile_unwrap_method() {
    let code = r#"
        let result = Some(42);
        let value = result.unwrap();
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse unwrap method");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile .unwrap() method");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("unwrap()") || rust_code.contains("unwrap ()"), "Should contain unwrap method call");
}

#[test]
fn test_transpile_expect_method() {
    let code = r#"
        let result = parse_number("42");
        let value = result.expect("Failed to parse number");
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse expect method");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile .expect() method");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("expect(") || rust_code.contains("expect ("), "Should contain expect method call");
    assert!(rust_code.contains("Failed to parse"), "Should preserve expect message");
}

#[test]
fn test_transpile_match_result() {
    let code = r#"
        match divide(10.0, 2.0) {
            Ok(value) => println("Result: {}", value),
            Err(error) => println("Error: {}", error),
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse match on Result");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile match on Result");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("Ok(") || rust_code.contains("Ok ("), "Should contain Ok pattern");
    assert!(rust_code.contains("Err(") || rust_code.contains("Err ("), "Should contain Err pattern");
}

#[test]
fn test_transpile_question_mark_operator() {
    let code = r#"
        fun process_file(path: &str) -> Result<String, Error> {
            let content = read_file(path)?;
            let processed = transform(content)?;
            Ok(processed)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse ? operator");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile ? operator");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("?"), "Should preserve ? operator");
}

#[test]
fn test_parse_doctest_attributes() {
    // This tests that we can parse doctest blocks with attributes
    let code = r#"
        /// # Examples
        /// 
        /// ```should_panic
        /// factorial(-1)
        /// ```
        /// 
        /// ```ignore
        /// let x = broken_code;
        /// ```
        /// 
        /// ```no_run
        /// expensive_operation();
        /// ```
        pub fn factorial(n: i32) -> i32 {
            assert!(n >= 0);
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse function with doctest attributes");
}

#[test]
fn test_book_example_documentation() {
    // This is the actual failing Ch17 example
    let code = r#"
        /// Calculates the factorial of a number.
        /// 
        /// # Examples
        /// 
        /// ```
        /// assert_eq!(factorial(0), 1)
        /// assert_eq!(factorial(5), 120)
        /// ```
        /// 
        /// ```should_panic
        /// factorial(-1)
        /// ```
        pub fn factorial(n: i32) -> i32 {
            assert!(n >= 0, "Factorial undefined for negative numbers");
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse book documentation example");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile book documentation example");
}