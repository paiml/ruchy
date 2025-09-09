use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Handle to a value in the slab allocator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabHandle {
    index: usize,
    generation: u32,
}

/// A slab allocator for persistent values that survive cell execution
pub struct SlabAllocator<T> {
    entries: Vec<SlabEntry<T>>,
    free_list: Vec<usize>,
    generation: AtomicUsize,
}

#[derive(Clone)]
struct SlabEntry<T> {
    value: Option<T>,
    generation: u32,
}

impl<T> SlabAllocator<T> {
    /// Create a new slab allocator
    pub fn new() -> Self {
        Self::with_capacity(64)
    }
    
    /// Create a slab allocator with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            generation: AtomicUsize::new(1),
        }
    }
    
    /// Insert a value and get its handle
    pub fn insert(&mut self, value: T) -> SlabHandle {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) as u32;
        
        if let Some(index) = self.free_list.pop() {
            // Reuse freed slot
            self.entries[index] = SlabEntry {
                value: Some(value),
                generation,
            };
            SlabHandle { index, generation }
        } else {
            // Allocate new slot
            let index = self.entries.len();
            self.entries.push(SlabEntry {
                value: Some(value),
                generation,
            });
            SlabHandle { index, generation }
        }
    }
    
    /// Get a reference to a value by handle
    pub fn get(&self, handle: SlabHandle) -> Option<&T> {
        self.entries.get(handle.index)
            .filter(|e| e.generation == handle.generation)
            .and_then(|e| e.value.as_ref())
    }
    
    /// Get a mutable reference to a value by handle
    pub fn get_mut(&mut self, handle: SlabHandle) -> Option<&mut T> {
        self.entries.get_mut(handle.index)
            .filter(|e| e.generation == handle.generation)
            .and_then(|e| e.value.as_mut())
    }
    
    /// Remove a value and return it
    pub fn remove(&mut self, handle: SlabHandle) -> Option<T> {
        let entry = self.entries.get_mut(handle.index)?;
        
        if entry.generation != handle.generation {
            return None;
        }
        
        self.free_list.push(handle.index);
        entry.generation = self.generation.fetch_add(1, Ordering::SeqCst) as u32;
        entry.value.take()
    }
    
    /// Promote a value from arena to slab (copy)
    pub fn promote<U>(&mut self, value: U) -> SlabHandle 
    where 
        T: From<U>
    {
        self.insert(T::from(value))
    }
    
    /// Get the number of allocated entries
    pub fn len(&self) -> usize {
        self.entries.len() - self.free_list.len()
    }
    
    /// Check if the slab is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.free_list.clear();
        self.generation.store(1, Ordering::SeqCst);
    }
    
    /// Compact the slab by removing freed entries
    pub fn compact(&mut self) -> HashMap<SlabHandle, SlabHandle> 
    where T: Clone
    {
        let mut remapping = HashMap::new();
        let mut new_entries = Vec::new();
        
        for (old_index, entry) in self.entries.iter().enumerate() {
            if entry.value.is_some() {
                let new_index = new_entries.len();
                new_entries.push(entry.clone());
                
                let old_handle = SlabHandle {
                    index: old_index,
                    generation: entry.generation,
                };
                let new_handle = SlabHandle {
                    index: new_index,
                    generation: entry.generation,
                };
                remapping.insert(old_handle, new_handle);
            }
        }
        
        self.entries = new_entries;
        self.free_list.clear();
        
        remapping
    }
}

impl<T> Default for SlabAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe slab allocator
pub struct ConcurrentSlab<T: Send + Sync> {
    inner: Arc<parking_lot::RwLock<SlabAllocator<T>>>,
}

impl<T: Send + Sync> ConcurrentSlab<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(SlabAllocator::new())),
        }
    }
    
    pub fn insert(&self, value: T) -> SlabHandle {
        self.inner.write().insert(value)
    }
    
    pub fn get(&self, handle: SlabHandle) -> Option<T> 
    where T: Clone
    {
        self.inner.read().get(handle).cloned()
    }
    
    pub fn remove(&self, handle: SlabHandle) -> Option<T> {
        self.inner.write().remove(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slab_insert_get() {
        let mut slab = SlabAllocator::new();
        
        let h1 = slab.insert("hello");
        let h2 = slab.insert("world");
        
        assert_eq!(slab.get(h1), Some(&"hello"));
        assert_eq!(slab.get(h2), Some(&"world"));
    }
    
    #[test]
    fn test_slab_remove() {
        let mut slab = SlabAllocator::new();
        
        let h = slab.insert(42);
        assert_eq!(slab.get(h), Some(&42));
        
        let removed = slab.remove(h);
        assert_eq!(removed, Some(42));
        assert_eq!(slab.get(h), None);
    }
    
    #[test]
    fn test_slab_reuse_slots() {
        let mut slab = SlabAllocator::new();
        
        let h1 = slab.insert(1);
        let _ = slab.remove(h1);
        let h2 = slab.insert(2);
        
        // Should reuse the same index but different generation
        assert_eq!(h1.index, h2.index);
        assert_ne!(h1.generation, h2.generation);
    }
    
    #[test]
    fn test_slab_compact() {
        let mut slab = SlabAllocator::new();
        
        let h1 = slab.insert(1);
        let h2 = slab.insert(2);
        let h3 = slab.insert(3);
        
        slab.remove(h2);
        
        let remapping = slab.compact();
        
        assert_eq!(slab.len(), 2);
        assert!(remapping.contains_key(&h1));
        assert!(remapping.contains_key(&h3));
        assert!(!remapping.contains_key(&h2));
    }
    
    #[test]
    fn test_concurrent_slab() {
        use std::thread;
        
        let slab = Arc::new(ConcurrentSlab::<i32>::new());
        let mut handles = vec![];
        
        for i in 0..10 {
            let slab_clone = Arc::clone(&slab);
            let handle = thread::spawn(move || {
                slab_clone.insert(i)
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        for handle in results {
            assert!(slab.get(handle).is_some());
        }
    }
}