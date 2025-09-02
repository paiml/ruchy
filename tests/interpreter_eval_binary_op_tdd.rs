// TDD Test Suite for Interpreter::eval_binary_op Complexity Reduction
// Current: 26 cyclomatic, 25 cognitive complexity
// Target: <20 for both metrics
// Strategy: Extract operator-category handlers (arithmetic, comparison, logical)

use ruchy::runtime::interpreter::Interpreter;
use ruchy::frontend::ast::BinaryOp as AstBinaryOp;
use ruchy::runtime::repl::Value;
use std::rc::Rc;

#[cfg(test)]
mod eval_binary_op_tdd {
    use super::*;

    fn create_test_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // Test arithmetic operations
    #[test]
    fn test_arithmetic_add() {
        let interpreter = create_test_interpreter();
        
        // Integer + Integer
        let result = interpreter.eval_binary_op(AstBinaryOp::Add, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(8));
        
        // Float + Float
        let result = interpreter.eval_binary_op(AstBinaryOp::Add, &Value::Float(2.5), &Value::Float(1.5));
        assert_eq!(result.unwrap(), Value::Float(4.0));
        
        // String + String
        let result = interpreter.eval_binary_op(
            AstBinaryOp::Add,
            &Value::String(Rc::new("Hello".to_string())),
            &Value::String(Rc::new(" World".to_string()))
        );
        assert_eq!(result.unwrap(), Value::String(Rc::new("Hello World".to_string())));
    }

