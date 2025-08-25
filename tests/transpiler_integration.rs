//! Integration tests for transpiler coverage improvement
//! Focus on simple, working patterns to boost coverage

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

/// Test complete program transpilation
#[test]
fn test_complete_programs() {
    let transpiler = Transpiler::new();
    
    let programs = [
        // Simple function program
        "fun main() { println(42) }",
        
        // Program with multiple functions
        "fun add(x: i32, y: i32) -> i32 { x + y }\nfun main() { add(1, 2) }",
        
        // Program with variables
        "fun main() { let x = 42; let y = x + 1; println(y) }",
        
        // Program with if-else
        "fun main() { if true { println(1) } else { println(0) } }",
        
        // Program with loops
        "fun main() { for i in 0..10 { println(i) } }",
        
        // Program with match
        "fun main() { match 1 { 1 => println(\"one\"), _ => println(\"other\") } }",
    ];
    
    for program in programs {
        let mut parser = Parser::new(program);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", program));
        
        let result = transpiler.transpile_to_program(&ast)
            .expect(&format!("Failed to transpile program: {}", program));
        let code = result.to_string();
        
        // Should generate valid Rust program structure
        assert!(code.contains("fn main"), "Program should contain main function: {}", program);
        assert!(code.contains("{"), "Program should have block structure: {}", program);
        assert!(code.contains("}"), "Program should have closing braces: {}", program);
    }
}

/// Test expression-only programs (one-liners)
#[test]
fn test_expression_programs() {
    let transpiler = Transpiler::new();
    
    let expressions = [
        "42",
        "1 + 2",
        "\"hello\"",
        "true && false",
        "[1, 2, 3]",
        "if true { 1 } else { 0 }",
    ];
    
    for expr in expressions {
        let mut parser = Parser::new(expr);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", expr));
        
        let result = transpiler.transpile_to_program(&ast)
            .expect(&format!("Failed to transpile expression: {}", expr));
        let code = result.to_string();
        
        // Expression programs should be wrapped in main
        assert!(code.contains("fn main"), "Expression '{}' should be wrapped in main", expr);
    }
}

/// Test all statement types
#[test]
fn test_all_statements() {
    let transpiler = Transpiler::new();
    
    let statements = [
        // Let statements
        "let x = 42",
        "let mut y = 0",
        "let z: i32 = 10",
        
        // Assignments
        "x = 42",
        "y += 1",
        
        // Expression statements
        "println(\"hello\")",
        "vec.push(1)",
        
        // Return statements
        "return 42",
        "return",
        
        // Break and continue
        "break",
        "continue",
    ];
    
    for stmt in statements {
        let mut parser = Parser::new(stmt);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", stmt));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile statement: {}", stmt));
        let code = result.to_string();
        
        // Statements should transpile to valid Rust
        assert!(!code.is_empty(), "Statement '{}' should produce output", stmt);
    }
}

/// Test all expression types
#[test]
fn test_all_expressions() {
    let transpiler = Transpiler::new();
    
    let expressions = [
        // Literals
        "42", "3.14", "true", "\"hello\"", "'c'", "()",
        
        // Binary operations
        "1 + 2", "5 - 3", "4 * 6", "10 / 2", "7 % 3",
        "5 > 3", "2 < 8", "3 == 3", "5 != 2",
        "true && false", "true || false",
        
        // Unary operations
        "-42", "!true",
        
        // Variables
        "x", "my_var",
        
        // Function calls
        "foo()", "bar(1, 2)",
        
        // Method calls
        "obj.method()", "vec.push(1)",
        
        // Arrays
        "[1, 2, 3]", "[]",
        
        // Tuples
        "(1, 2)", "()",
        
        // Blocks
        "{ 42 }", "{ let x = 1; x + 1 }",
        
        // If expressions
        "if true { 1 } else { 0 }",
        
        // Lambdas
        "|x| x + 1", "|| 42",
    ];
    
    for expr in expressions {
        let mut parser = Parser::new(expr);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", expr));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile expression: {}", expr));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "Expression '{}' should produce output", expr);
    }
}

