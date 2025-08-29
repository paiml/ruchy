//! Property-based tests for the parser
//!
//! [TEST-COV-012] Increase parser test coverage with property testing

use ruchy::Parser;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_integer_parsing(n in any::<i64>()) {
        let input = n.to_string();
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a single integer literal
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Integer"));
    }
    
    #[test]
    fn test_float_parsing(n in any::<f64>().prop_filter("Must be finite", |f| f.is_finite())) {
        let input = format!("{:.6}", n);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a float literal
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Float"));
    }
    
    #[test]
    fn test_string_parsing(s in "\\PC*") {
        let input = format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a string literal
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("String"));
    }
    
    #[test]
    fn test_identifier_parsing(s in "[a-zA-Z_][a-zA-Z0-9_]*") {
        // Skip keywords
        if matches!(s.as_str(), "if" | "else" | "while" | "for" | "let" | "fun" | "return" | 
                   "true" | "false" | "null" | "match" | "enum" | "struct" | "impl" | 
                   "pub" | "mut" | "const" | "static" | "type" | "trait" | "mod" | "use") {
            return Ok(());
        }
        
        let mut parser = Parser::new(&s);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an identifier
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Identifier"));
    }
    
    #[test]
    fn test_binary_expr_parsing(a in 0i32..100, b in 0i32..100, 
                                op in prop::sample::select(vec!["+", "-", "*", "/", "%"])) {
        let input = format!("{} {} {}", a, op, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a binary expression
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_array_parsing(items in prop::collection::vec(0i32..100, 0..10)) {
        let input = format!("[{}]", items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an array/list
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("List") || debug_str.contains("Array"));
    }
    
    #[test]
    fn test_tuple_parsing(a in 0i32..100, b in 0i32..100) {
        let input = format!("({}, {})", a, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a tuple
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Tuple"));
    }
    
    #[test]
    fn test_if_expr_parsing(cond in prop::bool::ANY, a in 0i32..100, b in 0i32..100) {
        let input = format!("if {} {{ {} }} else {{ {} }}", cond, a, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an if expression
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("If"));
    }
    
    #[test]
    fn test_function_call_parsing(name in "[a-z][a-z0-9_]*", 
                                  args in prop::collection::vec(0i32..100, 0..5)) {
        let input = format!("{}({})", name, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "));
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a function call
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Call"));
    }
    
    #[test]
    fn test_lambda_parsing(param in "[a-z][a-z0-9]*", body in 0i32..100) {
        let input = format!("|{}| {}", param, body);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a lambda
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Lambda"));
    }
    
    #[test]
    fn test_object_parsing(keys in prop::collection::vec("[a-z]+", 1..5)) {
        let fields = keys.iter().enumerate()
            .map(|(i, k)| format!("{}: {}", k, i))
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("{{ {} }}", fields);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an object
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Object") || debug_str.contains("Block"));
    }
    
    #[test]
    fn test_range_parsing(start in 0i32..100, end in 0i32..100) {
        let input = format!("{}..{}", start, end);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a range
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Range"));
    }
    
    #[test]
    fn test_inclusive_range_parsing(start in 0i32..100, end in 0i32..100) {
        let input = format!("{}..={}", start, end);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an inclusive range
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Range"));
    }
    
    #[test]
    fn test_method_call_parsing(obj in "[a-z]+", method in "[a-z]+") {
        let input = format!("{}.{}()", obj, method);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a method call
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("MethodCall") || debug_str.contains("Call"));
    }
    
    #[test]
    fn test_index_parsing(arr in "[a-z]+", idx in 0usize..10) {
        let input = format!("{}[{}]", arr, idx);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an index operation
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Index"));
    }
    
    #[test]
    fn test_unary_parsing(op in prop::sample::select(vec!["-", "!"]), val in 0i32..100) {
        let input = format!("{}{}", op, val);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a unary expression
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Unary"));
    }
    
    #[test]
    fn test_comparison_parsing(a in 0i32..100, b in 0i32..100,
                              op in prop::sample::select(vec!["<", ">", "<=", ">=", "==", "!="])) {
        let input = format!("{} {} {}", a, op, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a comparison
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_logical_parsing(a in prop::bool::ANY, b in prop::bool::ANY,
                           op in prop::sample::select(vec!["&&", "||"])) {
        let input = format!("{} {} {}", a, op, b);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a logical expression
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_assignment_parsing(var in "[a-z][a-z0-9]*", val in 0i32..100) {
        let input = format!("let {} = {}", var, val);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a let binding
        let debug_str = format!("{:?}", expr);
        prop_assert!(debug_str.contains("Let"));
    }
}