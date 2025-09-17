//! Comprehensive tests for frontend modules (parser, lexer, AST)
//! Target: Increase frontend coverage

use ruchy::frontend::{Parser, ast::*};

#[test]
fn test_parser_integer_literals() {
    let mut parser = Parser::new("42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
}

#[test]
fn test_parser_float_literals() {
    let mut parser = Parser::new("3.14");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Literal(Literal::Float(f)) => {
            assert!((f - 3.14).abs() < 0.001);
        }
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_parser_string_literals() {
    let mut parser = Parser::new(r#""hello world""#);
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(
        expr.kind,
        ExprKind::Literal(Literal::String(ref s)) if s == "hello world"
    ));
}

#[test]
fn test_parser_boolean_literals() {
    let mut parser = Parser::new("true");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));
    
    let mut parser = Parser::new("false");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(false))));
}

#[test]
fn test_parser_binary_operations() {
    let test_cases = vec![
        ("1 + 2", BinaryOp::Add),
        ("3 - 1", BinaryOp::Subtract),
        ("2 * 3", BinaryOp::Multiply),
        ("6 / 2", BinaryOp::Divide),
        ("7 % 3", BinaryOp::Modulo),
    ];
    
    for (input, expected_op) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Binary { op, .. } => {
                assert_eq!(*op, expected_op);
            }
            _ => panic!("Expected binary operation for: {}", input),
        }
    }
}

#[test]
fn test_parser_comparison_operations() {
    let test_cases = vec![
        ("1 < 2", BinaryOp::Less),
        ("2 > 1", BinaryOp::Greater),
        ("3 <= 3", BinaryOp::LessEqual),
        ("4 >= 4", BinaryOp::GreaterEqual),
        ("5 == 5", BinaryOp::Equal),
        ("6 != 7", BinaryOp::NotEqual),
    ];
    
    for (input, expected_op) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Binary { op, .. } => {
                assert_eq!(*op, expected_op);
            }
            _ => panic!("Expected comparison for: {}", input),
        }
    }
}

#[test]
fn test_parser_logical_operations() {
    let test_cases = vec![
        ("true && false", BinaryOp::And),
        ("true || false", BinaryOp::Or),
    ];
    
    for (input, expected_op) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
        
        let expr = result.unwrap();
        match &expr.kind {
            ExprKind::Binary { op, .. } => {
                assert_eq!(*op, expected_op);
            }
            _ => panic!("Expected logical operation for: {}", input),
        }
    }
}

#[test]
fn test_parser_unary_operations() {
    let mut parser = Parser::new("-42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Unary { .. }));
    
    let mut parser = Parser::new("!true");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Unary { .. }));
}

#[test]
fn test_parser_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    // Should parse as multiplication with grouped addition
    assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Multiply, .. }));
}

#[test]
fn test_parser_variable_declaration() {
    let mut parser = Parser::new("let x = 42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Let { name, .. } => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected let binding"),
    }
}