/// Test function definitions
#[test]
fn test_function_definitions() {
    let transpiler = Transpiler::new();
    
    let functions = [
        // Simple function
        "fun test() { }",
        "fun test() -> i32 { 42 }",
        
        // Function with parameters
        "fun add(x: i32, y: i32) -> i32 { x + y }",
        
        // Function with generics (if supported)
        "fun identity<T>(x: T) -> T { x }",
        
        // Async function
        "async fun fetch() -> String { \"data\" }",
        
        // Function with multiple statements
        "fun complex() -> i32 { let x = 1; let y = 2; x + y }",
    ];
    
    for func in functions {
        let mut parser = Parser::new(func);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", func));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile function: {}", func));
        let code = result.to_string();
        
        assert!(code.contains("fn"), "Function '{}' should contain 'fn' keyword", func);
    }
}

/// Test control flow structures
#[test]
fn test_control_flow() {
    let transpiler = Transpiler::new();
    
    let control_flows = [
        // If-else
        "if x > 0 { x } else { -x }",
        "if x > 0 { 1 } else if x < 0 { -1 } else { 0 }",
        
        // Match
        "match x { 0 => \"zero\", _ => \"other\" }",
        
        // For loop
        "for i in 0..10 { println(i) }",
        "for x in vec { x * 2 }",
        
        // While loop
        "while x > 0 { x = x - 1 }",
        
        // Loop
        "loop { break }",
    ];
    
    for flow in control_flows {
        let mut parser = Parser::new(flow);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", flow));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile control flow: {}", flow));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "Control flow '{}' should produce output", flow);
    }
}

/// Test type system features
#[test]
fn test_type_features() {
    let transpiler = Transpiler::new();
    
    let type_features = [
        // Type annotations
        "let x: i32 = 42",
        "let y: String = \"hello\"",
        "let z: Vec<i32> = vec![1, 2, 3]",
        
        // Generic types
        "let opt: Option<i32> = Some(42)",
        "let res: Result<i32, String> = Ok(42)",
        
        // Struct definitions
        "struct Point { x: i32, y: i32 }",
        
        // Enum definitions
        "enum Color { Red, Green, Blue }",
        
        // Type aliases
        "type NodeId = i32",
    ];
    
    for feature in type_features {
        let mut parser = Parser::new(feature);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", feature));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile type feature: {}", feature));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "Type feature '{}' should produce output", feature);
    }
}

/// Test error handling
#[test]
fn test_error_handling() {
    let transpiler = Transpiler::new();
    
    let error_handling = [
        // Result types
        "Ok(42)",
        "Err(\"error\")",
        
        // Option types
        "Some(42)",
        "None",
        
        // Try operator
        "foo()?",
        
        // Panic
        "panic(\"error\")",
    ];
    
    for handling in error_handling {
        let mut parser = Parser::new(handling);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", handling));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile error handling: {}", handling));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "Error handling '{}' should produce output", handling);
    }
}

/// Test operators
#[test]
fn test_operators() {
    let transpiler = Transpiler::new();
    
    let operators = [
        // Arithmetic
        "a + b", "a - b", "a * b", "a / b", "a % b",
        
        // Comparison
        "a > b", "a < b", "a >= b", "a <= b", "a == b", "a != b",
        
        // Logical
        "a && b", "a || b", "!a",
        
        // Bitwise
        "a & b", "a | b", "a ^ b", "a << b", "a >> b",
        
        // Assignment
        "a = b", "a += b", "a -= b", "a *= b", "a /= b",
        
        // Range
        "0..10", "0..=10",
        
        // Member access
        "obj.field", "obj.method()",
        
        // Index
        "arr[0]",
    ];
    
    for op in operators {
        let mut parser = Parser::new(op);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", op));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile operator: {}", op));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "Operator '{}' should produce output", op);
    }
}

/// Test string features
#[test]
fn test_string_features() {
    let transpiler = Transpiler::new();
    
    let string_features = [
        // String literals
        "\"hello\"",
        "\"world\"",
        
        // String interpolation
        "f\"Hello {name}\"",
        "f\"x = {x}, y = {y}\"",
        
        // String methods
        "\"hello\".to_upper()",
        "\"WORLD\".to_lower()",
        "\"test\".len()",
        
        // String concatenation
        "\"hello\" + \" \" + \"world\"",
    ];
    
    for feature in string_features {
        let mut parser = Parser::new(feature);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", feature));
        
        let result = transpiler.transpile(&ast)
            .expect(&format!("Failed to transpile string feature: {}", feature));
        let code = result.to_string();
        
        assert!(!code.is_empty(), "String feature '{}' should produce output", feature);
    }
}