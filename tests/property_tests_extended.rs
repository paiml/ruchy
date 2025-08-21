#![allow(clippy::unwrap_used, clippy::uninlined_format_args, clippy::redundant_closure_for_method_calls)]
//! Extended property-based tests for Ruchy compiler

use proptest::prelude::*;
use ruchy::Parser;
use ruchy::runtime::Repl;

proptest! {
    /// Test that arithmetic operations preserve mathematical properties
    #[test]
    fn prop_arithmetic_commutative(a in -1000i32..1000, b in -1000i32..1000) {
        let mut repl = Repl::new().unwrap();
        
        // Addition is commutative
        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);
        let result1 = repl.eval(&expr1).unwrap();
        let result2 = repl.eval(&expr2).unwrap();
        prop_assert_eq!(result1, result2, "Addition not commutative");
        
        // Multiplication is commutative
        let expr3 = format!("{} * {}", a, b);
        let expr4 = format!("{} * {}", b, a);
        let result3 = repl.eval(&expr3).unwrap();
        let result4 = repl.eval(&expr4).unwrap();
        prop_assert_eq!(result3, result4, "Multiplication not commutative");
    }
    
    /// Test that string operations don't panic
    #[test]
    fn prop_string_handling(s in "\\PC*") {
        let mut repl = Repl::new().unwrap();
        
        // String literal handling
        let expr = format!(r#"let x = "{}""#, s.escape_default());
        let _ = repl.eval(&expr); // Should not panic
        
        // String interpolation
        let expr2 = format!(r#"f"Value: {{{}}}""#, s.escape_default());
        let _ = repl.eval(&expr2); // Should not panic
    }
    
    /// Test that identifiers are parsed correctly
    #[test]
    fn prop_identifier_parsing(s in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
        let input = format!("let {} = 42", s);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse valid identifier: {}", s);
    }
    
    /// Test that numbers are parsed and evaluated correctly
    #[test]
    fn prop_number_roundtrip(n in -1_000_000i64..1_000_000) {
        let mut repl = Repl::new().unwrap();
        let expr = n.to_string();
        let result = repl.eval(&expr).unwrap();
        prop_assert_eq!(result, n.to_string(), "Number roundtrip failed");
    }
    
    /// Test that boolean operations follow logic laws
    #[test]
    fn prop_boolean_logic(a: bool, b: bool) {
        let mut repl = Repl::new().unwrap();
        
        // De Morgan's law: !(a && b) == !a || !b
        let expr1 = format!("!({} && {})", a, b);
        let expr2 = format!("!{} || !{}", a, b);
        let result1 = repl.eval(&expr1).unwrap();
        let result2 = repl.eval(&expr2).unwrap();
        prop_assert_eq!(result1, result2, "De Morgan's law failed");
        
        // Double negation: !!a == a
        let expr3 = format!("!!{}", a);
        let result3 = repl.eval(&expr3).unwrap();
        prop_assert_eq!(result3, a.to_string(), "Double negation failed");
    }
    
    /// Test that list operations preserve length
    #[test]
    fn prop_list_length(items in prop::collection::vec(0i32..100, 0..20)) {
        let mut repl = Repl::new().unwrap();
        
        let list_str = format!("[{}]", items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
        let expr = format!("{}.len()", list_str);
        let result = repl.eval(&expr).unwrap();
        prop_assert_eq!(result, items.len().to_string(), "List length incorrect");
    }
    
    /// Test that function definitions and calls work
    #[test]
    fn prop_function_identity(x in -1000i32..1000) {
        let mut repl = Repl::new().unwrap();
        
        // Define identity function
        repl.eval("fun identity(x: i32) -> i32 { x }").unwrap();
        
        // Call with value
        let expr = format!("identity({})", x);
        let result = repl.eval(&expr).unwrap();
        prop_assert_eq!(result, x.to_string(), "Identity function failed");
    }
    
    /// Test that match expressions are exhaustive
    #[test]
    fn prop_match_exhaustive(n in 0i32..10) {
        let mut repl = Repl::new().unwrap();
        
        let expr = format!(r#"
            match {} {{
                0 => "zero",
                1 => "one",
                2 => "two",
                _ => "other"
            }}
        "#, n);
        
        let result = repl.eval(&expr);
        prop_assert!(result.is_ok(), "Match expression failed for {}", n);
    }
    
    /// Test that loops terminate correctly
    #[test]
    fn prop_loop_termination(limit in 0i32..100) {
        let mut repl = Repl::new().unwrap();
        
        let expr = format!(r"
            let mut count = 0
            while count < {} {{
                count = count + 1
            }}
            count
        ", limit);
        
        let result = repl.eval(&expr).unwrap();
        prop_assert_eq!(result, limit.to_string(), "Loop didn't terminate correctly");
    }
    
    /// Test that block expressions return last value
    #[test]
    fn prop_block_return(a in -100i32..100, b in -100i32..100) {
        let mut repl = Repl::new().unwrap();
        
        let expr = format!(r"
            {{
                let x = {}
                let y = {}
                x + y
            }}
        ", a, b);
        
        let result = repl.eval(&expr).unwrap();
        let expected = (a + b).to_string();
        prop_assert_eq!(result, expected, "Block didn't return correct value");
    }
}