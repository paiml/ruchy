//! Comprehensive tests for Lazy Evaluation module
//! Target: Increase coverage from 0.17% to >50%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod lazy_tests {
    use crate::runtime::lazy::{LazyValue, LazyIterator};
    use crate::runtime::repl::Value;
    use std::rc::Rc;
    use anyhow::Result;
    
    // ========== LazyValue Tests ==========
    
    #[test]
    fn test_computed_value() {
        let value = Value::Int(42);
        let lazy = LazyValue::computed(value.clone());
        
        assert!(lazy.is_computed());
        
        let result = lazy.force();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }
    
    #[test]
    fn test_deferred_value() {
        let lazy = LazyValue::deferred(|| Ok(Value::Int(100)));
        
        // Initially not computed
        assert!(!lazy.is_computed());
        
        // Force evaluation
        let result = lazy.force();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(100));
        
        // Now it should be computed (cached)
        assert!(lazy.is_computed());
        
        // Second force should use cached value
        let result2 = lazy.force();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Value::Int(100));
    }
    
    #[test]
    fn test_deferred_error() {
        let lazy = LazyValue::deferred(|| {
            Err(anyhow::anyhow!("Computation failed"))
        });
        
        assert!(!lazy.is_computed());
        
        let result = lazy.force();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Computation failed"));
    }
    
    #[test]
    fn test_pipeline_value() {
        let source = LazyValue::computed(Value::Int(10));
        let pipeline = LazyValue::pipeline(source, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n * 2)),
                _ => Ok(v),
            }
        });
        
        assert!(!pipeline.is_computed());
        
        let result = pipeline.force();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(20));
    }
    
    #[test]
    fn test_pipeline_chain() {
        let source = LazyValue::computed(Value::Int(5));
        
        let pipeline1 = LazyValue::pipeline(source, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n + 10)),
                _ => Ok(v),
            }
        });
        
        let pipeline2 = LazyValue::pipeline(pipeline1, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n * 2)),
                _ => Ok(v),
            }
        });
        
        let result = pipeline2.force();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(30)); // (5 + 10) * 2
    }
    
    #[test]
    fn test_clone_lazy_value() {
        let lazy = LazyValue::deferred(|| Ok(Value::Int(42)));
        let cloned = lazy.clone();
        
        // Force original
        let result1 = lazy.force();
        assert!(result1.is_ok());
        
        // Force clone - should get same value
        let result2 = cloned.force();
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
    
    // ========== LazyIterator Tests ==========
    
    #[test]
    fn test_lazy_iterator_from_vec() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ];
        
        let iter = LazyIterator::from_vec(values.clone());
        let collected = iter.collect();
        
        assert!(collected.is_ok());
        assert_eq!(collected.unwrap(), values);
    }
    
    #[test]
    fn test_lazy_iterator_map() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ];
        
        let iter = LazyIterator::from_vec(values)
            .map(|v| {
                match v {
                    Value::Int(n) => Ok(Value::Int(n * 2)),
                    _ => Ok(v),
                }
            });
        
        let result = iter.collect();
        assert!(result.is_ok());
        
        let expected = vec![
            Value::Int(2),
            Value::Int(4),
            Value::Int(6),
        ];
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_lazy_iterator_filter() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];
        
        let iter = LazyIterator::from_vec(values)
            .filter(|v| {
                match v {
                    Value::Int(n) => Ok(*n % 2 == 0),
                    _ => Ok(false),
                }
            });
        
        let result = iter.collect();
        assert!(result.is_ok());
        
        let expected = vec![
            Value::Int(2),
            Value::Int(4),
        ];
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_lazy_iterator_take() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];
        
        let iter = LazyIterator::from_vec(values).take(3);
        
        let result = iter.collect();
        assert!(result.is_ok());
        
        let expected = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ];
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_lazy_iterator_skip() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];
        
        let iter = LazyIterator::from_vec(values).skip(2);
        
        let result = iter.collect();
        assert!(result.is_ok());
        
        let expected = vec![
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_lazy_iterator_chain() {
        let values = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];
        
        let iter = LazyIterator::from_vec(values)
            .filter(|v| {
                match v {
                    Value::Int(n) => Ok(*n > 2),
                    _ => Ok(false),
                }
            })
            .map(|v| {
                match v {
                    Value::Int(n) => Ok(Value::Int(n * 10)),
                    _ => Ok(v),
                }
            })
            .take(2);
        
        let result = iter.collect();
        assert!(result.is_ok());
        
        let expected = vec![
            Value::Int(30),
            Value::Int(40),
        ];
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_lazy_iterator_empty() {
        let iter = LazyIterator::from_vec(vec![]);
        let result = iter.collect();
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[test]
    fn test_lazy_iterator_single_element() {
        let iter = LazyIterator::from_vec(vec![Value::Int(42)]);
        let result = iter.collect();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::Int(42)]);
    }
    
    // ========== Complex Scenarios ==========
    
    #[test]
    fn test_complex_lazy_pipeline() {
        // Create a complex pipeline with multiple stages
        let source = LazyValue::computed(Value::Int(1));
        
        let stage1 = LazyValue::pipeline(source, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n + 5)),
                _ => Ok(v),
            }
        });
        
        let stage2 = LazyValue::pipeline(stage1, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n * 3)),
                _ => Ok(v),
            }
        });
        
        let stage3 = LazyValue::pipeline(stage2, |v| {
            match v {
                Value::Int(n) => Ok(Value::Int(n - 2)),
                _ => Ok(v),
            }
        });
        
        let result = stage3.force();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(16)); // ((1 + 5) * 3) - 2
    }
    
    #[test]
    fn test_deferred_with_side_effects() {
        use std::cell::Cell;
        
        let counter = Rc::new(Cell::new(0));
        let counter_clone = Rc::clone(&counter);
        
        let lazy = LazyValue::deferred(move || {
            counter_clone.set(counter_clone.get() + 1);
            Ok(Value::Int(counter_clone.get() as i64))
        });
        
        // First force - counter should be 1
        let result1 = lazy.force();
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), Value::Int(1));
        assert_eq!(counter.get(), 1);
        
        // Second force - should use cached value, counter stays at 1
        let result2 = lazy.force();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Value::Int(1));
        assert_eq!(counter.get(), 1);
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_lazy_value_never_panics(n: i64) {
            let lazy = LazyValue::computed(Value::Int(n));
            let _ = lazy.force();
            let _ = lazy.is_computed();
        }
        
        #[test]
        fn test_pipeline_preserves_value(n: i64) {
            let source = LazyValue::computed(Value::Int(n));
            let identity = LazyValue::pipeline(source, |v| Ok(v));
            
            let result = identity.force();
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), Value::Int(n));
        }
        
        #[test]
        fn test_iterator_operations_commute(
            values in prop::collection::vec(0i64..100, 0..20)
        ) {
            let ruchy_values: Vec<Value> = values.iter()
                .map(|&n| Value::Int(n))
                .collect();
            
            // Take then skip
            let iter1 = LazyIterator::from_vec(ruchy_values.clone())
                .take(10)
                .skip(5);
            let result1 = iter1.collect();
            
            // Just collect without transformations
            let iter2 = LazyIterator::from_vec(ruchy_values.clone());
            let all_values = iter2.collect().unwrap_or_default();
            let expected: Vec<Value> = all_values.into_iter()
                .take(10)
                .skip(5)
                .collect();
            
            if let Ok(r1) = result1 {
                prop_assert_eq!(r1, expected);
            }
        }
    }
}