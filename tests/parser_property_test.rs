//! Parser property tests - comprehensive grammar validation

use proptest::prelude::*;
use ruchy::Parser;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    
    /// Test that parser never panics on any input
    #[test]
    fn parser_never_panics_on_random_input(input: String) {
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Should not panic
    }
    
    /// Test that valid integer literals parse
    #[test]
    fn integer_literals_parse(n: i32) {
        let input = n.to_string();
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse integer: {}", input);
    }
    
    /// Test that valid string literals parse
    #[test]
    fn string_literals_parse(s in "[a-zA-Z0-9 ]{0,20}") {
        let input = format!("\"{}\"", s);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse string: {}", input);
    }
    
    /// Test let bindings
    #[test]
    fn let_bindings_parse(n: u8) {
        let input = format!("let x = {}", n);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse: {}", input);
    }
    
    /// Test list literals
    #[test]
    fn list_literals_parse(values: Vec<u8>) {
        let values_str = values.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("[{}]", values_str);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse list: {}", input);
    }
    
    /// Test rest patterns in lists
    #[test]
    fn rest_patterns_parse(vals: Vec<u8>) {
        if vals.len() < 2 {
            return Ok(()); // Skip if not enough values
        }
        let values_str = vals.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("let [first, ...rest] = [{}]", values_str);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse rest pattern: {}", input);
    }
    
    /// Test tuple destructuring
    #[test]
    fn tuple_destructuring_parse(a: u8, b: u8) {
        let input = format!("let (x, y) = ({}, {})", a, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse tuple destructuring: {}", input);
    }
    
    /// Test character literals
    #[test]
    fn character_literals_parse(c in "[a-zA-Z0-9]") {
        let input = format!("'{}'", c);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse character: {}", input);
    }
}