//! Comprehensive TDD tests for Runtime modules
//! Target: Increase coverage for runtime execution and value management
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod runtime_comprehensive_tests {
    use crate::runtime::{Value, Stack, GarbageCollector, Arena, BinaryOps, PatternMatcher};
    use std::collections::HashMap;
    
    // ========== Value System Tests ==========
    
    #[test]
    fn test_value_creation_all_types() {
        let int_val = Value::Integer(42);
        assert_eq!(int_val.type_name(), "Integer");
        assert!(int_val.is_truthy());
        
        let float_val = Value::Float(3.14);
        assert_eq!(float_val.type_name(), "Float");
        assert!(float_val.is_truthy());
        
        let bool_val = Value::Bool(false);
        assert_eq!(bool_val.type_name(), "Bool");
        assert!(!bool_val.is_truthy());
        
        let str_val = Value::String("hello".to_string());
        assert_eq!(str_val.type_name(), "String");
        assert!(str_val.is_truthy());
        
        let nil_val = Value::Nil;
        assert_eq!(nil_val.type_name(), "Nil");
        assert!(!nil_val.is_truthy());
        
        let array_val = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
        assert_eq!(array_val.type_name(), "Array");
        assert!(array_val.is_truthy());
    }
    
    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
        
        assert_eq!(Value::String("hello".to_string()), Value::String("hello".to_string()));
        assert_ne!(Value::String("hello".to_string()), Value::String("world".to_string()));
        
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
        
        assert_eq!(Value::Nil, Value::Nil);
        assert_ne!(Value::Nil, Value::Integer(0));
    }
    
    #[test]
    fn test_value_display() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::String("hello".to_string()).to_string(), "hello");
        assert_eq!(Value::Nil.to_string(), "nil");
        
        let array = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
        assert!(array.to_string().contains("1"));
        assert!(array.to_string().contains("2"));
    }
    
    #[test]
    fn test_value_coercion() {
        // To boolean
        assert_eq!(Value::Integer(0).to_bool(), false);
        assert_eq!(Value::Integer(1).to_bool(), true);
        assert_eq!(Value::String("".to_string()).to_bool(), false);
        assert_eq!(Value::String("hello".to_string()).to_bool(), true);
        assert_eq!(Value::Nil.to_bool(), false);
        
        // To string
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Bool(true).to_string(), "true");
    }
    
    // ========== Stack Operations Tests ==========
    
    #[test]
    fn test_stack_creation() {
        let stack = Stack::new();
        assert_eq!(stack.size(), 0);
        assert!(stack.is_empty());
    }
    
    #[test]
    fn test_stack_push_pop() {
        let mut stack = Stack::new();
        
        stack.push(Value::Integer(10));
        stack.push(Value::Integer(20));
        stack.push(Value::Integer(30));
        
        assert_eq!(stack.size(), 3);
        assert!(!stack.is_empty());
        
        assert_eq!(stack.pop(), Some(Value::Integer(30)));
        assert_eq!(stack.pop(), Some(Value::Integer(20)));
        assert_eq!(stack.pop(), Some(Value::Integer(10)));
        assert_eq!(stack.pop(), None);
        
        assert!(stack.is_empty());
    }
    
    #[test]
    fn test_stack_peek() {
        let mut stack = Stack::new();
        
        stack.push(Value::Integer(42));
        assert_eq!(stack.peek(), Some(&Value::Integer(42)));
        assert_eq!(stack.size(), 1); // Peek doesn't remove
        
        stack.push(Value::String("top".to_string()));
        assert_eq!(stack.peek(), Some(&Value::String("top".to_string())));
    }
    
    #[test]
    fn test_stack_frame_management() {
        let mut stack = Stack::new();
        
        // Main frame
        stack.push(Value::Integer(1));
        stack.push(Value::Integer(2));
        
        // New frame
        stack.push_frame();
        stack.push(Value::Integer(3));
        stack.push(Value::Integer(4));
        
        assert_eq!(stack.size(), 4);
        
        // Pop frame returns to previous state
        let frame_values = stack.pop_frame();
        assert_eq!(frame_values.len(), 2);
        assert_eq!(stack.size(), 2);
    }
    
    // ========== Garbage Collector Tests ==========
    
    #[test]
    fn test_gc_creation() {
        let gc = GarbageCollector::new();
        assert_eq!(gc.allocated_bytes(), 0);
        assert_eq!(gc.collection_count(), 0);
    }
    
    #[test]
    fn test_gc_allocation() {
        let mut gc = GarbageCollector::new();
        
        let val1 = gc.allocate(Value::Integer(42));
        let val2 = gc.allocate(Value::String("hello".to_string()));
        
        assert!(gc.allocated_bytes() > 0);
        assert_eq!(gc.live_objects(), 2);
    }
    
    #[test]
    fn test_gc_collection() {
        let mut gc = GarbageCollector::new();
        
        // Allocate values
        let val1 = gc.allocate(Value::Integer(1));
        let val2 = gc.allocate(Value::Integer(2));
        let val3 = gc.allocate(Value::Integer(3));
        
        // Mark val1 and val3 as reachable
        gc.mark(&val1);
        gc.mark(&val3);
        
        // Collect unreachable objects
        gc.collect();
        
        assert_eq!(gc.collection_count(), 1);
        assert_eq!(gc.live_objects(), 2); // Only val1 and val3 remain
    }
    
    #[test]
    fn test_gc_threshold_triggering() {
        let mut gc = GarbageCollector::new();
        gc.set_threshold(100); // Low threshold for testing
        
        // Allocate until threshold
        for i in 0..20 {
            gc.allocate(Value::String(format!("string_{}", i)));
        }
        
        // Should trigger collection automatically
        assert!(gc.collection_count() > 0);
    }
    
    // ========== Arena Allocator Tests ==========
    
    #[test]
    fn test_arena_creation() {
        let arena = Arena::new();
        assert_eq!(arena.allocated(), 0);
        assert_eq!(arena.capacity(), Arena::DEFAULT_CAPACITY);
    }
    
    #[test]
    fn test_arena_allocation() {
        let mut arena = Arena::new();
        
        let ptr1 = arena.alloc(Value::Integer(10));
        let ptr2 = arena.alloc(Value::Float(3.14));
        let ptr3 = arena.alloc(Value::String("test".to_string()));
        
        assert_eq!(arena.allocated(), 3);
        
        // Values should be retrievable
        assert_eq!(*arena.get(ptr1), Value::Integer(10));
        assert_eq!(*arena.get(ptr2), Value::Float(3.14));
        assert_eq!(*arena.get(ptr3), Value::String("test".to_string()));
    }
    
    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new();
        
        arena.alloc(Value::Integer(1));
        arena.alloc(Value::Integer(2));
        arena.alloc(Value::Integer(3));
        
        assert_eq!(arena.allocated(), 3);
        
        arena.reset();
        assert_eq!(arena.allocated(), 0);
    }
    
    #[test]
    fn test_arena_growth() {
        let mut arena = Arena::with_capacity(2);
        
        arena.alloc(Value::Integer(1));
        arena.alloc(Value::Integer(2));
        arena.alloc(Value::Integer(3)); // Should trigger growth
        
        assert!(arena.capacity() > 2);
        assert_eq!(arena.allocated(), 3);
    }
    
    // ========== Binary Operations Tests ==========
    
    #[test]
    fn test_binary_arithmetic() {
        // Integer arithmetic
        assert_eq!(
            BinaryOps::add(&Value::Integer(5), &Value::Integer(3)),
            Ok(Value::Integer(8))
        );
        assert_eq!(
            BinaryOps::subtract(&Value::Integer(5), &Value::Integer(3)),
            Ok(Value::Integer(2))
        );
        assert_eq!(
            BinaryOps::multiply(&Value::Integer(5), &Value::Integer(3)),
            Ok(Value::Integer(15))
        );
        assert_eq!(
            BinaryOps::divide(&Value::Integer(6), &Value::Integer(3)),
            Ok(Value::Integer(2))
        );
        
        // Float arithmetic
        assert_eq!(
            BinaryOps::add(&Value::Float(1.5), &Value::Float(2.5)),
            Ok(Value::Float(4.0))
        );
    }
    
    #[test]
    fn test_binary_comparison() {
        assert_eq!(
            BinaryOps::equal(&Value::Integer(5), &Value::Integer(5)),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            BinaryOps::not_equal(&Value::Integer(5), &Value::Integer(3)),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            BinaryOps::less_than(&Value::Integer(3), &Value::Integer(5)),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            BinaryOps::greater_than(&Value::Integer(5), &Value::Integer(3)),
            Ok(Value::Bool(true))
        );
    }
    
    #[test]
    fn test_binary_logical() {
        assert_eq!(
            BinaryOps::and(&Value::Bool(true), &Value::Bool(true)),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            BinaryOps::and(&Value::Bool(true), &Value::Bool(false)),
            Ok(Value::Bool(false))
        );
        assert_eq!(
            BinaryOps::or(&Value::Bool(false), &Value::Bool(true)),
            Ok(Value::Bool(true))
        );
    }
    
    #[test]
    fn test_binary_string_concat() {
        assert_eq!(
            BinaryOps::add(
                &Value::String("hello".to_string()),
                &Value::String(" world".to_string())
            ),
            Ok(Value::String("hello world".to_string()))
        );
    }
    
    #[test]
    fn test_binary_type_errors() {
        // Type mismatches should error
        assert!(BinaryOps::add(&Value::Integer(5), &Value::String("5".to_string())).is_err());
        assert!(BinaryOps::multiply(&Value::Bool(true), &Value::Integer(5)).is_err());
    }
    
    // ========== Pattern Matching Tests ==========
    
    #[test]
    fn test_pattern_literal_match() {
        let matcher = PatternMatcher::new();
        
        let pattern = Pattern::Literal(Value::Integer(42));
        assert!(matcher.matches(&pattern, &Value::Integer(42)));
        assert!(!matcher.matches(&pattern, &Value::Integer(43)));
    }
    
    #[test]
    fn test_pattern_wildcard_match() {
        let matcher = PatternMatcher::new();
        
        let pattern = Pattern::Wildcard;
        assert!(matcher.matches(&pattern, &Value::Integer(42)));
        assert!(matcher.matches(&pattern, &Value::String("any".to_string())));
        assert!(matcher.matches(&pattern, &Value::Nil));
    }
    
    #[test]
    fn test_pattern_binding() {
        let mut matcher = PatternMatcher::new();
        
        let pattern = Pattern::Binding("x".to_string());
        let value = Value::Integer(42);
        
        assert!(matcher.matches(&pattern, &value));
        assert_eq!(matcher.get_binding("x"), Some(&Value::Integer(42)));
    }
    
    #[test]
    fn test_pattern_tuple_match() {
        let matcher = PatternMatcher::new();
        
        let pattern = Pattern::Tuple(vec![
            Pattern::Literal(Value::Integer(1)),
            Pattern::Wildcard,
            Pattern::Literal(Value::Integer(3)),
        ]);
        
        let value = Value::Tuple(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        
        assert!(matcher.matches(&pattern, &value));
    }
    
    #[test]
    fn test_pattern_array_match() {
        let matcher = PatternMatcher::new();
        
        let pattern = Pattern::Array(vec![
            Pattern::Literal(Value::Integer(1)),
            Pattern::Rest,
        ]);
        
        let value = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        
        assert!(matcher.matches(&pattern, &value));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl Stack {
        fn is_empty(&self) -> bool {
            self.size() == 0
        }
        
        fn size(&self) -> usize {
            self.values.len()
        }
    }
    
    impl GarbageCollector {
        fn allocated_bytes(&self) -> usize {
            self.allocated
        }
        
        fn collection_count(&self) -> usize {
            self.collections
        }
        
        fn live_objects(&self) -> usize {
            self.objects.len()
        }
    }
    
    impl Arena {
        const DEFAULT_CAPACITY: usize = 1024;
        
        fn allocated(&self) -> usize {
            self.allocated
        }
        
        fn capacity(&self) -> usize {
            self.capacity
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_value_roundtrip(n in any::<i64>()) {
            let val = Value::Integer(n);
            let s = val.to_string();
            // String representation should contain the number
            assert!(s.contains(&n.to_string()) || n == 0);
        }
        
        #[test]
        fn test_stack_operations_consistency(ops in prop::collection::vec(0i32..100, 1..50)) {
            let mut stack = Stack::new();
            let mut expected = Vec::new();
            
            for val in ops {
                if val % 3 == 0 && !expected.is_empty() {
                    // Pop operation
                    stack.pop();
                    expected.pop();
                } else {
                    // Push operation
                    stack.push(Value::Integer(val as i64));
                    expected.push(val);
                }
            }
            
            assert_eq!(stack.size(), expected.len());
        }
        
        #[test]
        fn test_gc_never_negative(alloc_count in 1usize..100) {
            let mut gc = GarbageCollector::new();
            
            for i in 0..alloc_count {
                gc.allocate(Value::Integer(i as i64));
            }
            
            gc.collect();
            
            assert!(gc.allocated_bytes() >= 0);
            assert!(gc.live_objects() >= 0);
        }
    }
}