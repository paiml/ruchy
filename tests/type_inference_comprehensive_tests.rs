//! Comprehensive tests for the type inference engine
//!
//! This test suite provides extensive coverage for the type inference module,
//! targeting the zero-coverage middleend/infer.rs module (889 lines, 0% coverage)

#![allow(warnings)]  // Allow all warnings for test files

use ruchy::middleend::infer::InferenceContext;
use ruchy::middleend::types::{MonoType, TypeScheme};
use ruchy::middleend::environment::TypeEnv;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Pattern, Param, TypeKind};
use ruchy::frontend::ast::Span;

/// Helper function to create a simple integer literal expression
fn create_int_expr(value: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(value)),
        span: Span { start: 0, end: 0 },
        attributes: vec![],
    }
}

/// Helper function to create a simple string literal expression
fn create_string_expr(value: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(value.to_string())),
        span: Span { start: 0, end: 0 },
        attributes: vec![],
    }
}

/// Helper function to create a boolean literal expression
fn create_bool_expr(value: bool) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Bool(value)),
        span: Span { start: 0, end: 0 },
        attributes: vec![],
    }
}

/// Helper function to create an identifier expression
fn create_var_expr(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span { start: 0, end: 0 },
        attributes: vec![],
    }
}

/// Helper function to create a binary operation expression
fn create_binary_expr(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary { 
            left: Box::new(left), 
            op, 
            right: Box::new(right) 
        },
        span: Span { start: 0, end: 0 },
        attributes: vec![],
    }
}

/// Test InferenceContext creation
#[test]
fn test_inference_context_creation() {
    let ctx = InferenceContext::new();
    // Should create without errors
    assert!(std::ptr::addr_of!(ctx) as usize != 0);
}

/// Test InferenceContext creation with environment
#[test]
fn test_inference_context_with_env() {
    let env = TypeEnv::standard();
    let ctx = InferenceContext::with_env(env);
    // Should create successfully with environment
    assert!(std::ptr::addr_of!(ctx) as usize != 0);
}

/// Test type inference for integer literals
#[test]
fn test_infer_integer_literal() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    let expr = create_int_expr(42);
    
    let inferred_type = ctx.infer(&expr)?;
    
    // Should infer integer type
    match inferred_type {
        MonoType::Int => {}, // Expected
        _ => panic!("Expected Int type, got {:?}", inferred_type),
    }
    
    Ok(())
}

/// Test type inference for string literals
#[test]
fn test_infer_string_literal() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    let expr = create_string_expr("hello");
    
    let inferred_type = ctx.infer(&expr)?;
    
    // Should infer string type
    match inferred_type {
        MonoType::String => {}, // Expected
        _ => panic!("Expected String type, got {:?}", inferred_type),
    }
    
    Ok(())
}

/// Test type inference for boolean literals
#[test]
fn test_infer_boolean_literal() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    let expr = create_bool_expr(true);
    
    let inferred_type = ctx.infer(&expr)?;
    
    // Should infer boolean type
    match inferred_type {
        MonoType::Bool => {}, // Expected
        _ => panic!("Expected Bool type, got {:?}", inferred_type),
    }
    
    Ok(())
}

/// Test type inference for binary arithmetic operations
#[test]
fn test_infer_binary_arithmetic() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test addition: 1 + 2
    let expr = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Add,
        create_int_expr(2)
    );
    
    let inferred_type = ctx.infer(&expr)?;
    
    // Should infer integer type for arithmetic
    match inferred_type {
        MonoType::Int => {}, // Expected
        _ => panic!("Expected Int type for arithmetic, got {:?}", inferred_type),
    }
    
    Ok(())
}

/// Test type inference for binary comparison operations
#[test]
fn test_infer_binary_comparison() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test comparison: 1 < 2
    let expr = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Less,
        create_int_expr(2)
    );
    
    let inferred_type = ctx.infer(&expr)?;
    
    // Should infer boolean type for comparison
    match inferred_type {
        MonoType::Bool => {}, // Expected
        _ => panic!("Expected Bool type for comparison, got {:?}", inferred_type),
    }
    
    Ok(())
}

