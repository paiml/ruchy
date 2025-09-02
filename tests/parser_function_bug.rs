//! CRITICAL PARSER BUG TEST - Fun token not handled in expressions.rs
//! This test MUST FAIL initially, then PASS after fix (TDD requirement)

use ruchy::Parser;

#[test] 
fn test_fun_keyword_parsing() {
    let mut parser = Parser::new("fun main() { println(\"Hello\") }");
    let result = parser.parse();
    
    // This MUST fail initially - parser doesn't handle Fun token in expressions.rs
    assert!(result.is_ok(), "Parser should handle 'fun' keyword in top-level expressions");
}

#[test]
fn test_fun_with_parameters() {
    let mut parser = Parser::new("fun greet(name: str) { println(\"Hello, {}!\", name) }");
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle 'fun' with typed parameters");
}

#[test]  
fn test_multiple_functions() {
    let code = r#"
        fun add(a: int, b: int) -> int {
            a + b
        }
        
        fun main() {
            println(add(1, 2))
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle multiple functions");
}

#[test]
fn test_function_with_body() {
    let code = r#"
        fun factorial(n: int) -> int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle functions with complex bodies");
}

#[test]
fn test_function_return_type() {
    let mut parser = Parser::new("fun get_answer() -> int { 42 }");
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle function return types");
}