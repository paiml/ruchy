//! Comprehensive tests for backend modules (transpiler, code generation)
//! Target: Increase backend coverage

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::*;

fn create_literal_expr(lit: Literal) -> Expr {
    Expr {
        kind: ExprKind::Literal(lit),
        span: Span::new(0, 1),
        attributes: vec![],
    }
}

fn create_binary_expr(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    }
}

#[test]
fn test_transpile_integer_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal_expr(Literal::Integer(42));
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_float_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal_expr(Literal::Float(3.14));
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("3.14"));
}

#[test]
fn test_transpile_string_literal() {
    let transpiler = Transpiler::new();
    let expr = create_literal_expr(Literal::String("hello".to_string()));
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("hello") || code.contains("\"hello\""));
}

#[test]
fn test_transpile_bool_literal() {
    let transpiler = Transpiler::new();
    
    let true_expr = create_literal_expr(Literal::Bool(true));
    let result = transpiler.transpile_expr(&true_expr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("true"));
    
    let false_expr = create_literal_expr(Literal::Bool(false));
    let result = transpiler.transpile_expr(&false_expr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("false"));
}

#[test]
fn test_transpile_binary_addition() {
    let transpiler = Transpiler::new();
    let left = create_literal_expr(Literal::Integer(1));
    let right = create_literal_expr(Literal::Integer(2));
    let expr = create_binary_expr(left, BinaryOp::Add, right);
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("1") && code.contains("2"));
    assert!(code.contains("+"));
}

#[test]
fn test_transpile_binary_subtraction() {
    let transpiler = Transpiler::new();
    let left = create_literal_expr(Literal::Integer(5));
    let right = create_literal_expr(Literal::Integer(3));
    let expr = create_binary_expr(left, BinaryOp::Subtract, right);
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("-"));
}

#[test]
fn test_transpile_binary_multiplication() {
    let transpiler = Transpiler::new();
    let left = create_literal_expr(Literal::Integer(3));
    let right = create_literal_expr(Literal::Integer(4));
    let expr = create_binary_expr(left, BinaryOp::Multiply, right);
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("*"));
}

#[test]
fn test_transpile_binary_division() {
    let transpiler = Transpiler::new();
    let left = create_literal_expr(Literal::Integer(10));
    let right = create_literal_expr(Literal::Integer(2));
    let expr = create_binary_expr(left, BinaryOp::Divide, right);
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("/"));
}

#[test]
fn test_transpile_comparison_operators() {
    let transpiler = Transpiler::new();
    let ops = vec![
        BinaryOp::Less,
        BinaryOp::Greater,
        BinaryOp::LessEqual,
        BinaryOp::GreaterEqual,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
    ];
    
    for op in ops {
        let left = create_literal_expr(Literal::Integer(1));
        let right = create_literal_expr(Literal::Integer(2));
        let expr = create_binary_expr(left, op, right);
        
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok(), "Failed to transpile {:?}", op);
    }
}

#[test]
fn test_transpile_logical_operators() {
    let transpiler = Transpiler::new();
    
    let true_expr = create_literal_expr(Literal::Bool(true));
    let false_expr = create_literal_expr(Literal::Bool(false));
    
    // Test AND
    let and_expr = create_binary_expr(true_expr.clone(), BinaryOp::And, false_expr.clone());
    let result = transpiler.transpile_expr(&and_expr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("&&"));
    
    // Test OR
    let or_expr = create_binary_expr(true_expr, BinaryOp::Or, false_expr);
    let result = transpiler.transpile_expr(&or_expr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("||"));
}

#[test]
fn test_transpile_unary_negation() {
    let transpiler = Transpiler::new();
    let inner = create_literal_expr(Literal::Integer(42));
    let expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(inner),
        },
        span: Span::new(0, 3),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("-"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let inner = create_literal_expr(Literal::Bool(true));
    let expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(inner),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("!"));
}

#[test]
fn test_transpile_identifier() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("my_var".to_string()),
        span: Span::new(0, 6),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("my_var"));
}

