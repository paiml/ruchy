//! TDD tests for lazy.rs module
//! Target: Improve lazy.rs from 0% to 90%+ coverage

use ruchy::runtime::lazy::{LazyCache, LazyIterator, LazyValue};
use ruchy::runtime::repl::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_lazy_value_computed_creation() {
    let lazy = LazyValue::computed(Value::Integer(42));
    assert!(lazy.is_computed());
}

#[test]
fn test_lazy_value_computed_force() {
    let lazy = LazyValue::computed(Value::String(Rc::new("test".to_string())));
    let result = lazy.force().unwrap();
    assert_eq!(result, Value::String(Rc::new("test".to_string())));
}

#[test]
fn test_lazy_value_computed_clone() {
    let lazy = LazyValue::computed(Value::Bool(true));
    let cloned = lazy.clone();
    assert!(cloned.is_computed());
    assert_eq!(cloned.force().unwrap(), Value::Bool(true));
}

#[test]
fn test_lazy_value_deferred_creation() {
    let lazy = LazyValue::deferred(|| Ok(Value::Integer(100)));
    assert!(!lazy.is_computed());
}

#[test]
fn test_lazy_value_deferred_force() {
    let lazy = LazyValue::deferred(|| Ok(Value::Float(3.14)));
    let result = lazy.force().unwrap();
    assert_eq!(result, Value::Float(3.14));
    assert!(lazy.is_computed());
}

#[test]
fn test_lazy_value_deferred_caching() {
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);

    let lazy = LazyValue::deferred(move || {
        *counter_clone.borrow_mut() += 1;
        Ok(Value::Integer(999))
    });

    // First force
    lazy.force().unwrap();
    assert_eq!(*counter.borrow(), 1);

    // Second force should use cache
    lazy.force().unwrap();
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn test_lazy_value_deferred_error() {
    let lazy = LazyValue::deferred(|| Err(anyhow::anyhow!("computation failed")));

    let result = lazy.force();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("computation failed"));
}

#[test]
fn test_lazy_value_deferred_clone() {
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);

    let lazy = LazyValue::deferred(move || {
        *counter_clone.borrow_mut() += 1;
        Ok(Value::Integer(77))
    });

    let cloned = lazy.clone();

    // Force original
    lazy.force().unwrap();
    assert_eq!(*counter.borrow(), 1);

    // Force clone should also be cached
    cloned.force().unwrap();
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn test_lazy_value_pipeline_creation() {
    let source = LazyValue::computed(Value::Integer(10));
    let pipeline = LazyValue::pipeline(source, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n * 2))
        } else {
            Ok(v)
        }
    });

    assert!(!pipeline.is_computed()); // Pipelines are never considered computed
}

#[test]
fn test_lazy_value_pipeline_force() {
    let source = LazyValue::computed(Value::Integer(5));
    let pipeline = LazyValue::pipeline(source, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n + 10))
        } else {
            Ok(v)
        }
    });

    let result = pipeline.force().unwrap();
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_lazy_value_pipeline_nested() {
    let source = LazyValue::computed(Value::Integer(2));
    let stage1 = LazyValue::pipeline(source, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n * 3))
        } else {
            Ok(v)
        }
    });
    let stage2 = LazyValue::pipeline(stage1, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n + 4))
        } else {
            Ok(v)
        }
    });

    let result = stage2.force().unwrap();
    assert_eq!(result, Value::Integer(10)); // (2 * 3) + 4 = 10
}

#[test]
fn test_lazy_value_pipeline_with_deferred() {
    let source = LazyValue::deferred(|| Ok(Value::Integer(7)));
    let pipeline = LazyValue::pipeline(source, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::String(Rc::new(format!("number: {n}"))))
        } else {
            Ok(v)
        }
    });

    let result = pipeline.force().unwrap();
    assert_eq!(result, Value::String(Rc::new("number: 7".to_string())));
}

#[test]
fn test_lazy_value_pipeline_error_propagation() {
    let source = LazyValue::deferred(|| Err(anyhow::anyhow!("source error")));
    let pipeline = LazyValue::pipeline(source, |v| Ok(v));

    let result = pipeline.force();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("source error"));
}

