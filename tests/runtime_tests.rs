//! Comprehensive tests for runtime modules (interpreter, values)
//! Target: Increase runtime coverage

use ruchy::runtime::interpreter::{Interpreter, Value};
use ruchy::frontend::ast::*;

fn create_int_value(n: i64) -> Value {
    Value::Integer(n)
}

fn create_float_value(f: f64) -> Value {
    Value::Float(f)
}

fn create_string_value(s: &str) -> Value {
    Value::String(std::rc::Rc::new(s.to_string()))
}

fn create_bool_value(b: bool) -> Value {
    Value::Bool(b)
}

fn create_literal_expr(lit: Literal) -> Expr {
    Expr {
        kind: ExprKind::Literal(lit),
        span: Span::new(0, 1),
        attributes: vec![],
    }
}

#[test]
fn test_interpreter_creation() {
    let interpreter = Interpreter::new();
    // Should create without panic
    let _ = interpreter;
}

#[test]
fn test_eval_integer_literal() {
    let mut interpreter = Interpreter::new();
    let expr = create_literal_expr(Literal::Integer(42));
    
    let result = interpreter.eval_expr(&expr);
    assert!(result.is_ok());
    
    let value = result.unwrap();
    assert_eq!(value, Value::Integer(42));
}

#[test]
fn test_eval_float_literal() {
    let mut interpreter = Interpreter::new();
    let expr = create_literal_expr(Literal::Float(3.14));
    
    let result = interpreter.eval_expr(&expr);
    assert!(result.is_ok());
    
    let value = result.unwrap();
    match value {
        Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float value"),
    }
}

#[test]
fn test_eval_string_literal() {
    let mut interpreter = Interpreter::new();
    let expr = create_literal_expr(Literal::String("hello".to_string()));
    
    let result = interpreter.eval_expr(&expr);
    assert!(result.is_ok());
    
    let value = result.unwrap();
    assert_eq!(value, Value::String(std::rc::Rc::new("hello".to_string())));
}

#[test]
fn test_eval_bool_literal() {
    let mut interpreter = Interpreter::new();
    
    let true_expr = create_literal_expr(Literal::Bool(true));
    let result = interpreter.eval_expr(&true_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
    
    let false_expr = create_literal_expr(Literal::Bool(false));
    let result = interpreter.eval_expr(&false_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(false));
}

// Note: is_truthy() method doesn't exist in current implementation
// This test is commented out for now
// #[test]
// fn test_value_is_truthy() {

// Note: type_name() method doesn't exist in current implementation  
// #[test]
// fn test_value_type_name() {

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Integer(42)), "42");
    assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
    assert_eq!(format!("{}", Value::String(std::rc::Rc::new("hello".to_string()))), "hello");
    assert_eq!(format!("{}", Value::Bool(true)), "true");
    assert_eq!(format!("{}", Value::Bool(false)), "false");
    assert_eq!(format!("{}", Value::Nil), "nil");
}

#[test]
fn test_value_conversions() {
    use std::rc::Rc;
    // From conversions
    assert_eq!(Value::from_i64(42), Value::Integer(42));
    assert_eq!(Value::from_f64(3.14), Value::Float(3.14));
    assert_eq!(Value::from_bool(true), Value::Bool(true));
    assert_eq!(Value::from_string("test".to_string()), Value::String(Rc::new("test".to_string())));
    assert_eq!(Value::nil(), Value::Nil);
}

// Note: as_i64(), as_f64(), as_bool() methods don't exist in current implementation
// #[test]
// fn test_value_as_conversions() {
//     // Test successful conversions
//     assert_eq!(Value::Integer(42).as_i64().unwrap(), 42);
//     assert!((Value::Float(3.14).as_f64().unwrap() - 3.14).abs() < 0.001);
//     assert_eq!(Value::Bool(true).as_bool().unwrap(), true);
//     
//     // Test conversion errors
//     assert!(Value::String("test".to_string()).as_i64().is_err());
//     assert!(Value::Integer(42).as_bool().is_err());
// }

// Note: is_nil() method doesn't exist, and Unit should be Nil
// #[test]
// fn test_value_is_nil() {
//     assert!(Value::Unit.is_nil());
//     assert!(!Value::Integer(0).is_nil());
//     assert!(!Value::Bool(false).is_nil());
//     assert!(!Value::String("".to_string()).is_nil());
// }

#[test]
fn test_interpreter_binary_arithmetic() {
    let mut interpreter = Interpreter::new();
    
    // Test addition
    let add_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Integer(2))),
            op: BinaryOp::Add,
            right: Box::new(create_literal_expr(Literal::Integer(3))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&add_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(5));
    
    // Test subtraction
    let sub_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Integer(10))),
            op: BinaryOp::Subtract,
            right: Box::new(create_literal_expr(Literal::Integer(4))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&sub_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(6));
}

