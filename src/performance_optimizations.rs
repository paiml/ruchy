//! Performance optimizations for the Ruchy compiler
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// AST-based compilation cache with LRU eviction
pub struct CompilationCache {
    /// Cache entries with access tracking
    cache: HashMap<String, CacheEntry>,
    /// Maximum cache size in entries
    max_size: usize,
    /// Cache statistics
    hits: u64,
    misses: u64,
    evictions: u64,
}

#[derive(Clone)]
struct CacheEntry {
    /// Compiled output
    pub output: String,
    /// Last access time for LRU tracking
    pub last_access: Instant,
    /// Compilation time
    pub compile_time: Duration,
    /// Memory usage
    pub memory_bytes: usize,
}

impl CompilationCache {
    /// Create new compilation cache with specified capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::performance_optimizations::CompilationCache;
    ///
    /// let cache = CompilationCache::new(1000);
    /// assert_eq!(cache.len(), 0);
    /// assert_eq!(cache.capacity(), 1000);
    /// ```
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Get cached compilation result
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::performance_optimizations::CompilationCache;
    ///
    /// let mut cache = CompilationCache::new(100);
    /// cache.insert("test".to_string(), "output".to_string(), std::time::Duration::from_millis(100), 512);
    ///
    /// let result = cache.get("test");
    /// assert!(result.is_some());
    /// assert_eq!(result.unwrap(), "output");
    /// ```
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.cache.get_mut(key) {
            entry.last_access = Instant::now();
            self.hits += 1;
            Some(entry.output.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Insert compilation result into cache
    pub fn insert(
        &mut self,
        key: String,
        output: String,
        compile_time: Duration,
        memory_bytes: usize,
    ) {
        // Evict least recently used entries if at capacity
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let entry = CacheEntry {
            output,
            last_access: Instant::now(),
            compile_time,
            memory_bytes,
        };

        self.cache.insert(key, entry);
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find LRU entry
        let (lru_key, _) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, v)| (k.clone(), v.clone()))
            .expect("Cache not empty");

        self.cache.remove(&lru_key);
        self.evictions += 1;
    }

    /// Get cache statistics
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::performance_optimizations::CompilationCache;
    ///
    /// let mut cache = CompilationCache::new(100);
    /// cache.insert("test".to_string(), "output".to_string(), std::time::Duration::from_millis(100), 512);
    /// cache.get("test");
    /// cache.get("missing");
    ///
    /// let stats = cache.stats();
    /// assert_eq!(stats.total_requests(), 2);
    /// assert_eq!(stats.hit_rate(), 0.5);
    /// ```
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            size: self.cache.len(),
            capacity: self.max_size,
        }
    }

    /// Current number of cached entries
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Whether cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Maximum cache capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get memory usage of cached entries
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.cache.values().map(|entry| entry.memory_bytes).sum()
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    /// Total cache requests
    #[must_use]
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Cache hit rate (0.0 to 1.0)
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Cache utilization (0.0 to 1.0)
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.size as f64 / self.capacity as f64
        }
    }
}

/// Thread-safe parser pool for reuse
///
/// Note: This is a simplified version that focuses on cache management.
/// For a full implementation, consider parser state management.
pub struct ParserPool {
    /// Parser cache size
    max_size: usize,
    /// Pool statistics
    stats: Arc<Mutex<PoolStats>>,
}

#[derive(Debug, Default)]
struct PoolStats {
    /// Total parsers created
    created: u64,
    /// Total parsers borrowed
    borrowed: u64,
    /// Total parsers returned
    returned: u64,
    /// Current pool size
    current_size: usize,
}

impl ParserPool {
    /// Create new parser pool
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::performance_optimizations::ParserPool;
    ///
    /// let pool = ParserPool::new(10);
    /// assert_eq!(pool.capacity(), 10);
    /// ```
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            stats: Arc::new(Mutex::new(PoolStats::default())),
            max_size,
        }
    }

    /// Create a new parser (simplified implementation)
    ///
    /// In a full implementation, this would manage a pool of reusable parsers
    pub fn create_parser<'a>(&self, input: &'a str) -> crate::frontend::Parser<'a> {
        let mut stats = self.stats.lock().expect("Lock poisoned");
        stats.created += 1;
        stats.borrowed += 1;
        crate::frontend::Parser::new(input)
    }

    /// Get pool capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Get current pool size (simplified implementation)
    #[must_use]
    pub fn size(&self) -> usize {
        let stats = self.stats.lock().expect("Lock poisoned");
        stats.current_size
    }

    /// Get pool statistics
    #[must_use]
    pub fn stats(&self) -> PoolStatsSummary {
        let stats = self.stats.lock().expect("Lock poisoned");
        PoolStatsSummary {
            created: stats.created,
            borrowed: stats.borrowed,
            returned: stats.returned,
            current_size: stats.current_size,
        }
    }
}