#[test]
fn test_lazy_value_pipeline_transform_error() {
    let source = LazyValue::computed(Value::Integer(1));
    let pipeline = LazyValue::pipeline(source, |_| Err(anyhow::anyhow!("transform error")));

    let result = pipeline.force();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("transform error"));
}

#[test]
fn test_lazy_value_pipeline_clone() {
    let source = LazyValue::computed(Value::Integer(3));
    let pipeline = LazyValue::pipeline(source, |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n * n))
        } else {
            Ok(v)
        }
    });

    let cloned = pipeline.clone();

    let result1 = pipeline.force().unwrap();
    let result2 = cloned.force().unwrap();

    assert_eq!(result1, Value::Integer(9));
    assert_eq!(result2, Value::Integer(9));
}

#[test]
fn test_lazy_iterator_from_vec() {
    let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let lazy_iter = LazyIterator::from_vec(values.clone());

    let result = lazy_iter.collect().unwrap();
    assert_eq!(result, values);
}

#[test]
fn test_lazy_iterator_map_integers() {
    let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let lazy_iter = LazyIterator::from_vec(values);
    let mapped = lazy_iter.map(|v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n * 10))
        } else {
            Ok(v)
        }
    });

    let result = mapped.collect().unwrap();
    assert_eq!(
        result,
        vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)]
    );
}

#[test]
fn test_lazy_iterator_map_strings() {
    let values = vec![
        Value::String(Rc::new("a".to_string())),
        Value::String(Rc::new("b".to_string())),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let mapped = lazy_iter.map(|v| {
        if let Value::String(s) = v {
            Ok(Value::String(Rc::new(s.to_uppercase())))
        } else {
            Ok(v)
        }
    });

    let result = mapped.collect().unwrap();
    assert_eq!(
        result,
        vec![
            Value::String(Rc::new("A".to_string())),
            Value::String(Rc::new("B".to_string())),
        ]
    );
}

#[test]
fn test_lazy_iterator_map_error() {
    let values = vec![Value::Integer(1), Value::Integer(2)];
    let lazy_iter = LazyIterator::from_vec(values);
    let mapped = lazy_iter.map(|_| Err(anyhow::anyhow!("map error")));

    let result = mapped.collect();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("map error"));
}

#[test]
fn test_lazy_iterator_filter_even_numbers() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
        Value::Integer(5),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let filtered = lazy_iter.filter(|v| {
        if let Value::Integer(n) = v {
            Ok(n % 2 == 0)
        } else {
            Ok(false)
        }
    });

    let result = filtered.collect().unwrap();
    assert_eq!(result, vec![Value::Integer(2), Value::Integer(4)]);
}

#[test]
fn test_lazy_iterator_filter_strings() {
    let values = vec![
        Value::String(Rc::new("abc".to_string())),
        Value::String(Rc::new("a".to_string())),
        Value::String(Rc::new("abcd".to_string())),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let filtered = lazy_iter.filter(|v| {
        if let Value::String(s) = v {
            Ok(s.len() > 2)
        } else {
            Ok(false)
        }
    });

    let result = filtered.collect().unwrap();
    assert_eq!(
        result,
        vec![
            Value::String(Rc::new("abc".to_string())),
            Value::String(Rc::new("abcd".to_string())),
        ]
    );
}

#[test]
fn test_lazy_iterator_filter_error() {
    let values = vec![Value::Integer(1), Value::Integer(2)];
    let lazy_iter = LazyIterator::from_vec(values);
    let filtered = lazy_iter.filter(|_| Err(anyhow::anyhow!("filter error")));

    let result = filtered.collect();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("filter error"));
}

#[test]
fn test_lazy_iterator_take() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
        Value::Integer(5),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let taken = lazy_iter.take(3);

    let result = taken.collect().unwrap();
    assert_eq!(
        result,
        vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]
    );
}

#[test]
fn test_lazy_iterator_take_more_than_available() {
    let values = vec![Value::Integer(1), Value::Integer(2)];
    let lazy_iter = LazyIterator::from_vec(values.clone());
    let taken = lazy_iter.take(10);

    let result = taken.collect().unwrap();
    assert_eq!(result, values);
}

