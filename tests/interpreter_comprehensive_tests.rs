#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::approx_constant)]
#![allow(clippy::unreadable_literal)]
//! Comprehensive tests for the runtime interpreter
//!
//! This test suite provides extensive coverage for the interpreter module,
//! focusing on Value types, operations, and evaluation.

#![allow(clippy::unwrap_used)]  // Tests are allowed to use unwrap
#![allow(clippy::panic)]  // Tests are allowed to panic on unexpected conditions

use ruchy::runtime::interpreter::{Interpreter, Value};
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Span};
use std::rc::Rc;

/// Helper function to create test expressions
fn create_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::new(0, 10))
}

/// Test Value creation and basic operations
#[test]
fn test_value_creation() {
    // Integer value
    let int_val = Value::from_i64(42);
    assert_eq!(int_val, Value::Integer(42));
    
    // Float value
    let float_val = Value::from_f64(3.14159265);
    assert_eq!(float_val, Value::Float(3.14159265));
    
    // Boolean value
    let bool_val = Value::from_bool(true);
    assert_eq!(bool_val, Value::Bool(true));
    
    // String value
    let string_val = Value::from_string("hello".to_string());
    if let Value::String(s) = string_val {
        assert_eq!(&**s, "hello");
    } else {
        panic!("Expected string value");
    }
    
    // Nil value
    let nil_val = Value::Nil;
    assert_eq!(nil_val, Value::Nil);
}

/// Test Value array operations
#[test]
fn test_value_array_operations() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ];
    let array = Value::Array(Rc::new(values));
    
    if let Value::Array(arr) = array {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Integer(1));
        assert_eq!(arr[2], Value::Integer(3));
    } else {
        panic!("Expected array value");
    }
}

/// Test interpreter creation and initialization
#[test]
fn test_interpreter_creation() {
    let _interpreter = Interpreter::new();
    // Should create successfully with empty environment
    // No panic means test passed
}

/// Test evaluating literals
#[test]
fn test_eval_literals() {
    let mut interpreter = Interpreter::new();
    
    // Integer literal
    let int_expr = create_expr(ExprKind::Literal(Literal::Integer(100)));
    let result = interpreter.eval_expr(&int_expr).unwrap();
    assert_eq!(result, Value::Integer(100));
    
    // Float literal
    let float_expr = create_expr(ExprKind::Literal(Literal::Float(2.718)));
    let result = interpreter.eval_expr(&float_expr).unwrap();
    assert_eq!(result, Value::Float(2.718));
    
    // Boolean literal
    let bool_expr = create_expr(ExprKind::Literal(Literal::Bool(false)));
    let result = interpreter.eval_expr(&bool_expr).unwrap();
    assert_eq!(result, Value::Bool(false));
    
    // String literal
    let string_expr = create_expr(ExprKind::Literal(Literal::String("test".to_string())));
    let result = interpreter.eval_expr(&string_expr).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "test");
    }
}

/// Test binary operations
#[test]
fn test_eval_binary_operations() {
    let mut interpreter = Interpreter::new();
    
    // Addition
    let add_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        op: BinaryOp::Add,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(20)))),
    });
    let result = interpreter.eval_expr(&add_expr).unwrap();
    assert_eq!(result, Value::Integer(30));
    
    // Subtraction
    let sub_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(50)))),
        op: BinaryOp::Subtract,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(15)))),
    });
    let result = interpreter.eval_expr(&sub_expr).unwrap();
    assert_eq!(result, Value::Integer(35));
    
    // Multiplication
    let mul_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(6)))),
        op: BinaryOp::Multiply,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(7)))),
    });
    let result = interpreter.eval_expr(&mul_expr).unwrap();
    assert_eq!(result, Value::Integer(42));
    
    // Division
    let div_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(100)))),
        op: BinaryOp::Divide,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(4)))),
    });
    let result = interpreter.eval_expr(&div_expr).unwrap();
    assert_eq!(result, Value::Integer(25));
}

