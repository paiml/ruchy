//! TDD tests for TypeCast functionality
//! 
//! These tests drive the implementation of type casting (as operator)
//! which is needed for Ch04 practical patterns in the book

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_transpile_i32_to_f64() {
    let code = "let x = 42; x as f64";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile i32 to f64 cast");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("as f64"), "Should contain 'as f64' in output");
}

#[test]
fn test_transpile_f64_to_i32() {
    let code = "let x = 42.5; x as i32";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile f64 to i32 cast");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("as i32"), "Should contain 'as i32' in output");
}

#[test]
fn test_transpile_complex_cast_expression() {
    let code = "(raw_score as f64) / (max_score as f64) * 100.0";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile complex cast expression");
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("as f64"), "Should contain type casts");
    // Note: 100.0 becomes 100f64 in transpiled code, that's fine
    assert!(rust_code.contains("100"), "Should preserve numeric literal");
}


#[test]
fn test_book_example_process_score() {
    // This is the actual failing example from Ch04
    let code = r#"
        fun process_score(raw_score: i32, max_score: i32) -> f64 {
            if max_score <= 0 {
                return 0.0;
            }
            let percentage = (raw_score as f64) / (max_score as f64) * 100.0;
            percentage
        }
        
        fun main() {
            let score = process_score(85, 100);
            println("Score: {}", score);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse book example");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile book example with type casts");
}

#[test]
fn test_cast_with_method_call() {
    let code = "(percentage * 10.0).round() / 10.0";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile method call on cast expression");
}

