//! WebAssembly Heap Memory Management
//!
//! Provides generational garbage collection for WASM memory.

/// Heap allocator for WASM with generational GC
pub struct WasmHeap {
    /// Young generation for short-lived objects
    young: Vec<u8>,
    /// Old generation for long-lived objects
    old: Vec<u8>,
    /// GC roots
    roots: Vec<usize>,
}

impl WasmHeap {
    /// Create a new WASM heap with default capacity
    pub fn new() -> Self {
        Self {
            young: Vec::with_capacity(256 * 1024),    // 256KB
            old: Vec::with_capacity(2 * 1024 * 1024), // 2MB
            roots: Vec::new(),
        }
    }

    /// Perform minor garbage collection (young generation only)
    pub fn minor_gc(&mut self) {
        self.young.clear();
    }

    /// Perform major garbage collection (mark and compact)
    pub fn major_gc(&mut self) {
        // Mark phase
        let mut marked = vec![false; self.old.len()];
        for &root in &self.roots {
            if root < marked.len() {
                marked[root] = true;
            }
        }
        // Compact phase (simplified)
        let mut compacted = Vec::new();
        for (i, &is_marked) in marked.iter().enumerate() {
            if is_marked && i < self.old.len() {
                compacted.push(self.old[i]);
            }
        }
        self.old = compacted;
    }

    /// Get young generation size
    pub fn young_size(&self) -> usize {
        self.young.len()
    }

    /// Get old generation size
    pub fn old_size(&self) -> usize {
        self.old.len()
    }

    /// Add a root for GC marking
    pub fn add_root(&mut self, root: usize) {
        self.roots.push(root);
    }

    /// Clear all roots
    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }

    /// Allocate bytes in young generation
    pub fn alloc_young(&mut self, data: &[u8]) -> usize {
        let offset = self.young.len();
        self.young.extend_from_slice(data);
        offset
    }

    /// Promote data from young to old generation
    pub fn promote(&mut self, offset: usize, len: usize) {
        if offset + len <= self.young.len() {
            let data: Vec<u8> = self.young[offset..offset + len].to_vec();
            self.old.extend(data);
        }
    }
}

