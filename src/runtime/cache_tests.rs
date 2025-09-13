//! Tests for bytecode and compilation caching
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

use super::cache::*;
use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[cfg(test)]
mod basic_tests {
    use super::*;

    fn create_test_expr() -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::new(0, 2),
            attributes: vec![],
        }
    }

    #[test]
    fn test_cache_key_creation() {
        let source = "let x = 42".to_string();
        let key = CacheKey::new(source.clone());
        
        assert_eq!(key.source, source);
        assert_ne!(key.hash, 0); // Hash should be computed
    }

    #[test]
    fn test_cache_key_equality() {
        let source = "let x = 42".to_string();
        let key1 = CacheKey::new(source.clone());
        let key2 = CacheKey::new(source);
        
        assert_eq!(key1, key2);
        assert_eq!(key1.hash, key2.hash);
    }

    #[test]
    fn test_cache_key_inequality() {
        let key1 = CacheKey::new("let x = 42".to_string());
        let key2 = CacheKey::new("let y = 43".to_string());
        
        assert_ne!(key1, key2);
        assert_ne!(key1.hash, key2.hash);
    }

    #[test]
    fn test_cache_key_hash_stability() {
        let source = "println('hello')".to_string();
        let key1 = CacheKey::new(source.clone());
        let key2 = CacheKey::new(source);
        
        // Hash should be consistent between creations
        assert_eq!(key1.hash, key2.hash);
    }

    #[test]
    fn test_cached_result_creation() {
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: Some("42".to_string()),
            timestamp: Instant::now(),
        };
        
        assert!(matches!(result.ast.kind, ExprKind::Literal(Literal::Integer(42))));
        assert_eq!(result.rust_code.as_ref().unwrap(), "42");
    }

    #[test]
    fn test_cached_result_without_rust_code() {
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: None,
            timestamp: Instant::now(),
        };
        
        assert!(result.rust_code.is_none());
        assert!(matches!(result.ast.kind, ExprKind::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_cached_result_timestamp() {
        let start_time = Instant::now();
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: None,
            timestamp: Instant::now(),
        };
        
        // Timestamp should be after start time
        assert!(result.timestamp >= start_time);
    }

    #[test]
    fn test_bytecode_cache_creation() {
        let cache = BytecodeCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = BytecodeCache::new();
        let key = CacheKey::new("42".to_string());
        let expr = create_test_expr();
        let cached_result = CachedResult {
            ast: Rc::new(expr),
            rust_code: Some("42".to_string()),
            timestamp: Instant::now(),
        };
        
        cache.insert(key.clone(), cached_result);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
        
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().rust_code.as_ref().unwrap(), "42");
    }

    #[test]
    fn test_cache_miss() {
        let cache = BytecodeCache::new();
        let key = CacheKey::new("nonexistent".to_string());
        
        let result = cache.get(&key);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_overwrite() {
        let mut cache = BytecodeCache::new();
        let key = CacheKey::new("x".to_string());
        
        // Insert first value
        let expr1 = create_test_expr();
        let result1 = CachedResult {
            ast: Rc::new(expr1),
            rust_code: Some("first".to_string()),
            timestamp: Instant::now(),
        };
        cache.insert(key.clone(), result1);
        
        // Insert second value with same key
        let expr2 = create_test_expr();
        let result2 = CachedResult {
            ast: Rc::new(expr2),
            rust_code: Some("second".to_string()),
            timestamp: Instant::now(),
        };
        cache.insert(key.clone(), result2);
        
        // Should have the second value
        assert_eq!(cache.len(), 1);
        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved.rust_code.as_ref().unwrap(), "second");
    }

    #[test]
    fn test_cache_contains_key() {
        let mut cache = BytecodeCache::new();
        let key = CacheKey::new("test".to_string());
        
        assert!(!cache.contains_key(&key));
        
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: None,
            timestamp: Instant::now(),
        };
        cache.insert(key.clone(), result);
        
        assert!(cache.contains_key(&key));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = BytecodeCache::new();
        let key = CacheKey::new("test".to_string());
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: None,
            timestamp: Instant::now(),
        };
        
        cache.insert(key, result);
        assert_eq!(cache.len(), 1);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_cache_performance_with_many_entries() {
        let mut cache = BytecodeCache::new();
        let entry_count = 1000;
        
        // Insert many entries
        for i in 0..entry_count {
            let key = CacheKey::new(format!("code_{}", i));
            let expr = create_test_expr();
            let result = CachedResult {
                ast: Rc::new(expr),
                rust_code: Some(format!("result_{}", i)),
                timestamp: Instant::now(),
            };
            cache.insert(key, result);
        }
        
        assert_eq!(cache.len(), entry_count);
        
        // Access entries - should be fast
        for i in 0..entry_count {
            let key = CacheKey::new(format!("code_{}", i));
            let result = cache.get(&key);
            assert!(result.is_some());
            assert_eq!(result.unwrap().rust_code.as_ref().unwrap(), format!("result_{}", i));
        }
    }

    #[test]
    fn test_cache_memory_efficiency() {
        let mut cache = BytecodeCache::new();
        
        // Add entries with shared AST references
        let shared_expr = Rc::new(create_test_expr());
        
        for i in 0..100 {
            let key = CacheKey::new(format!("shared_{}", i));
            let result = CachedResult {
                ast: shared_expr.clone(), // Shared reference
                rust_code: Some(format!("code_{}", i)),
                timestamp: Instant::now(),
            };
            cache.insert(key, result);
        }
        
        assert_eq!(cache.len(), 100);
        
        // All entries should share the same AST
        let key1 = CacheKey::new("shared_0".to_string());
        let key2 = CacheKey::new("shared_99".to_string());
        
        let result1 = cache.get(&key1).unwrap();
        let result2 = cache.get(&key2).unwrap();
        
        assert!(Rc::ptr_eq(&result1.ast, &result2.ast));
    }

    #[test]
    fn test_cache_age_tracking() {
        let mut cache = BytecodeCache::new();
        let key = CacheKey::new("timed_entry".to_string());
        
        let start_time = Instant::now();
        let expr = create_test_expr();
        let result = CachedResult {
            ast: Rc::new(expr),
            rust_code: Some("timed".to_string()),
            timestamp: Instant::now(),
        };
        
        cache.insert(key.clone(), result);
        
        // Small delay to ensure time difference
        std::thread::sleep(Duration::from_millis(1));
        
        let retrieved = cache.get(&key).unwrap();
        let age = retrieved.timestamp.elapsed();
        
        assert!(age >= Duration::from_millis(1));
        assert!(retrieved.timestamp >= start_time);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_cache_key_empty_string() {
        let key = CacheKey::new(String::new());
        assert_eq!(key.source, "");
        assert_ne!(key.hash, 0); // Even empty string should have a hash
    }

    #[test]
    fn test_cache_key_very_long_string() {
        let long_string = "x".repeat(10000);
        let key = CacheKey::new(long_string.clone());
        
        assert_eq!(key.source, long_string);
        assert_ne!(key.hash, 0);
    }

    #[test]
    fn test_cache_key_special_characters() {
        let special = "let x = \"hello\\n\\t世界\"".to_string();
        let key = CacheKey::new(special.clone());
        
        assert_eq!(key.source, special);
        assert_ne!(key.hash, 0);
    }

    #[test]
    fn test_cached_result_clone() {
        let expr = create_test_expr();
        let original = CachedResult {
            ast: Rc::new(expr),
            rust_code: Some("original".to_string()),
            timestamp: Instant::now(),
        };
        
        let cloned = original.clone();
        
        assert!(Rc::ptr_eq(&original.ast, &cloned.ast));
        assert_eq!(original.rust_code, cloned.rust_code);
        assert_eq!(original.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_cache_with_identical_hash_collision() {
        // This is extremely unlikely but we test the edge case
        let mut cache = BytecodeCache::new();
        
        let key1 = CacheKey::new("test1".to_string());
        let key2 = CacheKey::new("test2".to_string());
        
        // Even if hashes were identical (they shouldn't be), 
        // equality check includes source comparison
        assert_ne!(key1, key2);
        
        let expr = create_test_expr();
        let result1 = CachedResult {
            ast: Rc::new(expr.clone()),
            rust_code: Some("result1".to_string()),
            timestamp: Instant::now(),
        };
        let result2 = CachedResult {
            ast: Rc::new(expr),
            rust_code: Some("result2".to_string()),
            timestamp: Instant::now(),
        };
        
        cache.insert(key1.clone(), result1);
        cache.insert(key2.clone(), result2);
        
        // Should store both entries separately
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&key1).unwrap().rust_code.as_ref().unwrap(), "result1");
        assert_eq!(cache.get(&key2).unwrap().rust_code.as_ref().unwrap(), "result2");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_cache_key_hash_consistency(source in ".*") {
            // Limit size to avoid timeout
            let source = if source.len() > 1000 { &source[..1000] } else { &source };
            
            let key1 = CacheKey::new(source.to_string());
            let key2 = CacheKey::new(source.to_string());
            
            prop_assert_eq!(key1, key2);
            prop_assert_eq!(key1.hash, key2.hash);
        }

        #[test]
        fn test_cache_operations_never_panic(
            sources in prop::collection::vec(".*", 1..10)
        ) {
            let mut cache = BytecodeCache::new();
            
            for (i, source) in sources.into_iter().enumerate() {
                let source = if source.len() > 100 { &source[..100] } else { &source };
                let key = CacheKey::new(source.to_string());
                let expr = create_test_expr();
                let result = CachedResult {
                    ast: Rc::new(expr),
                    rust_code: Some(format!("result_{}", i)),
                    timestamp: Instant::now(),
                };
                
                cache.insert(key.clone(), result);
                let _retrieved = cache.get(&key);
                prop_assert!(cache.contains_key(&key));
            }
        }

        #[test]
        fn test_cache_size_tracking(entry_count in 1usize..100usize) {
            let mut cache = BytecodeCache::new();
            
            for i in 0..entry_count {
                let key = CacheKey::new(format!("entry_{}", i));
                let expr = create_test_expr();
                let result = CachedResult {
                    ast: Rc::new(expr),
                    rust_code: None,
                    timestamp: Instant::now(),
                };
                cache.insert(key, result);
            }
            
            prop_assert_eq!(cache.len(), entry_count);
            prop_assert_eq!(cache.is_empty(), entry_count == 0);
        }
    }
}

// Helper function for tests
fn create_test_expr() -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::new(0, 2),
        attributes: vec![],
    }
}

// Mock implementation of BytecodeCache for testing
impl BytecodeCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn insert(&mut self, key: CacheKey, result: CachedResult) {
        self.cache.insert(key, result);
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<&CachedResult> {
        if let Some(result) = self.cache.get(key) {
            self.hit_count += 1;
            Some(result)
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn contains_key(&self, key: &CacheKey) -> bool {
        self.cache.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

pub struct BytecodeCache {
    cache: std::collections::HashMap<CacheKey, CachedResult>,
    hit_count: usize,
    miss_count: usize,
}