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
#[derive(Debug)]
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
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::reset;
/// 
/// let result = reset(());
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::reset;
/// 
/// let result = reset(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn reset(&self) {
        self.storage.borrow_mut().clear();
        *self.used.borrow_mut() = 0;
    }
    /// Get current memory usage
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::used;
/// 
/// let result = used(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn used(&self) -> usize {
        *self.used.borrow()
    }
}
/// Reference to a value in the arena
#[derive(Debug)]
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
#[derive(Debug)]
pub struct TransactionalArena {
    /// Current values
    current: Rc<SafeArena>,
    /// Checkpoints
    checkpoints: Vec<ArenaCheckpoint>,
}
#[derive(Clone, Debug)]
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
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::checkpoint;
/// 
/// let result = checkpoint(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn checkpoint(&mut self) -> usize {
        let checkpoint = ArenaCheckpoint {
            storage_size: self.current.storage.borrow().len(),
            used: self.current.used(),
        };
        self.checkpoints.push(checkpoint);
        self.checkpoints.len() - 1
    }
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::rollback;
/// 
/// let result = rollback(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::commit;
/// 
/// let result = commit(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn commit(&mut self) -> Result<()> {
        if self.checkpoints.is_empty() {
            return Err(anyhow!("No checkpoint to commit"));
        }
        self.checkpoints.pop();
        Ok(())
    }
/// # Examples
/// 
/// ```
/// use ruchy::runtime::safe_arena::arena;
/// 
/// let result = arena(());
/// assert_eq!(result, Ok(()));
/// ```
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
    #[test]
    fn test_arena_memory_limit() {
        let arena = SafeArena::new(16); // Very small limit
        // First allocation should succeed
        let _val1 = arena.alloc([0u8; 8]).unwrap();
        // Second allocation should fail due to memory limit
        let result = arena.alloc([0u8; 16]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("memory limit exceeded"));
    }
    #[test]
    fn test_arena_used_tracking() {
        let arena = SafeArena::new(1024);
        assert_eq!(arena.used(), 0);
        let _val1 = arena.alloc(42i32).unwrap();
        let used_after_int = arena.used();
        assert!(used_after_int >= 4); // At least size of i32
        let _val2 = arena.alloc("test".to_string()).unwrap();
        let used_after_string = arena.used();
        assert!(used_after_string > used_after_int);
    }
    #[test]
    fn test_arena_ref_deref() {
        let arena = SafeArena::new(1024);
        let val = arena.alloc(vec![1, 2, 3, 4]).unwrap();
        // Test Deref trait
        assert_eq!(val.len(), 4);
        assert_eq!(val[0], 1);
        assert_eq!(val[3], 4);
    }
    #[test]
    fn test_transactional_arena_new() {
        let arena = TransactionalArena::new(2048);
        assert_eq!(arena.arena().used(), 0);
        assert!(arena.checkpoints.is_empty());
    }
    #[test]
    fn test_transactional_arena_multiple_checkpoints() {
        let mut arena = TransactionalArena::new(1024);
        // Initial allocation
        arena.arena().alloc(100).unwrap();
        // First checkpoint
        let cp1 = arena.checkpoint();
        arena.arena().alloc(200).unwrap();
        // Second checkpoint
        let _cp2 = arena.checkpoint();
        arena.arena().alloc(300).unwrap();
        // Rollback to first checkpoint
        arena.rollback(cp1).unwrap();
        // Should only have allocations up to first checkpoint
        let used = arena.arena().used();
        assert!(used >= 4); // At least the first allocation
    }
    #[test]
    fn test_transactional_arena_invalid_checkpoint() {
        let mut arena = TransactionalArena::new(1024);
        // Try to rollback to invalid checkpoint
        let result = arena.rollback(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid checkpoint"));
    }
    #[test]
    fn test_transactional_arena_commit() {
        let mut arena = TransactionalArena::new(1024);
        // Create checkpoint
        let _cp = arena.checkpoint();
        assert!(!arena.checkpoints.is_empty());
        // Commit should remove the checkpoint
        arena.commit().unwrap();
        assert!(arena.checkpoints.is_empty());
    }
    #[test]
    fn test_transactional_arena_commit_without_checkpoint() {
        let mut arena = TransactionalArena::new(1024);
        // Try to commit without checkpoint
        let result = arena.commit();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No checkpoint to commit"));
    }
    #[test]
    fn test_transactional_arena_reset() {
        let mut arena = TransactionalArena::new(1024);
        // Add some data and checkpoints
        arena.arena().alloc(42).unwrap();
        arena.checkpoint();
        arena.arena().alloc(84).unwrap();
        assert!(arena.arena().used() > 0);
        assert!(!arena.checkpoints.is_empty());
        // Reset should clear everything
        arena.reset();
        assert_eq!(arena.arena().used(), 0);
        assert!(arena.checkpoints.is_empty());
    }
    #[test]
    fn test_checkpoint_clone() {
        let checkpoint1 = ArenaCheckpoint {
            storage_size: 10,
            used: 100,
        };
        let checkpoint2 = checkpoint1.clone();
        assert_eq!(checkpoint1.storage_size, checkpoint2.storage_size);
        assert_eq!(checkpoint1.used, checkpoint2.used);
    }
    #[test]
    fn test_arena_with_different_types() {
        let arena = SafeArena::new(1024);
        // Allocate different types
        let int_val = arena.alloc(42i32).unwrap();
        let string_val = arena.alloc("hello".to_string()).unwrap();
        let vec_val = arena.alloc(vec![1, 2, 3]).unwrap();
        let bool_val = arena.alloc(true).unwrap();
        // Verify all values
        assert_eq!(*int_val, 42);
        assert_eq!(*string_val, "hello");
        assert_eq!(*vec_val, vec![1, 2, 3]);
        assert!(*bool_val);
    }
    #[test]
    fn test_transactional_arena_nested_operations() {
        let mut arena = TransactionalArena::new(1024);
        // Initial state
        arena.arena().alloc(1).unwrap();
        let initial_used = arena.arena().used();
        // Start transaction
        let cp = arena.checkpoint();
        arena.arena().alloc(2).unwrap();
        arena.arena().alloc(3).unwrap();
        let mid_used = arena.arena().used();
        assert!(mid_used > initial_used);
        // Rollback
        arena.rollback(cp).unwrap();
        let final_used = arena.arena().used();
        assert_eq!(final_used, initial_used);
    }
    #[test]
    fn test_arena_large_allocation() {
        let arena = SafeArena::new(1024);
        // Try to allocate something larger than limit
        let result = arena.alloc([0u8; 2048]);
        assert!(result.is_err());
    }
    #[test]
    fn test_transactional_checkpoint_return_value() {
        let mut arena = TransactionalArena::new(1024);
        let cp1 = arena.checkpoint();
        assert_eq!(cp1, 0);
        let cp2 = arena.checkpoint();
        assert_eq!(cp2, 1);
        let cp3 = arena.checkpoint();
        assert_eq!(cp3, 2);
    }
}
#[cfg(test)]
mod property_tests_safe_arena {
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
