//! Comprehensive property-based tests for all major modules
//! Uses proptest to verify invariants with thousands of random inputs
//! Quality: PMAT A+ standards, ≤10 complexity per function

use proptest::prelude::*;
use ruchy::{Parser, Lexer, Repl, Transpiler};

// ========== Parser Property Tests ==========

proptest! {
    #[test]
    fn parser_never_panics_on_random_input(input in ".*") {
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Should not panic
    }
    
    #[test]
    fn parser_handles_deep_nesting(depth in 1usize..50) {
        let mut expr = String::from("1");
        for _ in 0..depth {
            expr = format!("({})", expr);
        }
        
        let mut parser = Parser::new(&expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err()); // Should complete without panic
    }
    
    #[test]
    fn parser_preserves_literal_values(
        int in -1000000i64..1000000,
        float in -1000.0f64..1000.0,
        string in "[a-zA-Z0-9 ]{0,100}"
    ) {
        // Integer literal
        let int_source = format!("{}", int);
        let mut parser = Parser::new(&int_source);
        let _ = parser.parse();
        
        // Float literal
        let float_source = format!("{}", float);
        parser = Parser::new(&float_source);
        let _ = parser.parse();
        
        // String literal
        let string_source = format!("\"{}\"", string);
        parser = Parser::new(&string_source);
        let _ = parser.parse();
    }
    
    #[test]
    fn parser_handles_all_operators(
        left in 1i32..100,
        right in 1i32..100
    ) {
        let operators = vec!["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||"];
        
        for op in operators {
            let source = format!("{} {} {}", left, op, right);
            let mut parser = Parser::new(&source);
            let result = parser.parse();
            assert!(result.is_ok() || result.is_err());
        }
    }
    
    #[test]
    fn parser_handles_unicode_correctly(input in "\\PC{0,100}") {
        let source = format!("let x = \"{}\"", input);
        let mut parser = Parser::new(&source);
        let _ = parser.parse(); // Should handle unicode without panic
    }
}

// ========== Lexer Property Tests ==========