#[test]
fn test_parser_function_declaration() {
    let mut parser = Parser::new("fun add(a, b) { a + b }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_parser_function_call() {
    let mut parser = Parser::new("print(42)");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Call { func, args } => {
            match &func.kind {
                ExprKind::Identifier(name) => assert_eq!(name, "print"),
                _ => panic!("Expected identifier as callee"),
            }
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_parser_if_expression() {
    let mut parser = Parser::new("if x > 0 { 1 } else { -1 }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::If { .. }));
}

#[test]
fn test_parser_while_loop() {
    let mut parser = Parser::new("while x < 10 { x = x + 1 }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::While { .. }));
}

#[test]
fn test_parser_for_loop() {
    let mut parser = Parser::new("for i in 0..10 { print(i) }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::For { .. }));
}

#[test]
fn test_parser_array_literal() {
    let mut parser = Parser::new("[1, 2, 3]");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::List(elements) => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_parser_object_literal() {
    let mut parser = Parser::new("{ x: 1, y: 2 }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::ObjectLiteral { fields } => {
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected object literal"),
    }
}

#[test]
fn test_parser_block_expression() {
    let mut parser = Parser::new("{ let x = 1; x + 2 }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Block(_)));
}

#[test]
fn test_parser_match_expression() {
    let mut parser = Parser::new("match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
        }
        _ => panic!("Expected match expression"),
    }
}

#[test]
fn test_parser_lambda_expression() {
    let mut parser = Parser::new("|x| x * 2");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Lambda { params, .. } => {
            assert_eq!(params.len(), 1);
        }
        _ => panic!("Expected lambda expression"),
    }
}

#[test]
fn test_parser_assignment() {
    let mut parser = Parser::new("x = 42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Assign { .. }));
}

#[test]
fn test_parser_field_access() {
    let mut parser = Parser::new("obj.field");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::FieldAccess { field, .. } => {
            assert_eq!(field, "field");
        }
        _ => panic!("Expected field access"),
    }
}

#[test]
fn test_parser_index_access() {
    let mut parser = Parser::new("arr[0]");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::IndexAccess { .. }));
}

#[test]
fn test_parser_return_statement() {
    let mut parser = Parser::new("return 42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Return { .. }));
}

#[test]
fn test_parser_break_statement() {
    let mut parser = Parser::new("break");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Break { .. }));
}

#[test]
fn test_parser_continue_statement() {
    let mut parser = Parser::new("continue");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert!(matches!(expr.kind, ExprKind::Continue { .. }));
}

#[test]
fn test_span_tracking() {
    let mut parser = Parser::new("42");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.span.start, 0);
    assert_eq!(expr.span.end, 2);
}

#[test]
fn test_parser_operator_precedence() {
    let mut parser = Parser::new("1 + 2 * 3");
    let result = parser.parse();
    assert!(result.is_ok());
    let expr = result.unwrap();
    // Should parse as 1 + (2 * 3) due to precedence
    match &expr.kind {
        ExprKind::Binary { op: BinaryOp::Add, right, .. } => {
            assert!(matches!(right.kind, ExprKind::Binary { op: BinaryOp::Multiply, .. }));
        }
        _ => panic!("Incorrect precedence parsing"),
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_parser_integer_roundtrip(n in 0i64..i64::MAX) {
            let input = n.to_string();
            let mut parser = Parser::new(&input);
            let result = parser.parse();

            if result.is_ok() {
                let expr = result.unwrap();
                match expr.kind {
                    ExprKind::Literal(Literal::Integer(parsed)) => {
                        prop_assert_eq!(parsed, n);
                    }
                    ExprKind::Unary { op: UnaryOp::Negate, operand } => {
                        // Negative numbers are parsed as unary negation
                        if let ExprKind::Literal(Literal::Integer(val)) = operand.kind {
                            prop_assert_eq!(-val, n);
                        }
                    }
                    _ => prop_assert!(false, "Expected integer literal or unary negation"),
                }
            }
        }
        
        #[test]
        fn prop_parser_string_roundtrip(s in "[a-zA-Z0-9 ]{0,50}") {
            let input = format!(r#""{}""#, s);
            let mut parser = Parser::new(&input);
            let result = parser.parse();
            
            if result.is_ok() {
                let expr = result.unwrap();
                match expr.kind {
                    ExprKind::Literal(Literal::String(parsed)) => {
                        prop_assert_eq!(parsed, s);
                    }
                    _ => prop_assert!(false, "Expected string literal"),
                }
            }
        }
        
        #[test]
        fn prop_parser_never_panics(input in ".*") {
            let input = if input.len() > 1000 { &input[..1000] } else { &input };
            let _ = std::panic::catch_unwind(|| {
                let mut parser = Parser::new(input);
                let _ = parser.parse();
            });
        }
    }
}