#[test]
fn test_lazy_iterator_take_zero() {
    let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let lazy_iter = LazyIterator::from_vec(values);
    let taken = lazy_iter.take(0);

    let result = taken.collect().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_lazy_iterator_skip() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
        Value::Integer(5),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let skipped = lazy_iter.skip(2);

    let result = skipped.collect().unwrap();
    assert_eq!(
        result,
        vec![Value::Integer(3), Value::Integer(4), Value::Integer(5)]
    );
}

#[test]
fn test_lazy_iterator_skip_more_than_available() {
    let values = vec![Value::Integer(1), Value::Integer(2)];
    let lazy_iter = LazyIterator::from_vec(values);
    let skipped = lazy_iter.skip(10);

    let result = skipped.collect().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_lazy_iterator_skip_zero() {
    let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let lazy_iter = LazyIterator::from_vec(values.clone());
    let skipped = lazy_iter.skip(0);

    let result = skipped.collect().unwrap();
    assert_eq!(result, values);
}

#[test]
fn test_lazy_iterator_chaining() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
        Value::Integer(5),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let chained = lazy_iter
        .map(|v| {
            if let Value::Integer(n) = v {
                Ok(Value::Integer(n * 2))
            } else {
                Ok(v)
            }
        })
        .filter(|v| {
            if let Value::Integer(n) = v {
                Ok(n > &5)
            } else {
                Ok(false)
            }
        })
        .take(2);

    let result = chained.collect().unwrap();
    assert_eq!(result, vec![Value::Integer(6), Value::Integer(8)]); // 3*2, 4*2
}

#[test]
fn test_lazy_iterator_first_with_values() {
    let values = vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)];
    let lazy_iter = LazyIterator::from_vec(values);

    let first = lazy_iter.first().unwrap();
    assert_eq!(first, Some(Value::Integer(10)));
}

#[test]
fn test_lazy_iterator_first_empty() {
    let values = vec![];
    let lazy_iter = LazyIterator::from_vec(values);

    let first = lazy_iter.first().unwrap();
    assert_eq!(first, None);
}

