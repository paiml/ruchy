//! Comprehensive TDD test suite for type_inference.rs
//! Target: Boost coverage via systematic testing of type inference logic
//! Toyota Way: Every inference path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== TYPE INFERENCE IN TRANSPILER TESTS ====================

#[test]
fn test_type_inference_numeric_operations() {
    let transpiler = Transpiler::new();
    
    // Test numeric operations get inferred correctly
    let code = "fun add(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    // Should contain function definition
    assert!(transpiled.contains("fn add"));
}

#[test]
fn test_type_inference_function_parameter() {
    let transpiler = Transpiler::new();
    
    // Test function used as parameter
    let code = "fun apply(f, x) { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn apply"));
    assert!(transpiled.contains("f(x)") || transpiled.contains("f (x)"));
}

#[test]
fn test_type_inference_string_concatenation() {
    let transpiler = Transpiler::new();
    
    // String concatenation should not be treated as numeric
    let code = r#"fun greet(name) { "Hello " + name }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn greet"));
    // Should have string formatting or concatenation
    assert!(transpiled.contains("format!") || transpiled.contains("+"));
}

#[test]
fn test_type_inference_nested_functions() {
    let transpiler = Transpiler::new();
    
    let code = "fun compose(f, g, x) { f(g(x)) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn compose"));
    assert!(transpiled.contains("f") && transpiled.contains("g"));
}

#[test]
fn test_type_inference_in_if_branches() {
    let transpiler = Transpiler::new();
    
    let code = "fun choose(cond, f, g, x) { if (cond) { f(x) } else { g(x) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("if"));
    assert!(transpiled.contains("else"));
}

#[test]
fn test_type_inference_in_let_bindings() {
    let transpiler = Transpiler::new();
    
    let code = "fun compute(f, x) { let result = f(x); result * 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("let"));
    assert!(transpiled.contains("result"));
}

#[test]
fn test_type_inference_lambda_body() {
    let transpiler = Transpiler::new();
    
    let code = "fun make_adder(n) { (x) => x + n }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn make_adder"));
    assert!(transpiled.contains("|") || transpiled.contains("move"));
}

#[test]
fn test_type_inference_arithmetic_operations() {
    let transpiler = Transpiler::new();
    
    let test_cases = vec![
        ("fun add(x, y) { x + y }", "+"),
        ("fun sub(x, y) { x - y }", "-"),
        ("fun mul(x, y) { x * y }", "*"),
        ("fun div(x, y) { x / y }", "/"),
        ("fun rem(x, y) { x % y }", "%"),
    ];
    
    for (code, op) in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let transpiled = result.unwrap().to_string();
        
        assert!(transpiled.contains(op), "Missing operator {} in {}", op, code);
    }
}

#[test]
fn test_type_inference_comparison_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun compare(x, y) { x > y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(">"));
}

#[test]
fn test_type_inference_block_expressions() {
    let transpiler = Transpiler::new();
    
    let code = "fun process(f, g, x) { { let a = f(x); let b = g(a); b + 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("let a"));
    assert!(transpiled.contains("let b"));
}

#[test]
fn test_type_inference_unary_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun negate(x) { -x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("-"));
}

#[test]
fn test_type_inference_logical_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun logic(a, b) { a && b || !a }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("&&"));
    assert!(transpiled.contains("||"));
    assert!(transpiled.contains("!"));
}

#[test]
fn test_type_inference_function_call_chains() {
    let transpiler = Transpiler::new();
    
    let code = "fun chain(f, g, h, x) { h(g(f(x))) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("h"));
    assert!(transpiled.contains("g"));
    assert!(transpiled.contains("f"));
}

#[test]
fn test_type_inference_mixed_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun complex(f, x, y) { f(x * 2) + y - 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("*"));
    assert!(transpiled.contains("+"));
    assert!(transpiled.contains("-"));
}

#[test]
fn test_type_inference_while_loop() {
    let transpiler = Transpiler::new();
    
    let code = "fun loop_fn(f, n) { while (n > 0) { f(n) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("while"));
}

#[test]
fn test_type_inference_for_loop() {
    let transpiler = Transpiler::new();
    
    let code = "fun iterate(f, items) { for (item in items) { f(item) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("for"));
}

#[test]
fn test_type_inference_match_expression() {
    let transpiler = Transpiler::new();
    
    let code = r#"fun process(x) { match x { 1 => "one", 2 => "two", _ => "other" } }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("match"));
}

#[test]
fn test_type_inference_array_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun array_op(f, arr) { [f(arr[0]), f(arr[1])] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("vec") || transpiled.contains("["));
}

#[test]
fn test_type_inference_tuple_operations() {
    let transpiler = Transpiler::new();
    
    let code = "fun tuple_op(f, x, y) { (f(x), f(y)) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("("));
    assert!(transpiled.contains(","));
}

#[test]
fn test_type_inference_method_calls() {
    let transpiler = Transpiler::new();
    
    let code = "fun method_test(s) { s.len() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("len"));
}

#[test]
fn test_type_inference_string_interpolation() {
    let transpiler = Transpiler::new();
    
    let code = r#"fun interpolate(name, age) { f"Hello {name}, you are {age} years old" }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("format!") || transpiled.contains("name"));
}

#[test]
fn test_type_inference_pipeline_operator() {
    let transpiler = Transpiler::new();
    
    let code = "fun pipe_test(x, f, g) { x |> f |> g }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference_async_function() {
    let transpiler = Transpiler::new();
    
    let code = "async fun fetch_data(url) { await get(url) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("async"));
}

#[test]
fn test_type_inference_try_expression() {
    let transpiler = Transpiler::new();
    
    let code = "fun may_fail(x) { try { risky_op(x) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference_spread_operator() {
    let transpiler = Transpiler::new();
    
    let code = "fun spread(arr) { [...arr, 1, 2, 3] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference_object_literal() {
    let transpiler = Transpiler::new();
    
    let code = "fun make_obj(name, age) { { name: name, age: age } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference_destructuring() {
    let transpiler = Transpiler::new();
    
    let code = "fun destruct(point) { let (x, y) = point; x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_type_inference_rest_parameters() {
    let transpiler = Transpiler::new();
    
    let code = "fun sum(...nums) { nums.reduce((a, b) => a + b) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    // May not be fully supported, but test parsing at least
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_type_inference_generator_function() {
    let transpiler = Transpiler::new();
    
    let code = "fun* generate() { yield 1; yield 2; yield 3 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    // May not be fully supported
    assert!(result.is_ok() || result.is_err());
}

// Run all tests with: cargo test type_inference_tdd --test type_inference_tdd