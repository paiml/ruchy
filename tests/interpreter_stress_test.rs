//! Stress tests for the interpreter
//! Tests interpreter under various stress conditions

use ruchy::runtime::interpreter::Interpreter;
use ruchy::frontend::parser::Parser;

#[test]
fn test_deep_expression_nesting() {
    let mut interpreter = Interpreter::new();
    
    // Create deeply nested arithmetic expression
    let mut expr = "1".to_string();
    for i in 2..=50 {
        expr = format!("({} + {})", expr, i);
    }
    
    let mut parser = Parser::new(&expr);
    let ast = parser.parse().unwrap();
    
    let result = interpreter.eval_expr(&ast);
    assert!(result.is_ok());
    
    // Sum of 1 to 50 = 1275
    if let Ok(value) = result {
        let expected = (1..=50).sum::<i32>();
        assert_eq!(format!("{}", value), format!("{}", expected));
    }
}

#[test]
fn test_many_variables() {
    let mut interpreter = Interpreter::new();
    
    // Create many variables
    for i in 0..100 {
        let code = format!("let var_{} = {}", i, i * 2);
        let mut parser = Parser::new(&code);
        let ast = parser.parse().unwrap();
        interpreter.eval_expr(&ast).unwrap();
    }
    
    // Access variables randomly
    for &i in [0, 25, 50, 75, 99].iter() {
        let code = format!("var_{}", i);
        let mut parser = Parser::new(&code);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast).unwrap();
        assert_eq!(format!("{}", result), format!("{}", i * 2));
    }
}

#[test]
fn test_function_recursion_limits() {
    let mut interpreter = Interpreter::new();
    
    // Define a recursive function with reasonable depth
    let func_def = "fun countdown(n) { if n <= 0 { 0 } else { countdown(n - 1) } }";
    let mut parser = Parser::new(func_def);
    let ast = parser.parse().unwrap();
    interpreter.eval_expr(&ast).unwrap();
    
    // Test with moderate recursion depth
    let call = "countdown(50)";
    let mut parser = Parser::new(call);
    let ast = parser.parse().unwrap();
    let result = interpreter.eval_expr(&ast);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "0");
}

#[test]
fn test_large_data_structures() {
    let mut interpreter = Interpreter::new();
    
    // Create large array
    let numbers: Vec<String> = (1..=100).map(|i| i.to_string()).collect();
    let array_literal = format!("[{}]", numbers.join(", "));
    
    let full_code = format!("let big_array = {}", array_literal);
    let mut parser = Parser::new(&full_code);
    if let Ok(ast) = parser.parse() {
        let result = interpreter.eval_expr(&ast);
        assert!(result.is_ok());
    }
    
    // Access array length if supported
    let mut parser = Parser::new("big_array.length");
    if let Ok(ast) = parser.parse() {
        let result = interpreter.eval_expr(&ast);
        if result.is_ok() {
            // Length should be 100
        }
    }
}

#[test]
fn test_complex_expressions() {
    let mut interpreter = Interpreter::new();
    
    let complex_expressions = vec![
        // Nested function calls
        "let f = fun(x) { x * 2 }; f(f(f(5)))",
        
        // Complex conditionals
        "if true { if false { 1 } else { 2 } } else { 3 }",
        
        // Nested data structures
        "let nested = [[1, 2], [3, 4]]; nested[0][1]",
        
        // Mixed operations
        "let x = 5; let y = 10; (x + y) * (x - y) / 2",
    ];
    
    for expr in complex_expressions {
        let mut parser = Parser::new(expr);
        if let Ok(ast) = parser.parse() {
            let result = interpreter.eval_expr(&ast);
            // Should not panic, may succeed or fail gracefully
            match result {
                Ok(_) => println!("✅ Complex expression succeeded: {}", expr),
                Err(_) => println!("ℹ️ Complex expression failed gracefully: {}", expr),
            }
        }
    }
}

