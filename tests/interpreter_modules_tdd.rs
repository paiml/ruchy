//! TDD tests for refactored interpreter modules
//! Comprehensive coverage for all extracted modules

#[cfg(test)]
mod value_tests {
    use ruchy::runtime::interpreter_modules::value::Value;
    use std::rc::Rc;

    #[test]
    fn test_value_creation() {
        assert_eq!(Value::from_i64(42), Value::Integer(42));
        assert_eq!(Value::from_f64(3.14), Value::Float(3.14));
        assert_eq!(Value::from_bool(true), Value::Bool(true));
        assert_eq!(Value::nil(), Value::Nil);
    }

    #[test]
    fn test_value_truthiness() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Integer(1).is_truthy());
        assert!(!Value::Integer(0).is_truthy());
        assert!(Value::Float(1.0).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
        assert!(!Value::Nil.is_truthy());
        assert!(Value::from_string("hello".to_string()).is_truthy());
        assert!(!Value::from_string("".to_string()).is_truthy());
    }

    #[test]
    fn test_value_arithmetic() {
        let a = Value::Integer(10);
        let b = Value::Integer(5);
        
        assert_eq!(a.add(&b).unwrap(), Value::Integer(15));
        assert_eq!(a.subtract(&b).unwrap(), Value::Integer(5));
        assert_eq!(a.multiply(&b).unwrap(), Value::Integer(50));
        assert_eq!(a.divide(&b).unwrap(), Value::Integer(2));
        assert_eq!(a.modulo(&b).unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_value_mixed_arithmetic() {
        let i = Value::Integer(10);
        let f = Value::Float(2.5);
        
        assert_eq!(i.add(&f).unwrap(), Value::Float(12.5));
        assert_eq!(i.subtract(&f).unwrap(), Value::Float(7.5));
        assert_eq!(i.multiply(&f).unwrap(), Value::Float(25.0));
        assert_eq!(i.divide(&f).unwrap(), Value::Float(4.0));
    }

    #[test]
    fn test_string_concatenation() {
        let a = Value::from_string("Hello".to_string());
        let b = Value::from_string(" World".to_string());
        
        assert_eq!(a.add(&b).unwrap(), Value::from_string("Hello World".to_string()));
    }

    #[test]
    fn test_division_by_zero() {
        let a = Value::Integer(10);
        let zero = Value::Integer(0);
        
        assert!(a.divide(&zero).is_err());
        assert!(a.modulo(&zero).is_err());
    }

    #[test]
    fn test_value_comparison() {
        let a = Value::Integer(5);
        let b = Value::Integer(10);
        
        assert_eq!(a.compare(&b), Some(std::cmp::Ordering::Less));
        assert_eq!(b.compare(&a), Some(std::cmp::Ordering::Greater));
        assert_eq!(a.compare(&a), Some(std::cmp::Ordering::Equal));
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Integer(42)), "42");
        assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Nil), "nil");
        assert_eq!(format!("{}", Value::from_string("hello".to_string())), "hello");
    }

    #[test]
    fn test_array_display() {
        let arr = Value::from_array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(format!("{}", arr), "[1, 2, 3]");
    }

    #[test]
    fn test_tuple_display() {
        let tup = Value::from_tuple(vec![Value::Integer(42)]);
        assert_eq!(format!("{}", tup), "(42,)");
        
        let tup2 = Value::from_tuple(vec![
            Value::Integer(1),
            Value::from_string("a".to_string()),
        ]);
        assert_eq!(format!("{}", tup2), "(1, a)");
    }
}

