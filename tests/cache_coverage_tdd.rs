//! TDD tests for cache.rs module
//! Target: Improve cache.rs from 81.74% to 95%+

use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use ruchy::runtime::cache::{BytecodeCache, CacheKey, CacheStats, CachedResult, ExpressionCache};
use std::rc::Rc;
use std::time::Duration;

#[test]
fn test_cache_key_new() {
    let source = "println(\"hello\")".to_string();
    let _key = CacheKey::new(source);

    // Can't access private fields, but can test creation doesn't panic
    // and the key can be used for equality comparisons
}

#[test]
fn test_cache_key_equality() {
    let source = "let x = 42".to_string();
    let key1 = CacheKey::new(source.clone());
    let key2 = CacheKey::new(source);
    let key3 = CacheKey::new("let y = 42".to_string());

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
    assert_ne!(key2, key3);
}

#[test]
fn test_cache_key_clone() {
    let original = CacheKey::new("clone_test".to_string());
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

#[test]
fn test_cached_result_fields() {
    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    });

    let result = CachedResult {
        ast: expr.clone(),
        rust_code: Some("42".to_string()),
        timestamp: std::time::Instant::now(),
    };

    assert!(Rc::ptr_eq(&result.ast, &expr));
    assert_eq!(result.rust_code, Some("42".to_string()));
}

#[test]
fn test_cached_result_clone() {
    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Float(3.14)),
        span: Span::default(),
        attributes: vec![],
    });

    let original = CachedResult {
        ast: expr,
        rust_code: None,
        timestamp: std::time::Instant::now(),
    };

    let cloned = original.clone();

    assert!(Rc::ptr_eq(&original.ast, &cloned.ast));
    assert_eq!(original.rust_code, cloned.rust_code);
}

#[test]
fn test_bytecode_cache_new() {
    let _cache = BytecodeCache::new();
    // Just verify it can be created without panicking
}

#[test]
fn test_bytecode_cache_with_capacity() {
    let _cache = BytecodeCache::with_capacity(500);
    // Just verify it can be created with custom capacity
}

#[test]
fn test_bytecode_cache_get_empty() {
    let cache = BytecodeCache::new();
    let result = cache.get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_bytecode_cache_insert_and_get() {
    let cache = BytecodeCache::new();
    let source = "42".to_string();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    });

    cache.insert(source.clone(), expr.clone(), Some("42".to_string()));

    let result = cache.get(&source);
    assert!(result.is_some());

    let result = result.unwrap();
    assert!(Rc::ptr_eq(&result.ast, &expr));
    assert_eq!(result.rust_code, Some("42".to_string()));
}

#[test]
fn test_bytecode_cache_clear() {
    let cache = BytecodeCache::new();
    let source = "test".to_string();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::String("test".to_string())),
        span: Span::default(),
        attributes: vec![],
    });

    cache.insert(source.clone(), expr, None);

    // Check it's cached
    assert!(cache.get(&source).is_some());

    // Clear cache
    cache.clear();

    // Check it's gone
    assert!(cache.get(&source).is_none());
}

#[test]
fn test_bytecode_cache_stats() {
    let cache = BytecodeCache::new();

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);

    // Try to get something (should miss)
    cache.get("nonexistent");

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.size, 0);

    // Insert and get something (should hit)
    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    });

    cache.insert("test".to_string(), expr, None);
    cache.get("test");

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.size, 1);
}

#[test]
fn test_bytecode_cache_evict_older_than() {
    let cache = BytecodeCache::new();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(123)),
        span: Span::default(),
        attributes: vec![],
    });

    cache.insert("old_entry".to_string(), expr, None);

    // Check it's there
    assert!(cache.get("old_entry").is_some());

    // Evict entries older than 0 duration (should evict everything)
    cache.evict_older_than(Duration::from_secs(0));

    // Check it's gone
    assert!(cache.get("old_entry").is_none());
}

#[test]
fn test_bytecode_cache_lru_eviction() {
    let cache = BytecodeCache::with_capacity(2);

    let expr1 = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(1)),
        span: Span::default(),
        attributes: vec![],
    });

    let expr2 = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(2)),
        span: Span::default(),
        attributes: vec![],
    });

    let expr3 = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(3)),
        span: Span::default(),
        attributes: vec![],
    });

    cache.insert("one".to_string(), expr1, None);
    cache.insert("two".to_string(), expr2, None);

    // Both should be cached
    assert!(cache.get("one").is_some());
    assert!(cache.get("two").is_some());

    // Insert third item (should evict oldest)
    cache.insert("three".to_string(), expr3, None);

    // Third should be cached
    assert!(cache.get("three").is_some());

    // Due to LRU, one of the first two should have been evicted
    let stats = cache.stats();
    assert_eq!(stats.size, 2);
}