/// Test comparison operations
#[test]
fn test_eval_comparison_operations() {
    let mut interpreter = Interpreter::new();
    
    // Equal
    let eq_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        op: BinaryOp::Equal,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
    });
    let result = interpreter.eval_expr(&eq_expr).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // Not equal
    let ne_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        op: BinaryOp::NotEqual,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
    });
    let result = interpreter.eval_expr(&ne_expr).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // Less than
    let lt_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
        op: BinaryOp::Less,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
    });
    let result = interpreter.eval_expr(&lt_expr).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // Greater than
    let gt_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        op: BinaryOp::Greater,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
    });
    let result = interpreter.eval_expr(&gt_expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

/// Test logical operations
#[test]
fn test_eval_logical_operations() {
    let mut interpreter = Interpreter::new();
    
    // Logical AND
    let and_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
        op: BinaryOp::And,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Bool(false)))),
    });
    let result = interpreter.eval_expr(&and_expr).unwrap();
    assert_eq!(result, Value::Bool(false));
    
    // Logical OR
    let or_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
        op: BinaryOp::Or,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Bool(false)))),
    });
    let result = interpreter.eval_expr(&or_expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

/// Test unary operations
#[test]
fn test_eval_unary_operations() {
    let mut interpreter = Interpreter::new();
    
    // Negation
    let neg_expr = create_expr(ExprKind::Unary {
        op: UnaryOp::Negate,
        operand: Box::new(create_expr(ExprKind::Literal(Literal::Integer(42)))),
    });
    let result = interpreter.eval_expr(&neg_expr).unwrap();
    assert_eq!(result, Value::Integer(-42));
    
    // Logical NOT
    let not_expr = create_expr(ExprKind::Unary {
        op: UnaryOp::Not,
        operand: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
    });
    let result = interpreter.eval_expr(&not_expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

/// Test variable binding with let
#[test]
fn test_eval_let_binding() {
    let mut interpreter = Interpreter::new();
    
    // let x = 10; x + 5
    let let_expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        body: Box::new(create_expr(ExprKind::Binary {
            left: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
            op: BinaryOp::Add,
            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        })),
        is_mutable: false,
    });
    
    let result = interpreter.eval_expr(&let_expr).unwrap();
    assert_eq!(result, Value::Integer(15));
}

/// Test if-else expressions
#[test]
fn test_eval_if_else() {
    let mut interpreter = Interpreter::new();
    
    // if true then 10 else 20
    let if_expr = create_expr(ExprKind::If {
        condition: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
        then_branch: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        else_branch: Some(Box::new(create_expr(ExprKind::Literal(Literal::Integer(20))))),
    });
    let result = interpreter.eval_expr(&if_expr).unwrap();
    assert_eq!(result, Value::Integer(10));
    
    // if false then 10 else 20
    let if_false_expr = create_expr(ExprKind::If {
        condition: Box::new(create_expr(ExprKind::Literal(Literal::Bool(false)))),
        then_branch: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        else_branch: Some(Box::new(create_expr(ExprKind::Literal(Literal::Integer(20))))),
    });
    let result = interpreter.eval_expr(&if_false_expr).unwrap();
    assert_eq!(result, Value::Integer(20));
}

/// Test list expressions
#[test]
fn test_eval_list() {
    let mut interpreter = Interpreter::new();
    
    let list_expr = create_expr(ExprKind::List(vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
        create_expr(ExprKind::Literal(Literal::Integer(3))),
    ]));
    
    let result = interpreter.eval_expr(&list_expr).unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Integer(1));
        assert_eq!(arr[1], Value::Integer(2));
        assert_eq!(arr[2], Value::Integer(3));
    } else {
        panic!("Expected array value");
    }
}

/// Test block expressions
#[test]
fn test_eval_block() {
    let mut interpreter = Interpreter::new();
    
    // Block with multiple expressions, returns last
    let block_expr = create_expr(ExprKind::Block(vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
        create_expr(ExprKind::Literal(Literal::Integer(3))),
    ]));
    
    let result = interpreter.eval_expr(&block_expr).unwrap();
    assert_eq!(result, Value::Integer(3));
}

