//! Shared binary operation evaluation
//! Extracted to reduce duplication across interpreter and REPL

use crate::frontend::ast::BinaryOp;
use crate::runtime::Value;
use anyhow::{Result, bail};

/// Evaluate a binary operation on two values
pub fn evaluate_binary_op(op: &BinaryOp, lhs: &Value, rhs: &Value) -> Result<Value> {
    match op {
        BinaryOp::Add => evaluate_add(lhs, rhs),
        BinaryOp::Subtract => evaluate_subtract(lhs, rhs),
        BinaryOp::Multiply => evaluate_multiply(lhs, rhs),
        BinaryOp::Divide => evaluate_divide(lhs, rhs),
        BinaryOp::Modulo => evaluate_modulo(lhs, rhs),
        BinaryOp::Power => evaluate_power(lhs, rhs),
        
        BinaryOp::Equal => Ok(Value::Bool(values_equal(lhs, rhs))),
        BinaryOp::NotEqual => Ok(Value::Bool(!values_equal(lhs, rhs))),
        BinaryOp::Less => evaluate_less(lhs, rhs),
        BinaryOp::LessEqual => evaluate_less_equal(lhs, rhs),
        BinaryOp::Greater => evaluate_greater(lhs, rhs),
        BinaryOp::GreaterEqual => evaluate_greater_equal(lhs, rhs),
        
        BinaryOp::And => evaluate_and(lhs, rhs),
        BinaryOp::Or => evaluate_or(lhs, rhs),
        
        BinaryOp::BitwiseAnd => evaluate_bitwise_and(lhs, rhs),
        BinaryOp::BitwiseOr => evaluate_bitwise_or(lhs, rhs),
        BinaryOp::BitwiseXor => evaluate_bitwise_xor(lhs, rhs),
        BinaryOp::LeftShift => evaluate_left_shift(lhs, rhs),
        
        BinaryOp::NullCoalesce => Ok(if matches!(lhs, Value::Unit) { 
            rhs.clone() 
        } else { 
            lhs.clone() 
        }),
    }
}

fn evaluate_add(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b)))
        }
        (Value::List(a), Value::List(b)) => {
            let mut result = a.clone();
            result.extend(b.clone());
            Ok(Value::List(result))
        }
        _ => bail!("Cannot add {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_subtract(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        _ => bail!("Cannot subtract {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_multiply(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::String(s), Value::Int(n)) | (Value::Int(n), Value::String(s)) => {
            if *n < 0 {
                bail!("Cannot repeat string negative times");
            }
            Ok(Value::String(s.repeat(*n as usize)))
        }
        _ => bail!("Cannot multiply {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_divide(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                bail!("Division by zero");
            }
            Ok(Value::Int(a / b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b.abs() < f64::EPSILON {
                bail!("Division by zero");
            }
            Ok(Value::Float(a / b))
        }
        _ => bail!("Cannot divide {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_modulo(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                bail!("Modulo by zero");
            }
            Ok(Value::Int(a % b))
        }
        _ => bail!("Cannot modulo {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_power(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b < 0 {
                bail!("Negative exponent not supported for integers");
            }
            Ok(Value::Int(a.pow(*b as u32)))
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        _ => bail!("Cannot exponentiate {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_less(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        _ => bail!("Cannot compare {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_less_equal(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
        _ => bail!("Cannot compare {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_greater(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
        _ => bail!("Cannot compare {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_greater_equal(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
        _ => bail!("Cannot compare {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_and(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
        _ => bail!("Cannot AND {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_or(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
        _ => bail!("Cannot OR {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_bitwise_and(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
        _ => bail!("Cannot bitwise AND {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_bitwise_or(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
        _ => bail!("Cannot bitwise OR {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_bitwise_xor(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
        _ => bail!("Cannot bitwise XOR {:?} and {:?}", lhs, rhs),
    }
}

fn evaluate_left_shift(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b < 0 || *b >= 64 {
                bail!("Invalid shift amount: {}", b);
            }
            Ok(Value::Int(a << b))
        }
        _ => bail!("Cannot left shift {:?} by {:?}", lhs, rhs),
    }
}

/// Check if two values are equal
fn values_equal(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Unit, Value::Unit) => true,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Char(a), Value::Char(b)) => a == b,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Tuple(a), Value::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        _ => false,
    }
}