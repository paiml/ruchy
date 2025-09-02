//! Property-based tests for execution invariants
//! 
//! Tests that certain properties hold for all valid inputs

use proptest::prelude::*;
use ruchy::runtime::repl::Repl;

/// Generate valid arithmetic expressions
fn arithmetic_expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("0"),
        Just("1"),
        Just("42"),
        Just("100"),
    ];
    
    leaf.prop_recursive(3, 16, 5, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} + {}", a, b)),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} - {}", a, b)),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} * {}", a, b)),
            inner.prop_map(|x| format!("({})", x)),
        ]
    })
}

/// Generate valid boolean expressions
fn boolean_expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("true"),
        Just("false"),
    ];
    
    leaf.prop_recursive(3, 16, 5, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} && {}", a, b)),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} || {}", a, b)),
            inner.prop_map(|x| format!("!{}", x)),
            inner.prop_map(|x| format!("({})", x)),
        ]
    })
}

/// Generate valid list expressions
fn list_expr() -> impl Strategy<Value = String> {
    prop::collection::vec(0..100i32, 0..10)
        .prop_map(|v| format!("[{}]", v.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")))
}

proptest! {
    /// Test that evaluation is deterministic
    #[test]
    fn eval_deterministic(expr in arithmetic_expr()) {
        let mut repl1 = Repl::new().unwrap();
        let mut repl2 = Repl::new().unwrap();
        
        let result1 = repl1.eval(&expr);
        let result2 = repl2.eval(&expr);
        
        match (result1, result2) {
            (Ok(v1), Ok(v2)) => prop_assert_eq!(v1.to_string(), v2.to_string()),
            (Err(_), Err(_)) => (), // Both errored, that's consistent
            _ => prop_assert!(false, "Inconsistent results for expression: {}", expr),
        }
    }
    
    /// Test that arithmetic operations don't panic
    #[test]
    fn arithmetic_no_panic(expr in arithmetic_expr()) {
        let mut repl = Repl::new().unwrap();
        // Just ensure it doesn't panic - error is OK
        let _ = repl.eval(&expr);
    }
    
    /// Test that boolean operations always return bool
    #[test]
    fn boolean_returns_bool(expr in boolean_expr()) {
        let mut repl = Repl::new().unwrap();
        if let Ok(result) = repl.eval(&expr) {
            let s = result.to_string();
            prop_assert!(s == "true" || s == "false", 
                         "Boolean expression returned non-bool: {}", s);
        }
    }
    
    /// Test that list operations preserve length
    #[test]
    fn list_map_preserves_length(list in list_expr()) {
        let mut repl = Repl::new().unwrap();
        let map_expr = format!("{}.map(|x| x * 2)", list);
        
        if let (Ok(orig), Ok(mapped)) = (repl.eval(&list), repl.eval(&map_expr)) {
            let orig_str = orig.to_string();
            let mapped_str = mapped.to_string();
            
            // Count commas as a proxy for list length
            let orig_len = orig_str.matches(',').count();
            let mapped_len = mapped_str.matches(',').count();
            
            prop_assert_eq!(orig_len, mapped_len, 
                           "Map changed list length: {} -> {}", orig_str, mapped_str);
        }
    }
    
    /// Test that string operations don't panic
    #[test]
    fn string_operations_safe(s in "[a-zA-Z0-9 ]{0,100}") {
        let mut repl = Repl::new().unwrap();
        
        // Test various string operations
        let exprs = vec![
            format!(r#""{}".len()"#, s),
            format!(r#""{}".to_upper()"#, s),
            format!(r#""{}".to_lower()"#, s),
            format!(r#""  {}  ".trim()"#, s),
        ];
        
        for expr in exprs {
            let _ = repl.eval(&expr); // Don't panic
        }
    }
}

/// Test roundtrip properties
mod roundtrip {
    use super::*;
    use ruchy::{Parser, Transpiler};
    
    proptest! {
        /// Test that simple expressions roundtrip through parse/transpile
        #[test]
        fn parse_transpile_arithmetic(expr in arithmetic_expr()) {
            let parser = Parser::new(&expr);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                // Just ensure transpilation doesn't panic
                let _ = transpiler.transpile(&ast);
            }
        }
    }
}