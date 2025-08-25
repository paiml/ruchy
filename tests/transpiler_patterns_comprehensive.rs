//! Comprehensive pattern transpilation tests
//! Target: Boost patterns.rs from 14% to 70% coverage

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::similar_names)]

use ruchy::{Transpiler, Parser};

/// Test all pattern types in match expressions
#[test]
fn test_match_all_pattern_types() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Wildcard pattern
        (r#"match x { _ => 42 }"#, vec!["match", "_", "=>", "42"]),
        
        // Literal patterns
        (r#"match x { 0 => "zero", 1 => "one", _ => "other" }"#, vec!["match", "0", "=>", "zero", "1", "=>", "one"]),
        (r#"match x { true => 1, false => 0 }"#, vec!["match", "true", "=>", "1", "false", "=>", "0"]),
        (r#"match x { 'a' => 1, 'b' => 2, _ => 3 }"#, vec!["match", "'a'", "=>", "1", "'b'", "=>", "2"]),
        (r#"match x { "hello" => 1, "world" => 2, _ => 3 }"#, vec!["match", "hello", "=>", "1", "world", "=>", "2"]),
        
        // Identifier patterns
        (r#"match x { y => y + 1 }"#, vec!["match", "y", "=>", "y", "+", "1"]),
        (r#"match x { Some(y) => y, None => 0 }"#, vec!["match", "Some", "y", "=>", "y", "None", "=>", "0"]),
        
        // Tuple patterns
        (r#"match pair { (a, b) => a + b }"#, vec!["match", "(", "a", ",", "b", ")", "=>", "a", "+", "b"]),
        (r#"match triple { (x, y, z) => x + y + z }"#, vec!["match", "(", "x", ",", "y", ",", "z", ")", "=>"]),
        (r#"match pair { (0, y) => y, (x, 0) => x, (x, y) => x + y }"#, vec!["match", "(", "0", ",", "y", ")", "=>", "y"]),
        
        // Array/List patterns
        (r#"match arr { [] => 0, [x] => x, [x, y] => x + y, _ => -1 }"#, vec!["match", "[]", "=>", "0", "[", "x", "]", "=>", "x"]),
        (r#"match list { [first, second, ..rest] => first }"#, vec!["match", "[", "first", ",", "second", ",", "..", "rest", "]"]),
        (r#"match list { [head, ..] => head }"#, vec!["match", "[", "head", ",", "..", "]", "=>", "head"]),
        
        // Struct patterns
        (r#"match point { Point { x, y } => x + y }"#, vec!["match", "Point", "{", "x", ",", "y", "}", "=>", "x", "+", "y"]),
        (r#"match point { Point { x: 0, y } => y, Point { x, y: 0 } => x, _ => 0 }"#, vec!["match", "Point", "{", "x", ":", "0"]),
        (r#"match config { Config { debug: true, .. } => 1, _ => 0 }"#, vec!["match", "Config", "{", "debug", ":", "true", "..", "}"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Pattern match '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test nested patterns
#[test]
fn test_nested_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Nested tuples
        (r#"match x { ((a, b), c) => a + b + c }"#, vec!["match", "((", "a", ",", "b", ")", ",", "c", ")", "=>"]),
        (r#"match x { (Some(a), Some(b)) => a + b, _ => 0 }"#, vec!["match", "(", "Some", "a", "Some", "b", ")", "=>"]),
        
        // Nested arrays
        (r#"match x { [[a, b], [c, d]] => a + b + c + d }"#, vec!["match", "[[", "a", ",", "b", "]", ",", "[", "c", ",", "d", "]]"]),
        
        // Nested structs
        (r#"match x { Outer { inner: Inner { value } } => value }"#, vec!["match", "Outer", "{", "inner", ":", "Inner", "{", "value"]),
        
        // Mixed nesting
        (r#"match x { (Point { x, y }, z) => x + y + z }"#, vec!["match", "(", "Point", "{", "x", ",", "y", "}", ",", "z"]),
        (r#"match x { [Some(a), None, Some(b)] => a + b }"#, vec!["match", "[", "Some", "a", "None", "Some", "b", "]"]),
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

/// Test pattern guards
#[test]
fn test_pattern_guards() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple guards
        (r#"match x { n if n > 0 => "positive", n if n < 0 => "negative", _ => "zero" }"#, 
         vec!["match", "n", "if", "n", ">", "0", "=>", "positive"]),
        
        // Guards with complex conditions
        (r#"match x { n if n > 0 && n < 10 => "single digit", _ => "other" }"#,
         vec!["match", "n", "if", "n", ">", "0", "&&", "n", "<", "10"]),
        
        // Guards with pattern destructuring
        (r#"match pair { (a, b) if a > b => a - b, (a, b) => b - a }"#,
         vec!["match", "(", "a", ",", "b", ")", "if", "a", ">", "b"]),
        
        // Guards with struct patterns
        (r#"match point { Point { x, y } if x == y => "diagonal", _ => "other" }"#,
         vec!["match", "Point", "{", "x", ",", "y", "}", "if", "x", "==", "y"]),
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

/// Test or patterns (pattern alternatives)
#[test]
fn test_or_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple or patterns
        (r#"match x { 1 | 2 | 3 => "low", _ => "high" }"#, vec!["match", "1", "|", "2", "|", "3", "=>", "low"]),
        
        // Or patterns with identifiers
        (r#"match x { Some(1) | Some(2) => "found", _ => "not found" }"#, 
         vec!["match", "Some", "1", "|", "Some", "2", "=>", "found"]),
        
        // Or patterns in tuples
        (r#"match pair { (0, _) | (_, 0) => "has zero", _ => "no zero" }"#,
         vec!["match", "(", "0", ",", "_", ")", "|", "(", "_", ",", "0", ")"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Or pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test range patterns
#[test]
fn test_range_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Inclusive ranges
        (r#"match x { 1..=10 => "small", 11..=100 => "medium", _ => "large" }"#,
         vec!["match", "1", "..=", "10", "=>", "small"]),
        
        // Exclusive ranges
        (r#"match x { 0..10 => "single digit", _ => "other" }"#,
         vec!["match", "0", "..", "10", "=>", "single"]),
        
        // Character ranges
        (r#"match ch { 'a'..='z' => "lowercase", 'A'..='Z' => "uppercase", _ => "other" }"#,
         vec!["match", "'a'", "..=", "'z'", "=>", "lowercase"]),
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

/// Test rest patterns (.. syntax)
#[test]
fn test_rest_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Rest in arrays
        (r#"match arr { [first, ..rest] => first }"#, vec!["match", "[", "first", ",", "..", "rest", "]"]),
        (r#"match arr { [first, second, ..] => first + second }"#, vec!["match", "[", "first", ",", "second", ",", "..", "]"]),
        (r#"match arr { [.., last] => last }"#, vec!["match", "[", "..", ",", "last", "]"]),
        
        // Rest in tuples
        (r#"match tuple { (first, ..) => first }"#, vec!["match", "(", "first", ",", "..", ")"]),
        
        // Rest in structs
        (r#"match s { Struct { field, .. } => field }"#, vec!["match", "Struct", "{", "field", ",", "..", "}"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Rest pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test complex real-world pattern matching scenarios
#[test]
fn test_complex_pattern_scenarios() {
    let transpiler = Transpiler::new();
    
    // Result pattern matching
    let code = r#"
        match result {
            Ok(Some(value)) if value > 0 => value,
            Ok(Some(_)) => 0,
            Ok(None) => -1,
            Err(msg) => panic(msg)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse complex pattern");
    let result = transpiler.transpile(&ast).expect("Failed to transpile complex pattern");
    let transpiled = result.to_string();
    
    assert!(transpiled.contains("Ok"));
    assert!(transpiled.contains("Some"));
    assert!(transpiled.contains("value"));
    assert!(transpiled.contains("if"));
    assert!(transpiled.contains("Err"));
    
    // Enum with data pattern matching
    let code2 = r#"
        match event {
            Event::Click { x, y } => handle_click(x, y),
            Event::Key(code) => handle_key(code),
            Event::Resize(w, h) => handle_resize(w, h),
            _ => ()
        }
    "#;
    
    let mut parser2 = Parser::new(code2);
    let ast2 = parser2.parse().expect("Failed to parse enum pattern");
    let result2 = transpiler.transpile(&ast2).expect("Failed to transpile enum pattern");
    let transpiled2 = result2.to_string();
    
    assert!(transpiled2.contains("Event"));
    assert!(transpiled2.contains("Click"));
    assert!(transpiled2.contains("Key"));
    assert!(transpiled2.contains("Resize"));
}

/// Test let pattern destructuring
#[test]
fn test_let_pattern_destructuring() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Tuple destructuring
        ("let (a, b) = (1, 2)", vec!["let", "(", "a", ",", "b", ")", "=", "(", "1", ",", "2", ")"]),
        ("let (x, y, z) = triple", vec!["let", "(", "x", ",", "y", ",", "z", ")", "=", "triple"]),
        
        // Array destructuring
        ("let [first, second] = [1, 2]", vec!["let", "[", "first", ",", "second", "]", "=", "vec"]),
        ("let [head, ..tail] = list", vec!["let", "[", "head", ",", "..", "tail", "]", "=", "list"]),
        
        // Struct destructuring
        ("let Point { x, y } = point", vec!["let", "Point", "{", "x", ",", "y", "}", "=", "point"]),
        ("let Config { host, port, .. } = config", vec!["let", "Config", "{", "host", ",", "port", ",", "..", "}"]),
        
        // Nested destructuring
        ("let ((a, b), c) = nested", vec!["let", "((", "a", ",", "b", ")", ",", "c", ")", "=", "nested"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Let destructuring '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test for-loop pattern destructuring
#[test]
fn test_for_loop_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple iteration
        ("for x in list { println(x) }", vec!["for", "x", "in", "list", "println"]),
        
        // Tuple destructuring in for loop
        ("for (k, v) in map { println(k, v) }", vec!["for", "(", "k", ",", "v", ")", "in", "map"]),
        
        // Struct destructuring in for loop
        ("for Point { x, y } in points { x + y }", vec!["for", "Point", "{", "x", ",", "y", "}", "in", "points"]),
        
        // Array pattern in for loop
        ("for [a, b] in pairs { a * b }", vec!["for", "[", "a", ",", "b", "]", "in", "pairs"]),
        
        // Enumerate pattern
        ("for (i, value) in list.enumerate() { println(i, value) }", vec!["for", "(", "i", ",", "value", ")", "in"]),
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

/// Test function parameter patterns
#[test]
fn test_function_parameter_patterns() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Tuple parameters
        ("fun add((a, b): (i32, i32)) -> i32 { a + b }", vec!["fn", "add", "(", "a", ",", "b", ")", ":", "(", "i32"]),
        
        // Struct parameters
        ("fun distance(Point { x, y }: Point) -> f64 { sqrt(x*x + y*y) }", 
         vec!["fn", "distance", "Point", "{", "x", ",", "y", "}", ":", "Point"]),
        
        // Array parameters
        ("fun sum([a, b, c]: [i32; 3]) -> i32 { a + b + c }",
         vec!["fn", "sum", "[", "a", ",", "b", ",", "c", "]", ":"]),
        
        // Ignored parameters
        ("fun ignore(_: i32, b: i32) -> i32 { b }", vec!["fn", "ignore", "_", ":", "i32", "b", ":", "i32"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Function parameter pattern '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}