/// Summary of pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatsSummary {
    pub created: u64,
    pub borrowed: u64,
    pub returned: u64,
    pub current_size: usize,
}

/// Memory-efficient string interning
pub struct StringInterner {
    /// Interned strings
    strings: HashMap<String, usize>,
    /// String storage by ID
    storage: Vec<String>,
    /// Next available ID
    next_id: usize,
}

impl StringInterner {
    /// Create new string interner
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::performance_optimizations::StringInterner;
    ///
    /// let mut interner = StringInterner::new();
    /// let id1 = interner.intern("hello");
    /// let id2 = interner.intern("hello");
    /// assert_eq!(id1, id2); // Same string gets same ID
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            storage: Vec::new(),
            next_id: 0,
        }
    }

    /// Intern a string and return its ID
    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.next_id;
            self.strings.insert(s.to_string(), id);
            self.storage.push(s.to_string());
            self.next_id += 1;
            id
        }
    }

    /// Get string by ID
    #[must_use]
    pub fn get(&self, id: usize) -> Option<&str> {
        self.storage.get(id).map(String::as_str)
    }

    /// Get number of interned strings
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Check if interner is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Clear all interned strings
    pub fn clear(&mut self) {
        self.strings.clear();
        self.storage.clear();
        self.next_id = 0;
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_compilation_cache_basic() {
        let mut cache = CompilationCache::new(2);

        // Test insertion and retrieval
        cache.insert(
            "test1".to_string(),
            "output1".to_string(),
            Duration::from_millis(100),
            512,
        );
        assert_eq!(cache.get("test1"), Some("output1".to_string()));
        assert_eq!(cache.len(), 1);

        // Test miss
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn test_compilation_cache_lru_eviction() {
        let mut cache = CompilationCache::new(2);

        // Fill cache
        cache.insert(
            "a".to_string(),
            "output_a".to_string(),
            Duration::from_millis(100),
            256,
        );
        cache.insert(
            "b".to_string(),
            "output_b".to_string(),
            Duration::from_millis(100),
            256,
        );
        assert_eq!(cache.len(), 2);

        // Access 'a' to make it more recently used
        cache.get("a");

        // Insert 'c' should evict 'b' (least recently used)
        cache.insert(
            "c".to_string(),
            "output_c".to_string(),
            Duration::from_millis(100),
            256,
        );
        assert_eq!(cache.len(), 2);
        assert!(cache.get("a").is_some());
        assert!(cache.get("c").is_some());
        assert!(cache.get("b").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CompilationCache::new(100);

        // Add some data
        cache.insert(
            "test".to_string(),
            "output".to_string(),
            Duration::from_millis(100),
            512,
        );

        // Generate some hits and misses
        cache.get("test");
        cache.get("missing");

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests(), 2);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        // Test basic interning
        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello"); // Same string

        assert_ne!(id1, id2);
        assert_eq!(id1, id3); // Same string gets same ID

        // Test retrieval
        assert_eq!(interner.get(id1), Some("hello"));
        assert_eq!(interner.get(id2), Some("world"));
        assert_eq!(interner.len(), 2); // Only 2 unique strings
    }

    #[test]
    fn test_parser_pool_basic() {
        let pool = ParserPool::new(5);

        // Create parser
        let _parser1 = pool.create_parser("42");

        // Pool should work
        assert_eq!(pool.capacity(), 5);

        // Should be able to create another
        let _parser2 = pool.create_parser("true");

        let stats = pool.stats();
        assert_eq!(stats.created, 2);
        assert_eq!(stats.borrowed, 2);
    }

    #[test]
    fn test_cache_memory_usage() {
        let mut cache = CompilationCache::new(10);

        cache.insert(
            "small".to_string(),
            "x".to_string(),
            Duration::from_millis(50),
            100,
        );
        cache.insert(
            "large".to_string(),
            "y".repeat(1000),
            Duration::from_millis(200),
            2000,
        );

        let total_memory = cache.memory_usage();
        assert_eq!(total_memory, 2100); // 100 + 2000
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CompilationCache::new(10);
        cache.insert(
            "test".to_string(),
            "output".to_string(),
            Duration::from_millis(100),
            512,
        );
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}
