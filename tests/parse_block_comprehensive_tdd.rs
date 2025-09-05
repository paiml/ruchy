//! Comprehensive TDD safety net for parse_block refactoring
//! Target: 25 complexity → ≤10 with systematic function extraction
//! Focus: Cover all parsing paths before refactoring complexity hotspot

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind, Literal};
    
    // Helper function (complexity: 3)
    fn parse_block_expr(input: &str) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        Ok(expr)
    }
    
    // Helper to extract block contents (complexity: 3)
    fn extract_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        if let ExprKind::Block(exprs) = &expr.kind {
            Some(exprs)
        } else {
            None
        }
    }
    
    // Basic block parsing tests (complexity: 2 each)
    #[test]
    fn test_parse_empty_block() {
        let result = parse_block_expr("{}");
        assert!(result.is_ok(), "Failed to parse empty block");
        
        let expr = result.unwrap();
        // Empty blocks become unit literals
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Unit)));
    }
    
    #[test]
    fn test_parse_single_expression_block() {
        let result = parse_block_expr("{ 42 }");
        assert!(result.is_ok(), "Failed to parse single expression block");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_multiple_expression_block() {
        let result = parse_block_expr("{ 1; 2; 3 }");
        assert!(result.is_ok(), "Failed to parse multiple expression block");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 3);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_without_semicolons() {
        let result = parse_block_expr("{ 1\n2\n3 }");
        assert!(result.is_ok(), "Failed to parse block without semicolons");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 3);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    // Let statement tests (complexity: 3 each)
    #[test]
    fn test_parse_block_with_let_statement() {
        let result = parse_block_expr("{ let x = 42; x + 1 }");
        assert!(result.is_ok(), "Failed to parse block with let statement");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            // Should contain a let expression
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_let_expression() {
        let result = parse_block_expr("{ let x = 42 in x + 1 }");
        assert!(result.is_ok(), "Failed to parse block with let expression");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_multiple_let_statements() {
        let result = parse_block_expr("{ let x = 10; let y = 20; x + y }");
        assert!(result.is_ok(), "Failed to parse block with multiple let statements");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            // Should be nested let expressions
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_let_without_semicolon() {
        let result = parse_block_expr("{ let x = 42\nx + 1 }");
        assert!(result.is_ok(), "Failed to parse let without semicolon");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    // Object literal detection tests (complexity: 3 each)
    #[test]
    fn test_parse_object_literal_basic() {
        let result = parse_block_expr("{ name: \"John\", age: 30 }");
        assert!(result.is_ok(), "Failed to parse object literal");
        
        let expr = result.unwrap();
        // Should be parsed as object literal, not block
        assert!(matches!(expr.kind, ExprKind::ObjectLiteral { .. }));
    }
    
    #[test]
    fn test_parse_object_literal_with_spread() {
        let result = parse_block_expr("{ ...other, name: \"John\" }");
        assert!(result.is_ok(), "Failed to parse object literal with spread");
        
        let expr = result.unwrap();
        assert!(matches!(expr.kind, ExprKind::ObjectLiteral { .. }));
    }
    
    #[test]
    fn test_parse_object_literal_string_keys() {
        let result = parse_block_expr("{ \"first-name\": \"John\", \"last-name\": \"Doe\" }");
        assert!(result.is_ok(), "Failed to parse object literal with string keys");
        
        let expr = result.unwrap();
        assert!(matches!(expr.kind, ExprKind::ObjectLiteral { .. }));
    }
    
    #[test]
    fn test_distinguish_block_vs_object() {
        // This should be a block (expression statement)
        let block_result = parse_block_expr("{ x + y }");
        assert!(block_result.is_ok());
        let block_expr = block_result.unwrap();
        assert!(matches!(block_expr.kind, ExprKind::Block(_)));
        
        // This should be an object (key: value pattern)
        let obj_result = parse_block_expr("{ x: y }");
        assert!(obj_result.is_ok());
        let obj_expr = obj_result.unwrap();
        assert!(matches!(obj_expr.kind, ExprKind::ObjectLiteral { .. }));
    }
    
    // Complex block expression tests (complexity: 4 each)
    #[test]
    fn test_parse_nested_blocks() {
        let result = parse_block_expr("{ { 1; 2 }; { 3; 4 } }");
        assert!(result.is_ok(), "Failed to parse nested blocks");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 2);
            // Each should be a block
            assert!(matches!(exprs[0].kind, ExprKind::Block(_)));
            assert!(matches!(exprs[1].kind, ExprKind::Block(_)));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_control_flow() {
        let result = parse_block_expr("{ if true { 1 } else { 2 }; 3 }");
        assert!(result.is_ok(), "Failed to parse block with control flow");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 2);
            assert!(matches!(exprs[0].kind, ExprKind::If { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_function_calls() {
        let result = parse_block_expr("{ calculate(x, y); other_func() }");
        assert!(result.is_ok(), "Failed to parse block with function calls");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 2);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_mixed_statements() {
        let result = parse_block_expr("{ let x = 10; let y = x * 2; if y > 15 { y } else { 0 } }");
        assert!(result.is_ok(), "Failed to parse complex mixed statements");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            // Should be nested let expressions
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    // Edge case tests (complexity: 3 each)
    #[test]
    fn test_parse_block_trailing_semicolon() {
        let result = parse_block_expr("{ 1; 2; }");
        assert!(result.is_ok(), "Failed to parse block with trailing semicolon");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 2);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_with_whitespace() {
        let result = parse_block_expr("{\n  1;\n  2;\n  3\n}");
        assert!(result.is_ok(), "Failed to parse block with whitespace");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 3);
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_single_let_no_body() {
        let result = parse_block_expr("{ let x = 42 }");
        assert!(result.is_ok(), "Failed to parse single let without body");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    // Error handling tests (complexity: 2 each)
    #[test]
    fn test_parse_block_missing_closing_brace() {
        let result = parse_block_expr("{ 1; 2");
        assert!(result.is_err(), "Should fail on missing closing brace");
    }
    
    #[test]
    fn test_parse_block_invalid_let_syntax() {
        let result = parse_block_expr("{ let x; x }");
        // This should either parse successfully or fail gracefully
        let _ = result; // Not asserting specific behavior for this edge case
    }
    
    #[test]
    fn test_parse_block_incomplete_expression() {
        let result = parse_block_expr("{ 1 + }");
        assert!(result.is_err(), "Should fail on incomplete expression");
    }
    
    // Integration tests (complexity: 5 each)
    #[test]
    fn test_parse_complex_block_structure() {
        let input = "{ 
            let data = [1, 2, 3, 4];
            let result = if data.len() > 0 {
                data.iter().sum()
            } else {
                0
            };
            println!(\"Result: {}\", result);
            result
        }";
        
        let result = parse_block_expr(input);
        assert!(result.is_ok(), "Failed to parse complex block structure");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            // Should be a nested let structure
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
    
    #[test]
    fn test_parse_block_vs_object_disambiguation() {
        // Test various edge cases for block vs object detection
        let test_cases = vec![
            ("{ x }", true),       // Block: single identifier
            ("{ x: y }", false),   // Object: key-value pair
            ("{ f() }", true),     // Block: function call
            ("{ ...x }", false),   // Object: spread operator
            ("{ 1 + 2 }", true),   // Block: arithmetic
            ("{ a: b, c: d }", false), // Object: multiple key-value pairs
        ];
        
        for (input, should_be_block) in test_cases {
            let result = parse_block_expr(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            
            let expr = result.unwrap();
            if should_be_block {
                assert!(
                    matches!(expr.kind, ExprKind::Block(_) | ExprKind::Literal(Literal::Unit)),
                    "Expected block for: {}",
                    input
                );
            } else {
                assert!(
                    matches!(expr.kind, ExprKind::ObjectLiteral { .. }),
                    "Expected object for: {}",
                    input
                );
            }
        }
    }
    
    #[test]
    fn test_parse_deeply_nested_let_statements() {
        let input = "{ 
            let a = 1;
            let b = a + 2;
            let c = b * 3;
            let d = c - 4;
            d + 5
        }";
        
        let result = parse_block_expr(input);
        assert!(result.is_ok(), "Failed to parse deeply nested lets");
        
        let expr = result.unwrap();
        if let Some(exprs) = extract_block_exprs(&expr) {
            assert_eq!(exprs.len(), 1);
            // Should be nested let expressions
            assert!(matches!(exprs[0].kind, ExprKind::Let { .. }));
        } else {
            panic!("Expected Block expression");
        }
    }
}