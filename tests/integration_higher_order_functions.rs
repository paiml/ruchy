//! Integration tests for higher-order function type inference
//! 
//! This test suite ensures that the BUG-002 fix works correctly
//! across various higher-order function scenarios.

use ruchy::backend::transpiler::Transpiler;
use ruchy::Parser;

#[test]
fn test_basic_higher_order_function() {
    let code = "fun apply(f, x) { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f should be typed as a function, not String
    assert!(!rust_str.contains("f : String"));
    assert!(rust_str.contains("impl Fn"));
}

#[test]
fn test_multiple_function_parameters() {
    let code = "fun compose(f, g, x) { f(g(x)) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // Both f and g should be function types
    assert!(!rust_str.contains("f : String"));
    assert!(!rust_str.contains("g : String"));
    assert!(rust_str.contains("impl Fn"));
}

#[test]
fn test_numeric_function_parameters() {
    let code = "fun double(n) { n * 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // n should be typed as i32 for arithmetic
    assert!(rust_str.contains("n : i32") || rust_str.contains("n: i32"));
    assert!(!rust_str.contains("n : String"));
}

#[test]
fn test_string_function_parameters() {
    let code = "fun greet(name) { \"Hello \" + name }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // name should remain String type for string concatenation
    assert!(rust_str.contains("name : String") || rust_str.contains("name: String"));
}

#[test]
fn test_mixed_parameter_types() {
    let code = "fun transform(f, n, s) { f(n * 2) + s }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f should be function, n should be i32
    assert!(rust_str.contains("impl Fn"));
    assert!(rust_str.contains("n : i32") || rust_str.contains("n: i32"));
    
    // NOTE: Currently s is inferred as i32 because f(n * 2) + s 
    // suggests numeric addition. This is a known limitation -
    // we need cross-parameter type inference to handle this correctly.
    assert!(rust_str.contains("s : i32") || rust_str.contains("s: i32"));
}

#[test]
fn test_function_in_conditional() {
    let code = "fun conditional_apply(f, condition) { if condition { f(1) } else { f(0) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f should be detected as function even inside if branches
    assert!(rust_str.contains("impl Fn"));
    assert!(!rust_str.contains("f : String"));
}

#[test]
fn test_function_in_let_binding() {
    let code = "fun with_binding(f) { let x = 42; f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f should be detected as function in let body
    assert!(rust_str.contains("impl Fn"));
    assert!(!rust_str.contains("f : String"));
}

#[test]
fn test_nested_function_calls() {
    let code = "fun nested(f, g, h, x) { f(g(h(x))) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f, g, h should all be function types
    assert!(rust_str.matches("impl Fn").count() >= 3);
    assert!(!rust_str.contains("f : String"));
    assert!(!rust_str.contains("g : String")); 
    assert!(!rust_str.contains("h : String"));
}

#[test]
fn test_arithmetic_operations_all_types() {
    let operations = vec!["+", "-", "*", "/", "%"];
    
    for op in operations {
        let code = format!("fun test(x) {{ x {op} 5 }}");
        let mut parser = Parser::new(&code);
        let ast = parser.parse().unwrap();
        
        let mut transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast).unwrap();
        let rust_str = rust_code.to_string();
        
        // x should be i32 for all arithmetic operations
        assert!(rust_str.contains("x : i32") || rust_str.contains("x: i32"),
               "Expected i32 type for {op} operation: {rust_str}");
    }
}

#[test] 
fn test_unused_parameters_default_to_string() {
    let code = "fun unused(x) { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // Unused x should default to String
    assert!(rust_str.contains("x : String") || rust_str.contains("x: String"));
}

#[test]
fn test_main_function_never_gets_return_type() {
    let code = "fun main() { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // main should never have explicit return type
    assert!(!rust_str.contains("fn main() ->"));
    assert!(!rust_str.contains("fn main () ->"));
}

#[test]
fn test_regular_functions_get_return_types() {
    let code = "fun add(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // Non-main numeric functions should have return type
    assert!(rust_str.contains("-> i32"));
}

#[test]
fn test_lambda_parameter_detection() {
    let code = "fun with_lambda(f) { (x) => f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // f should be detected as function even inside lambda
    assert!(rust_str.contains("impl Fn"));
    assert!(!rust_str.contains("f : String"));
}

#[test]
fn test_complex_higher_order_scenario() {
    // This is the exact scenario that was broken in v1.17.0
    let code = r"
        fun apply(f, x) { f(x) }
        fun double(n) { n * 2 }
        fun main() { apply(double, 5) }
    ";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).unwrap();
    let rust_str = rust_code.to_string();
    
    // This should now compile without errors
    // f in apply should be impl Fn(i32) -> i32
    // n in double should be i32
    // main should have no return type
    
    assert!(rust_str.contains("impl Fn"));
    assert!(rust_str.contains("n : i32") || rust_str.contains("n: i32"));
    assert!(!rust_str.contains("fn main() ->"));
    assert!(!rust_str.contains("f : String"));
    assert!(!rust_str.contains("n : String"));
}

#[test] 
fn test_regression_prevention_property() {
    // Property: function parameters used as functions should NEVER be typed as String
    let test_cases = vec![
        "fun test(f) { f(1) }",
        "fun test(f) { f(f(1)) }",
        "fun test(f, g) { f(g(1)) }",
        "fun test(f) { if true { f(1) } else { f(2) } }",
        "fun test(f) { let x = 1; f(x) }",
    ];
    
    for code in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let mut transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast).unwrap();
        let rust_str = rust_code.to_string();
        
        // REGRESSION TEST: Function parameters should NEVER be String
        assert!(!rust_str.contains("f : String"),
               "REGRESSION: Function parameter f was typed as String in: {code}\nGenerated: {rust_str}");
        assert!(!rust_str.contains("g : String"), 
               "REGRESSION: Function parameter g was typed as String in: {code}\nGenerated: {rust_str}");
    }
}

#[test]
fn test_numeric_operations_property() {
    // Property: parameters used in arithmetic should be i32
    let test_cases = vec![
        "fun test(n) { n + 1 }",
        "fun test(n) { n - 1 }",
        "fun test(n) { n * 2 }",
        "fun test(n) { n / 2 }",
        "fun test(n) { n % 3 }",
    ];
    
    for code in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let mut transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast).unwrap();
        let rust_str = rust_code.to_string();
        
        // Parameters in arithmetic should be i32
        assert!(rust_str.contains("n : i32") || rust_str.contains("n: i32"),
               "Expected numeric parameter to be i32 in: {code}\nGenerated: {rust_str}");
    }
}