    #[test]
    fn test_arithmetic_subtract() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Subtract, &Value::Integer(10), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(7));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Subtract, &Value::Float(10.5), &Value::Float(2.5));
        assert_eq!(result.unwrap(), Value::Float(8.0));
    }

    #[test]
    fn test_arithmetic_multiply() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Multiply, &Value::Integer(4), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(12));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Multiply, &Value::Float(2.5), &Value::Float(4.0));
        assert_eq!(result.unwrap(), Value::Float(10.0));
    }

    #[test]
    fn test_arithmetic_divide() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Divide, &Value::Integer(12), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(4));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Divide, &Value::Float(10.0), &Value::Float(2.0));
        assert_eq!(result.unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_arithmetic_modulo() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Modulo, &Value::Integer(10), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_arithmetic_power() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Power, &Value::Integer(2), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Integer(8));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Power, &Value::Float(2.0), &Value::Float(3.0));
        assert_eq!(result.unwrap(), Value::Float(8.0));
    }

    // Test comparison operations
    #[test]
    fn test_comparison_equal() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Equal, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Equal, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(false));
        
        let result = interpreter.eval_binary_op(
            AstBinaryOp::Equal,
            &Value::String(Rc::new("hello".to_string())),
            &Value::String(Rc::new("hello".to_string()))
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_comparison_not_equal() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::NotEqual, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::NotEqual, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_comparison_less_than() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Less, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Less, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(false));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Less, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_comparison_greater_than() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Greater, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::Greater, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_comparison_less_equal() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::LessEqual, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::LessEqual, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::LessEqual, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_comparison_greater_equal() {
        let interpreter = create_test_interpreter();
        
        let result = interpreter.eval_binary_op(AstBinaryOp::GreaterEqual, &Value::Integer(5), &Value::Integer(3));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::GreaterEqual, &Value::Integer(5), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = interpreter.eval_binary_op(AstBinaryOp::GreaterEqual, &Value::Integer(3), &Value::Integer(5));
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    // Test logical operations
    #[test]
    fn test_logical_and() {
        let interpreter = create_test_interpreter();
        
        // True && True = True (returns right operand)
        let result = interpreter.eval_binary_op(AstBinaryOp::And, &Value::Bool(true), &Value::Integer(42));
        assert_eq!(result.unwrap(), Value::Integer(42));
        
        // False && True = False (returns left operand)
        let result = interpreter.eval_binary_op(AstBinaryOp::And, &Value::Bool(false), &Value::Integer(42));
        assert_eq!(result.unwrap(), Value::Bool(false));
        
        // Test truthy values
        let result = interpreter.eval_binary_op(
            AstBinaryOp::And, 
            &Value::String(Rc::new("hello".to_string())),
            &Value::Integer(123)
        );
        assert_eq!(result.unwrap(), Value::Integer(123));
    }

    #[test]
    fn test_logical_or() {
        let interpreter = create_test_interpreter();
        
        // True || False = True (returns left operand)
        let result = interpreter.eval_binary_op(AstBinaryOp::Or, &Value::Integer(42), &Value::Bool(false));
        assert_eq!(result.unwrap(), Value::Integer(42));
        
        // False || True = True (returns right operand)
        let result = interpreter.eval_binary_op(AstBinaryOp::Or, &Value::Bool(false), &Value::Integer(42));
        assert_eq!(result.unwrap(), Value::Integer(42));
        
        // Nil || 123 = 123 (returns right operand)
        let result = interpreter.eval_binary_op(AstBinaryOp::Or, &Value::Nil, &Value::Integer(123));
        assert_eq!(result.unwrap(), Value::Integer(123));
    }

    // Test error cases
    #[test]
    fn test_unsupported_operations() {
        let interpreter = create_test_interpreter();
        
        // Division by zero should error
        let result = interpreter.eval_binary_op(AstBinaryOp::Divide, &Value::Integer(10), &Value::Integer(0));
        assert!(result.is_err());
        
        // Type mismatch should error for incompatible operations
        let result = interpreter.eval_binary_op(AstBinaryOp::Add, &Value::Integer(5), &Value::Bool(true));
        assert!(result.is_err());
    }

    // Test refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_eval_arithmetic_op() {
            // Test extracted arithmetic operation handler
            let interpreter = create_test_interpreter();
            
            // This would test the extracted eval_arithmetic_op once implemented
            // let result = interpreter.eval_arithmetic_op(AstBinaryOp::Add, &Value::Integer(2), &Value::Integer(3));
            // assert_eq!(result.unwrap(), Value::Integer(5));
        }

        #[test]
        fn test_eval_comparison_op() {
            // Test extracted comparison operation handler
            let interpreter = create_test_interpreter();
            
            // This would test the extracted eval_comparison_op once implemented
            // let result = interpreter.eval_comparison_op(AstBinaryOp::Equal, &Value::Integer(5), &Value::Integer(5));
            // assert_eq!(result.unwrap(), Value::Bool(true));
        }

        #[test]
        fn test_eval_logical_op() {
            // Test extracted logical operation handler
            let interpreter = create_test_interpreter();
            
            // This would test the extracted eval_logical_op once implemented
            // let result = interpreter.eval_logical_op(AstBinaryOp::And, &Value::Bool(true), &Value::Integer(42));
            // assert_eq!(result.unwrap(), Value::Integer(42));
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl Interpreter {
    // Main method becomes a dispatcher (complexity ~5)
    fn eval_binary_op(&self, op: AstBinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Add | AstBinaryOp::Subtract | AstBinaryOp::Multiply | 
            AstBinaryOp::Divide | AstBinaryOp::Modulo | AstBinaryOp::Power => {
                self.eval_arithmetic_op(op, left, right)
            }
            AstBinaryOp::Equal | AstBinaryOp::NotEqual | AstBinaryOp::Less | 
            AstBinaryOp::Greater | AstBinaryOp::LessEqual | AstBinaryOp::GreaterEqual => {
                self.eval_comparison_op(op, left, right)
            }
            AstBinaryOp::And | AstBinaryOp::Or => {
                self.eval_logical_op(op, left, right)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Binary operator not yet implemented: {op:?}"
            ))),
        }
    }

    // Extract arithmetic logic (complexity ~8)
    fn eval_arithmetic_op(&self, op: AstBinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Add => self.add_values(left, right),
            AstBinaryOp::Subtract => self.sub_values(left, right),
            AstBinaryOp::Multiply => self.mul_values(left, right),
            AstBinaryOp::Divide => self.div_values(left, right),
            AstBinaryOp::Modulo => self.modulo_values(left, right),
            AstBinaryOp::Power => self.power_values(left, right),
            _ => unreachable!("Non-arithmetic operation passed to eval_arithmetic_op"),
        }
    }

    // Extract comparison logic (complexity ~8)
    fn eval_comparison_op(&self, op: AstBinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Equal => Ok(Value::from_bool(self.equal_values(left, right))),
            AstBinaryOp::NotEqual => Ok(Value::from_bool(!self.equal_values(left, right))),
            AstBinaryOp::Less => Ok(Value::from_bool(self.less_than_values(left, right)?)),
            AstBinaryOp::Greater => Ok(Value::from_bool(self.greater_than_values(left, right)?)),
            AstBinaryOp::LessEqual => {
                let less = self.less_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(less || equal))
            }
            AstBinaryOp::GreaterEqual => {
                let greater = self.greater_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(greater || equal))
            }
            _ => unreachable!("Non-comparison operation passed to eval_comparison_op"),
        }
    }

    // Extract logical logic (complexity ~4)
    fn eval_logical_op(&self, op: AstBinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::And => {
                if left.is_truthy() {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            }
            AstBinaryOp::Or => {
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            _ => unreachable!("Non-logical operation passed to eval_logical_op"),
        }
    }
}
*/