/// Test string concatenation
#[test]
fn test_string_concatenation() {
    let mut interpreter = Interpreter::new();
    
    let concat_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::String("Hello, ".to_string())))),
        op: BinaryOp::Add,
        right: Box::new(create_expr(ExprKind::Literal(Literal::String("World!".to_string())))),
    });
    
    let result = interpreter.eval_expr(&concat_expr).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "Hello, World!");
    } else {
        panic!("Expected string value");
    }
}

/// Test float arithmetic
#[test]
fn test_float_arithmetic() {
    let mut interpreter = Interpreter::new();
    
    // Float addition
    let add_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Float(3.14)))),
        op: BinaryOp::Add,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Float(2.86)))),
    });
    let result = interpreter.eval_expr(&add_expr).unwrap();
    if let Value::Float(f) = result {
        assert!((f - 6.0).abs() < 0.001);
    }
    
    // Float multiplication
    let mul_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Float(2.5)))),
        op: BinaryOp::Multiply,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Float(4.0)))),
    });
    let result = interpreter.eval_expr(&mul_expr).unwrap();
    assert_eq!(result, Value::Float(10.0));
}

/// Test modulo operation
#[test]
fn test_modulo_operation() {
    let mut interpreter = Interpreter::new();
    
    let mod_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        op: BinaryOp::Modulo,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(3)))),
    });
    
    let result = interpreter.eval_expr(&mod_expr).unwrap();
    assert_eq!(result, Value::Integer(1));
}

/// Test power operation
#[test]
fn test_power_operation() {
    let mut interpreter = Interpreter::new();
    
    let pow_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(2)))),
        op: BinaryOp::Power,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(8)))),
    });
    
    let result = interpreter.eval_expr(&pow_expr).unwrap();
    assert_eq!(result, Value::Integer(256));
}

/// Test nested let bindings
#[test]
fn test_nested_let_bindings() {
    let mut interpreter = Interpreter::new();
    
    // let x = 10; let y = 20; x + y
    let nested_let = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        body: Box::new(create_expr(ExprKind::Let {
            name: "y".to_string(),
            type_annotation: None,
            value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(20)))),
            body: Box::new(create_expr(ExprKind::Binary {
                left: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
                op: BinaryOp::Add,
                right: Box::new(create_expr(ExprKind::Identifier("y".to_string()))),
            })),
            is_mutable: false,
        })),
        is_mutable: false,
    });
    
    let result = interpreter.eval_expr(&nested_let).unwrap();
    assert_eq!(result, Value::Integer(30));
}

/// Test empty block
#[test]
fn test_empty_block() {
    let mut interpreter = Interpreter::new();
    
    let empty_block = create_expr(ExprKind::Block(vec![]));
    let result = interpreter.eval_expr(&empty_block).unwrap();
    assert_eq!(result, Value::Nil);
}

/// Test chained comparisons
#[test]
fn test_chained_comparisons() {
    let mut interpreter = Interpreter::new();
    
    // (5 < 10) && (10 < 15)
    let chained = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Binary {
            left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
            op: BinaryOp::Less,
            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        })),
        op: BinaryOp::And,
        right: Box::new(create_expr(ExprKind::Binary {
            left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
            op: BinaryOp::Less,
            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(15)))),
        })),
    });
    
    let result = interpreter.eval_expr(&chained).unwrap();
    assert_eq!(result, Value::Bool(true));
}

/// Test truthiness of values
#[test]
fn test_value_truthiness() {
    // Test nil is falsy
    assert!(!Value::Nil.is_truthy());
    
    // Test false is falsy
    assert!(!Value::Bool(false).is_truthy());
    
    // Test true is truthy
    assert!(Value::Bool(true).is_truthy());
    
    // Test integers are truthy (even 0)
    assert!(Value::Integer(0).is_truthy());
    assert!(Value::Integer(42).is_truthy());
    
    // Test floats are truthy
    assert!(Value::Float(0.0).is_truthy());
    assert!(Value::Float(3.1415).is_truthy());
    
    // Test strings are truthy (even empty)
    assert!(Value::from_string(String::new()).is_truthy());
    assert!(Value::from_string("hello".to_string()).is_truthy());
}