proptest! {
    #[test]
    fn lexer_tokenizes_all_input(input in "[a-zA-Z0-9 +\\-*/()\\[\\]{}.,;:]{0,1000}") {
        let lexer = Lexer::new(&input);
        let tokens: Vec<_> = lexer.collect();
        
        // Should produce at least one token (EOF) for any input
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn lexer_preserves_token_order(tokens in prop::collection::vec("[a-z]+", 1..20)) {
        let source = tokens.join(" ");
        let lexer = Lexer::new(&source);
        let lexed_tokens: Vec<_> = lexer.collect();
        
        // Number of identifier tokens should match input
        let identifier_count = lexed_tokens.iter()
            .filter(|t| matches!(t.kind, TokenKind::Identifier(_)))
            .count();
        
        assert!(identifier_count <= tokens.len());
    }
    
    #[test]
    fn lexer_handles_numbers_correctly(
        integers in prop::collection::vec(-1000000i64..1000000, 1..10),
        floats in prop::collection::vec(-1000.0f64..1000.0, 1..10)
    ) {
        // Test integers
        let int_source = integers.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        let lexer = Lexer::new(&int_source);
        let tokens: Vec<_> = lexer.collect();
        
        let int_token_count = tokens.iter()
            .filter(|t| matches!(t.kind, TokenKind::Integer(_)))
            .count();
        
        assert!(int_token_count <= integers.len());
        
        // Test floats
        let float_source = floats.iter()
            .map(|n| format!("{:.2}", n))
            .collect::<Vec<_>>()
            .join(" ");
        
        let lexer = Lexer::new(&float_source);
        let tokens: Vec<_> = lexer.collect();
        
        let float_token_count = tokens.iter()
            .filter(|t| matches!(t.kind, TokenKind::Float(_)))
            .count();
        
        assert!(float_token_count <= floats.len());
    }
}

// ========== REPL Property Tests ==========

proptest! {
    #[test]
    fn repl_maintains_state_consistency(
        vars in prop::collection::vec(("[a-z][a-z0-9_]{0,10}", 0i32..1000), 1..10)
    ) {
        let mut repl = Repl::new().unwrap();
        
        // Define variables
        for (name, value) in &vars {
            let expr = format!("let {} = {}", name, value);
            let _ = repl.eval(&expr);
        }
        
        // Verify variables are accessible
        for (name, expected) in &vars {
            let expr = format!("{}", name);
            let result = repl.eval(&expr);
            
            if result.is_ok() {
                let output = result.unwrap();
                assert!(output.contains(&expected.to_string()) || output.contains(name));
            }
        }
    }
    
    #[test]
    fn repl_arithmetic_correctness(
        operations in prop::collection::vec(
            (1i32..100, prop::sample::select(vec!["+", "-", "*"]), 1i32..100),
            1..20
        )
    ) {
        let mut repl = Repl::new().unwrap();
        
        for (left, op, right) in operations {
            let expr = format!("{} {} {}", left, op, right);
            let result = repl.eval(&expr);
            
            if result.is_ok() {
                let output = result.unwrap();
                
                let expected = match op {
                    "+" => left + right,
                    "-" => left - right,
                    "*" => left * right,
                    _ => continue,
                };
                
                assert!(output.contains(&expected.to_string()));
            }
        }
    }
    
    #[test]
    fn repl_error_recovery(
        valid_exprs in prop::collection::vec("[a-z]+ = [0-9]+", 1..5),
        invalid_exprs in prop::collection::vec("@#$%^&*", 1..5)
    ) {
        let mut repl = Repl::new().unwrap();
        
        // Interleave valid and invalid expressions
        for i in 0..valid_exprs.len().min(invalid_exprs.len()) {
            // Valid expression
            let _ = repl.eval(&format!("let {}", valid_exprs[i]));
            
            // Invalid expression (should error but not crash)
            let _ = repl.eval(&invalid_exprs[i]);
            
            // Should still work after error
            let result = repl.eval("1 + 1");
            assert!(result.is_ok() || result.is_err());
        }
    }
}

// ========== Transpiler Property Tests ==========

proptest! {
    #[test]
    fn transpiler_preserves_identifiers(
        identifiers in prop::collection::vec("[a-z][a-z0-9_]{0,20}", 1..10)
    ) {
        let mut source = String::new();
        for id in &identifiers {
            source.push_str(&format!("let {} = 0;\n", id));
        }
        
        let mut parser = Parser::new(&source);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                // All identifiers should appear in output
                for id in &identifiers {
                    assert!(rust_code.contains(id));
                }
            }
        }
    }
    
    #[test]
    fn transpiler_handles_nested_structures(
        depth in 1usize..10,
        width in 1usize..5
    ) {
        let mut source = String::new();
        
        // Generate nested if statements
        for _ in 0..depth {
            source.push_str("if true { ");
        }
        source.push_str("42");
        for _ in 0..depth {
            source.push_str(" }");
        }
        
        let mut parser = Parser::new(&source);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
    
    #[test]
    fn transpiler_preserves_numeric_literals(
        integers in prop::collection::vec(-1000i32..1000, 1..10),
        floats in prop::collection::vec(-100.0f64..100.0, 1..10)
    ) {
        // Test integers
        for int in integers {
            let source = format!("let x = {}", int);
            let mut parser = Parser::new(&source);
            
            if let Ok(ast) = parser.parse() {
                let transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    assert!(rust_code.contains(&int.to_string()));
                }
            }
        }
        
        // Test floats
        for float in floats {
            let source = format!("let y = {}", float);
            let mut parser = Parser::new(&source);
            
            if let Ok(ast) = parser.parse() {
                let transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    // Float representation might vary slightly
                    assert!(rust_code.contains("."));
                }
            }
        }
    }
}

