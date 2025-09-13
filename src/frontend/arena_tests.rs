//! Tests for Arena memory allocator
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

use super::arena::*;
use std::rc::Rc;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let arena = Arena::new();
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }

    #[test]
    fn test_arena_default() {
        let arena = Arena::default();
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }

    #[test]
    fn test_alloc_integer() {
        let arena = Arena::new();
        let value = arena.alloc(42i32);
        
        assert_eq!(*value, 42);
        assert_eq!(arena.total_allocated(), 1);
        assert_eq!(arena.num_items(), 1);
    }

    #[test]
    fn test_alloc_string() {
        let arena = Arena::new();
        let value = arena.alloc("hello".to_string());
        
        assert_eq!(*value, "hello");
        assert_eq!(arena.total_allocated(), 1);
        assert_eq!(arena.num_items(), 1);
    }

    #[test]
    fn test_alloc_multiple_values() {
        let arena = Arena::new();
        
        let int_val = arena.alloc(100i32);
        let str_val = arena.alloc("world".to_string());
        let bool_val = arena.alloc(true);
        
        assert_eq!(*int_val, 100);
        assert_eq!(*str_val, "world");
        assert_eq!(*bool_val, true);
        
        assert_eq!(arena.total_allocated(), 3);
        assert_eq!(arena.num_items(), 3);
    }

    #[test]
    fn test_alloc_same_type_multiple_times() {
        let arena = Arena::new();
        
        let val1 = arena.alloc(1i32);
        let val2 = arena.alloc(2i32);
        let val3 = arena.alloc(3i32);
        
        assert_eq!(*val1, 1);
        assert_eq!(*val2, 2);
        assert_eq!(*val3, 3);
        
        assert_eq!(arena.total_allocated(), 3);
        assert_eq!(arena.num_items(), 3);
    }

    #[test]
    fn test_clear_arena() {
        let arena = Arena::new();
        
        arena.alloc(42i32);
        arena.alloc("test".to_string());
        
        assert_eq!(arena.total_allocated(), 2);
        assert_eq!(arena.num_items(), 2);
        
        arena.clear();
        
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }

    #[test]
    fn test_arena_reuse_after_clear() {
        let arena = Arena::new();
        
        // Allocate some values
        arena.alloc(1i32);
        arena.alloc(2i32);
        arena.clear();
        
        // Allocate new values after clear
        let new_val = arena.alloc(100i32);
        assert_eq!(*new_val, 100);
        assert_eq!(arena.total_allocated(), 1);
        assert_eq!(arena.num_items(), 1);
    }

    #[test]
    fn test_rc_sharing() {
        let arena = Arena::new();
        let value = arena.alloc(42i32);
        
        // Clone the Rc to test shared ownership
        let shared_value = value.clone();
        assert_eq!(*shared_value, 42);
        assert_eq!(*value, *shared_value);
        
        // Both should point to the same data
        assert!(Rc::ptr_eq(&value, &shared_value));
    }

    #[test]
    fn test_alloc_complex_type() {
        #[derive(Debug, PartialEq)]
        struct ComplexType {
            id: u32,
            name: String,
            active: bool,
        }
        
        let arena = Arena::new();
        let complex = ComplexType {
            id: 123,
            name: "test_item".to_string(),
            active: true,
        };
        
        let allocated = arena.alloc(complex);
        
        assert_eq!(allocated.id, 123);
        assert_eq!(allocated.name, "test_item");
        assert_eq!(allocated.active, true);
        
        assert_eq!(arena.total_allocated(), 1);
    }

    #[test]
    fn test_arena_zero_allocation() {
        let arena = Arena::new();
        
        // Test arena state without any allocations
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
        
        // Clear empty arena
        arena.clear();
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_large_number_of_allocations() {
        let arena = Arena::new();
        let count = 1000;
        
        for i in 0..count {
            arena.alloc(i);
        }
        
        assert_eq!(arena.total_allocated(), count);
        assert_eq!(arena.num_items(), count);
    }

    #[test]
    fn test_mixed_type_allocations() {
        let arena = Arena::new();
        
        // Allocate different types
        arena.alloc(42i32);
        arena.alloc(3.14f64);
        arena.alloc("string".to_string());
        arena.alloc(true);
        arena.alloc(vec![1, 2, 3]);
        
        assert_eq!(arena.total_allocated(), 5);
        assert_eq!(arena.num_items(), 5);
    }

    #[test]
    fn test_allocation_after_partial_clear() {
        let arena = Arena::new();
        
        // Allocate, clear, then allocate again
        arena.alloc(1i32);
        arena.alloc(2i32);
        
        let count_before = arena.total_allocated();
        arena.clear();
        
        arena.alloc(3i32);
        arena.alloc(4i32);
        
        assert_eq!(count_before, 2);
        assert_eq!(arena.total_allocated(), 2);
        assert_eq!(arena.num_items(), 2);
    }

    #[test]
    fn test_arena_statistics() {
        let arena = Arena::new();
        
        // Test statistics tracking
        assert_eq!(arena.total_allocated(), 0);
        
        arena.alloc(1);
        assert_eq!(arena.total_allocated(), 1);
        assert_eq!(arena.num_items(), 1);
        
        arena.alloc(2);
        assert_eq!(arena.total_allocated(), 2);
        assert_eq!(arena.num_items(), 2);
        
        arena.clear();
        assert_eq!(arena.total_allocated(), 0);
        assert_eq!(arena.num_items(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_alloc_never_panics_with_integers(value in i32::MIN..i32::MAX) {
            let arena = Arena::new();
            let allocated = arena.alloc(value);
            prop_assert_eq!(*allocated, value);
            prop_assert_eq!(arena.total_allocated(), 1);
        }

        #[test]
        fn test_alloc_never_panics_with_strings(value in "[a-zA-Z0-9 ]{0,100}") {
            let arena = Arena::new();
            let allocated = arena.alloc(value.clone());
            prop_assert_eq!(*allocated, value);
            prop_assert_eq!(arena.total_allocated(), 1);
        }

        #[test]
        fn test_multiple_allocations_maintain_count(
            count in 1usize..100usize
        ) {
            let arena = Arena::new();
            
            for i in 0..count {
                arena.alloc(i);
            }
            
            prop_assert_eq!(arena.total_allocated(), count);
            prop_assert_eq!(arena.num_items(), count);
        }

        #[test]
        fn test_clear_always_resets_to_zero(
            initial_count in 1usize..50usize
        ) {
            let arena = Arena::new();
            
            // Allocate some values
            for i in 0..initial_count {
                arena.alloc(i);
            }
            
            prop_assert_eq!(arena.total_allocated(), initial_count);
            
            // Clear and verify reset
            arena.clear();
            prop_assert_eq!(arena.total_allocated(), 0);
            prop_assert_eq!(arena.num_items(), 0);
        }

        #[test]
        fn test_allocation_after_clear_works(
            before_count in 1usize..20usize,
            after_count in 1usize..20usize
        ) {
            let arena = Arena::new();
            
            // Allocate before
            for i in 0..before_count {
                arena.alloc(i);
            }
            
            arena.clear();
            
            // Allocate after
            for i in 0..after_count {
                arena.alloc(i + 1000); // Different values
            }
            
            prop_assert_eq!(arena.total_allocated(), after_count);
            prop_assert_eq!(arena.num_items(), after_count);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Test allocating AST-like structures
    #[derive(Debug, PartialEq)]
    struct MockAstNode {
        id: u32,
        node_type: String,
        children: Vec<Rc<MockAstNode>>,
    }

    #[test]
    fn test_ast_allocation_pattern() {
        let arena = Arena::new();
        
        // Create leaf nodes
        let leaf1 = arena.alloc(MockAstNode {
            id: 1,
            node_type: "Literal".to_string(),
            children: vec![],
        });
        
        let leaf2 = arena.alloc(MockAstNode {
            id: 2,
            node_type: "Literal".to_string(),
            children: vec![],
        });
        
        // Create parent node referencing children
        let parent = arena.alloc(MockAstNode {
            id: 3,
            node_type: "BinaryOp".to_string(),
            children: vec![leaf1.clone(), leaf2.clone()],
        });
        
        assert_eq!(arena.total_allocated(), 3);
        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].id, 1);
        assert_eq!(parent.children[1].id, 2);
    }

    #[test]
    fn test_arena_with_recursive_structures() {
        let arena = Arena::new();
        
        // Simulate building a small AST tree
        let mut nodes = Vec::new();
        
        for i in 0..10 {
            let node = arena.alloc(MockAstNode {
                id: i,
                node_type: format!("Node{}", i),
                children: vec![],
            });
            nodes.push(node);
        }
        
        assert_eq!(arena.total_allocated(), 10);
        assert_eq!(nodes.len(), 10);
        
        // Verify all nodes are properly allocated
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node.id, i as u32);
        }
    }
}