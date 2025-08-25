//! Pattern transpilation test coverage
//! Toyota Way: Target low-coverage pattern module (14%)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::similar_names)]

use ruchy::{Transpiler, Parser};

/// Test pattern matching in match expressions
#[test]
fn test_transpile_match_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Literal patterns
        (r#"match x { 1 => "one", _ => "other" }"#, vec!["match", "1", "=>", "one", "_"]),
        
        // Identifier patterns
        (r#"match x { y => y + 1 }"#, vec!["match", "y", "=>", "y", "+", "1"]),
        
        // Tuple patterns
        (r#"match pair { (a, b) => a + b }"#, vec!["match", "pair", "(", "a", ",", "b", ")", "=>", "a", "+", "b"]),
        
        // Array patterns
        (r#"match arr { [first, second] => first + second }"#, vec!["match", "arr", "first", "second", "=>"]),
        
        // Struct patterns
        (r#"match obj { {x: a, y: b} => a + b }"#, vec!["match", "obj", "x", "y", "=>", "a", "+", "b"]),
        
        // Wildcard pattern
        (r#"match x { _ => 42 }"#, vec!["match", "_", "=>", "42"]),
        
        // Multiple patterns
        (r#"match x { 1 | 2 | 3 => "low", _ => "high" }"#, vec!["match", "1", "2", "3", "=>", "low"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Match pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test pattern guards
#[test]
fn test_transpile_pattern_guards() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        (r#"match x { n if n > 0 => "positive", _ => "other" }"#, vec!["match", "if", "n", ">", "0", "=>", "positive"]),
        (r#"match x { n if n < 0 => -n, n => n }"#, vec!["match", "if", "n", "<", "0", "=>", "-", "n"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Pattern guard '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test let patterns
#[test]
fn test_transpile_let_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple binding
        ("let x = 42", vec!["let", "x", "=", "42"]),
        
        // Tuple destructuring
        ("let (a, b) = (1, 2)", vec!["let", "(", "a", ",", "b", ")", "=", "(", "1", ",", "2", ")"]),
        
        // Array destructuring
        ("let [first, second] = [1, 2]", vec!["let", "first", "second", "vec", "1", "2"]),
        
        // Struct destructuring
        ("let {x, y} = point", vec!["let", "x", "y", "=", "point"]),
        
        // Nested patterns
        ("let ((a, b), c) = ((1, 2), 3)", vec!["let", "a", "b", "c", "1", "2", "3"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Let pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test function parameter patterns
#[test]
fn test_transpile_function_param_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple parameters
        ("fun test(x: i32, y: i32) -> i32 { x + y }", vec!["fn", "test", "x", "i32", "y", "i32"]),
        
        // Tuple parameters
        ("fun add_pair((a, b): (i32, i32)) -> i32 { a + b }", vec!["fn", "add_pair", "a", "b", "i32"]),
        
        // Struct parameters  
        ("fun process({x, y}: Point) -> i32 { x + y }", vec!["fn", "process", "x", "y", "Point"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Function param pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test for loop patterns
#[test]
fn test_transpile_for_loop_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple iteration
        ("for x in vec { println(x) }", vec!["for", "x", "in", "vec", "println"]),
        
        // Tuple iteration
        ("for (k, v) in map { println(k) }", vec!["for", "k", "v", "in", "map", "println"]),
        
        // Array pattern in for loop
        ("for [a, b] in pairs { a + b }", vec!["for", "a", "b", "in", "pairs"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "For loop pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test nested patterns
#[test]
fn test_transpile_nested_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Nested tuples
        (r#"match x { ((a, b), c) => a + b + c }"#, vec!["match", "a", "b", "c", "=>", "+"]),
        
        // Nested arrays
        (r#"match x { [[a, b], [c, d]] => a + b + c + d }"#, vec!["match", "a", "b", "c", "d", "=>"]),
        
        // Mixed nesting
        (r#"match x { (a, [b, c]) => a + b + c }"#, vec!["match", "a", "b", "c", "=>"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Nested pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test range patterns
#[test]
fn test_transpile_range_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        (r#"match x { 1..10 => "small", _ => "large" }"#, vec!["match", "1", ".", "10", "=>", "small"]),
        (r#"match x { 1..=10 => "inclusive", _ => "other" }"#, vec!["match", "1", ".", "=", "10", "=>"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Range pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test type ascription in patterns
#[test]
fn test_transpile_pattern_type_ascription() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        ("let x: i32 = 42", vec!["let", "x", "i32", "42"]),
        ("let (a, b): (i32, i32) = (1, 2)", vec!["let", "a", "b", "i32", "1", "2"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Pattern type ascription '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}