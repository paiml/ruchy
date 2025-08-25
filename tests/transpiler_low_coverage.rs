//! Tests targeting low-coverage transpiler modules
//! Toyota Way: Systematic testing to reach 70% coverage target

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

/// Test result type transpilation
#[test]
fn test_transpile_result_types() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Ok constructor
        ("Ok(42)", vec!["Ok", "42"]),
        
        // Err constructor
        ("Err(\"error\")", vec!["Err", "\"error\""]),
        
        // Result type annotation
        ("let x: Result<i32, String> = Ok(42)", vec!["let", "x", "Result", "i32", "String", "Ok", "42"]),
        
        // Pattern matching on Result
        (r#"match result { Ok(val) => val, Err(e) => 0 }"#, vec!["match", "result", "Ok", "val", "Err", "e", "0"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result type '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test option type transpilation
#[test]
fn test_transpile_option_types() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Some constructor
        ("Some(42)", vec!["Some", "42"]),
        
        // None constructor
        ("None", vec!["None"]),
        
        // Option type annotation
        ("let x: Option<i32> = Some(42)", vec!["let", "x", "Option", "i32", "Some", "42"]),
        
        // Pattern matching on Option
        (r#"match opt { Some(val) => val, None => 0 }"#, vec!["match", "opt", "Some", "val", "None", "0"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Option type '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test type inference edge cases
#[test]
fn test_transpile_type_inference_edge_cases() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Generic function
        ("fun identity<T>(x: T) -> T { x }", vec!["fn", "identity", "<", "T", ">", "x", "T"]),
        
        // Type alias
        ("type Point = (i32, i32)", vec!["type", "Point", "i32", "i32"]),
        
        // Complex generic type
        ("let map: HashMap<String, Vec<i32>> = HashMap::new()", vec!["let", "map", "HashMap", "String", "Vec", "i32"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Type inference case '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test dataframe operations transpilation
#[test]
fn test_transpile_dataframe_operations() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // DataFrame literal
        (r#"df![{"col1": [1, 2, 3]}]"#, vec!["DataFrame", "col1", "1", "2", "3"]),
        
        // DataFrame select
        (r#"df.select(["col1", "col2"])"#, vec!["df", "select", "col1", "col2"]),
        
        // DataFrame filter
        (r#"df.filter(col("age") > 18)"#, vec!["df", "filter", "col", "age", ">", "18"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "DataFrame operation '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test actor model transpilation
#[test]
fn test_transpile_actors() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Actor definition
        (r#"actor Counter { state count: i32 = 0 }"#, vec!["struct", "Counter", "count", "i32"]),
        
        // Actor message send
        ("counter ! Increment", vec!["counter", "send", "Increment"]),
        
        // Actor spawn
        ("spawn Counter { count: 0 }", vec!["spawn", "Counter", "count", "0"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Actor '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test complex nested expressions
#[test]
fn test_transpile_complex_nested() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Nested if-else
        ("if x > 0 { if y > 0 { 1 } else { 2 } } else { 3 }", vec!["if", "x", ">", "0", "if", "y", ">", "0", "1", "else", "2", "else", "3"]),
        
        // Nested match
        (r#"match x { Some(y) => match y { 0 => "zero", _ => "other" }, None => "none" }"#, vec!["match", "x", "Some", "y", "match", "y", "0", "zero", "other", "None", "none"]),
        
        // Nested function calls
        ("f(g(h(x)))", vec!["f", "g", "h", "x"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Complex nested '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test edge cases and error conditions
#[test]
fn test_transpile_edge_cases() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Empty block
        ("{}", vec!["{"]),
        
        // Nested empty blocks
        ("{ {} }", vec!["{", "{"]),
        
        // Very long identifier
        ("let very_long_identifier_name_that_exceeds_normal_length = 42", vec!["let", "very_long_identifier_name_that_exceeds_normal_length", "42"]),
        
        // Unicode identifiers
        ("let π = 3.14", vec!["let", "π", "3.14"]),
        
        // Multiple semicolons
        ("x = 1; ; y = 2", vec!["x", "1", "y", "2"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Edge case '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test all unary operators
#[test]
fn test_transpile_all_unary_ops() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Negation
        ("-42", vec!["-", "42"]),
        ("-x", vec!["-", "x"]),
        
        // Logical NOT
        ("!true", vec!["!", "true"]),
        ("!flag", vec!["!", "flag"]),
        
        // Bitwise NOT
        ("~5", vec!["!", "5"]),  // Transpiles to ! in Rust
        ("~bits", vec!["!", "bits"]),
        
        // Reference
        ("&value", vec!["&", "value"]),
        
        // Dereference
        ("*ptr", vec!["*", "ptr"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Unary op '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test try operator and error handling
#[test]
fn test_transpile_try_operator() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Try operator
        ("result?", vec!["result", "?"]),
        
        // Try in expression
        ("let x = foo()?", vec!["let", "x", "=", "foo", "()", "?"]),
        
        // Chain of try operators
        ("a()?.b()?.c()?", vec!["a", "?", "b", "?", "c", "?"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Try operator '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test macro invocations
#[test]
fn test_transpile_macros() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // println macro
        (r#"println!("Hello")"#, vec!["println", "!", "\"Hello\""]),
        
        // vec macro
        ("vec![1, 2, 3]", vec!["vec", "!", "1", "2", "3"]),
        
        // format macro
        (r#"format!("{}", x)"#, vec!["format", "!", "x"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Macro '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}