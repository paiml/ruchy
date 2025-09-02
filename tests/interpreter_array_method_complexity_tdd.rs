// TDD Test Suite for Interpreter::eval_array_higher_order_method Complexity Reduction
// Current: 32 cyclomatic, 57 cognitive complexity
// Target: <20 for both metrics
// Strategy: Extract individual method handlers

use ruchy::runtime::interpreter::Interpreter;
use ruchy::frontend::value::Value;
use std::rc::Rc;

#[cfg(test)]
mod interpreter_array_method_refactoring {
    use super::*;

    // Helper to create a test interpreter
    fn create_test_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // Helper to create test closure value
    fn create_test_closure() -> Value {
        // Create a simple closure that doubles its input for testing
        // This would need proper closure construction in real implementation
        Value::Closure {
            params: vec!["x".to_string()],
            body: Box::new(ruchy::frontend::ast::Expr {
                kind: ruchy::frontend::ast::ExprKind::Binary {
                    left: Box::new(ruchy::frontend::ast::Expr {
                        kind: ruchy::frontend::ast::ExprKind::Identifier("x".to_string()),
                        span: (0, 1).into(),
                    }),
                    op: ruchy::frontend::ast::BinaryOp::Multiply,
                    right: Box::new(ruchy::frontend::ast::Expr {
                        kind: ruchy::frontend::ast::ExprKind::Integer(2),
                        span: (2, 3).into(),
                    }),
                },
                span: (0, 3).into(),
            }),
            env: Rc::new(std::collections::HashMap::new()),
        }
    }

    #[test]
    fn test_map_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let closure = create_test_closure();
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "map", &[closure]);
        assert!(result.is_ok());
        
        if let Ok(Value::Array(result_arr)) = result {
            assert_eq!(result_arr.len(), 3);
        } else {
            panic!("Expected array result from map");
        }
    }

    #[test]
    fn test_filter_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let closure = create_test_closure(); // Would filter based on some condition
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "filter", &[closure]);
        assert!(result.is_ok());
        
        if let Ok(Value::Array(result_arr)) = result {
            assert!(result_arr.len() <= 4); // Filter reduces size
        } else {
            panic!("Expected array result from filter");
        }
    }

    #[test]
    fn test_reduce_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let closure = create_test_closure();
        let initial_value = Value::Integer(0);
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "reduce", &[closure, initial_value]);
        assert!(result.is_ok());
        
        // Reduce should return a single value, not an array
        assert!(!matches!(result.unwrap(), Value::Array(_)));
    }

    #[test]
    fn test_any_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let closure = create_test_closure();
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "any", &[closure]);
        assert!(result.is_ok());
        
        if let Ok(Value::Bool(_)) = result {
            // any returns a boolean
        } else {
            panic!("Expected boolean result from any");
        }
    }

    #[test]
    fn test_all_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let closure = create_test_closure();
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "all", &[closure]);
        assert!(result.is_ok());
        
        if let Ok(Value::Bool(_)) = result {
            // all returns a boolean
        } else {
            panic!("Expected boolean result from all");
        }
    }

    #[test]
    fn test_find_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let closure = create_test_closure();
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "find", &[closure]);
        assert!(result.is_ok());
        
        // find returns either the found item or Nil
        assert!(matches!(result.unwrap(), Value::Integer(_) | Value::Nil));
    }

    #[test]
    fn test_invalid_method() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![Value::Integer(1)]);
        let closure = create_test_closure();
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "invalid_method", &[closure]);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("Unknown array method"));
        }
    }

    #[test]
    fn test_wrong_argument_count() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![Value::Integer(1)]);
        
        // map expects 1 argument, provide 0
        let result = interpreter.eval_array_higher_order_method(&test_array, "map", &[]);
        assert!(result.is_err());
        
        // reduce expects 2 arguments, provide 1
        let closure = create_test_closure();
        let result = interpreter.eval_array_higher_order_method(&test_array, "reduce", &[closure]);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_closure_argument() {
        let mut interpreter = create_test_interpreter();
        let test_array = Rc::new(vec![Value::Integer(1)]);
        let non_closure = Value::Integer(42);
        
        let result = interpreter.eval_array_higher_order_method(&test_array, "map", &[non_closure]);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("expects a function argument"));
        }
    }

    // Tests for refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_eval_map_method() {
            // Test individual map method implementation
            let mut interpreter = create_test_interpreter();
            let test_array = Rc::new(vec![Value::Integer(1), Value::Integer(2)]);
            let closure = create_test_closure();
            
            // This would test the extracted eval_map_method once implemented
            // let result = interpreter.eval_map_method(&test_array, &[closure]);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_eval_filter_method() {
            // Test individual filter method implementation
            let mut interpreter = create_test_interpreter();
            let test_array = Rc::new(vec![Value::Integer(1), Value::Integer(2)]);
            let closure = create_test_closure();
            
            // This would test the extracted eval_filter_method once implemented
            // let result = interpreter.eval_filter_method(&test_array, &[closure]);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_eval_reduce_method() {
            // Test individual reduce method implementation
            let mut interpreter = create_test_interpreter();
            let test_array = Rc::new(vec![Value::Integer(1), Value::Integer(2)]);
            let closure = create_test_closure();
            let initial = Value::Integer(0);
            
            // This would test the extracted eval_reduce_method once implemented
            // let result = interpreter.eval_reduce_method(&test_array, &[closure, initial]);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_validate_closure_argument() {
            // Test the validation helper
            let closure = create_test_closure();
            let non_closure = Value::Integer(42);
            
            // This would test the extracted validate_closure_argument helper
            // assert!(Interpreter::validate_closure_argument(&closure).is_ok());
            // assert!(Interpreter::validate_closure_argument(&non_closure).is_err());
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl Interpreter {
    // Extract map logic (complexity ~8)
    fn eval_map_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "map")?;
        let mut result = Vec::new();
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            result.push(func_result);
        }
        Ok(Value::Array(Rc::new(result)))
    }

    // Extract filter logic (complexity ~8)
    fn eval_filter_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_single_closure_argument(args, "filter")?;
        let mut result = Vec::new();
        for item in arr.iter() {
            let func_result = self.eval_function_call_value(&args[0], std::slice::from_ref(item))?;
            if func_result.is_truthy() {
                result.push(item.clone());
            }
        }
        Ok(Value::Array(Rc::new(result)))
    }

    // Extract reduce logic (complexity ~6)
    fn eval_reduce_method(&mut self, arr: &Rc<Vec<Value>>, args: &[Value]) -> Result<Value, InterpreterError> {
        self.validate_reduce_arguments(args)?;
        let mut accumulator = args[1].clone();
        for item in arr.iter() {
            accumulator = self.eval_function_call_value(&args[0], &[accumulator, item.clone()])?;
        }
        Ok(accumulator)
    }

    // Extract validation helpers (complexity ~2 each)
    fn validate_single_closure_argument(&self, args: &[Value], method_name: &str) -> Result<(), InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError(format!("{} expects 1 argument", method_name)));
        }
        if !matches!(&args[0], Value::Closure { .. }) {
            return Err(InterpreterError::RuntimeError(format!("{} expects a function argument", method_name)));
        }
        Ok(())
    }

    fn validate_reduce_arguments(&self, args: &[Value]) -> Result<(), InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError("reduce expects 2 arguments".to_string()));
        }
        if !matches!(&args[0], Value::Closure { .. }) {
            return Err(InterpreterError::RuntimeError("reduce expects a function and initial value".to_string()));
        }
        Ok(())
    }
}
*/