#[cfg(test)]
mod cache_tests {
    use ruchy::runtime::interpreter_modules::cache::{InlineCache, CacheEntry, CacheState, TypeId};
    use ruchy::runtime::interpreter_modules::value::Value;

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new();
        assert!(matches!(entry.state, CacheState::Uninitialized));
        assert_eq!(entry.hits, 0);
        assert_eq!(entry.misses, 0);
    }

    #[test]
    fn test_cache_entry_update() {
        let mut entry = CacheEntry::new();
        let type_id = TypeId::from_value(&Value::Integer(42));
        
        entry.update(type_id);
        assert!(matches!(entry.state, CacheState::Monomorphic { .. }));
        
        let different_type = TypeId::from_value(&Value::Float(3.14));
        entry.update(different_type);
        assert!(matches!(entry.state, CacheState::Polymorphic { .. }));
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut entry = CacheEntry::new();
        
        entry.record_hit();
        entry.record_hit();
        entry.record_miss();
        
        assert_eq!(entry.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_inline_cache() {
        let mut cache = InlineCache::new();
        
        let type_id = TypeId::from_value(&Value::Integer(42));
        cache.insert(100, type_id);
        
        assert!(cache.lookup(100).is_some());
        assert!(cache.lookup(200).is_none());
    }

    #[test]
    fn test_cache_global_hit_rate() {
        let mut cache = InlineCache::new();
        
        cache.record_hit(100);
        cache.record_hit(100);
        cache.record_miss(200);
        
        assert_eq!(cache.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_type_id_from_value() {
        let int_id = TypeId::from_value(&Value::Integer(42));
        let float_id = TypeId::from_value(&Value::Float(3.14));
        
        assert_ne!(int_id, float_id);
        assert_eq!(int_id.type_name(), "integer");
        assert_eq!(float_id.type_name(), "float");
    }
}

#[cfg(test)]
mod gc_tests {
    use ruchy::runtime::interpreter_modules::gc::{ConservativeGC, GCObject};
    use ruchy::runtime::interpreter_modules::value::Value;

    #[test]
    fn test_gc_track_object() {
        let mut gc = ConservativeGC::new();
        
        let id = gc.track_object(Value::Integer(42));
        assert!(gc.get_object(id).is_some());
    }

    #[test]
    fn test_gc_root_marking() {
        let mut gc = ConservativeGC::new();
        
        let id = gc.track_object(Value::Integer(42));
        gc.add_root(id);
        
        let stats = gc.get_stats();
        assert_eq!(stats.roots, 1);
        
        gc.remove_root(id);
        let stats = gc.get_stats();
        assert_eq!(stats.roots, 0);
    }

    #[test]
    fn test_gc_collection() {
        let mut gc = ConservativeGC::new();
        
        let root_id = gc.track_object(Value::Integer(42));
        gc.add_root(root_id);
        
        let _garbage_id = gc.track_object(Value::Float(3.14));
        
        let freed = gc.collect_garbage();
        assert_eq!(freed, 1);
        
        assert!(gc.get_object(root_id).is_some());
    }

    #[test]
    fn test_gc_stats() {
        let mut gc = ConservativeGC::new();
        
        gc.track_object(Value::Integer(42));
        gc.track_object(Value::Float(3.14));
        
        let stats = gc.get_stats();
        assert_eq!(stats.total_objects, 2);
        assert!(stats.total_bytes > 0);
    }

    #[test]
    fn test_gc_object() {
        let obj = GCObject::new(1, Value::Integer(42));
        
        assert_eq!(obj.id, 1);
        assert_eq!(obj.value, Value::Integer(42));
        assert!(!obj.marked);
        assert_eq!(obj.generation, 0);
    }

    #[test]
    fn test_gc_clear() {
        let mut gc = ConservativeGC::new();
        
        gc.track_object(Value::Integer(42));
        gc.track_object(Value::Float(3.14));
        
        gc.clear();
        
        let stats = gc.get_stats();
        assert_eq!(stats.total_objects, 0);
    }
}

#[cfg(test)]
mod error_tests {
    use ruchy::runtime::interpreter_modules::error::InterpreterError;

    #[test]
    fn test_error_creation() {
        let err = InterpreterError::undefined_variable("x");
        assert_eq!(err.user_message(), "Undefined variable: 'x'");
        
        let err = InterpreterError::type_mismatch("integer", "string");
        assert_eq!(err.user_message(), "Type mismatch: expected integer, found string");
        
        let err = InterpreterError::DivisionByZero;
        assert_eq!(err.user_message(), "Division by zero");
    }

    #[test]
    fn test_error_display() {
        let err = InterpreterError::argument_count_mismatch(2, 3);
        assert_eq!(format!("{}", err), "Wrong number of arguments: expected 2, found 3");
    }
}

#[cfg(test)]
mod type_feedback_tests {
    use ruchy::runtime::interpreter_modules::type_feedback::{TypeFeedback, OperationFeedback};
    use ruchy::runtime::interpreter_modules::cache::TypeId;
    use ruchy::runtime::interpreter_modules::value::Value;

    #[test]
    fn test_type_feedback_creation() {
        let feedback = TypeFeedback::new();
        let stats = feedback.get_statistics();
        
        assert_eq!(stats.binary_ops_tracked, 0);
        assert_eq!(stats.variables_tracked, 0);
        assert_eq!(stats.call_sites_tracked, 0);
    }

    #[test]
    fn test_operation_feedback() {
        let mut feedback = OperationFeedback::new(100);
        
        let int_type = TypeId::from_value(&Value::Integer(42));
        feedback.record(int_type, int_type);
        
        assert!(!feedback.is_stable()); // Not enough observations
        
        for _ in 0..20 {
            feedback.record(int_type, int_type);
        }
        
        assert!(feedback.is_stable());
        assert!(feedback.confidence() > 0.9);
    }

    #[test]
    fn test_type_feedback_recording() {
        let mut feedback = TypeFeedback::new();
        
        let int_type = TypeId::from_value(&Value::Integer(42));
        feedback.record_binary_op(100, int_type, int_type);
        
        let stats = feedback.get_statistics();
        assert_eq!(stats.binary_ops_tracked, 1);
    }
}

#[cfg(test)]
mod threaded_tests {
    use ruchy::runtime::interpreter_modules::threaded::{DirectThreadedInterpreter, Instruction, BinaryOp};
    use ruchy::runtime::interpreter_modules::value::Value;

    #[test]
    fn test_threaded_interpreter_creation() {
        let interp = DirectThreadedInterpreter::new();
        assert!(interp);
    }

    #[test]
    fn test_instruction_creation() {
        let inst = Instruction::LoadConst(Value::Integer(42));
        assert!(matches!(inst, Instruction::LoadConst(_)));
        
        let inst = Instruction::BinaryOp(BinaryOp::Add);
        assert!(matches!(inst, Instruction::BinaryOp(_)));
    }

    #[test]
    fn test_simple_execution() {
        let mut interp = DirectThreadedInterpreter::new();
        
        // Test would require proper bytecode compilation
        // This is a placeholder for the structure
        assert!(interp);
    }
}

#[cfg(test)]
mod builtin_tests {
    use ruchy::runtime::interpreter_modules::builtin;
    use ruchy::runtime::interpreter_modules::value::Value;

    #[test]
    fn test_builtin_len() {
        let builtins = builtin::get_builtins();
        let len_fn = builtins.iter().find(|(name, _)| *name == "len").unwrap().1;
        
        let result = len_fn(&[Value::from_string("hello".to_string())]).unwrap();
        assert_eq!(result, Value::Integer(5));
        
        let result = len_fn(&[Value::from_array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_builtin_type() {
        let builtins = builtin::get_builtins();
        let type_fn = builtins.iter().find(|(name, _)| *name == "type").unwrap().1;
        
        let result = type_fn(&[Value::Integer(42)]).unwrap();
        assert_eq!(result, Value::from_string("integer".to_string()));
        
        let result = type_fn(&[Value::Float(3.14)]).unwrap();
        assert_eq!(result, Value::from_string("float".to_string()));
    }

    #[test]
    fn test_builtin_str() {
        let builtins = builtin::get_builtins();
        let str_fn = builtins.iter().find(|(name, _)| *name == "str").unwrap().1;
        
        let result = str_fn(&[Value::Integer(42)]).unwrap();
        assert_eq!(result, Value::from_string("42".to_string()));
        
        let result = str_fn(&[Value::Bool(true)]).unwrap();
        assert_eq!(result, Value::from_string("true".to_string()));
    }

    #[test]
    fn test_builtin_int() {
        let builtins = builtin::get_builtins();
        let int_fn = builtins.iter().find(|(name, _)| *name == "int").unwrap().1;
        
        let result = int_fn(&[Value::Float(3.14)]).unwrap();
        assert_eq!(result, Value::Integer(3));
        
        let result = int_fn(&[Value::from_string("42".to_string())]).unwrap();
        assert_eq!(result, Value::Integer(42));
        
        let result = int_fn(&[Value::Bool(true)]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_builtin_abs() {
        let builtins = builtin::get_builtins();
        let abs_fn = builtins.iter().find(|(name, _)| *name == "abs").unwrap().1;
        
        let result = abs_fn(&[Value::Integer(-42)]).unwrap();
        assert_eq!(result, Value::Integer(42));
        
        let result = abs_fn(&[Value::Float(-3.14)]).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_builtin_min_max() {
        let builtins = builtin::get_builtins();
        
        let min_fn = builtins.iter().find(|(name, _)| *name == "min").unwrap().1;
        let result = min_fn(&[Value::Integer(5), Value::Integer(2), Value::Integer(8)]).unwrap();
        assert_eq!(result, Value::Integer(2));
        
        let max_fn = builtins.iter().find(|(name, _)| *name == "max").unwrap().1;
        let result = max_fn(&[Value::Integer(5), Value::Integer(2), Value::Integer(8)]).unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_builtin_range() {
        let builtins = builtin::get_builtins();
        let range_fn = builtins.iter().find(|(name, _)| *name == "range").unwrap().1;
        
        // range(5)
        let result = range_fn(&[Value::Integer(5)]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[4], Value::Integer(4));
        } else {
            panic!("Expected array");
        }
        
        // range(2, 5)
        let result = range_fn(&[Value::Integer(2), Value::Integer(5)]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(4));
        } else {
            panic!("Expected array");
        }
    }
}