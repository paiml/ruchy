//! TDD Tests for Comprehensive Error Handling Implementation (ERROR-001)
//! 
//! These tests drive the implementation of error handling needed for Ch17
//! Following strict TDD: Red -> Green -> Refactor

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_try_catch_basic() {
    let code = r#"
        fun risky_operation() {
            try {
                let result = dangerous_function();
                println(result);
            } catch (e) {
                println("Error occurred: {}", e);
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse try-catch block: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile try-catch: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("match") || rust_code.contains("if let"), 
        "Should generate error handling code: {}", rust_code);
}

#[test]
fn test_try_catch_finally() {
    let code = r#"
        fun cleanup_example() {
            try {
                let file = open_file("data.txt");
                process(file);
            } catch (e) {
                log_error(e);
            } finally {
                cleanup_resources();
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse try-catch-finally: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile try-catch-finally: {:?}", transpiled.err());
}

#[test]
fn test_result_type_ok() {
    let code = r#"
        fun divide(a: i32, b: i32) -> Result<i32, String> {
            if b == 0 {
                return Err("Division by zero");
            }
            Ok(a / b)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Result<T,E> return type: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile Result type: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("Result<i32, String>"), 
        "Should preserve Result type: {}", rust_code);
    assert!(rust_code.contains("Ok") && rust_code.contains("Err"), 
        "Should have Ok/Err constructors: {}", rust_code);
}

#[test]
fn test_option_type() {
    let code = r#"
        fun find_user(id: i32) -> Option<User> {
            if id > 0 {
                Some(User { id: id, name: "Alice" })
            } else {
                None
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Option<T> type: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile Option type: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("Option<User>"), 
        "Should preserve Option type: {}", rust_code);
}

#[test]
fn test_panic_macro() {
    let code = r#"
        fun validate_input(x: i32) {
            if x < 0 {
                panic!("Input must be non-negative, got {}", x);
            }
            println("Valid input: {}", x);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse panic! macro: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile panic! macro: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("panic!"), 
        "Should generate panic! macro: {}", rust_code);
}

#[test]
fn test_assert_macro() {
    let code = r#"
        fun test_function() {
            let x = 5;
            assert!(x > 0, "x must be positive");
            assert_eq!(x, 5);
            assert_ne!(x, 10);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse assert macros: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile assert macros: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("assert!") || rust_code.contains("assert"), 
        "Should generate assert macros: {}", rust_code);
}

#[test]
fn test_question_mark_operator() {
    let code = r#"
        fun read_config() -> Result<Config, Error> {
            let contents = read_file("config.json")?;
            let config = parse_json(contents)?;
            Ok(config)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse ? operator: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile ? operator: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("?"), 
        "Should preserve ? operator: {}", rust_code);
}

#[test]
fn test_unwrap_methods() {
    let code = r#"
        fun process_optional(opt: Option<i32>) {
            let value = opt.unwrap();
            let safe_value = opt.unwrap_or(0);
            let computed = opt.unwrap_or_else(|| calculate_default());
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse unwrap methods: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile unwrap methods: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("unwrap"), 
        "Should have unwrap methods: {}", rust_code);
}

#[test]
fn test_error_propagation() {
    let code = r#"
        fun chain_operations() -> Result<i32, String> {
            let a = operation1()?;
            let b = operation2(a)?;
            let c = operation3(b)?;
            Ok(c)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse error propagation: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile error propagation: {:?}", transpiled.err());
}

#[test]
fn test_custom_error_types() {
    let code = r#"
        enum MyError {
            NotFound,
            InvalidInput(String),
            IoError { path: String, code: i32 }
        }
        
        fun risky_op() -> Result<(), MyError> {
            Err(MyError::NotFound)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse custom error enum: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile custom error: {:?}", transpiled.err());
}

#[test]
fn test_match_on_result() {
    let code = r#"
        fun handle_result(res: Result<i32, String>) {
            match res {
                Ok(value) => println("Success: {}", value),
                Err(msg) => println("Error: {}", msg)
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Result matching: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile Result matching: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("match") && rust_code.contains("Ok") && rust_code.contains("Err"), 
        "Should generate Result match: {}", rust_code);
}

#[test]
fn test_doctest_attributes() {
    let code = r#"
        /// ```should_panic
        /// let x = panic!("This should panic");
        /// ```
        fun test_panic() {
            panic!("Expected panic");
        }
        
        /// ```ignore
        /// expensive_operation();
        /// ```
        fun ignored_test() {
            // This test is ignored
        }
        
        /// ```no_run
        /// infinite_loop();
        /// ```
        fun no_run_example() {
            loop {}
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse doctest attributes: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile with doctest attributes: {:?}", transpiled.err());
}

#[test]
fn test_comprehensive_error_handling() {
    let code = r#"
        enum AppError {
            Io(String),
            Parse(String),
            Network { code: i32, message: String }
        }
        
        fun main() -> Result<(), AppError> {
            try {
                let config = load_config()?;
                let data = fetch_data(&config)?;
                process_data(data)?;
                Ok(())
            } catch (e) {
                match e {
                    AppError::Io(msg) => {
                        eprintln!("IO Error: {}", msg);
                        Err(e)
                    },
                    AppError::Parse(msg) => {
                        eprintln!("Parse Error: {}", msg);
                        Err(e)
                    },
                    AppError::Network { code, message } => {
                        eprintln!("Network Error {}: {}", code, message);
                        Err(e)
                    }
                }
            } finally {
                cleanup();
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse comprehensive error handling: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile comprehensive error handling: {:?}", transpiled.err());
}