//! Comprehensive Result type transpilation tests
//! Target: Boost result_type.rs from 12% to 70% coverage

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

/// Test Result type constructors
#[test]
fn test_result_constructors() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Ok constructor
        ("Ok(42)", vec!["Ok", "(", "42", ")"]),
        ("Ok(\"success\")", vec!["Ok", "(", "\"success\"", ")"]),
        ("Ok(vec![1, 2, 3])", vec!["Ok", "(", "vec", "!", "[", "1", ",", "2", ",", "3", "]", ")"]),
        
        // Err constructor
        ("Err(\"error\")", vec!["Err", "(", "\"error\"", ")"]),
        ("Err(404)", vec!["Err", "(", "404", ")"]),
        ("Err(Error::NotFound)", vec!["Err", "(", "Error", "::", "NotFound", ")"]),
        
        // Nested Results
        ("Ok(Ok(42))", vec!["Ok", "(", "Ok", "(", "42", ")", ")"]),
        ("Ok(Err(\"inner error\"))", vec!["Ok", "(", "Err", "(", "\"inner error\"", ")", ")"]),
        ("Err(Ok(0))", vec!["Err", "(", "Ok", "(", "0", ")", ")"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result constructor '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test Result pattern matching
#[test]
fn test_result_pattern_matching() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Basic Result matching
        (r#"match result { Ok(val) => val, Err(e) => 0 }"#, 
         vec!["match", "result", "Ok", "(", "val", ")", "=>", "val", "Err", "(", "e", ")", "=>", "0"]),
        
        // Result with specific error matching
        (r#"match result { Ok(x) => x * 2, Err("not found") => -1, Err(_) => 0 }"#,
         vec!["match", "Ok", "(", "x", ")", "=>", "x", "*", "2", "Err", "(", "\"not found\"", ")", "=>", "-1"]),
        
        // Nested Result matching
        (r#"match result { Ok(Ok(val)) => val, Ok(Err(e)) => panic(e), Err(e) => panic(e) }"#,
         vec!["match", "Ok", "(", "Ok", "(", "val", ")", ")", "=>", "val"]),
        
        // Result with destructuring
        (r#"match result { Ok((a, b)) => a + b, Err(_) => 0 }"#,
         vec!["match", "Ok", "(", "(", "a", ",", "b", ")", ")", "=>", "a", "+", "b"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result matching '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test ? operator (try operator)
#[test]
fn test_try_operator() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple try
        ("result?", vec!["result", "?"]),
        ("foo()?", vec!["foo", "(", ")", "?"]),
        
        // Chained try
        ("foo()?.bar()?", vec!["foo", "(", ")", "?", ".", "bar", "(", ")", "?"]),
        ("a()?.b()?.c()?", vec!["a", "(", ")", "?", ".", "b", "(", ")", "?", ".", "c", "(", ")", "?"]),
        
        // Try with field access
        ("config.get(\"key\")?", vec!["config", ".", "get", "(", "\"key\"", ")", "?"]),
        ("result?.field", vec!["result", "?", ".", "field"]),
        
        // Try in expressions
        ("let x = foo()?", vec!["let", "x", "=", "foo", "(", ")", "?"]),
        ("let y = x? + 1", vec!["let", "y", "=", "x", "?", "+", "1"]),
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

/// Test Result combinators
#[test]
fn test_result_combinators() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // map
        ("result.map(|x| x * 2)", vec!["result", ".", "map", "(", "|", "x", "|", "x", "*", "2", ")"]),
        
        // map_err
        ("result.map_err(|e| format(\"Error: {}\", e))", vec!["result", ".", "map_err", "(", "|", "e", "|"]),
        
        // and_then
        ("result.and_then(|x| Ok(x + 1))", vec!["result", ".", "and_then", "(", "|", "x", "|", "Ok", "(", "x", "+", "1"]),
        
        // or_else
        ("result.or_else(|_| Ok(default))", vec!["result", ".", "or_else", "(", "|", "_", "|", "Ok", "(", "default"]),
        
        // unwrap_or
        ("result.unwrap_or(0)", vec!["result", ".", "unwrap_or", "(", "0", ")"]),
        
        // unwrap_or_else
        ("result.unwrap_or_else(|_| default())", vec!["result", ".", "unwrap_or_else", "(", "|", "_", "|", "default"]),
        
        // unwrap_or_default
        ("result.unwrap_or_default()", vec!["result", ".", "unwrap_or_default", "(", ")"]),
        
        // is_ok and is_err
        ("result.is_ok()", vec!["result", ".", "is_ok", "(", ")"]),
        ("result.is_err()", vec!["result", ".", "is_err", "(", ")"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result combinator '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test Result type annotations
#[test]
fn test_result_type_annotations() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Basic Result types
        ("let x: Result<i32, String> = Ok(42)", vec!["let", "x", ":", "Result", "<", "i32", ",", "String", ">", "=", "Ok"]),
        ("let y: Result<(), Error> = Err(Error::NotFound)", vec!["let", "y", ":", "Result", "<", "(", ")", ",", "Error", ">"]),
        
        // Nested Result types
        ("let z: Result<Result<i32, String>, Error> = Ok(Ok(42))", 
         vec!["let", "z", ":", "Result", "<", "Result", "<", "i32", ",", "String", ">", ",", "Error", ">"]),
        
        // Function returning Result
        ("fun divide(a: i32, b: i32) -> Result<i32, String> { if b == 0 { Err(\"division by zero\") } else { Ok(a / b) } }",
         vec!["fn", "divide", "->", "Result", "<", "i32", ",", "String", ">", "if", "b", "==", "0", "Err"]),
        
        // Result in generic types
        ("let vec: Vec<Result<i32, String>> = vec![Ok(1), Err(\"error\")]",
         vec!["let", "vec", ":", "Vec", "<", "Result", "<", "i32", ",", "String", ">", ">"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result type annotation '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test error propagation patterns
#[test]
fn test_error_propagation() {
    let transpiler = Transpiler::new();
    
    // Early return with ?
    let code = r#"
        fun process() -> Result<i32, Error> {
            let x = step1()?;
            let y = step2(x)?;
            let z = step3(y)?;
            Ok(z)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse error propagation");
    let result = transpiler.transpile(&ast).expect("Failed to transpile error propagation");
    let transpiled = result.to_string();
    
    assert!(transpiled.contains("step1"));
    assert!(transpiled.contains("step2"));
    assert!(transpiled.contains("step3"));
    assert!(transpiled.contains("?"));
    assert!(transpiled.contains("Ok"));
    
    // Try block (if supported)
    let code2 = r#"
        let result = try {
            let a = foo()?;
            let b = bar()?;
            a + b
        }
    "#;
    
    let mut parser2 = Parser::new(code2);
    let ast2 = parser2.parse().expect("Failed to parse try block");
    let result2 = transpiler.transpile(&ast2).expect("Failed to transpile try block");
    let transpiled2 = result2.to_string();
    
    assert!(transpiled2.contains("try"));
    assert!(transpiled2.contains("foo"));
    assert!(transpiled2.contains("bar"));
    assert!(transpiled2.contains("?"));
}

/// Test Result with custom error types
#[test]
fn test_custom_error_types() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Enum error types
        (r#"enum AppError { NotFound, InvalidInput(String), NetworkError }"#,
         vec!["enum", "AppError", "NotFound", "InvalidInput", "(", "String", ")", "NetworkError"]),
        
        // Returning custom errors
        (r#"Err(AppError::NotFound)"#, vec!["Err", "(", "AppError", "::", "NotFound", ")"]),
        (r#"Err(AppError::InvalidInput("bad input"))"#, 
         vec!["Err", "(", "AppError", "::", "InvalidInput", "(", "\"bad input\"", ")", ")"]),
        
        // Matching custom errors
        (r#"match result { Ok(v) => v, Err(AppError::NotFound) => 0, Err(_) => -1 }"#,
         vec!["match", "Ok", "Err", "(", "AppError", "::", "NotFound", ")", "=>", "0"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Custom error '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test Result conversion methods
#[test]
fn test_result_conversions() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // ok() and err() methods
        ("result.ok()", vec!["result", ".", "ok", "(", ")"]),
        ("result.err()", vec!["result", ".", "err", "(", ")"]),
        
        // as_ref and as_mut
        ("result.as_ref()", vec!["result", ".", "as_ref", "(", ")"]),
        ("result.as_mut()", vec!["result", ".", "as_mut", "(", ")"]),
        
        // transpose
        ("opt_result.transpose()", vec!["opt_result", ".", "transpose", "(", ")"]),
        
        // From/Into conversions
        ("Result::from(value)", vec!["Result", "::", "from", "(", "value", ")"]),
        ("value.into()", vec!["value", ".", "into", "(", ")"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result conversion '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test Result in async contexts
#[test]
fn test_result_async() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Async function returning Result
        ("async fun fetch() -> Result<String, Error> { Ok(\"data\") }",
         vec!["async", "fn", "fetch", "->", "Result", "<", "String", ",", "Error", ">", "Ok"]),
        
        // Await with Result
        ("let data = fetch().await?", vec!["let", "data", "=", "fetch", "(", ")", ".", "await", "?"]),
        
        // Async block with Result
        ("async { Ok(42) }", vec!["async", "{", "Ok", "(", "42", ")", "}"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Async Result '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test Result with collect
#[test]
fn test_result_collect() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Collecting into Result<Vec<_>, _>
        ("results.collect::<Result<Vec<_>, _>>()", 
         vec!["results", ".", "collect", "::", "<", "Result", "<", "Vec", "<", "_", ">", ",", "_", ">", ">", "(", ")"]),
        
        // Iterator of Results
        ("vec.iter().map(|x| parse(x)).collect::<Result<Vec<i32>, ParseError>>()",
         vec!["vec", ".", "iter", "(", ")", ".", "map", "(", "|", "x", "|", "parse", "(", "x", ")", ")", ".", "collect"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", input));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", input));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Result collect '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}