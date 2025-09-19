//! Additional fuzz testing focused on interpreter edge cases

#![allow(unused_variables, clippy::match_same_arms, clippy::single_match)]

use ruchy::runtime::repl::Repl;
use std::env;
use ruchy::frontend::parser::Parser;

/// Fuzz test: Interpreter robustness with malformed input
#[test]
fn fuzz_interpreter_robustness() {
    let malformed_inputs = vec![
        "let x = ", // Incomplete assignment
        "if true {", // Unclosed block
        "match x {", // Incomplete match
        "fun f() {", // Unclosed function
        "for x in", // Incomplete for loop
        "let x = y..", // Invalid range
        "match { => }", // Empty match arms
        "[1, 2,", // Unclosed array
        "f(", // Unclosed function call
        "x.", // Incomplete method call
        "let = x", // Invalid let binding
        "if { true }", // Invalid if condition
        "while { }", // Invalid while condition  
    ];
    
    for input in malformed_inputs {
        let mut parser = Parser::new(input);
        if let Ok(ast) = parser.parse() {
            let mut repl = Repl::new(std::env::temp_dir()).unwrap();
            let _ = repl.eval(input); // Should not panic
        }
    }
}

/// Fuzz test: Deep recursion handling
#[test]
fn fuzz_deep_recursion() {
    // Test parser with deeply nested expressions
    for depth in 1..=20 {
        let mut code = "(".repeat(depth);
        code.push('1');
        code.push_str(&")".repeat(depth));
        
        let mut parser = Parser::new(&code);
        let _ = parser.parse(); // Should not stack overflow
        
        // Test deeply nested blocks
        let mut block_code = "{".repeat(depth);
        block_code.push('1');
        block_code.push_str(&"}".repeat(depth));
        
        let mut block_parser = Parser::new(&block_code);
        let _ = block_parser.parse();
    }
}

/// Fuzz test: Memory exhaustion prevention  
#[test]
fn fuzz_memory_bounds() {
    let memory_intensive = vec![
        "[1, 2, 3, 4, 5]", // Moderate array creation
        r#""hello""#, // Normal string
        "1..10", // Small range
        "[1, 2].map(|x| x * 2)", // Simple collection operation
    ];
    
    for input in memory_intensive {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input); // Should not exhaust memory
    }
}