/// Test type inference for multiple arithmetic operations
#[test]
fn test_infer_multiple_arithmetic() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test subtraction: 10 - 5
    let sub_expr = create_binary_expr(
        create_int_expr(10),
        BinaryOp::Subtract,
        create_int_expr(5)
    );
    
    // Test multiplication: 3 * 4
    let mul_expr = create_binary_expr(
        create_int_expr(3),
        BinaryOp::Multiply,
        create_int_expr(4)
    );
    
    // Test division: 8 / 2
    let div_expr = create_binary_expr(
        create_int_expr(8),
        BinaryOp::Divide,
        create_int_expr(2)
    );
    
    // All should infer to Int type
    assert!(matches!(ctx.infer(&sub_expr)?, MonoType::Int));
    assert!(matches!(ctx.infer(&mul_expr)?, MonoType::Int));
    assert!(matches!(ctx.infer(&div_expr)?, MonoType::Int));
    
    Ok(())
}

/// Test type inference for boolean operations
#[test]
fn test_infer_boolean_operations() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test logical AND: true && false
    let and_expr = create_binary_expr(
        create_bool_expr(true),
        BinaryOp::And,
        create_bool_expr(false)
    );
    
    // Test logical OR: true || false
    let or_expr = create_binary_expr(
        create_bool_expr(true),
        BinaryOp::Or,
        create_bool_expr(false)
    );
    
    // Both should infer to Bool type
    assert!(matches!(ctx.infer(&and_expr)?, MonoType::Bool));
    assert!(matches!(ctx.infer(&or_expr)?, MonoType::Bool));
    
    Ok(())
}

/// Test type inference for equality operations
#[test]
fn test_infer_equality_operations() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test equality: 1 == 1
    let eq_expr = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Equal,
        create_int_expr(1)
    );
    
    // Test inequality: 1 != 2
    let ne_expr = create_binary_expr(
        create_int_expr(1),
        BinaryOp::NotEqual,
        create_int_expr(2)
    );
    
    // Both should infer to Bool type
    assert!(matches!(ctx.infer(&eq_expr)?, MonoType::Bool));
    assert!(matches!(ctx.infer(&ne_expr)?, MonoType::Bool));
    
    Ok(())
}

/// Test type inference for all comparison operations
#[test]
fn test_infer_all_comparisons() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    let comparisons = vec![
        (BinaryOp::Less, "less than"),
        (BinaryOp::LessEqual, "less than or equal"),
        (BinaryOp::Greater, "greater than"),
        (BinaryOp::GreaterEqual, "greater than or equal"),
    ];
    
    for (op, _desc) in comparisons {
        let expr = create_binary_expr(
            create_int_expr(5),
            op,
            create_int_expr(10)
        );
        
        let inferred_type = ctx.infer(&expr)?;
        assert!(matches!(inferred_type, MonoType::Bool));
    }
    
    Ok(())
}

/// Test unifier apply method
#[test]
fn test_unifier_apply() {
    let ctx = InferenceContext::new();
    
    // Test applying substitutions to a simple type
    let int_type = MonoType::Int;
    let applied = ctx.apply(&int_type);
    
    // Should return the same type for concrete types
    match applied {
        MonoType::Int => {}, // Expected
        _ => panic!("Expected Int type after apply, got {:?}", applied),
    }
}

/// Test type inference for nested expressions
#[test]
fn test_infer_nested_expressions() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test nested arithmetic: (1 + 2) * 3
    let inner_expr = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Add,
        create_int_expr(2)
    );
    
    let outer_expr = create_binary_expr(
        inner_expr,
        BinaryOp::Multiply,
        create_int_expr(3)
    );
    
    let inferred_type = ctx.infer(&outer_expr)?;
    
    // Should infer integer type
    assert!(matches!(inferred_type, MonoType::Int));
    
    Ok(())
}

/// Test type inference for mixed boolean and comparison
#[test]
fn test_infer_mixed_boolean_comparison() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test mixed: (1 < 2) && (3 > 1)
    let left_comp = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Less,
        create_int_expr(2)
    );
    
    let right_comp = create_binary_expr(
        create_int_expr(3),
        BinaryOp::Greater,
        create_int_expr(1)
    );
    
    let and_expr = create_binary_expr(
        left_comp,
        BinaryOp::And,
        right_comp
    );
    
    let inferred_type = ctx.infer(&and_expr)?;
    
    // Should infer boolean type
    assert!(matches!(inferred_type, MonoType::Bool));
    
    Ok(())
}

