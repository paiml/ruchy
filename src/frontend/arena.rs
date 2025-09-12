//! Arena-based memory allocator for AST nodes (safe version)
//!
//! This module provides an efficient arena allocator for AST nodes that:
//! - Reduces allocation overhead by pooling allocations
//! - Improves cache locality by keeping related nodes close
//! - Enables fast bulk deallocation when the arena is dropped
//! - Uses only safe Rust code
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
/// Memory pool for reusing allocations
pub struct Arena {
    /// Track total allocated items for statistics
    total_allocated: RefCell<usize>,
    /// Storage for allocated values (using Rc for shared ownership)
    storage: RefCell<Vec<Rc<dyn std::any::Any>>>,
}
impl Arena {
    /// Create a new arena allocator
    pub fn new() -> Self {
        Self {
            total_allocated: RefCell::new(0),
            storage: RefCell::new(Vec::new()),
        }
    }
    /// Allocate a value in the arena (returns Rc for shared ownership)
    pub fn alloc<T: 'static>(&self, value: T) -> Rc<T> {
        *self.total_allocated.borrow_mut() += 1;
        let rc = Rc::new(value);
        self.storage
            .borrow_mut()
            .push(rc.clone() as Rc<dyn std::any::Any>);
        rc
    }
    /// Get total items allocated
    pub fn total_allocated(&self) -> usize {
        *self.total_allocated.borrow()
    }
    /// Get number of items in storage
    pub fn num_items(&self) -> usize {
        self.storage.borrow().len()
    }
    /// Clear all allocations (for reuse)
    pub fn clear(&self) {
        self.storage.borrow_mut().clear();
        *self.total_allocated.borrow_mut() = 0;
    }
}
impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
/// String interner for deduplicating strings in the AST
pub struct StringInterner {
    map: RefCell<HashMap<String, Rc<str>>>,
}
impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
        }
    }
    /// Intern a string, returning an Rc that can be cheaply cloned
    pub fn intern(&self, s: &str) -> Rc<str> {
        let mut map = self.map.borrow_mut();
        if let Some(interned) = map.get(s) {
            Rc::clone(interned)
        } else {
            let rc: Rc<str> = Rc::from(s);
            map.insert(s.to_string(), Rc::clone(&rc));
            rc
        }
    }
    /// Get statistics about the interner
    pub fn stats(&self) -> (usize, usize) {
        let map = self.map.borrow();
        let total_size: usize = map.values().map(|s| s.len()).sum();
        (map.len(), total_size)
    }
    /// Clear the interner
    pub fn clear(&self) {
        self.map.borrow_mut().clear();
    }
}
impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
/// Fast bump allocator for sequential allocations
pub struct BumpAllocator<T> {
    /// Pre-allocated vector with capacity
    storage: RefCell<Vec<T>>,
    /// Track allocations
    count: RefCell<usize>,
}
impl<T> BumpAllocator<T> {
    /// Create a new bump allocator with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: RefCell::new(Vec::with_capacity(capacity)),
            count: RefCell::new(0),
        }
    }
    /// Allocate a value, returning its index
    pub fn alloc(&self, value: T) -> usize {
        let mut storage = self.storage.borrow_mut();
        let index = storage.len();
        storage.push(value);
        *self.count.borrow_mut() += 1;
        index
    }
    /// Get a reference to an allocated value by index
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.storage.borrow().get(index).cloned()
    }
    /// Get the number of allocated items
    pub fn len(&self) -> usize {
        self.storage.borrow().len()
    }
    /// Check if allocator is empty
    pub fn is_empty(&self) -> bool {
        self.storage.borrow().is_empty()
    }
    /// Clear all allocations
    pub fn clear(&self) {
        self.storage.borrow_mut().clear();
        *self.count.borrow_mut() = 0;
    }
}
impl<T> Default for BumpAllocator<T> {
    fn default() -> Self {
        Self::with_capacity(128)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arena_basic() {
        let arena = Arena::new();
        let x = arena.alloc(42i32);
        assert_eq!(*x, 42);
        let y = arena.alloc("hello".to_string());
        assert_eq!(y.as_ref(), "hello");
        assert_eq!(arena.total_allocated(), 2);
    }
    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");
        assert_eq!(&*s1, "hello");
        assert_eq!(&*s2, "hello");
        assert_eq!(&*s3, "world");
        // Check that identical strings share the same Rc
        assert!(Rc::ptr_eq(&s1, &s2));
        assert!(!Rc::ptr_eq(&s1, &s3));
        let (num_strings, _) = interner.stats();
        assert_eq!(num_strings, 2); // Only "hello" and "world"
    }
    #[test]
    fn test_bump_allocator() {
        let alloc = BumpAllocator::with_capacity(10);
        let idx1 = alloc.alloc(42i32);
        let idx2 = alloc.alloc(100i32);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(alloc.get(idx1), Some(42));
        assert_eq!(alloc.get(idx2), Some(100));
        assert_eq!(alloc.len(), 2);
    }
    #[test]
    fn test_arena_many_allocations() {
        let arena = Arena::new();
        for i in 0..1000 {
            let _x = arena.alloc(i);
        }
        assert_eq!(arena.total_allocated(), 1000);
        assert_eq!(arena.num_items(), 1000);
        arena.clear();
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }
}
