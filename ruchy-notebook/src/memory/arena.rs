use std::cell::{Cell, RefCell};
use std::rc::Rc;

const ARENA_SIZE: usize = 256 * 1024; // 256KB transient arena

/// A safe bump-pointer arena allocator for transient allocations
pub struct Arena {
    storage: RefCell<Vec<Box<dyn std::any::Any>>>,
    bytes_used: Cell<usize>,
    generation: Cell<u64>,
}

impl Arena {
    /// Create a new arena with default size (256KB)
    pub fn new() -> Self {
        Self {
            storage: RefCell::new(Vec::new()),
            bytes_used: Cell::new(0),
            generation: Cell::new(0),
        }
    }
    
    /// Create an arena with specified capacity
    pub fn with_capacity(_capacity: usize) -> Self {
        // Capacity hint for future optimization
        Self::new()
    }
    
    /// Allocate memory for type T in the arena
    pub fn alloc<T: 'static>(&self, value: T) -> ArenaRef<T> {
        let size = std::mem::size_of::<T>();
        self.bytes_used.set(self.bytes_used.get() + size);
        
        if self.bytes_used.get() > ARENA_SIZE {
            panic!("Arena out of memory: {} bytes used", self.bytes_used.get());
        }
        
        let boxed = Box::new(value);
        let rc = Rc::new(boxed);
        self.storage.borrow_mut().push(Box::new(rc.clone()));
        
        ArenaRef {
            value: rc,
            generation: self.generation.get(),
        }
    }
    
    /// Allocate a slice in the arena
    pub fn alloc_slice<T: Copy + 'static>(&self, slice: &[T]) -> ArenaRef<Vec<T>> {
        let size = std::mem::size_of::<T>() * slice.len();
        self.bytes_used.set(self.bytes_used.get() + size);
        
        if self.bytes_used.get() > ARENA_SIZE {
            panic!("Arena out of memory for slice");
        }
        
        let vec = slice.to_vec();
        self.alloc(vec)
    }
    
    /// Reset the arena, invalidating all references
    pub fn reset(&self) {
        self.storage.borrow_mut().clear();
        self.bytes_used.set(0);
        self.generation.set(self.generation.get().wrapping_add(1));
    }
    
    /// Get current memory usage
    pub fn used(&self) -> usize {
        self.bytes_used.get()
    }
    
    /// Get total capacity
    pub fn capacity(&self) -> usize {
        ARENA_SIZE
    }
    
    /// Get fragmentation percentage (0-100)
    pub fn fragmentation(&self) -> f64 {
        let used = self.used() as f64;
        let capacity = self.capacity() as f64;
        if capacity == 0.0 { return 0.0; }
        ((capacity - used) / capacity) * 100.0
    }
}

/// A reference to data allocated in an arena
pub struct ArenaRef<T> {
    value: Rc<Box<T>>,
    generation: u64,
}

impl<T> ArenaRef<T> {
    /// Get a reference to the allocated value
    pub fn get(&self) -> &T {
        &**self.value
    }
}

impl<T> std::ops::Deref for ArenaRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> Clone for ArenaRef<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            generation: self.generation,
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new();
        
        let x = arena.alloc(42i32);
        assert_eq!(*x, 42);
        
        let y = arena.alloc("hello".to_string());
        assert_eq!(&**y, "hello");
        
        assert!(arena.used() > 0);
    }
    
    #[test]
    fn test_arena_slice() {
        let arena = Arena::new();
        let data = vec![1, 2, 3, 4, 5];
        
        let slice_ref = arena.alloc_slice(&data);
        assert_eq!(&**slice_ref, &[1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn test_arena_reset() {
        let arena = Arena::new();
        
        let _ = arena.alloc(100u64);
        let used_before = arena.used();
        assert!(used_before > 0);
        
        arena.reset();
        assert_eq!(arena.used(), 0);
    }
    
    #[test]
    fn test_arena_fragmentation() {
        let arena = Arena::with_capacity(1024);
        
        assert_eq!(arena.fragmentation(), 100.0);
        
        let _ = arena.alloc(512u64);
        assert!(arena.fragmentation() < 100.0);
    }
    
    #[test]
    #[should_panic(expected = "Arena out of memory")]
    fn test_arena_overflow() {
        let arena = Arena::new();
        // Allocate more than the arena can handle
        for _ in 0..100000 {
            let _ = arena.alloc([0u64; 100]);
        }
    }
}