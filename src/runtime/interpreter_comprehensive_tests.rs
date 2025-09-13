//! Comprehensive TDD tests for Interpreter module
//! Target: Increase coverage from 59.22% to 85%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod interpreter_comprehensive_tests {
    use crate::runtime::interpreter::{Interpreter, Value};
    use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp};

    // ========== Value Creation and Conversion Tests ==========

    #[test]
    fn test_value_creation() {
        // Test all Value creation methods
        let int_val = Value::from_i64(42);
        assert_eq!(int_val, Value::Integer(42));
        
        let float_val = Value::from_f64(3.14);
        assert_eq!(float_val, Value::Float(3.14));
        
        let bool_val = Value::from_bool(true);
        assert_eq!(bool_val, Value::Bool(true));
        
        let nil_val = Value::nil();
        assert!(nil_val.is_nil());
        
        let string_val = Value::from_string("hello".to_string());
        if let Value::String(s) = &string_val {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_value_array_creation() {
        let elements = vec![Value::from_i64(1), Value::from_i64(2), Value::from_i64(3)];
        let array_val = Value::from_array(elements.clone());
        
        if let Value::Array(arr) = &array_val {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        } else {
            panic!("Expected Array value");
        }
    }

    #[test]
    fn test_value_truthiness() {
        assert!(Value::from_bool(true).is_truthy());
        assert!(!Value::from_bool(false).is_truthy());
        assert!(!Value::nil().is_truthy());
        
        // Numbers should be truthy (following common language conventions)
        assert!(Value::from_i64(42).is_truthy());
        assert!(Value::from_i64(0).is_truthy()); // 0 is truthy in Ruchy
        assert!(Value::from_f64(3.14).is_truthy());
    }

    #[test]
    fn test_value_type_conversions() {
        let int_val = Value::from_i64(42);
        assert_eq!(int_val.as_i64().unwrap(), 42);
        assert!(int_val.as_f64().is_err()); // Should fail for wrong type
        assert!(int_val.as_bool().is_err()); // Should fail for wrong type
        
        let float_val = Value::from_f64(3.14);
        assert_eq!(float_val.as_f64().unwrap(), 3.14);
        assert!(float_val.as_i64().is_err());
        
        let bool_val = Value::from_bool(true);
        assert_eq!(bool_val.as_bool().unwrap(), true);
        assert!(bool_val.as_i64().is_err());
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::from_i64(42).type_name(), "integer");
        assert_eq!(Value::from_f64(3.14).type_name(), "float");
        assert_eq!(Value::from_bool(true).type_name(), "boolean");
        assert_eq!(Value::nil().type_name(), "nil");
        assert_eq!(Value::from_string("test".to_string()).type_name(), "string");
        assert_eq!(Value::from_array(vec![]).type_name(), "array");
    }

    // ========== Interpreter Core Functionality Tests ==========

    #[test]
    fn test_interpreter_creation() {
        let interpreter = Interpreter::new();
        
        // Test that interpreter starts with empty state
        let bindings = interpreter.get_global_bindings();
        // Should have some built-in functions, but we can test it's not empty
        assert!(!bindings.is_empty()); // Built-ins should be present
    }

    #[test]
    fn test_interpreter_basic_evaluation() {
        let mut interpreter = Interpreter::new();
        
        // Test integer literal evaluation
        let int_expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Default::default(),
        };
        let result = interpreter.eval_expr(&int_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
        
        // Test float literal evaluation  
        let float_expr = Expr {
            kind: ExprKind::Literal(Literal::Float(3.14)),
            span: Default::default(),
        };
        let result = interpreter.eval_expr(&float_expr).unwrap();
        assert_eq!(result, Value::Float(3.14));
        
        // Test boolean literal evaluation
        let bool_expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Default::default(),
        };
        let result = interpreter.eval_expr(&bool_expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_interpreter_string_evaluation() {
        let mut interpreter = Interpreter::new();
        
        let string_expr = Expr {
            kind: ExprKind::Literal(Literal::String("hello world".to_string())),
            span: Default::default(),
        };
        let result = interpreter.eval_expr(&string_expr).unwrap();
        
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_interpreter_arithmetic_operations() {
        let mut interpreter = Interpreter::new();
        
        // Test addition
        let add_expr = create_binary_expr(
            Literal::Integer(5),
            BinaryOp::Add,
            Literal::Integer(3)
        );
        let result = interpreter.eval_expr(&add_expr).unwrap();
        assert_eq!(result, Value::Integer(8));
        
        // Test subtraction
        let sub_expr = create_binary_expr(
            Literal::Integer(10),
            BinaryOp::Subtract,
            Literal::Integer(4)
        );
        let result = interpreter.eval_expr(&sub_expr).unwrap();
        assert_eq!(result, Value::Integer(6));
        
        // Test multiplication
        let mul_expr = create_binary_expr(
            Literal::Integer(6),
            BinaryOp::Multiply,
            Literal::Integer(7)
        );
        let result = interpreter.eval_expr(&mul_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_interpreter_float_arithmetic() {
        let mut interpreter = Interpreter::new();
        
        // Test float addition
        let add_expr = create_binary_expr(
            Literal::Float(2.5),
            BinaryOp::Add,
            Literal::Float(1.5)
        );
        let result = interpreter.eval_expr(&add_expr).unwrap();
        assert_eq!(result, Value::Float(4.0));
        
        // Test mixed arithmetic (int + float)
        let mixed_expr = create_binary_expr(
            Literal::Integer(10),
            BinaryOp::Add,
            Literal::Float(2.5)
        );
        let result = interpreter.eval_expr(&mixed_expr).unwrap();
        assert_eq!(result, Value::Float(12.5));
    }

    // ========== Stack Operations Tests ==========

    #[test]
    fn test_interpreter_stack_operations() {
        let mut interpreter = Interpreter::new();
        
        // Test stack push
        let value = Value::from_i64(42);
        assert!(interpreter.push(value.clone()).is_ok());
        
        // Test stack peek
        let peeked = interpreter.peek(0).unwrap();
        assert_eq!(peeked, value);
        
        // Test stack pop
        let popped = interpreter.pop().unwrap();
        assert_eq!(popped, value);
        
        // Test stack underflow
        assert!(interpreter.pop().is_err());
    }

    #[test]
    fn test_interpreter_stack_depth() {
        let mut interpreter = Interpreter::new();
        
        // Test multiple pushes and peeks at different depths
        interpreter.push(Value::from_i64(1)).unwrap();
        interpreter.push(Value::from_i64(2)).unwrap();
        interpreter.push(Value::from_i64(3)).unwrap();
        
        assert_eq!(interpreter.peek(0).unwrap(), Value::Integer(3)); // Top
        assert_eq!(interpreter.peek(1).unwrap(), Value::Integer(2)); // Middle
        assert_eq!(interpreter.peek(2).unwrap(), Value::Integer(1)); // Bottom
        
        // Test out-of-bounds peek
        assert!(interpreter.peek(10).is_err());
    }

    // ========== Global Binding Tests ==========

    #[test]
    fn test_global_binding_operations() {
        let mut interpreter = Interpreter::new();
        
        // Test setting global binding
        let name = "test_var".to_string();
        let value = Value::from_i64(42);
        interpreter.set_global_binding(name.clone(), value.clone());
        
        // Test getting global bindings
        let bindings = interpreter.get_global_bindings();
        assert_eq!(bindings.get(&name), Some(&value));
        
        // Test current bindings (should be same as global for now)
        let current_bindings = interpreter.get_current_bindings();
        assert_eq!(current_bindings.get(&name), Some(&value));
    }

    // ========== Garbage Collection Tests ==========

    #[test]
    fn test_gc_tracking() {
        let mut interpreter = Interpreter::new();
        
        // Test tracking objects
        let array_value = Value::from_array(vec![Value::from_i64(1), Value::from_i64(2)]);
        let id = interpreter.gc_track(array_value);
        assert_eq!(id, 0); // First tracked object should have ID 0
        
        // Test GC stats
        let stats = interpreter.gc_stats();
        assert_eq!(stats.objects_before, 1);
    }

    #[test]
    fn test_gc_allocation_helpers() {
        let mut interpreter = Interpreter::new();
        
        // Test GC-managed array allocation
        let elements = vec![Value::from_i64(1), Value::from_i64(2), Value::from_i64(3)];
        let array_val = interpreter.gc_alloc_array(elements);
        
        if let Value::Array(arr) = array_val {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array value");
        }
        
        // Test GC-managed string allocation
        let string_val = interpreter.gc_alloc_string("test string".to_string());
        if let Value::String(s) = string_val {
            assert_eq!(s.as_ref(), "test string");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_gc_configuration() {
        let mut interpreter = Interpreter::new();
        
        // Test GC threshold setting
        interpreter.gc_set_threshold(100);
        
        // Test auto-collection setting
        interpreter.gc_set_auto_collect(true);
        interpreter.gc_set_auto_collect(false);
        
        // Test manual collection
        let stats = interpreter.gc_collect();
        assert_eq!(stats.objects_after, 0); // Should be 0 after collection
        
        // Test GC clear
        interpreter.gc_clear();
        let info = interpreter.gc_info();
        assert_eq!(info.total_objects, 0);
    }

    // ========== String Evaluation Tests ==========

    #[test]
    fn test_eval_string_basic() {
        let mut interpreter = Interpreter::new();
        
        // Test simple arithmetic expression
        let result = interpreter.eval_string("2 + 3").unwrap();
        assert_eq!(result, Value::Integer(5));
        
        // Test string literal
        let result = interpreter.eval_string(r#""hello""#).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_eval_string_error_handling() {
        let mut interpreter = Interpreter::new();
        
        // Test parse error handling
        let result = interpreter.eval_string("invalid syntax +++");
        assert!(result.is_err());
        
        // Test empty string
        let result = interpreter.eval_string("");
        assert!(result.is_err()); // Empty input should be an error
    }

    // ========== Type Feedback and Caching Tests ==========

    #[test]
    fn test_type_feedback() {
        let mut interpreter = Interpreter::new();
        
        // Test getting type feedback stats
        let stats = interpreter.get_type_feedback_stats();
        assert_eq!(stats.monomorphic_sites, 0); // Should start with 0
        
        // Test getting specialization candidates
        let candidates = interpreter.get_specialization_candidates();
        assert!(candidates.is_empty()); // Should start empty
        
        // Test clearing type feedback
        interpreter.clear_type_feedback();
        let stats_after_clear = interpreter.get_type_feedback_stats();
        assert_eq!(stats_after_clear.monomorphic_sites, 0);
    }

    #[test]
    fn test_field_caching() {
        let mut interpreter = Interpreter::new();
        
        // Test cache stats
        let stats = interpreter.get_cache_stats();
        assert!(stats.is_empty()); // Should start empty
        
        // Test cache clearing
        interpreter.clear_caches();
        let stats_after_clear = interpreter.get_cache_stats();
        assert!(stats_after_clear.is_empty());
    }

    // ========== Helper Functions (≤10 complexity each) ==========

    /// Helper: Create binary expression for testing
    fn create_binary_expr(left: Literal, op: BinaryOp, right: Literal) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    kind: ExprKind::Literal(left),
                    span: Default::default(),
                }),
                op,
                right: Box::new(Expr {
                    kind: ExprKind::Literal(right),
                    span: Default::default(),
                }),
            },
            span: Default::default(),
        }
    }

    // ========== Property Tests ==========

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_value_creation_never_panics(i: i64, f: f64, b: bool, s: String) {
            // Property: Value creation should never panic
            let _int_val = Value::from_i64(i);
            let _float_val = Value::from_f64(f);
            let _bool_val = Value::from_bool(b);
            let _string_val = Value::from_string(s);
            let _nil_val = Value::nil();
        }

        #[test]
        fn test_interpreter_stack_never_panics(values in prop::collection::vec(any::<i64>(), 0..100)) {
            // Property: Stack operations should never panic (but may return errors)
            let mut interpreter = Interpreter::new();
            
            for value in values {
                let _ = interpreter.push(Value::from_i64(value));
            }
            
            // Try to pop all values
            loop {
                if interpreter.pop().is_err() {
                    break;
                }
            }
        }

        #[test]
        fn test_gc_operations_never_panic(element_count in 0usize..50) {
            // Property: GC operations should never panic
            let mut interpreter = Interpreter::new();
            
            // Create various objects for GC tracking
            for i in 0..element_count {
                let array = interpreter.gc_alloc_array(vec![Value::from_i64(i as i64)]);
                let _id = interpreter.gc_track(array);
            }
            
            // GC operations should not panic
            let _stats = interpreter.gc_collect();
            let _info = interpreter.gc_info();
            interpreter.gc_clear();
        }
    }
}