#[test]
fn test_transpile_array_literal() {
    let transpiler = Transpiler::new();
    let elements = vec![
        create_literal_expr(Literal::Integer(1)),
        create_literal_expr(Literal::Integer(2)),
        create_literal_expr(Literal::Integer(3)),
    ];
    let expr = Expr {
        kind: ExprKind::List(elements),
        span: Span::new(0, 9),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("vec!") || code.contains("["));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

#[test]
fn test_transpile_block_expression() {
    let transpiler = Transpiler::new();
    let statements = vec![
        create_literal_expr(Literal::Integer(42)),
    ];
    let expr = Expr {
        kind: ExprKind::Block(statements),
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("{") || code.contains("42"));
}

#[test]
fn test_transpile_if_expression() {
    let transpiler = Transpiler::new();
    let condition = create_literal_expr(Literal::Bool(true));
    let then_branch = create_literal_expr(Literal::Integer(1));
    let else_branch = Some(Box::new(create_literal_expr(Literal::Integer(2))));
    
    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        },
        span: Span::new(0, 20),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("if"));
}

#[test]
fn test_transpile_function_call() {
    let transpiler = Transpiler::new();
    let callee = Expr {
        kind: ExprKind::Identifier("print".to_string()),
        span: Span::new(0, 5),
        attributes: vec![],
    };
    let args = vec![create_literal_expr(Literal::String("Hello".to_string()))];
    
    let expr = Expr {
        kind: ExprKind::Call {
            func: Box::new(callee),
            args,
        },
        span: Span::new(0, 15),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("print") || code.contains("("));
}

#[test]
fn test_transpile_let_binding() {
    let transpiler = Transpiler::new();
    let value = create_literal_expr(Literal::Integer(42));
    
    let expr = Expr {
        kind: ExprKind::Let {
            name: "x".to_string(),
            value: Box::new(value),
            body: Box::new(create_literal_expr(Literal::Integer(0))), // placeholder body
            type_annotation: None,
            is_mutable: false,
        },
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("let") || code.contains("x"));
}

#[test]
fn test_transpile_assignment() {
    let transpiler = Transpiler::new();
    let target = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::new(0, 1),
        attributes: vec![],
    };
    let value = create_literal_expr(Literal::Integer(100));
    
    let expr = Expr {
        kind: ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        },
        span: Span::new(0, 8),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("=") || code.contains("x"));
}

#[test]
fn test_transpile_while_loop() {
    let transpiler = Transpiler::new();
    let condition = create_literal_expr(Literal::Bool(true));
    let body = create_literal_expr(Literal::Integer(42));
    
    let expr = Expr {
        kind: ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
        },
        span: Span::new(0, 20),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("while") || code.contains("loop"));
}

#[test]
fn test_transpile_return_statement() {
    let transpiler = Transpiler::new();
    let value = Some(Box::new(create_literal_expr(Literal::Integer(42))));
    
    let expr = Expr {
        kind: ExprKind::Return { value },
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("return"));
}

#[test]
fn test_transpile_break_statement() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Break { label: None },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("break"));
}

#[test]
fn test_transpile_continue_statement() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Continue { label: None },
        span: Span::new(0, 8),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_expr(&expr);
    assert!(result.is_ok());
    
    let code = result.unwrap().to_string();
    assert!(code.contains("continue"));
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_transpile_integer_never_panics(n in i64::MIN..i64::MAX) {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::Integer(n));
            
            // Should never panic
            let _ = transpiler.transpile_expr(&expr);
        }
        
        #[test]
        fn prop_transpile_string_never_panics(s in ".*") {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::String(s));
            
            // Should never panic
            let _ = transpiler.transpile_expr(&expr);
        }
        
        #[test]
        fn prop_transpile_preserves_integer_value(n in i64::MIN..i64::MAX) {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::Integer(n));
            
            if let Ok(code) = transpiler.transpile_expr(&expr) {
                let code_str = code.to_string();
                // The integer value should appear in the generated code
                prop_assert!(code_str.contains(&n.to_string()));
            }
        }
    }
}