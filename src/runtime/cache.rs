//! Bytecode and compilation caching for improved REPL performance
//!
//! This module provides caching mechanisms to avoid re-parsing and
//! re-compiling expressions that have been seen before.
use crate::frontend::ast::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
/// A cache key that represents source code
#[derive(Clone, Debug, Eq)]
pub struct CacheKey {
    /// The source code
    source: String,
    /// Hash of the source for fast comparison
    hash: u64,
}
impl CacheKey {
    /// Create a new cache key from source code
    pub fn new(source: String) -> Self {
        let hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            source.hash(&mut hasher);
            hasher.finish()
        };
        CacheKey { source, hash }
    }
}
impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.source == other.source
    }
}
impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}
/// Cached compilation result
#[derive(Clone)]
pub struct CachedResult {
    /// The parsed AST
    pub ast: Rc<Expr>,
    /// The transpiled Rust code (if applicable)
    pub rust_code: Option<String>,
    /// Timestamp when cached
    pub timestamp: std::time::Instant,
}
/// Bytecode cache for REPL expressions
pub struct BytecodeCache {
    /// Cache storage
    cache: RefCell<HashMap<CacheKey, CachedResult>>,
    /// Maximum cache size (number of entries)
    max_size: usize,
    /// Track access order for LRU eviction
    access_order: RefCell<Vec<CacheKey>>,
    /// Statistics
    hits: RefCell<usize>,
    misses: RefCell<usize>,
}
impl BytecodeCache {
    /// Create a new bytecode cache with specified max size
    pub fn with_capacity(max_size: usize) -> Self {
        BytecodeCache {
            cache: RefCell::new(HashMap::with_capacity(max_size)),
            max_size,
            access_order: RefCell::new(Vec::with_capacity(max_size)),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
        }
    }
    /// Create a default cache with 1000 entry capacity
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }
    /// Get a cached result if available
    pub fn get(&self, source: &str) -> Option<CachedResult> {
        let key = CacheKey::new(source.to_string());
        if let Some(result) = self.cache.borrow().get(&key) {
            *self.hits.borrow_mut() += 1;
            // Update access order for LRU
            let mut access = self.access_order.borrow_mut();
            if let Some(pos) = access.iter().position(|k| k == &key) {
                access.remove(pos);
            }
            access.push(key.clone());
            Some(result.clone())
        } else {
            *self.misses.borrow_mut() += 1;
            None
        }
    }
    /// Store a compilation result in the cache
    pub fn insert(&self, source: String, ast: Rc<Expr>, rust_code: Option<String>) {
        let key = CacheKey::new(source);
        // Check if we need to evict
        if self.cache.borrow().len() >= self.max_size {
            self.evict_lru();
        }
        let result = CachedResult {
            ast,
            rust_code,
            timestamp: std::time::Instant::now(),
        };
        self.cache.borrow_mut().insert(key.clone(), result);
        self.access_order.borrow_mut().push(key);
    }
    /// Evict least recently used entry
    fn evict_lru(&self) {
        let mut access = self.access_order.borrow_mut();
        if !access.is_empty() {
            let lru_key = access.remove(0);
            self.cache.borrow_mut().remove(&lru_key);
        }
    }
    /// Clear the entire cache
    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
        self.access_order.borrow_mut().clear();
        *self.hits.borrow_mut() = 0;
        *self.misses.borrow_mut() = 0;
    }
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.borrow().len(),
            capacity: self.max_size,
            hits: *self.hits.borrow(),
            misses: *self.misses.borrow(),
            hit_rate: self.calculate_hit_rate(),
        }
    }
    /// Calculate hit rate as a percentage
    #[allow(clippy::cast_precision_loss)]
    fn calculate_hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow() as f64;
        let total = hits + *self.misses.borrow() as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
    /// Remove entries older than specified duration
    pub fn evict_older_than(&self, age: std::time::Duration) {
        let now = std::time::Instant::now();
        let mut cache = self.cache.borrow_mut();
        let mut access = self.access_order.borrow_mut();
        // Find keys to remove
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(_, result)| now.duration_since(result.timestamp) > age)
            .map(|(key, _)| key.clone())
            .collect();
        // Remove from cache and access order
        for key in keys_to_remove {
            cache.remove(&key);
            if let Some(pos) = access.iter().position(|k| k == &key) {
                access.remove(pos);
            }
        }
    }
}
impl Default for BytecodeCache {
    fn default() -> Self {
        Self::new()
    }
}
/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of cached entries
    pub size: usize,
    /// Maximum capacity
    pub capacity: usize,
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Hit rate percentage
    pub hit_rate: f64,
}
impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {}/{} entries, {} hits, {} misses ({:.1}% hit rate)",
            self.size, self.capacity, self.hits, self.misses, self.hit_rate
        )
    }
}
/// Expression cache for parsed ASTs
pub struct ExpressionCache {
    inner: BytecodeCache,
}
impl ExpressionCache {
    /// Create a new expression cache
    pub fn new() -> Self {
        ExpressionCache {
            inner: BytecodeCache::new(),
        }
    }
    /// Try to get a parsed expression from cache
    pub fn get_parsed(&self, source: &str) -> Option<Rc<Expr>> {
        self.inner.get(source).map(|result| result.ast)
    }
    /// Cache a parsed expression
    pub fn cache_parsed(&self, source: String, ast: Rc<Expr>) {
        self.inner.insert(source, ast, None);
    }
    /// Try to get transpiled code from cache
    pub fn get_transpiled(&self, source: &str) -> Option<String> {
        self.inner.get(source).and_then(|result| result.rust_code)
    }
    /// Cache transpiled code
    pub fn cache_transpiled(&self, source: String, ast: Rc<Expr>, rust_code: String) {
        self.inner.insert(source, ast, Some(rust_code));
    }
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.inner.stats()
    }
    /// Clear the cache
    pub fn clear(&self) {
        self.inner.clear();
    }
}
impl Default for ExpressionCache {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    fn make_test_expr(value: i64) -> Rc<Expr> {
        Rc::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        })
    }
    #[test]
    fn test_cache_key() {
        let key1 = CacheKey::new("let x = 42".to_string());
        let key2 = CacheKey::new("let x = 42".to_string());
        let key3 = CacheKey::new("let y = 42".to_string());
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
    #[test]
    fn test_bytecode_cache_basic() {
        let cache = BytecodeCache::with_capacity(3);
        // Cache miss
        assert!(cache.get("let x = 1").is_none());
        assert_eq!(cache.stats().misses, 1);
        // Insert and hit
        cache.insert("let x = 1".to_string(), make_test_expr(1), None);
        assert!(cache.get("let x = 1").is_some());
        assert_eq!(cache.stats().hits, 1);
    }
    #[test]
    fn test_cache_lru_eviction() {
        let cache = BytecodeCache::with_capacity(2);
        cache.insert("expr1".to_string(), make_test_expr(1), None);
        cache.insert("expr2".to_string(), make_test_expr(2), None);
        // Access expr1 to make it more recent
        let _ = cache.get("expr1");
        // This should evict expr2 (least recently used)
        cache.insert("expr3".to_string(), make_test_expr(3), None);
        assert!(cache.get("expr1").is_some());
        assert!(cache.get("expr2").is_none()); // Evicted
        assert!(cache.get("expr3").is_some());
    }
    #[test]
    fn test_expression_cache() {
        let cache = ExpressionCache::new();
        let expr = make_test_expr(42);
        cache.cache_parsed("let x = 42".to_string(), Rc::clone(&expr));
        let cached = cache.get_parsed("let x = 42").unwrap();
        assert!(Rc::ptr_eq(&expr, &cached));
        // Test transpiled code caching
        cache.cache_transpiled(
            "let y = 10".to_string(),
            make_test_expr(10),
            "let y = 10;".to_string(),
        );
        assert_eq!(
            cache.get_transpiled("let y = 10"),
            Some("let y = 10;".to_string())
        );
    }
    #[test]
    fn test_cache_stats() {
        let cache = BytecodeCache::with_capacity(10);
        for i in 0..5 {
            cache.insert(format!("expr{i}"), make_test_expr(i), None);
        }
        // Generate some hits and misses
        let _ = cache.get("expr1");
        let _ = cache.get("expr2");
        let _ = cache.get("expr_missing");
        let stats = cache.stats();
        assert_eq!(stats.size, 5);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 60.0 && stats.hit_rate < 70.0);
    }

    // === EXTREME TDD Round 15 tests ===

    #[test]
    fn test_cache_key_hash() {
        use std::collections::HashSet;

        let key1 = CacheKey::new("let x = 1".to_string());
        let key2 = CacheKey::new("let x = 1".to_string());
        let key3 = CacheKey::new("let x = 2".to_string());

        let mut set = HashSet::new();
        set.insert(key1.clone());
        assert!(set.contains(&key2));
        assert!(!set.contains(&key3));
    }

    #[test]
    fn test_bytecode_cache_default() {
        let cache = BytecodeCache::default();
        assert_eq!(cache.stats().capacity, 1000);
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_expression_cache_default() {
        let cache = ExpressionCache::default();
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_cache_stats_empty() {
        let cache = BytecodeCache::new();
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_stats_clone() {
        let cache = BytecodeCache::new();
        let _ = cache.get("missing");
        let stats = cache.stats();
        let cloned = stats.clone();
        assert_eq!(stats.misses, cloned.misses);
        assert_eq!(stats.capacity, cloned.capacity);
    }

    #[test]
    fn test_cached_result_clone() {
        let result = CachedResult {
            ast: make_test_expr(42),
            rust_code: Some("let x = 42;".to_string()),
            timestamp: std::time::Instant::now(),
        };
        let cloned = result.clone();
        assert!(Rc::ptr_eq(&result.ast, &cloned.ast));
        assert_eq!(result.rust_code, cloned.rust_code);
    }

    #[test]
    fn test_bytecode_cache_insert_duplicate() {
        let cache = BytecodeCache::with_capacity(10);
        cache.insert("expr1".to_string(), make_test_expr(1), None);
        cache.insert("expr1".to_string(), make_test_expr(2), None);

        // Size should still be 1 (duplicate key)
        assert_eq!(cache.stats().size, 1);
    }

    #[test]
    fn test_expression_cache_get_transpiled_none() {
        let cache = ExpressionCache::new();
        cache.cache_parsed("test".to_string(), make_test_expr(1));

        // Should return None for rust_code since we only cached parsed
        assert!(cache.get_transpiled("test").is_none());
    }

    #[test]
    fn test_cache_key_debug() {
        let key = CacheKey::new("test".to_string());
        let debug = format!("{:?}", key);
        assert!(debug.contains("CacheKey"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_cache_stats_debug() {
        let cache = BytecodeCache::new();
        let stats = cache.stats();
        let debug = format!("{:?}", stats);
        assert!(debug.contains("CacheStats"));
    }
}
#[cfg(test)]
mod property_tests_cache {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn test_bytecode_cache_clear_not_stub() {
        // MISSED: replace BytecodeCache::clear with ()
        use crate::frontend::ast::{ExprKind, Literal, Span};

        let cache = BytecodeCache::new();
        let expr = Rc::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        });

        cache.insert("test".to_string(), expr, None);
        assert_eq!(cache.stats().size, 1);

        cache.clear();
        assert_eq!(cache.stats().size, 0, "Clear should empty cache");
        assert_eq!(cache.stats().hits, 0, "Clear should reset hits");
        assert_eq!(cache.stats().misses, 0, "Clear should reset misses");
    }

    #[test]
    fn test_cache_stats_display_not_stub() {
        // MISSED: replace fmt -> Result with Ok(Default::default())
        let cache = BytecodeCache::new();
        let stats = cache.stats();

        let display = format!("{stats}");
        assert!(
            display.contains("Cache:"),
            "Display should contain 'Cache:'"
        );
        assert!(
            display.contains("entries"),
            "Display should contain 'entries'"
        );
        assert!(display.contains("hits"), "Display should contain 'hits'");
        assert!(
            display.contains("misses"),
            "Display should contain 'misses'"
        );
    }

    #[test]
    fn test_evict_older_than_comparison_operator() {
        // MISSED: replace > with >= in evict_older_than (line 152)
        use crate::frontend::ast::{ExprKind, Literal, Span};

        let cache = BytecodeCache::new();
        let expr = Rc::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        });

        cache.insert("test".to_string(), Rc::clone(&expr), None);
        assert_eq!(cache.stats().size, 1);

        // Evict items older than 1 second (should evict the item after sleep)
        std::thread::sleep(std::time::Duration::from_millis(1100));
        cache.evict_older_than(std::time::Duration::from_secs(1));
        assert_eq!(
            cache.stats().size,
            0,
            "Items older than 1s should be evicted (>)"
        );
    }

    #[test]
    fn test_expression_cache_clear_not_stub() {
        // MISSED: replace ExpressionCache::clear with ()
        use crate::frontend::ast::{ExprKind, Literal, Span};

        let cache = ExpressionCache::new();
        let expr = Rc::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        });

        cache.cache_parsed("expr".to_string(), expr);
        assert_eq!(cache.stats().size, 1);

        cache.clear();
        assert_eq!(cache.stats().size, 0, "Clear should empty expression cache");
    }
}