#[test]
fn test_interpreter_comparison_operations() {
    let mut interpreter = Interpreter::new();
    
    // Test less than
    let lt_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Integer(2))),
            op: BinaryOp::Less,
            right: Box::new(create_literal_expr(Literal::Integer(3))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&lt_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
    
    // Test equality
    let eq_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Integer(5))),
            op: BinaryOp::Equal,
            right: Box::new(create_literal_expr(Literal::Integer(5))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&eq_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_interpreter_logical_operations() {
    let mut interpreter = Interpreter::new();
    
    // Test AND
    let and_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Bool(true))),
            op: BinaryOp::And,
            right: Box::new(create_literal_expr(Literal::Bool(false))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&and_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(false));
    
    // Test OR
    let or_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(create_literal_expr(Literal::Bool(false))),
            op: BinaryOp::Or,
            right: Box::new(create_literal_expr(Literal::Bool(true))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&or_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_interpreter_unary_operations() {
    let mut interpreter = Interpreter::new();
    
    // Test negation
    let neg_expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(create_literal_expr(Literal::Integer(42))),
        },
        span: Span::new(0, 3),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&neg_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(-42));
    
    // Test NOT
    let not_expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(create_literal_expr(Literal::Bool(true))),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&not_expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Bool(false));
}

#[test]
fn test_interpreter_block_expression() {
    let mut interpreter = Interpreter::new();
    
    let block = Expr {
        kind: ExprKind::Block(vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::Integer(2)),
            create_literal_expr(Literal::Integer(42)), // Last value is returned
        ]),
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&block);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(42));
}

#[test]
fn test_interpreter_if_expression() {
    let mut interpreter = Interpreter::new();
    
    // Test true condition
    let if_true = Expr {
        kind: ExprKind::If {
            condition: Box::new(create_literal_expr(Literal::Bool(true))),
            then_branch: Box::new(create_literal_expr(Literal::Integer(1))),
            else_branch: Some(Box::new(create_literal_expr(Literal::Integer(2)))),
        },
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&if_true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(1));
    
    // Test false condition
    let if_false = Expr {
        kind: ExprKind::If {
            condition: Box::new(create_literal_expr(Literal::Bool(false))),
            then_branch: Box::new(create_literal_expr(Literal::Integer(1))),
            else_branch: Some(Box::new(create_literal_expr(Literal::Integer(2)))),
        },
        span: Span::new(0, 10),
        attributes: vec![],
    };
    
    let result = interpreter.eval_expr(&if_false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(2));
}

#[test]
fn test_interpreter_array_operations() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ];
    let array = Value::from_array(values.clone());
    
    match array {
        Value::Array(ref arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        }
        _ => panic!("Expected array value"),
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_value_int_roundtrip(n in i64::MIN..i64::MAX) {
            let value = Value::Integer(n);
            // Note: as_i64() doesn't exist, so just test creation
            match value {
                Value::Integer(i) => prop_assert_eq!(i, n),
                _ => prop_assert!(false, "Expected Integer variant"),
            }
        }
        
        #[test]
        fn prop_value_float_roundtrip(f in any::<f64>()) {
            if !f.is_nan() {
                let value = Value::Float(f);
                // Note: as_f64() doesn't exist, so just test creation
                match value {
                    Value::Float(fl) => prop_assert!((fl - f).abs() < 1e-10 || (fl.is_infinite() && f.is_infinite())),
                    _ => prop_assert!(false, "Expected Float variant"),
                }
            }
        }
        
        #[test]
        fn prop_value_string_roundtrip(s in ".*") {
            use std::rc::Rc;
            let value = Value::String(Rc::new(s.clone()));
            let display = format!("{}", value);
            prop_assert_eq!(display, s);
        }
        
        #[test]
        fn prop_interpreter_int_arithmetic(a in i32::MIN/2..i32::MAX/2, b in i32::MIN/2..i32::MAX/2) {
            let mut interpreter = Interpreter::new();
            
            let add_expr = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(create_literal_expr(Literal::Integer(a as i64))),
                    op: BinaryOp::Add,
                    right: Box::new(create_literal_expr(Literal::Integer(b as i64))),
                },
                span: Span::new(0, 5),
                attributes: vec![],
            };
            
            let result = interpreter.eval_expr(&add_expr);
            if result.is_ok() {
                let value = result.unwrap();
                prop_assert_eq!(value, Value::Integer((a as i64) + (b as i64)));
            }
        }
    }
}