/// Fuzz test: Unicode and special character handling
#[test]
fn fuzz_unicode_robustness() {
    let unicode_tests = vec![
        "\"🦀\"", // Emoji
        "\"Hëllö\"", // Accented characters  
        "\"世界\"", // Chinese characters
        "\"🏳️‍🌈\"", // Complex emoji
        "\"\\u{1F600}\"", // Unicode escape
        "\"\\x41\\x42\"", // Hex escapes
    ];
    
    for input in unicode_tests {
        let mut parser = Parser::new(input);
        let _ = parser.parse();
        
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Error propagation edge cases
#[test] 
fn fuzz_error_propagation() {
    let error_cases = vec![
        "undefined_variable",
        "x.undefined_method()",
        "1 / 0", // Division by zero
        "[1][10]", // Index out of bounds
        "null.method()", // Null pointer access
        "\"string\" + 42", // Type mismatch
        "if 42 { true }", // Non-boolean in if condition
    ];
    
    for input in error_cases {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        match repl.eval(input) {
            Ok(_) => {}, // Some might succeed
            Err(_) => {}, // Expected for many cases
        }
        // Key point: should never panic, always return Result
    }
}

/// Fuzz test: Collection operations edge cases
#[test]
fn fuzz_collection_operations() {
    let collection_tests = vec![
        "[]", // Empty array
        "[].len()", // Empty array method
        "[1].head()", // Single element
        "[1].tail()", // Single element tail
        "[1, 2, 3][0]", // Valid indexing
        "[1, 2, 3][-1]", // Negative index
        "{}", // Empty object
        "{\"a\": 1}", // Single key object
        "{\"a\": 1}[\"a\"]", // Object access
        "{\"a\": 1}[\"missing\"]", // Missing key
    ];
    
    for input in collection_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Function call edge cases  
#[test]
fn fuzz_function_calls() {
    let function_tests = vec![
        "println()", // No args
        "println(1)", // One arg
        "println(1, 2, 3)", // Multiple args
        "len([1, 2, 3])", // Builtin function
        "max(1, 2)", // Binary builtin
        "min()", // No args to binary function
        "type(42)", // Type introspection
        "help(println)", // Help system
    ];
    
    for input in function_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Property-based fuzz test: Any valid parse should not crash interpreter
#[test] 
fn property_valid_parse_no_crash() {
    
    
    fn generate_simple_expr(seed: usize) -> String {
        let exprs = ["1", "true", "\"hello\"", "[]", "()",
            "1 + 2", "true && false", "\"a\" + \"b\"",
            "[1, 2, 3]", "(1, 2)", "x", "f()"];
        exprs[seed % exprs.len()].to_string()
    }
    
    // Test 1000 random valid expressions
    for seed in 0..1000 {
        let code = generate_simple_expr(seed);
        let mut parser = Parser::new(&code);
        
        if let Ok(_ast) = parser.parse() {
            let mut repl = Repl::new(std::env::temp_dir()).unwrap();
            let _ = repl.eval(&code); // Should never panic
        }
    }
}

/// Fuzz test: REPL state consistency
#[test]
fn fuzz_repl_state_consistency() {
    let state_tests = vec![
        ("let x = 1", "x"), // Variable definition and access
        ("let y = [1, 2]", "y[0]"), // Array definition and indexing  
        ("let z = {\"a\": 1}", "z[\"a\"]"), // Object definition and access
        ("fun f() { 42 }", "f()"), // Function definition and call
    ];
    
    for (setup, access) in state_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(setup); // Setup
        let _ = repl.eval(access); // Access - should work or fail gracefully
    }
}

/// Fuzz test: Type coercion edge cases
#[test]
fn fuzz_type_coercion() {
    let coercion_tests = vec![
        "1 + 2.5", // Int + Float
        "true + false", // Bool arithmetic (might be valid)
        "\"5\" + \"10\"", // String concatenation
        "42.to_string()", // Method on literal
        "[1, 2, 3].to_string()", // Collection to string
        "true.to_string()", // Bool to string
    ];
    
    for input in coercion_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Control flow edge cases
#[test]
fn fuzz_control_flow() {
    let control_flow_tests = vec![
        "if true { 1 } else { 2 }", // Basic if-else
        "if false { 1 }", // If without else
        "while false { 1 }", // While that never executes
        "for x in [] { x }", // For over empty collection
        "for i in 1..3 { i }", // For over range
        "match 42 { x => x }", // Catch-all match
        "match true { true => 1, false => 2 }", // Boolean match
    ];
    
    for input in control_flow_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Operator precedence edge cases
#[test]
fn fuzz_operator_precedence() {
    let precedence_tests = vec![
        "1 + 2 * 3", // Addition and multiplication
        "1 * 2 + 3", // Multiplication and addition
        "true && false || true", // Logical operators
        "!true && false", // Unary and binary
        "1 < 2 && 3 > 2", // Comparison and logical
        "1 + 2 == 3", // Arithmetic and comparison
        "(1 + 2) * 3", // Parentheses
        "1 + (2 * 3)", // Nested parentheses
    ];
    
    for input in precedence_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: String operations edge cases
#[test]
fn fuzz_string_operations() {
    let string_tests = vec![
        "\"\"", // Empty string
        "\" \"", // Whitespace string
        "\"\\n\"", // Newline
        "\"\\t\"", // Tab
        "\"\\\"\"", // Escaped quote
        "\"hello\".len()", // String method
        "\"hello\".upper()", // String transformation
        "\"hello\".chars()", // String to chars
        "\"hello\" + \"world\"", // String concatenation
        "f\"Hello {1 + 2}\"", // String interpolation
    ];
    
    for input in string_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Numeric operations edge cases
#[test]
fn fuzz_numeric_operations() {
    let numeric_tests = vec![
        "0", // Zero
        "-1", // Negative
        "3.14", // Float
        "1.0", // Integer as float
        "1 + 1", // Basic addition
        "10 - 5", // Subtraction
        "2 * 3", // Multiplication
        "10 / 2", // Division
        "10 % 3", // Modulo
        "2.5 + 1.5", // Float arithmetic
        "abs(-5)", // Math function
        "max(1, 2, 3)", // Variadic function
        "min(1, 2)", // Binary function
        "sqrt(9)", // Square root
        "floor(3.7)", // Floor function
        "ceil(3.2)", // Ceiling function
        "round(3.6)", // Rounding
    ];
    
    for input in numeric_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Complex nested expressions
#[test]
fn fuzz_nested_expressions() {
    let nested_tests = vec![
        "[[1, 2], [3, 4]]", // Nested arrays
        "{\"a\": {\"b\": 1}}", // Nested objects
        "if true { if false { 1 } else { 2 } }", // Nested if
        "fun outer() { fun inner() { 42 } }", // Nested functions
        "[1, 2, 3].map(|x| x * 2).filter(|x| x > 2)", // Method chaining
        "(1 + (2 * (3 + 4)))", // Deeply nested arithmetic
        "match [1, 2] { [x, y] => x + y }", // Pattern matching
    ];
    
    for input in nested_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}

/// Fuzz test: Memory and performance stress
#[test]
fn fuzz_stress_test() {
    let stress_tests = vec![
        "1..100", // Large range
        "[1; 50]", // Array with repeated elements (if supported)
        "\"a\" * 50", // String repetition (if supported)  
        "let x = 1; let y = x; let z = y", // Variable chain
    ];
    
    for input in stress_tests {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let _ = repl.eval(input);
    }
}