impl Default for WasmHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_new() {
        let heap = WasmHeap::new();
        assert!(heap.young.is_empty());
        assert!(heap.old.is_empty());
        assert!(heap.roots.is_empty());
    }

    #[test]
    fn test_heap_default() {
        let heap = WasmHeap::default();
        assert!(heap.young.is_empty());
    }

    #[test]
    fn test_heap_minor_gc() {
        let mut heap = WasmHeap::new();
        heap.young.push(1);
        heap.young.push(2);
        assert_eq!(heap.young.len(), 2);
        heap.minor_gc();
        assert!(heap.young.is_empty());
    }

    #[test]
    fn test_heap_major_gc_empty() {
        let mut heap = WasmHeap::new();
        heap.major_gc();
        assert!(heap.old.is_empty());
    }

    #[test]
    fn test_heap_major_gc_with_roots() {
        let mut heap = WasmHeap::new();
        heap.old = vec![1, 2, 3, 4, 5];
        heap.roots = vec![0, 2, 4]; // Mark indices 0, 2, 4
        heap.major_gc();
        // Only marked items should remain
        assert_eq!(heap.old.len(), 3);
    }

    #[test]
    fn test_heap_major_gc_out_of_bounds_root() {
        let mut heap = WasmHeap::new();
        heap.old = vec![1, 2, 3];
        heap.roots = vec![100]; // Out of bounds root
        heap.major_gc();
        // No items marked, so old should be empty
        assert!(heap.old.is_empty());
    }

    #[test]
    fn test_heap_with_data() {
        let mut heap = WasmHeap::new();
        // Add data to young gen
        heap.young.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(heap.young.len(), 5);

        // Minor GC clears young
        heap.minor_gc();
        assert!(heap.young.is_empty());
    }

    #[test]
    fn test_heap_major_gc_all_marked() {
        let mut heap = WasmHeap::new();
        heap.old = vec![10, 20, 30];
        heap.roots = vec![0, 1, 2]; // Mark all
        heap.major_gc();
        assert_eq!(heap.old.len(), 3);
    }

    #[test]
    fn test_heap_major_gc_none_marked() {
        let mut heap = WasmHeap::new();
        heap.old = vec![10, 20, 30];
        heap.roots = vec![]; // Mark none
        heap.major_gc();
        assert!(heap.old.is_empty());
    }

    #[test]
    fn test_heap_major_gc_partial_marks() {
        let mut heap = WasmHeap::new();
        heap.old = vec![10, 20, 30, 40, 50];
        heap.roots = vec![1, 3]; // Mark indices 1 and 3
        heap.major_gc();
        assert_eq!(heap.old.len(), 2);
        assert!(heap.old.contains(&20));
        assert!(heap.old.contains(&40));
    }

    #[test]
    fn test_young_size() {
        let mut heap = WasmHeap::new();
        assert_eq!(heap.young_size(), 0);
        heap.young.push(1);
        assert_eq!(heap.young_size(), 1);
    }

    #[test]
    fn test_old_size() {
        let mut heap = WasmHeap::new();
        assert_eq!(heap.old_size(), 0);
        heap.old.push(1);
        assert_eq!(heap.old_size(), 1);
    }

    #[test]
    fn test_add_root() {
        let mut heap = WasmHeap::new();
        assert!(heap.roots.is_empty());
        heap.add_root(0);
        heap.add_root(5);
        assert_eq!(heap.roots.len(), 2);
    }

    #[test]
    fn test_clear_roots() {
        let mut heap = WasmHeap::new();
        heap.roots = vec![0, 1, 2, 3];
        heap.clear_roots();
        assert!(heap.roots.is_empty());
    }

    #[test]
    fn test_alloc_young() {
        let mut heap = WasmHeap::new();
        let offset = heap.alloc_young(&[1, 2, 3]);
        assert_eq!(offset, 0);
        assert_eq!(heap.young_size(), 3);

        let offset2 = heap.alloc_young(&[4, 5]);
        assert_eq!(offset2, 3);
        assert_eq!(heap.young_size(), 5);
    }

    #[test]
    fn test_promote() {
        let mut heap = WasmHeap::new();
        heap.alloc_young(&[1, 2, 3, 4, 5]);
        heap.promote(1, 3); // Promote bytes at offset 1, length 3
        assert_eq!(heap.old_size(), 3);
        assert_eq!(heap.old, vec![2, 3, 4]);
    }

    #[test]
    fn test_promote_out_of_bounds() {
        let mut heap = WasmHeap::new();
        heap.alloc_young(&[1, 2, 3]);
        heap.promote(10, 5); // Out of bounds - should do nothing
        assert_eq!(heap.old_size(), 0);
    }

    // =====================================================================
    // Property Tests
    // =====================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(1000))]

            /// Property: Minor GC always clears young generation
            #[test]
            fn prop_minor_gc_clears_young(data in prop::collection::vec(any::<u8>(), 0..100)) {
                let mut heap = WasmHeap::new();
                heap.young = data;
                heap.minor_gc();
                prop_assert!(heap.young.is_empty());
            }

            /// Property: Major GC with no roots clears old generation
            #[test]
            fn prop_major_gc_no_roots_clears_old(data in prop::collection::vec(any::<u8>(), 0..100)) {
                let mut heap = WasmHeap::new();
                heap.old = data;
                heap.roots.clear();
                heap.major_gc();
                prop_assert!(heap.old.is_empty());
            }

            /// Property: Alloc increases young size by exact amount
            #[test]
            fn prop_alloc_increases_size(data in prop::collection::vec(any::<u8>(), 1..50)) {
                let mut heap = WasmHeap::new();
                let before = heap.young_size();
                heap.alloc_young(&data);
                prop_assert_eq!(heap.young_size(), before + data.len());
            }

            /// Property: Add root increases roots count
            #[test]
            fn prop_add_root_increases_count(root in any::<usize>()) {
                let mut heap = WasmHeap::new();
                let before = heap.roots.len();
                heap.add_root(root);
                prop_assert_eq!(heap.roots.len(), before + 1);
            }

            /// Property: Clear roots empties roots
            #[test]
            fn prop_clear_roots_empties(roots in prop::collection::vec(any::<usize>(), 0..20)) {
                let mut heap = WasmHeap::new();
                heap.roots = roots;
                heap.clear_roots();
                prop_assert!(heap.roots.is_empty());
            }
        }
    }
}
