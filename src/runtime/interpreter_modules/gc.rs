//! Conservative garbage collector
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

/// GC-managed object
#[derive(Debug, Clone)]
pub struct GCObject {
    pub id: usize,
    pub value: Value,
    pub marked: bool,
    pub generation: u32,
}

impl GCObject {
    /// Create new GC object
    pub fn new(id: usize, value: Value) -> Self {
        Self {
            id,
            value,
            marked: false,
            generation: 0,
        }
    }

    /// Mark object as reachable
    pub fn mark(&mut self) {
        self.marked = true;
    }

    /// Unmark object
    pub fn unmark(&mut self) {
        self.marked = false;
    }

    /// Promote to next generation
    pub fn promote(&mut self) {
        self.generation += 1;
    }

    /// Get estimated size in bytes
    pub fn size_estimate(&self) -> usize {
        estimate_value_size(&self.value)
    }
}

/// Conservative garbage collector
pub struct ConservativeGC {
    /// All allocated objects
    objects: HashMap<usize, GCObject>,
    /// Next object ID
    next_id: usize,
    /// Roots (always reachable)
    roots: HashSet<usize>,
    /// Allocation threshold for triggering GC
    allocation_threshold: usize,
    /// Current allocated memory
    allocated_bytes: usize,
    /// Total collections performed
    collections: u64,
    /// Total objects freed
    total_freed: u64,
    /// Auto-collect enabled
    auto_collect: bool,
}

impl ConservativeGC {
    /// Create new garbage collector
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            next_id: 1,
            roots: HashSet::new(),
            allocation_threshold: 1024 * 1024, // 1MB
            allocated_bytes: 0,
            collections: 0,
            total_freed: 0,
            auto_collect: true,
        }
    }

    /// Track a new object
    pub fn track_object(&mut self, value: Value) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let size = estimate_value_size(&value);
        self.allocated_bytes += size;

        let obj = GCObject::new(id, value);
        self.objects.insert(id, obj);

        // Check if we should collect
        if self.auto_collect && self.allocated_bytes > self.allocation_threshold {
            self.collect_garbage();
        }

        id
    }

    /// Mark object as root
    pub fn add_root(&mut self, id: usize) {
        self.roots.insert(id);
    }

    /// Remove root marking
    pub fn remove_root(&mut self, id: usize) {
        self.roots.remove(&id);
    }

    /// Get object by ID
    pub fn get_object(&self, id: usize) -> Option<&GCObject> {
        self.objects.get(&id)
    }

    /// Get mutable object by ID
    pub fn get_object_mut(&mut self, id: usize) -> Option<&mut GCObject> {
        self.objects.get_mut(&id)
    }

    /// Perform garbage collection
    pub fn collect_garbage(&mut self) -> usize {
        self.collections += 1;
        
        // Mark phase
        self.mark_phase();
        
        // Sweep phase
        let freed = self.sweep_phase();
        
        self.total_freed += freed as u64;
        freed
    }

    /// Mark phase - mark all reachable objects
    fn mark_phase(&mut self) {
        // Unmark all objects first
        for obj in self.objects.values_mut() {
            obj.unmark();
        }

        // Mark from roots
        let roots: Vec<usize> = self.roots.iter().copied().collect();
        for root_id in roots {
            self.mark_object(root_id);
        }
    }

    /// Mark an object and its children
    fn mark_object(&mut self, id: usize) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if obj.marked {
                return; // Already marked
            }
            obj.mark();

            // Mark referenced objects based on value type
            let refs = find_references(&obj.value);
            for ref_id in refs {
                self.mark_object(ref_id);
            }
        }
    }

    /// Sweep phase - remove unmarked objects
    fn sweep_phase(&mut self) -> usize {
        let mut to_remove = Vec::new();
        let mut freed_bytes = 0;

        for (id, obj) in &self.objects {
            if !obj.marked {
                to_remove.push(*id);
                freed_bytes += obj.size_estimate();
            }
        }

        let freed_count = to_remove.len();
        for id in to_remove {
            self.objects.remove(&id);
        }

        self.allocated_bytes = self.allocated_bytes.saturating_sub(freed_bytes);
        freed_count
    }

    /// Force a garbage collection
    pub fn force_collect(&mut self) -> usize {
        self.collect_garbage()
    }

    /// Get GC statistics
    pub fn get_stats(&self) -> GCStats {
        GCStats {
            total_objects: self.objects.len(),
            total_bytes: self.allocated_bytes,
            collections: self.collections,
            total_freed: self.total_freed,
            roots: self.roots.len(),
        }
    }

    /// Get detailed GC information
    pub fn get_info(&self) -> GCInfo {
        let mut generation_counts = HashMap::new();
        for obj in self.objects.values() {
            *generation_counts.entry(obj.generation).or_insert(0) += 1;
        }

        GCInfo {
            stats: self.get_stats(),
            generation_counts,
            threshold: self.allocation_threshold,
            auto_collect: self.auto_collect,
        }
    }

    /// Set collection threshold
    pub fn set_collection_threshold(&mut self, bytes: usize) {
        self.allocation_threshold = bytes;
    }

    /// Enable/disable auto-collection
    pub fn set_auto_collect(&mut self, enabled: bool) {
        self.auto_collect = enabled;
    }

    /// Clear all objects
    pub fn clear(&mut self) {
        self.objects.clear();
        self.roots.clear();
        self.allocated_bytes = 0;
        self.next_id = 1;
    }
}

impl Default for ConservativeGC {
    fn default() -> Self {
        Self::new()
    }
}

/// GC statistics
#[derive(Debug, Clone)]
pub struct GCStats {
    pub total_objects: usize,
    pub total_bytes: usize,
    pub collections: u64,
    pub total_freed: u64,
    pub roots: usize,
}

impl GCStats {
    /// Get average objects freed per collection
    pub fn avg_freed_per_collection(&self) -> f64 {
        if self.collections == 0 {
            0.0
        } else {
            self.total_freed as f64 / self.collections as f64
        }
    }
}

/// Detailed GC information
#[derive(Debug, Clone)]
pub struct GCInfo {
    pub stats: GCStats,
    pub generation_counts: HashMap<u32, usize>,
    pub threshold: usize,
    pub auto_collect: bool,
}

// Helper functions (complexity: ≤10)

/// Estimate size of a value in bytes
fn estimate_value_size(value: &Value) -> usize {
    match value {
        Value::Integer(_) => 8,
        Value::Float(_) => 8,
        Value::Bool(_) => 1,
        Value::Nil => 0,
        Value::String(s) => s.len() + 24, // String content + Rc overhead
        Value::Array(arr) => {
            let content_size: usize = arr.iter().map(estimate_value_size).sum();
            content_size + arr.len() * 8 + 24 // Content + pointers + Rc
        }
        Value::Tuple(tup) => {
            let content_size: usize = tup.iter().map(estimate_value_size).sum();
            content_size + tup.len() * 8 + 24
        }
        Value::Closure { params, .. } => {
            params.len() * 32 + 64 // Rough estimate
        }
    }
}

/// Find object references in a value
fn find_references(_value: &Value) -> Vec<usize> {
    // In a real implementation, this would extract object IDs
    // from compound values like arrays and closures
    Vec::new()
}