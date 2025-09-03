//! Comprehensive TDD test suite for result_type.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every Result type path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== RESULT CONSTRUCTOR TESTS ====================

#[test]
fn test_transpile_ok_constructor() {
    let transpiler = Transpiler::new();
    let code = "Ok(42)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Ok"));
    assert!(transpiled.contains("42"));
}

#[test]
fn test_transpile_err_constructor() {
    let transpiler = Transpiler::new();
    let code = r#"Err("error message")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Err"));
    assert!(transpiled.contains("error message"));
}

#[test]
fn test_transpile_result_type_annotation() {
    let transpiler = Transpiler::new();
    let code = "let x: Result<i32, String> = Ok(42)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Result"));
    assert!(transpiled.contains("Ok"));
}

// ==================== RESULT PATTERN MATCHING TESTS ====================

#[test]
fn test_transpile_result_match_ok_err() {
    let transpiler = Transpiler::new();
    let code = "match result { Ok(val) => val, Err(e) => 0 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("match"));
    assert!(transpiled.contains("Ok"));
    assert!(transpiled.contains("Err"));
}

#[test]
fn test_transpile_result_match_with_binding() {
    let transpiler = Transpiler::new();
    let code = "match parse_int(s) { Ok(n) => n * 2, Err(_) => -1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Ok"));
    assert!(transpiled.contains("Err"));
}

#[test]
fn test_transpile_nested_result_match() {
    let transpiler = Transpiler::new();
    let code = r#"
    match outer {
        Ok(inner_result) => match inner_result {
            Ok(val) => val,
            Err(e) => 0
        },
        Err(_) => -1
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("match"));
}

// ==================== QUESTION MARK OPERATOR TESTS ====================

#[test]
fn test_transpile_question_mark() {
    let transpiler = Transpiler::new();
    let code = "result?";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("?"));
}

#[test]
fn test_transpile_chained_question_mark() {
    let transpiler = Transpiler::new();
    let code = "parse_int(s)?.checked_add(1)?";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("?"));
}

#[test]
fn test_transpile_question_mark_in_function() {
    let transpiler = Transpiler::new();
    let code = r#"
    fun process(s: String) -> Result<i32, String> {
        let n = parse_int(s)?;
        Ok(n * 2)
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("?"));
    assert!(transpiled.contains("Ok"));
}

// ==================== RESULT COMBINATOR TESTS ====================

#[test]
fn test_transpile_result_map() {
    let transpiler = Transpiler::new();
    let code = "result.map(|x| x * 2)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("map"));
}

#[test]
fn test_transpile_result_map_err() {
    let transpiler = Transpiler::new();
    let code = "result.map_err(|e| format(\"Error: {}\", e))";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("map_err"));
}

#[test]
fn test_transpile_result_and_then() {
    let transpiler = Transpiler::new();
    let code = "result.and_then(|x| Ok(x + 1))";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("and_then"));
}

#[test]
fn test_transpile_result_or_else() {
    let transpiler = Transpiler::new();
    let code = "result.or_else(|e| Err(format(\"Failed: {}\", e)))";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("or_else"));
}

#[test]
fn test_transpile_result_unwrap() {
    let transpiler = Transpiler::new();
    let code = "result.unwrap()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("unwrap"));
}

#[test]
fn test_transpile_result_unwrap_or() {
    let transpiler = Transpiler::new();
    let code = "result.unwrap_or(0)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("unwrap_or"));
}

#[test]
fn test_transpile_result_unwrap_or_else() {
    let transpiler = Transpiler::new();
    let code = "result.unwrap_or_else(|_| default_value())";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("unwrap_or_else"));
}

#[test]
fn test_transpile_result_expect() {
    let transpiler = Transpiler::new();
    let code = r#"result.expect("Value must be present")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("expect"));
}

// ==================== RESULT CHAIN TESTS ====================

#[test]
fn test_transpile_result_chain_multiple() {
    let transpiler = Transpiler::new();
    let code = "parse_int(s).map(|n| n * 2).and_then(|n| Ok(n + 1))";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("map"));
    assert!(transpiled.contains("and_then"));
}

#[test]
fn test_transpile_result_is_ok() {
    let transpiler = Transpiler::new();
    let code = "result.is_ok()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("is_ok"));
}

#[test]
fn test_transpile_result_is_err() {
    let transpiler = Transpiler::new();
    let code = "result.is_err()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("is_err"));
}

// ==================== IF LET PATTERN TESTS ====================

#[test]
fn test_transpile_if_let_ok() {
    let transpiler = Transpiler::new();
    let code = "if let Ok(value) = result { value } else { 0 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("if let"));
    assert!(transpiled.contains("Ok"));
}

#[test]
fn test_transpile_if_let_err() {
    let transpiler = Transpiler::new();
    let code = r#"if let Err(e) = result { println("Error: {}", e) }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("if let"));
    assert!(transpiled.contains("Err"));
}

// ==================== RESULT IN FUNCTION SIGNATURE TESTS ====================

#[test]
fn test_transpile_function_returning_result() {
    let transpiler = Transpiler::new();
    let code = r#"
    fun divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 {
            Err("Division by zero")
        } else {
            Ok(a / b)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Result"));
    assert!(transpiled.contains("Ok"));
    assert!(transpiled.contains("Err"));
}

#[test]
fn test_transpile_function_with_result_parameter() {
    let transpiler = Transpiler::new();
    let code = "fun handle_result(r: Result<i32, String>) { }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Result"));
}

// ==================== RESULT HELPERS TEST ====================

#[test]
fn test_generate_result_helpers() {
    let helpers = Transpiler::generate_result_helpers();
    let code = helpers.to_string();
    
    assert!(code.contains("trait ResultExt"));
    assert!(code.contains("map_err_with"));
    assert!(code.contains("unwrap_or_else_with"));
    assert!(code.contains("and_then_with"));
    assert!(code.contains("or_else_with"));
}

// Run all tests with: cargo test result_type_tdd --test result_type_tdd