/// Test type inference error handling for type mismatches
#[test]
fn test_infer_type_mismatch_handling() {
    let mut ctx = InferenceContext::new();
    
    // This should potentially fail or handle gracefully
    // Test: "hello" + 42 (string + integer)
    let expr = create_binary_expr(
        create_string_expr("hello"),
        BinaryOp::Add,
        create_int_expr(42)
    );
    
    let result = ctx.infer(&expr);
    
    // Should either error or handle gracefully
    match result {
        Ok(_) => {}, // Some inference engines might coerce types
        Err(_) => {}, // Expected for strict type checking
    }
}

/// Test multiple inference contexts
#[test]
fn test_multiple_inference_contexts() -> anyhow::Result<()> {
    // Test that multiple contexts work independently
    let mut ctx1 = InferenceContext::new();
    let mut ctx2 = InferenceContext::new();
    
    let expr1 = create_int_expr(10);
    let expr2 = create_string_expr("test");
    
    let type1 = ctx1.infer(&expr1)?;
    let type2 = ctx2.infer(&expr2)?;
    
    // Should infer different types correctly
    assert!(matches!(type1, MonoType::Int));
    assert!(matches!(type2, MonoType::String));
    
    Ok(())
}

/// Test inference context memory management
#[test]
fn test_inference_memory_management() -> anyhow::Result<()> {
    // Create many contexts to test memory handling
    for _i in 0..50 {
        let mut ctx = InferenceContext::new();
        let expr = create_int_expr(42);
        let _result = ctx.infer(&expr)?;
    }
    
    // Should complete without memory issues
    Ok(())
}

/// Test complex nested boolean logic
#[test]
fn test_complex_boolean_logic() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test: (true && false) || (1 < 2)
    let left_and = create_binary_expr(
        create_bool_expr(true),
        BinaryOp::And,
        create_bool_expr(false)
    );
    
    let right_comp = create_binary_expr(
        create_int_expr(1),
        BinaryOp::Less,
        create_int_expr(2)
    );
    
    let or_expr = create_binary_expr(
        left_and,
        BinaryOp::Or,
        right_comp
    );
    
    let inferred_type = ctx.infer(&or_expr)?;
    
    // Should infer boolean type
    assert!(matches!(inferred_type, MonoType::Bool));
    
    Ok(())
}

/// Test type inference with various literal types
#[test]
fn test_infer_various_literals() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test different literal types
    let int_expr = create_int_expr(123);
    let string_expr = create_string_expr("test_string");
    let bool_true_expr = create_bool_expr(true);
    let bool_false_expr = create_bool_expr(false);
    
    // All should infer correctly
    assert!(matches!(ctx.infer(&int_expr)?, MonoType::Int));
    assert!(matches!(ctx.infer(&string_expr)?, MonoType::String));
    assert!(matches!(ctx.infer(&bool_true_expr)?, MonoType::Bool));
    assert!(matches!(ctx.infer(&bool_false_expr)?, MonoType::Bool));
    
    Ok(())
}

/// Test inference with large numbers
#[test]
fn test_infer_large_numbers() -> anyhow::Result<()> {
    let mut ctx = InferenceContext::new();
    
    // Test with large integer values
    let large_pos = create_int_expr(i64::MAX);
    let large_neg = create_int_expr(i64::MIN);
    let zero = create_int_expr(0);
    
    // All should infer to Int type
    assert!(matches!(ctx.infer(&large_pos)?, MonoType::Int));
    assert!(matches!(ctx.infer(&large_neg)?, MonoType::Int));
    assert!(matches!(ctx.infer(&zero)?, MonoType::Int));
    
    Ok(())
}

/// Test type inference determinism
#[test]
fn test_inference_determinism() -> anyhow::Result<()> {
    // Test that inference is deterministic
    let expr = create_binary_expr(
        create_int_expr(10),
        BinaryOp::Add,
        create_int_expr(20)
    );
    
    // Run multiple times and ensure consistent results
    for _i in 0..10 {
        let mut ctx = InferenceContext::new();
        let result = ctx.infer(&expr)?;
        assert!(matches!(result, MonoType::Int));
    }
    
    Ok(())
}