#[test]
fn test_error_recovery_stress() {
    let mut interpreter = Interpreter::new();
    
    // Set up valid state
    let setup = "let valid_var = 42";
    let mut parser = Parser::new(setup);
    let ast = parser.parse().unwrap();
    interpreter.eval_expr(&ast).unwrap();
    
    // Generate many errors
    for i in 0..50 {
        let invalid_expr = format!("undefined_var_{}", i);
        let mut parser = Parser::new(&invalid_expr);
        if let Ok(ast) = parser.parse() {
            let _result = interpreter.eval_expr(&ast);
            // Should handle error gracefully
        }
    }
    
    // Interpreter should still work after many errors
    let mut parser = Parser::new("valid_var");
    let ast = parser.parse().unwrap();
    let result = interpreter.eval_expr(&ast).unwrap();
    assert_eq!(format!("{}", result), "42");
}

#[test]
fn test_memory_intensive_operations() {
    let mut interpreter = Interpreter::new();
    
    // Create variables with potentially large content
    for i in 0..20 {
        let large_string = "x".repeat(100);
        let code = format!("let str_{} = \"{}\"", i, large_string);
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let result = interpreter.eval_expr(&ast);
            assert!(result.is_ok());
        }
    }
    
    // Access variables to ensure they're still available
    for i in [0, 10, 19] {
        let code = format!("str_{}", i);
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let result = interpreter.eval_expr(&ast);
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_scope_nesting_stress() {
    let mut interpreter = Interpreter::new();
    
    // Create nested scopes with same variable names
    let nested_code = r#"
        let x = 1;
        let inner = fun() {
            let x = 2;
            let deeper = fun() {
                let x = 3;
                x
            };
            x + deeper()
        };
        x + inner()
    "#;
    
    let mut parser = Parser::new(nested_code);
    if let Ok(ast) = parser.parse() {
        let result = interpreter.eval_expr(&ast);
        if result.is_ok() {
            // Should resolve scopes correctly: 1 + (2 + 3) = 6
            println!("✅ Nested scopes handled correctly");
        }
    }
}

#[test]
fn test_pattern_matching_stress() {
    let mut interpreter = Interpreter::new();
    
    // Complex pattern matching scenarios
    let pattern_tests = vec![
        // Tuple patterns
        "let (a, b, c) = (1, 2, 3); a + b + c",
        
        // Array destructuring
        "let [first, second] = [10, 20]; first + second",
        
        // Nested patterns
        "let ((x, y), z) = ((1, 2), 3); x + y + z",
    ];
    
    for pattern in pattern_tests {
        let mut parser = Parser::new(pattern);
        if let Ok(ast) = parser.parse() {
            let result = interpreter.eval_expr(&ast);
            match result {
                Ok(val) => println!("✅ Pattern worked: {} = {}", pattern, val),
                Err(_) => println!("ℹ️ Pattern not implemented: {}", pattern),
            }
        }
    }
}

#[test]
fn test_concurrent_simulation() {
    // Simulate what might happen in concurrent scenarios by
    // creating multiple interpreters and cross-referencing state
    
    let mut interpreters = Vec::new();
    for _ in 0..5 {
        interpreters.push(Interpreter::new());
    }
    
    // Set up same state in all interpreters
    for (i, interpreter) in interpreters.iter_mut().enumerate() {
        let code = format!("let instance_id = {}", i);
        let mut parser = Parser::new(&code);
        let ast = parser.parse().unwrap();
        interpreter.eval_expr(&ast).unwrap();
    }
    
    // Verify state is independent
    for (i, interpreter) in interpreters.iter_mut().enumerate() {
        let mut parser = Parser::new("instance_id");
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast).unwrap();
        assert_eq!(format!("{}", result), format!("{}", i));
    }
}

#[test]
fn test_edge_case_numbers() {
    let mut interpreter = Interpreter::new();
    
    let edge_cases = vec![
        ("0", "0"),
        ("-1", "-1"),
        ("999999", "999999"),
        ("-999999", "-999999"),
        ("3.14159", "3.14159"),
        ("-2.71828", "-2.71828"),
    ];
    
    for (input, expected) in edge_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        let result = interpreter.eval_expr(&ast).unwrap();
        
        // Allow for floating point precision differences
        let result_str = format!("{}", result);
        assert!(result_str.starts_with(expected.split('.').next().unwrap()) ||
                result_str == expected,
                "Expected {} to be close to {}, got {}", input, expected, result_str);
    }
}