#[test]
fn test_lazy_iterator_count_source() {
    let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let lazy_iter = LazyIterator::from_vec(values);

    let count = lazy_iter.count().unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_lazy_iterator_count_after_filter() {
    let values = vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
    ];
    let lazy_iter = LazyIterator::from_vec(values);
    let filtered = lazy_iter.filter(|v| {
        if let Value::Integer(n) = v {
            Ok(n % 2 == 0)
        } else {
            Ok(false)
        }
    });

    let count = filtered.count().unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_lazy_cache_new() {
    let cache = LazyCache::new();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_lazy_cache_default() {
    let cache = LazyCache::default();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_lazy_cache_get_or_compute_new_key() {
    let cache = LazyCache::new();
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);

    let result = cache
        .get_or_compute("test_key", || {
            *counter_clone.borrow_mut() += 1;
            Ok(Value::String(Rc::new("computed".to_string())))
        })
        .unwrap();

    assert_eq!(result, Value::String(Rc::new("computed".to_string())));
    assert_eq!(*counter.borrow(), 1);
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_lazy_cache_get_or_compute_existing_key() {
    let cache = LazyCache::new();
    let counter = Rc::new(RefCell::new(0));

    // First computation
    let counter_clone = Rc::clone(&counter);
    cache
        .get_or_compute("key", || {
            *counter_clone.borrow_mut() += 1;
            Ok(Value::Integer(42))
        })
        .unwrap();

    // Second call should use cache
    let counter_clone = Rc::clone(&counter);
    let result = cache
        .get_or_compute("key", || {
            *counter_clone.borrow_mut() += 1;
            Ok(Value::Integer(999)) // Different value, shouldn't be called
        })
        .unwrap();

    assert_eq!(result, Value::Integer(42)); // Original cached value
    assert_eq!(*counter.borrow(), 1); // Counter only incremented once
}

#[test]
fn test_lazy_cache_get_or_compute_error() {
    let cache = LazyCache::new();

    let result = cache.get_or_compute("error_key", || Err(anyhow::anyhow!("computation failed")));

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("computation failed"));
    assert_eq!(cache.size(), 0); // Error shouldn't be cached
}

#[test]
fn test_lazy_cache_multiple_keys() {
    let cache = LazyCache::new();

    cache
        .get_or_compute("key1", || Ok(Value::Integer(1)))
        .unwrap();
    cache
        .get_or_compute("key2", || Ok(Value::Integer(2)))
        .unwrap();
    cache
        .get_or_compute("key3", || Ok(Value::Integer(3)))
        .unwrap();

    assert_eq!(cache.size(), 3);

    let result1 = cache
        .get_or_compute("key1", || Ok(Value::Integer(999)))
        .unwrap();
    let result2 = cache
        .get_or_compute("key2", || Ok(Value::Integer(999)))
        .unwrap();
    let result3 = cache
        .get_or_compute("key3", || Ok(Value::Integer(999)))
        .unwrap();

    assert_eq!(result1, Value::Integer(1));
    assert_eq!(result2, Value::Integer(2));
    assert_eq!(result3, Value::Integer(3));
}

#[test]
fn test_lazy_cache_clear() {
    let cache = LazyCache::new();

    cache
        .get_or_compute("key1", || Ok(Value::Integer(1)))
        .unwrap();
    cache
        .get_or_compute("key2", || Ok(Value::Integer(2)))
        .unwrap();
    assert_eq!(cache.size(), 2);

    cache.clear();
    assert_eq!(cache.size(), 0);

    // After clear, computation should run again
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    cache
        .get_or_compute("key1", || {
            *counter_clone.borrow_mut() += 1;
            Ok(Value::Integer(999))
        })
        .unwrap();

    assert_eq!(*counter.borrow(), 1); // Computation ran
}

#[test]
fn test_lazy_cache_different_value_types() {
    let cache = LazyCache::new();

    cache
        .get_or_compute("int", || Ok(Value::Integer(42)))
        .unwrap();
    cache
        .get_or_compute("string", || Ok(Value::String(Rc::new("test".to_string()))))
        .unwrap();
    cache
        .get_or_compute("bool", || Ok(Value::Bool(true)))
        .unwrap();
    cache
        .get_or_compute("float", || Ok(Value::Float(3.14)))
        .unwrap();

    assert_eq!(cache.size(), 4);

    assert_eq!(
        cache
            .get_or_compute("int", || Ok(Value::Integer(0)))
            .unwrap(),
        Value::Integer(42)
    );
    assert_eq!(
        cache
            .get_or_compute("string", || Ok(Value::String(Rc::new("".to_string()))))
            .unwrap(),
        Value::String(Rc::new("test".to_string()))
    );
    assert_eq!(
        cache
            .get_or_compute("bool", || Ok(Value::Bool(false)))
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        cache
            .get_or_compute("float", || Ok(Value::Float(0.0)))
            .unwrap(),
        Value::Float(3.14)
    );
}

#[test]
fn test_complex_lazy_evaluation_workflow() {
    // Test combining LazyValue, LazyIterator, and LazyCache
    let cache = LazyCache::new();

    // Create some lazy values
    let lazy1 = LazyValue::deferred(|| Ok(Value::Integer(10)));
    let lazy2 = LazyValue::pipeline(LazyValue::computed(Value::Integer(5)), |v| {
        if let Value::Integer(n) = v {
            Ok(Value::Integer(n * 2))
        } else {
            Ok(v)
        }
    });

    // Force evaluation and cache results
    let result1 = lazy1.force().unwrap();
    let result2 = lazy2.force().unwrap();

    cache.get_or_compute("result1", || Ok(result1)).unwrap();
    cache.get_or_compute("result2", || Ok(result2)).unwrap();

    // Create iterator from cached values
    let values = vec![
        cache
            .get_or_compute("result1", || Ok(Value::Integer(0)))
            .unwrap(),
        cache
            .get_or_compute("result2", || Ok(Value::Integer(0)))
            .unwrap(),
    ];

    let lazy_iter = LazyIterator::from_vec(values);
    let processed = lazy_iter
        .map(|v| {
            if let Value::Integer(n) = v {
                Ok(Value::Integer(n + 1))
            } else {
                Ok(v)
            }
        })
        .collect()
        .unwrap();

    assert_eq!(processed, vec![Value::Integer(11), Value::Integer(11)]); // 10+1, 10+1
}
