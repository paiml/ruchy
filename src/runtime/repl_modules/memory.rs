//! Memory tracking module for bounded allocation
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Memory tracker for bounded allocation
/// Arena-style memory tracker for bounded evaluation
/// Provides fixed memory allocation with reset capability
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    allocated: Arc<AtomicUsize>,
    limit: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker with a specified limit
    pub fn new(limit: usize) -> Self {
        Self {
            allocated: Arc::new(AtomicUsize::new(0)),
            limit,
        }
    }

    /// Track memory allocation
    pub fn allocate(&self, bytes: usize) -> Result<(), String> {
        let current = self.allocated.load(Ordering::SeqCst);
        if current + bytes > self.limit {
            Err(format!(
                "Memory limit exceeded: {} + {} > {}",
                current, bytes, self.limit
            ))
        } else {
            self.allocated.fetch_add(bytes, Ordering::SeqCst);
            Ok(())
        }
    }

    /// Track memory deallocation
    pub fn deallocate(&self, bytes: usize) {
        self.allocated.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.allocated.load(Ordering::SeqCst)
    }

    /// Get memory limit
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Get remaining memory
    pub fn remaining(&self) -> usize {
        let current = self.current_usage();
        if current > self.limit {
            0
        } else {
            self.limit - current
        }
    }

    /// Check if allocation would exceed limit
    pub fn would_exceed(&self, bytes: usize) -> bool {
        self.current_usage() + bytes > self.limit
    }

    /// Reset memory tracker
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::SeqCst);
    }

    /// Get memory usage percentage
    pub fn usage_percentage(&self) -> f64 {
        let current = self.current_usage() as f64;
        let limit = self.limit as f64;
        (current / limit) * 100.0
    }

    /// Check if memory is nearly exhausted (>90% used)
    pub fn is_nearly_exhausted(&self) -> bool {
        self.usage_percentage() > 90.0
    }
}