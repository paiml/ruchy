//! Property-based tests for the parser
//!
//! [TEST-COV-013] Increase parser test coverage with property testing
//! Implements mathematical property verification using both proptest and quickcheck

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{ExprKind, Literal};
use proptest::prelude::*;

// Additional quickcheck-based property tests for robustness
#[cfg(test)]
mod quickcheck_tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    // Property: Parser never panics on any input
    #[quickcheck]
    fn property_parser_never_panics(input: String) -> TestResult {
        if input.len() > 1000 {
            return TestResult::discard();
        }
        
        let mut parser = Parser::new(&input);
        let _result = parser.parse();
        TestResult::passed()
    }

    // Property: Whitespace doesn't affect numeric parsing
    #[quickcheck]
    fn property_whitespace_agnostic_numbers(n: i64) -> TestResult {
        // Skip negative numbers as they need special handling for unary minus
        if n < 0 {
            return TestResult::discard();
        }
        
        let inputs = [
            n.to_string(),
            format!(" {n}"),
            format!("{n} "),
            format!(" {n} "),
            format!("  {n}  "),
            format!("\t{n}\n"),
        ];
        
        let mut results = Vec::new();
        for input in &inputs {
            let mut parser = Parser::new(input);
            if let Ok(expr) = parser.parse() {
                if let ExprKind::Literal(Literal::Integer(parsed)) = expr.kind {
                    results.push(parsed);
                } else {
                    return TestResult::failed();
                }
            } else {
                return TestResult::failed();
            }
        }
        
        TestResult::from_bool(results.iter().all(|&x| x == n))
    }

    // Property: Deep nesting doesn't cause stack overflow
    #[quickcheck]
    fn property_bounded_nesting(depth: u8) -> TestResult {
        let depth = (depth % 50) as usize;
        let mut input = "(".repeat(depth);
        input.push_str("42");
        input.push_str(&")".repeat(depth));
        
        let mut parser = Parser::new(&input);
        let _result = parser.parse(); // Should not crash
        TestResult::passed()
    }

    // Property: Operator precedence consistency
    #[quickcheck] 
    fn property_precedence_consistency() -> TestResult {
        // 2 + 3 * 4 should be 2 + (3 * 4), not (2 + 3) * 4
        let input = "2 + 3 * 4";
        let mut parser = Parser::new(input);
        
        if let Ok(expr) = parser.parse() {
            // Should be addition at top level with multiplication on right
            if let ExprKind::Binary { op, left: _, right } = expr.kind {
                if let ruchy::frontend::ast::BinaryOp::Add = op {
                    if let ExprKind::Binary { op: ruchy::frontend::ast::BinaryOp::Multiply, .. } = right.kind {
                        return TestResult::passed();
                    }
                }
            }
        }
        TestResult::failed()
    }
}

proptest! {
    #[test]
    fn test_integer_parsing(n in any::<i64>()) {
        let input = n.to_string();
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a single integer literal
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Integer"));
    }
    
    #[test]
    fn test_float_parsing(n in any::<f64>().prop_filter("Must be finite", |f| f.is_finite())) {
        let input = format!("{n:.6}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a float literal
        let debug_str = format!("{expr:?}");
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
        let debug_str = format!("{expr:?}");
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
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Identifier"));
    }
    
    #[test]
    fn test_binary_expr_parsing(a in 0i32..100, b in 0i32..100, 
                                op in prop::sample::select(vec!["+", "-", "*", "/", "%"])) {
        let input = format!("{a} {op} {b}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a binary expression
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_array_parsing(items in prop::collection::vec(0i32..100, 0..10)) {
        let input = format!("[{}]", items.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "));
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an array/list
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("List") || debug_str.contains("Array"));
    }
    
    #[test]
    fn test_tuple_parsing(a in 0i32..100, b in 0i32..100) {
        let input = format!("({a}, {b})");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a tuple
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Tuple"));
    }
    
    #[test]
    fn test_if_expr_parsing(cond in prop::bool::ANY, a in 0i32..100, b in 0i32..100) {
        let input = format!("if {cond} {{ {a} }} else {{ {b} }}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an if expression
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("If"));
    }
    
    #[test]
    fn test_function_call_parsing(name in "[a-z][a-z0-9_]*", 
                                  args in prop::collection::vec(0i32..100, 0..5)) {
        let input = format!("{}({})", name, args.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "));
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a function call
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Call"));
    }
    
    #[test]
    fn test_lambda_parsing(param in "[a-z][a-z0-9]*", body in 0i32..100) {
        let input = format!("|{param}| {body}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a lambda
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Lambda"));
    }
    
    #[test]
    fn test_object_parsing(keys in prop::collection::vec("[a-z]+", 1..5)) {
        let fields = keys.iter().enumerate()
            .map(|(i, k)| format!("{k}: {i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("{{ {fields} }}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an object
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Object") || debug_str.contains("Block"));
    }
    
    #[test]
    fn test_range_parsing(start in 0i32..100, end in 0i32..100) {
        let input = format!("{start}..{end}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a range
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Range"));
    }
    
    #[test]
    fn test_inclusive_range_parsing(start in 0i32..100, end in 0i32..100) {
        let input = format!("{start}..={end}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an inclusive range
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Range"));
    }
    
    #[test]
    fn test_method_call_parsing(obj in "[a-z]+", method in "[a-z][a-z0-9]*") {
        // Skip reserved keywords that might cause parsing issues
        if matches!(method.as_str(), "df" | "if" | "else" | "while" | "for" | "let" | "fun" | 
                   "return" | "true" | "false" | "null" | "match" | "enum" | "struct" | 
                   "impl" | "pub" | "mut" | "const" | "static" | "type" | "trait" | "mod" | "use") {
            return Ok(());
        }
        
        let input = format!("{obj}.{method}()");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse method call: {}", input);
        let expr = result.unwrap();
        // Should parse as a method call
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("MethodCall") || debug_str.contains("Call"));
    }
    
    #[test]
    fn test_index_parsing(arr in "[a-z]+", idx in 0usize..10) {
        let input = format!("{arr}[{idx}]");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as an index operation
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Index"));
    }
    
    #[test]
    fn test_unary_parsing(op in prop::sample::select(vec!["-", "!"]), val in 0i32..100) {
        let input = format!("{op}{val}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a unary expression
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Unary"));
    }
    
    #[test]
    fn test_comparison_parsing(a in 0i32..100, b in 0i32..100,
                              op in prop::sample::select(vec!["<", ">", "<=", ">=", "==", "!="])) {
        let input = format!("{a} {op} {b}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a comparison
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_logical_parsing(a in prop::bool::ANY, b in prop::bool::ANY,
                           op in prop::sample::select(vec!["&&", "||"])) {
        let input = format!("{a} {op} {b}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a logical expression
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Binary"));
    }
    
    #[test]
    fn test_assignment_parsing(var in "[a-z][a-z0-9]*", val in 0i32..100) {
        let input = format!("let {var} = {val}");
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as a let binding
        let debug_str = format!("{expr:?}");
        prop_assert!(debug_str.contains("Let"));
    }
}