#[test]
fn test_expression_cache_new() {
    let _cache = ExpressionCache::new();
    // Just verify it can be created
}

#[test]
fn test_expression_cache_get_parsed() {
    let cache = ExpressionCache::new();

    let result = cache.get_parsed("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_expression_cache_cache_parsed() {
    let cache = ExpressionCache::new();
    let source = "parsed_test".to_string();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Char('x')),
        span: Span::default(),
        attributes: vec![],
    });

    cache.cache_parsed(source.clone(), expr.clone());

    let result = cache.get_parsed(&source);
    assert!(result.is_some());
    assert!(Rc::ptr_eq(&result.unwrap(), &expr));
}

#[test]
fn test_expression_cache_get_transpiled() {
    let cache = ExpressionCache::new();

    let result = cache.get_transpiled("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_expression_cache_cache_transpiled() {
    let cache = ExpressionCache::new();
    let source = "transpiled_test".to_string();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
    });

    let rust_code = "()".to_string();

    cache.cache_transpiled(source.clone(), expr.clone(), rust_code.clone());

    let result = cache.get_transpiled(&source);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), rust_code);

    // Should also be able to get the parsed version
    let parsed = cache.get_parsed(&source);
    assert!(parsed.is_some());
    assert!(Rc::ptr_eq(&parsed.unwrap(), &expr));
}

#[test]
fn test_expression_cache_stats() {
    let cache = ExpressionCache::new();

    let stats = cache.stats();
    assert_eq!(stats.size, 0);
}

#[test]
fn test_expression_cache_clear() {
    let cache = ExpressionCache::new();

    let expr = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(999)),
        span: Span::default(),
        attributes: vec![],
    });

    cache.cache_parsed("clear_test".to_string(), expr);

    // Check it's cached
    assert!(cache.get_parsed("clear_test").is_some());

    cache.clear();

    // Check it's gone
    assert!(cache.get_parsed("clear_test").is_none());
}

#[test]
fn test_cache_stats_fields() {
    let stats = CacheStats {
        size: 7,
        capacity: 100,
        hits: 10,
        misses: 5,
        hit_rate: 66.7,
    };

    assert_eq!(stats.hits, 10);
    assert_eq!(stats.misses, 5);
    assert_eq!(stats.size, 7);
    assert_eq!(stats.hit_rate, 66.7);
}

#[test]
fn test_cache_with_different_expressions() {
    let cache = BytecodeCache::new();

    let expressions = vec![
        ("int", ExprKind::Literal(Literal::Integer(42))),
        ("float", ExprKind::Literal(Literal::Float(3.14))),
        (
            "string",
            ExprKind::Literal(Literal::String("test".to_string())),
        ),
        ("bool", ExprKind::Literal(Literal::Bool(false))),
        ("char", ExprKind::Literal(Literal::Char('z'))),
        ("unit", ExprKind::Literal(Literal::Unit)),
    ];

    for (name, kind) in expressions {
        let expr = Rc::new(Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
        });

        cache.insert(name.to_string(), expr.clone(), None);

        let result = cache.get(name);
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap().ast, &expr));
    }

    let stats = cache.stats();
    assert_eq!(stats.size, 6);
    assert_eq!(stats.hits, 6);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_expression_cache_overwrite() {
    let cache = ExpressionCache::new();
    let source = "overwrite".to_string();

    // First version
    let expr1 = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(1)),
        span: Span::default(),
        attributes: vec![],
    });
    cache.cache_parsed(source.clone(), expr1.clone());

    let result1 = cache.get_parsed(&source).unwrap();
    assert!(Rc::ptr_eq(&result1, &expr1));

    // Second version (should overwrite)
    let expr2 = Rc::new(Expr {
        kind: ExprKind::Literal(Literal::Integer(2)),
        span: Span::default(),
        attributes: vec![],
    });
    cache.cache_parsed(source.clone(), expr2.clone());

    let result2 = cache.get_parsed(&source).unwrap();
    assert!(Rc::ptr_eq(&result2, &expr2));
    assert!(!Rc::ptr_eq(&result2, &expr1));
}
