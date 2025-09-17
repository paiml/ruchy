//! Shared binary operation evaluation
//! Extracted to reduce duplication across interpreter and REPL
use crate::frontend::ast::BinaryOp;
use crate::runtime::Value;
use anyhow::{Result, bail};
///
/// let lhs = `Value::Int(5)`;
/// let rhs = `Value::Int(3)`;
/// let result = `evaluate_binary_op(&BinaryOp::Add`, &lhs, &`rhs).unwrap()`;
/// `assert_eq!(result`, `Value::Int(8)`);
/// ```
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
        BinaryOp::Gt => evaluate_greater(lhs, rhs), // Alias for Greater
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
            let mut result = String::with_capacity(a.len() + b.len());
            result.push_str(a);
            result.push_str(b);
            Ok(Value::String(result))
        }
        (Value::List(a), Value::List(b)) => {
            let mut result = Vec::with_capacity(a.len() + b.len());
            result.extend(a.iter().cloned());
            result.extend(b.iter().cloned());
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::BinaryOp;
    use crate::runtime::Value;

    // Test 1: Arithmetic Addition Operations
    #[test]
    fn test_addition_operations() {
        // Integer addition
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Int(5), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(8));

        // Float addition
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Float(5.5), &Value::Float(3.2)).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 8.7).abs() < f64::EPSILON);
        } else {
            panic!("Expected float result");
        }

        // String concatenation
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::String("hello".to_string()), &Value::String(" world".to_string())).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));

        // List concatenation
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(3), Value::Int(4)]);
        let result = evaluate_binary_op(&BinaryOp::Add, &list1, &list2).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]));

        // Empty string concatenation
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::String("".to_string()), &Value::String("test".to_string())).unwrap();
        assert_eq!(result, Value::String("test".to_string()));
    }

    // Test 2: Arithmetic Subtraction Operations
    #[test]
    fn test_subtraction_operations() {
        // Integer subtraction
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Int(10), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(7));

        // Negative result
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Int(3), &Value::Int(10)).unwrap();
        assert_eq!(result, Value::Int(-7));

        // Float subtraction
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Float(10.5), &Value::Float(3.2)).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 7.3).abs() < f64::EPSILON);
        } else {
            panic!("Expected float result");
        }

        // Zero result
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::Int(5), &Value::Int(5)).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    // Test 3: Multiplication Operations
    #[test]
    fn test_multiplication_operations() {
        // Integer multiplication
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Int(6), &Value::Int(7)).unwrap();
        assert_eq!(result, Value::Int(42));

        // Float multiplication
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Float(2.5), &Value::Float(4.0)).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 10.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float result");
        }

        // String repetition (string * int)
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::String("hi".to_string()), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::String("hihihi".to_string()));

        // String repetition (int * string)
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Int(2), &Value::String("ab".to_string())).unwrap();
        assert_eq!(result, Value::String("abab".to_string()));

        // Zero multiplication
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Int(42), &Value::Int(0)).unwrap();
        assert_eq!(result, Value::Int(0));

        // Empty string repetition
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::String("".to_string()), &Value::Int(5)).unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    // Test 4: Division Operations and Error Cases
    #[test]
    fn test_division_operations() {
        // Integer division
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Int(15), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(5));

        // Float division
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Float(10.0), &Value::Float(4.0)).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 2.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected float result");
        }

        // Integer division by zero - should error
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Int(10), &Value::Int(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));

        // Float division by zero - should error
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Float(10.0), &Value::Float(0.0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));

        // Negative division
        let result = evaluate_binary_op(&BinaryOp::Divide, &Value::Int(-12), &Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(-4));
    }

    // Test 5: Comparison Operations
    #[test]
    fn test_comparison_operations() {
        // Integer comparisons
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Int(3), &Value::Int(5)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::LessEqual, &Value::Int(3), &Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::GreaterEqual, &Value::Int(3), &Value::Int(3)).unwrap(), Value::Bool(true));

        // Float comparisons
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::Float(3.1), &Value::Float(3.2)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::Float(3.2), &Value::Float(3.1)).unwrap(), Value::Bool(true));

        // String comparisons (lexicographic)
        assert_eq!(evaluate_binary_op(&BinaryOp::Less, &Value::String("apple".to_string()), &Value::String("banana".to_string())).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Greater, &Value::String("zebra".to_string()), &Value::String("apple".to_string())).unwrap(), Value::Bool(true));
    }

    // Test 6: Equality Operations
    #[test]
    fn test_equality_operations() {
        // Basic equality
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Int(42), &Value::Int(42)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Int(42), &Value::Int(43)).unwrap(), Value::Bool(false));

        // String equality
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::String("test".to_string()), &Value::String("test".to_string())).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::String("test".to_string()), &Value::String("other".to_string())).unwrap(), Value::Bool(false));

        // Not equal
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::Int(42), &Value::Int(43)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::NotEqual, &Value::Int(42), &Value::Int(42)).unwrap(), Value::Bool(false));

        // Float equality with epsilon
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Float(1.0), &Value::Float(1.0)).unwrap(), Value::Bool(true));

        // Different types should not be equal
        assert_eq!(evaluate_binary_op(&BinaryOp::Equal, &Value::Int(42), &Value::String("42".to_string())).unwrap(), Value::Bool(false));
    }

    // Test 7: Logical Operations
    #[test]
    fn test_logical_operations() {
        // AND operations
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(true), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(false), &Value::Bool(true)).unwrap(), Value::Bool(false));
        assert_eq!(evaluate_binary_op(&BinaryOp::And, &Value::Bool(false), &Value::Bool(false)).unwrap(), Value::Bool(false));

        // OR operations
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(true), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(true), &Value::Bool(false)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(false), &Value::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_binary_op(&BinaryOp::Or, &Value::Bool(false), &Value::Bool(false)).unwrap(), Value::Bool(false));
    }

    // Test 8: Error Cases for Invalid Operations
    #[test]
    fn test_invalid_operation_errors() {
        // Cannot add int and bool
        let result = evaluate_binary_op(&BinaryOp::Add, &Value::Int(5), &Value::Bool(true));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot add"));

        // Cannot subtract string and int
        let result = evaluate_binary_op(&BinaryOp::Subtract, &Value::String("test".to_string()), &Value::Int(5));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot subtract"));

        // Cannot multiply bool and float
        let result = evaluate_binary_op(&BinaryOp::Multiply, &Value::Bool(true), &Value::Float(3.14));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot multiply"));

        // Cannot compare int and string
        let result = evaluate_binary_op(&BinaryOp::Less, &Value::Int(5), &Value::String("test".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot compare"));
    }
}
