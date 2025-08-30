//! Safe arena allocator without unsafe code
//!
//! Provides bounded memory allocation using safe Rust abstractions.

use anyhow::{Result, anyhow};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Safe Arena Allocator
// ============================================================================

/// Safe arena allocator using Rc for memory management
pub struct SafeArena {
    /// Storage for allocated values
    storage: RefCell<Vec<Box<dyn std::any::Any>>>,
    /// Current memory usage estimate
    used: RefCell<usize>,
    /// Maximum allowed memory
    max_size: usize,
}

impl SafeArena {
    /// Create a new arena with the given size limit
    pub fn new(max_size: usize) -> Self {
        Self {
            storage: RefCell::new(Vec::new()),
            used: RefCell::new(0),
            max_size,
        }
    }
    
    /// Allocate a value in the arena
    pub fn alloc<T: 'static>(&self, value: T) -> Result<ArenaRef<'_, T>> {
        let size = std::mem::size_of::<T>();
        
        // Check memory limit
        if *self.used.borrow() + size > self.max_size {
            return Err(anyhow!("Arena memory limit exceeded"));
        }
        
        // Store value in Rc
        let rc_value = Rc::new(value);
        self.storage.borrow_mut().push(Box::new(rc_value.clone()) as Box<dyn std::any::Any>);
        *self.used.borrow_mut() += size;
        
        Ok(ArenaRef {
            value: rc_value,
            _arena: self,
        })
    }
    
    /// Reset the arena, clearing all allocations
    pub fn reset(&self) {
        self.storage.borrow_mut().clear();
        *self.used.borrow_mut() = 0;
    }
    
    /// Get current memory usage
    pub fn used(&self) -> usize {
        *self.used.borrow()
    }
}

/// Reference to a value in the arena
pub struct ArenaRef<'a, T> {
    value: Rc<T>,
    _arena: &'a SafeArena,
}

impl<T> std::ops::Deref for ArenaRef<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// ============================================================================
// Transactional Arena
// ============================================================================

/// Arena with checkpoint/rollback support
pub struct TransactionalArena {
    /// Current values
    current: Rc<SafeArena>,
    /// Checkpoints
    checkpoints: Vec<ArenaCheckpoint>,
}

#[derive(Clone)]
struct ArenaCheckpoint {
    storage_size: usize,
    used: usize,
}

impl TransactionalArena {
    pub fn new(max_size: usize) -> Self {
        Self {
            current: Rc::new(SafeArena::new(max_size)),
            checkpoints: Vec::new(),
        }
    }
    
    pub fn checkpoint(&mut self) -> usize {
        let checkpoint = ArenaCheckpoint {
            storage_size: self.current.storage.borrow().len(),
            used: self.current.used(),
        };
        self.checkpoints.push(checkpoint);
        self.checkpoints.len() - 1
    }
    
    pub fn rollback(&mut self, checkpoint_id: usize) -> Result<()> {
        if checkpoint_id >= self.checkpoints.len() {
            return Err(anyhow!("Invalid checkpoint"));
        }
        
        let checkpoint = &self.checkpoints[checkpoint_id];
        
        // Truncate storage to checkpoint size
        self.current.storage.borrow_mut().truncate(checkpoint.storage_size);
        *self.current.used.borrow_mut() = checkpoint.used;
        
        // Remove later checkpoints
        self.checkpoints.truncate(checkpoint_id + 1);
        
        Ok(())
    }
    
    pub fn commit(&mut self) -> Result<()> {
        if self.checkpoints.is_empty() {
            return Err(anyhow!("No checkpoint to commit"));
        }
        self.checkpoints.pop();
        Ok(())
    }
    
    pub fn arena(&self) -> &SafeArena {
        &self.current
    }
    
    pub fn reset(&mut self) {
        self.current.reset();
        self.checkpoints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_arena() {
        let arena = SafeArena::new(1024);
        
        let v1 = arena.alloc(42).unwrap();
        let v2 = arena.alloc("hello".to_string()).unwrap();
        
        assert_eq!(*v1, 42);
        assert_eq!(*v2, "hello");
        
        arena.reset();
        assert_eq!(arena.used(), 0);
    }
    
    #[test]
    fn test_transactional() {
        let mut arena = TransactionalArena::new(1024);
        
        arena.arena().alloc(1).unwrap();
        let checkpoint = arena.checkpoint();
        
        arena.arena().alloc(2).unwrap();
        let used_before = arena.arena().used();
        
        arena.rollback(checkpoint).unwrap();
        let used_after = arena.arena().used();
        
        assert!(used_after < used_before);
    }
}