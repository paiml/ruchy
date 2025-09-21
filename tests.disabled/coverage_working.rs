//! Working test suite demonstrating coverage improvements
//! This compiles and runs to show our testing infrastructure works

use ruchy::performance_optimizations::{CompilationCache, ParserPool, StringInterner};

#[test]
fn test_compilation_cache_basic() {
    let mut cache = CompilationCache::new(100);

    // Test empty cache
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    // Test insertion and retrieval
    cache.insert(
        "test1".to_string(),
        "output1".to_string(),
        std::time::Duration::from_millis(100),
        512,
    );

    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());

    // Test cache hit
    let result = cache.get("test1");
    assert_eq!(result, Some("output1".to_string()));

    // Test cache miss
    let miss = cache.get("nonexistent");
    assert_eq!(miss, None);

    // Test statistics
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate(), 0.5);
}

#[test]
fn test_cache_lru_eviction() {
    let mut cache = CompilationCache::new(2);

    // Fill cache to capacity
    cache.insert(
        "a".to_string(),
        "val_a".to_string(),
        std::time::Duration::from_millis(10),
        100,
    );
    cache.insert(
        "b".to_string(),
        "val_b".to_string(),
        std::time::Duration::from_millis(10),
        100,
    );

    assert_eq!(cache.len(), 2);

    // Access 'a' to make it more recently used
    let _ = cache.get("a");

    // Insert third item - should evict 'b' (least recently used)
    cache.insert(
        "c".to_string(),
        "val_c".to_string(),
        std::time::Duration::from_millis(10),
        100,
    );

    assert_eq!(cache.len(), 2);
    assert!(cache.get("a").is_some());
    assert!(cache.get("b").is_none()); // Evicted
    assert!(cache.get("c").is_some());

    let stats = cache.stats();
    assert_eq!(stats.evictions, 1);
}

#[test]
fn test_string_interner_basic() {
    let mut interner = StringInterner::new();

    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());

    // Test interning
    let id1 = interner.intern("hello");
    let id2 = interner.intern("world");
    let id3 = interner.intern("hello"); // Same string

    assert_eq!(id1, id3); // Same string gets same ID
    assert_ne!(id1, id2); // Different strings get different IDs

    // Test retrieval
    assert_eq!(interner.get(id1), Some("hello"));
    assert_eq!(interner.get(id2), Some("world"));
    assert_eq!(interner.get(9999), None); // Invalid ID

    // Test size
    assert_eq!(interner.len(), 2); // Only 2 unique strings
    assert!(!interner.is_empty());
}

#[test]
fn test_string_interner_clear() {
    let mut interner = StringInterner::new();

    interner.intern("test1");
    interner.intern("test2");
    assert_eq!(interner.len(), 2);

    interner.clear();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());

    // Can intern again after clear
    let id = interner.intern("new");
    assert_eq!(interner.get(id), Some("new"));
}

#[test]
fn test_parser_pool_basic() {
    let pool = ParserPool::new(10);

    assert_eq!(pool.capacity(), 10);
    assert_eq!(pool.size(), 0);

    // Create parser from pool
    let _parser1 = pool.create_parser("let x = 42");
    let _parser2 = pool.create_parser("let y = 100");

    let stats = pool.stats();
    assert_eq!(stats.created, 2);
    assert_eq!(stats.borrowed, 2);
}

#[test]
fn test_cache_memory_tracking() {
    let mut cache = CompilationCache::new(10);

    cache.insert(
        "small".to_string(),
        "x".to_string(),
        std::time::Duration::from_millis(5),
        100,
    );
    cache.insert(
        "medium".to_string(),
        "y".repeat(10),
        std::time::Duration::from_millis(10),
        1000,
    );
    cache.insert(
        "large".to_string(),
        "z".repeat(100),
        std::time::Duration::from_millis(15),
        10000,
    );

    let memory = cache.memory_usage();
    assert_eq!(memory, 11100); // 100 + 1000 + 10000

    // Clear and check memory
    cache.clear();
    assert_eq!(cache.memory_usage(), 0);
}

#[test]
fn test_cache_utilization() {
    let cache = CompilationCache::new(10);
    let stats = cache.stats();

    assert_eq!(stats.utilization(), 0.0); // Empty cache
    assert_eq!(stats.capacity, 10);
    assert_eq!(stats.size, 0);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_cache_consistency(
            key in "[a-z]{1,10}",
            value in "[a-z]{1,20}",
            millis in 1u64..1000,
            bytes in 1usize..10000
        ) {
            let mut cache = CompilationCache::new(5);

            cache.insert(
                key.clone(),
                value.clone(),
                std::time::Duration::from_millis(millis),
                bytes
            );

            // What goes in must come out
            let retrieved = cache.get(&key);
            prop_assert_eq!(retrieved, Some(value));
        }

        #[test]
        fn prop_interner_deduplication(s1 in ".*", s2 in ".*") {
            let mut interner = StringInterner::new();

            let id1a = interner.intern(&s1);
            let id1b = interner.intern(&s1);
            let id2a = interner.intern(&s2);
            let id2b = interner.intern(&s2);

            // Same strings always get same IDs
            prop_assert_eq!(id1a, id1b);
            prop_assert_eq!(id2a, id2b);

            // Different strings get different IDs (unless they're equal)
            if s1 == s2 {
                prop_assert_eq!(id1a, id2a);
            } else {
                prop_assert_ne!(id1a, id2a);
            }
        }

        #[test]
        fn prop_cache_capacity_respected(capacity in 1usize..20) {
            let mut cache = CompilationCache::new(capacity);

            // Insert more than capacity
            for i in 0..capacity + 5 {
                cache.insert(
                    format!("key{i}"),
                    format!("val{i}"),
                    std::time::Duration::from_millis(10),
                    100
                );
            }

            // Size should never exceed capacity
            prop_assert!(cache.len() <= capacity);
        }

        #[test]
        fn prop_interner_retrieval(strings in prop::collection::vec(".*", 1..20)) {
            let mut interner = StringInterner::new();
            let mut ids = Vec::new();

            // Intern all strings
            for s in &strings {
                ids.push(interner.intern(s));
            }

            // All strings should be retrievable
            for (i, id) in ids.iter().enumerate() {
                prop_assert_eq!(interner.get(*id), Some(strings[i].as_str()));
            }
        }
    }
}