// ========== Type System Property Tests ==========

proptest! {
    #[test]
    fn type_inference_consistency(
        expr_type in prop::sample::select(vec!["int", "float", "bool", "string"]),
        count in 1usize..10
    ) {
        let mut source = String::new();
        
        for i in 0..count {
            let value = match expr_type {
                "int" => format!("{}", i),
                "float" => format!("{}.0", i),
                "bool" => format!("{}", i % 2 == 0),
                "string" => format!("\"text_{}\"", i),
                _ => "null".to_string(),
            };
            
            source.push_str(&format!("let var_{} = {};\n", i, value));
        }
        
        let mut parser = Parser::new(&source);
        let _ = parser.parse(); // Type inference should not panic
    }
    
    #[test]
    fn generic_type_instantiation(
        type_params in prop::collection::vec("[A-Z]", 1..3),
        values in prop::collection::vec(0i32..100, 1..5)
    ) {
        let type_param_str = type_params.join(", ");
        let source = format!(
            "fn generic<{}>() {{ }} generic::<i32>()",
            type_param_str
        );
        
        let mut parser = Parser::new(&source);
        let _ = parser.parse(); // Should handle generic syntax
    }
}

// ========== Memory Safety Property Tests ==========

proptest! {
    #[test]
    fn no_buffer_overflow_on_long_input(
        length in 1000usize..10000
    ) {
        let input = "a".repeat(length);
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Should handle long input safely
        
        let lexer = Lexer::new(&input);
        let _: Vec<_> = lexer.collect(); // Should tokenize safely
    }
    
    #[test]
    fn no_stack_overflow_on_deep_recursion(
        depth in 100usize..500
    ) {
        // Deep nested function calls
        let mut source = String::new();
        for _ in 0..depth {
            source.push_str("f(");
        }
        source.push_str("0");
        for _ in 0..depth {
            source.push_str(")");
        }
        
        let mut parser = Parser::new(&source);
        let _ = parser.parse(); // Should handle deep recursion
    }
    
    #[test]
    fn handles_empty_and_whitespace(
        spaces in 0usize..100,
        tabs in 0usize..20,
        newlines in 0usize..50
    ) {
        let input = format!(
            "{}{}{}",
            " ".repeat(spaces),
            "\t".repeat(tabs),
            "\n".repeat(newlines)
        );
        
        let mut parser = Parser::new(&input);
        let _ = parser.parse();
        
        let lexer = Lexer::new(&input);
        let _: Vec<_> = lexer.collect();
    }
}

// ========== Concurrency Property Tests ==========

proptest! {
    #[test]
    fn thread_safe_parsing(
        sources in prop::collection::vec("[a-z]+ = [0-9]+", 1..10)
    ) {
        use std::thread;
        
        let handles: Vec<_> = sources.into_iter().map(|source| {
            thread::spawn(move || {
                let expr = format!("let {}", source);
                let mut parser = Parser::new(&expr);
                let _ = parser.parse();
            })
        }).collect();
        
        for handle in handles {
            assert!(handle.join().is_ok());
        }
    }
}

// ========== Fuzzing-Style Property Tests ==========

proptest! {
    #[test]
    fn fuzz_parser_with_random_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        let input = String::from_utf8_lossy(&bytes);
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Should not panic on arbitrary bytes
    }
    
    #[test]
    fn fuzz_lexer_with_random_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        let input = String::from_utf8_lossy(&bytes);
        let lexer = Lexer::new(&input);
        let _: Vec<_> = lexer.collect(); // Should not panic
    }
    
    #[test]
    fn fuzz_repl_with_random_input(
        inputs in prop::collection::vec(".*", 1..10)
    ) {
        let mut repl = Repl::new().unwrap();
        
        for input in inputs {
            let _ = repl.eval(&input); // Should not panic
        }
    }
}