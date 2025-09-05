//! Inline caching for optimization
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use std::collections::HashMap;

/// State of a cache entry
#[derive(Debug, Clone, PartialEq)]
pub enum CacheState {
    /// Never seen before
    Uninitialized,
    /// Single type observed
    Monomorphic { type_id: TypeId },
    /// Multiple types observed
    Polymorphic { type_ids: Vec<TypeId> },
    /// Too many types - give up caching
    Megamorphic,
}

/// Type identifier for caching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

impl TypeId {
    /// Create type ID from value
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Integer(_) => TypeId(0),
            Value::Float(_) => TypeId(1),
            Value::Bool(_) => TypeId(2),
            Value::Nil => TypeId(3),
            Value::String(_) => TypeId(4),
            Value::Array(_) => TypeId(5),
            Value::Tuple(_) => TypeId(6),
            Value::Closure { .. } => TypeId(7),
        }
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self.0 {
            0 => "integer",
            1 => "float",
            2 => "boolean",
            3 => "nil",
            4 => "string",
            5 => "array",
            6 => "tuple",
            7 => "closure",
            _ => "unknown",
        }
    }
}

/// Cache entry for a specific location
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub state: CacheState,
    pub hits: u64,
    pub misses: u64,
    /// Cached operation result for monomorphic case
    pub cached_result: Option<Value>,
}

impl CacheEntry {
    /// Create new cache entry
    pub fn new() -> Self {
        Self {
            state: CacheState::Uninitialized,
            hits: 0,
            misses: 0,
            cached_result: None,
        }
    }

    /// Update cache with new type observation
    pub fn update(&mut self, type_id: TypeId) {
        match &mut self.state {
            CacheState::Uninitialized => {
                self.state = CacheState::Monomorphic { type_id };
            }
            CacheState::Monomorphic { type_id: current } => {
                if *current != type_id {
                    self.state = CacheState::Polymorphic {
                        type_ids: vec![*current, type_id],
                    };
                    self.cached_result = None;
                }
            }
            CacheState::Polymorphic { type_ids } => {
                if !type_ids.contains(&type_id) {
                    type_ids.push(type_id);
                    if type_ids.len() > 4 {
                        self.state = CacheState::Megamorphic;
                        self.cached_result = None;
                    }
                }
            }
            CacheState::Megamorphic => {}
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Check if cache is effective
    pub fn is_effective(&self) -> bool {
        self.hit_rate() > 0.8 && (self.hits + self.misses) > 10
    }
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Inline cache for optimizing operations
pub struct InlineCache {
    /// Cache entries indexed by location
    entries: HashMap<usize, CacheEntry>,
    /// Global hit count
    global_hits: u64,
    /// Global miss count
    global_misses: u64,
    /// Maximum cache size
    max_entries: usize,
}

impl InlineCache {
    /// Create new inline cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            global_hits: 0,
            global_misses: 0,
            max_entries: 10000,
        }
    }

    /// Look up cache entry for a location
    pub fn lookup(&self, location: usize) -> Option<&CacheEntry> {
        self.entries.get(&location)
    }

    /// Get mutable cache entry for a location
    pub fn lookup_mut(&mut self, location: usize) -> Option<&mut CacheEntry> {
        self.entries.get_mut(&location)
    }

    /// Insert or update cache entry
    pub fn insert(&mut self, location: usize, type_id: TypeId) -> &mut CacheEntry {
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&location) {
            self.evict_least_effective();
        }

        let entry = self.entries.entry(location).or_insert_with(CacheEntry::new);
        entry.update(type_id);
        entry
    }

    /// Record global hit
    pub fn record_hit(&mut self, location: usize) {
        self.global_hits += 1;
        if let Some(entry) = self.entries.get_mut(&location) {
            entry.record_hit();
        }
    }

    /// Record global miss
    pub fn record_miss(&mut self, location: usize) {
        self.global_misses += 1;
        if let Some(entry) = self.entries.get_mut(&location) {
            entry.record_miss();
        }
    }

    /// Get global hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.global_hits + self.global_misses;
        if total == 0 {
            0.0
        } else {
            self.global_hits as f64 / total as f64
        }
    }

    /// Evict least effective cache entry
    fn evict_least_effective(&mut self) {
        if let Some((location, _)) = self
            .entries
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.hit_rate()
                    .partial_cmp(&b.hit_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(k, v)| (*k, v.clone()))
        {
            self.entries.remove(&location);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut monomorphic = 0;
        let mut polymorphic = 0;
        let mut megamorphic = 0;
        let mut uninitialized = 0;

        for entry in self.entries.values() {
            match &entry.state {
                CacheState::Uninitialized => uninitialized += 1,
                CacheState::Monomorphic { .. } => monomorphic += 1,
                CacheState::Polymorphic { .. } => polymorphic += 1,
                CacheState::Megamorphic => megamorphic += 1,
            }
        }

        CacheStats {
            total_entries: self.entries.len(),
            monomorphic,
            polymorphic,
            megamorphic,
            uninitialized,
            global_hit_rate: self.hit_rate(),
            global_hits: self.global_hits,
            global_misses: self.global_misses,
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.global_hits = 0;
        self.global_misses = 0;
    }
}

impl Default for InlineCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for inline cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub monomorphic: usize,
    pub polymorphic: usize,
    pub megamorphic: usize,
    pub uninitialized: usize,
    pub global_hit_rate: f64,
    pub global_hits: u64,
    pub global_misses: u64,
}

impl CacheStats {
    /// Get percentage of monomorphic entries
    pub fn monomorphic_percentage(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.monomorphic as f64 / self.total_entries as f64) * 100.0
        }
    }

    /// Check if cache is healthy
    pub fn is_healthy(&self) -> bool {
        self.global_hit_rate > 0.7 && self.monomorphic_percentage() > 60.0
    }
}