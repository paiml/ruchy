// Comprehensive TDD Test Suite for src/runtime/safe_arena.rs
// Target: SafeArena allocator with 95%+ coverage
// Sprint 79: Push Coverage to 75%
//
// Quality Standards:
// - TDD methodology: Test-first development
// - Cyclomatic complexity ≤10 for all test functions
// - Property-based testing with thousands of iterations
// - Zero SATD comments

use ruchy::runtime::safe_arena::{SafeArena, ArenaRef};
use proptest::prelude::*;
use std::rc::Rc;

// Basic functionality tests
#[test]
fn test_arena_new() {
    let arena = SafeArena::new(1024);
    assert_eq!(arena.used(), 0);
}

#[test]
fn test_arena_new_with_different_sizes() {
    let sizes = [0, 1, 10, 100, 1024, 1_000_000];
    for size in sizes {
        let arena = SafeArena::new(size);
        assert_eq!(arena.used(), 0);
    }
}

#[test]
fn test_alloc_simple_value() {
    let arena = SafeArena::new(1024);
    let value = 42i32;

    let result = arena.alloc(value);
    assert!(result.is_ok());

    let arena_ref = result.unwrap();
    assert_eq!(*arena_ref, 42);
    assert!(arena.used() > 0);
}

#[test]
fn test_alloc_string() {
    let arena = SafeArena::new(1024);
    let value = String::from("Hello, Arena!");

    let result = arena.alloc(value.clone());
    assert!(result.is_ok());

    let arena_ref = result.unwrap();
    assert_eq!(*arena_ref, "Hello, Arena!");
}

#[test]
fn test_alloc_vector() {
    let arena = SafeArena::new(1024);
    let value = vec![1, 2, 3, 4, 5];

    let result = arena.alloc(value.clone());
    assert!(result.is_ok());

    let arena_ref = result.unwrap();
    assert_eq!(*arena_ref, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_alloc_struct() {
    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    let arena = SafeArena::new(1024);
    let value = TestStruct {
        id: 123,
        name: String::from("test"),
    };

    let result = arena.alloc(value.clone());
    assert!(result.is_ok());

    let arena_ref = result.unwrap();
    assert_eq!(*arena_ref, value);
}

#[test]
fn test_memory_limit_exceeded() {
    let arena = SafeArena::new(1); // Very small arena
    let value = [0u8; 100]; // Large value

    let result = arena.alloc(value);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("memory limit exceeded"));
}

#[test]
fn test_multiple_allocations() {
    let arena = SafeArena::new(1024);

    let ref1 = arena.alloc(10i32).unwrap();
    let ref2 = arena.alloc(20i32).unwrap();
    let ref3 = arena.alloc(30i32).unwrap();

    assert_eq!(*ref1, 10);
    assert_eq!(*ref2, 20);
    assert_eq!(*ref3, 30);

    assert!(arena.used() > 0);
}

#[test]
fn test_reset() {
    let arena = SafeArena::new(1024);

    // Allocate some values
    let _ = arena.alloc(42i32).unwrap();
    let _ = arena.alloc("test").unwrap();
    let _ = arena.alloc(vec![1, 2, 3]).unwrap();

    assert!(arena.used() > 0);

    // Reset the arena
    arena.reset();

    assert_eq!(arena.used(), 0);

    // Can allocate again after reset
    let result = arena.alloc(100i32);
    assert!(result.is_ok());
}

#[test]
fn test_reset_multiple_times() {
    let arena = SafeArena::new(1024);

    for _ in 0..10 {
        let _ = arena.alloc(42i32);
        assert!(arena.used() > 0);
        arena.reset();
        assert_eq!(arena.used(), 0);
    }
}

#[test]
fn test_used_tracking() {
    let arena = SafeArena::new(1024);

    let initial = arena.used();
    assert_eq!(initial, 0);

    let _ = arena.alloc(42i32);
    let after_first = arena.used();
    assert!(after_first > initial);

    let _ = arena.alloc(84i32);
    let after_second = arena.used();
    assert!(after_second > after_first);
}

#[test]
fn test_arena_ref_deref() {
    let arena = SafeArena::new(1024);
    let value = 42i32;

    let arena_ref = arena.alloc(value).unwrap();

    // Test deref
    let dereferenced: &i32 = &*arena_ref;
    assert_eq!(*dereferenced, 42);
}


#[test]
fn test_zero_sized_arena() {
    let arena = SafeArena::new(0);

    // Even a small allocation should fail
    let result = arena.alloc(1u8);
    assert!(result.is_err());
}

#[test]
fn test_exact_size_allocation() {
    let size = std::mem::size_of::<u64>();
    let arena = SafeArena::new(size);

    // Should succeed for exact size
    let result = arena.alloc(42u64);
    assert!(result.is_ok());

    // Next allocation should fail
    let result2 = arena.alloc(1u8);
    assert!(result2.is_err());
}

#[test]
fn test_different_types_in_arena() {
    let arena = SafeArena::new(4096);

    let int_ref = arena.alloc(42i32).unwrap();
    let string_ref = arena.alloc(String::from("hello")).unwrap();
    let vec_ref = arena.alloc(vec![1.0, 2.0, 3.0]).unwrap();
    let bool_ref = arena.alloc(true).unwrap();

    assert_eq!(*int_ref, 42);
    assert_eq!(*string_ref, "hello");
    assert_eq!(*vec_ref, vec![1.0, 2.0, 3.0]);
    assert_eq!(*bool_ref, true);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn test_arena_creation_never_panics(size in 0usize..1_000_000usize) {
            let arena = SafeArena::new(size);
            prop_assert_eq!(arena.used(), 0);
        }

        #[test]
        fn test_alloc_within_limit(
            arena_size in 100usize..10_000usize,
            value in 0i32..1000i32,
        ) {
            let arena = SafeArena::new(arena_size);
            let result = arena.alloc(value);

            if std::mem::size_of::<i32>() <= arena_size {
                prop_assert!(result.is_ok());
                let arena_ref = result.unwrap();
                prop_assert_eq!(*arena_ref, value);
            }
        }

        #[test]
        fn test_reset_always_clears(
            values in prop::collection::vec(0i32..1000, 0..10)
        ) {
            let arena = SafeArena::new(10_000);

            for value in values {
                let _ = arena.alloc(value);
            }

            arena.reset();
            prop_assert_eq!(arena.used(), 0);
        }

        #[test]
        fn test_used_monotonic_increase(
            values in prop::collection::vec(0u8..255, 1..20)
        ) {
            let arena = SafeArena::new(10_000);
            let mut prev_used = arena.used();

            for value in values {
                if let Ok(_) = arena.alloc(value) {
                    let current_used = arena.used();
                    prop_assert!(current_used >= prev_used);
                    prev_used = current_used;
                }
            }
        }

        #[test]
        fn test_allocation_preserves_value(
            s in "[a-zA-Z0-9]{0,100}"
        ) {
            let arena = SafeArena::new(10_000);

            if let Ok(arena_ref) = arena.alloc(s.clone()) {
                let deref: &String = &*arena_ref;
                prop_assert_eq!(deref, &s);
            }
        }
    }
}

// Stress tests
#[test]
fn test_many_small_allocations() {
    let arena = SafeArena::new(100_000);
    let mut refs = Vec::new();

    for i in 0..1000 {
        match arena.alloc(i) {
            Ok(r) => refs.push(r),
            Err(_) => break,
        }
    }

    // Verify all values are still accessible
    for (i, r) in refs.iter().enumerate() {
        assert_eq!(**r, i);
    }
}

#[test]
fn test_alternating_alloc_reset() {
    let arena = SafeArena::new(1024);

    for i in 0..100 {
        let result = arena.alloc(i);
        assert!(result.is_ok());

        if i % 10 == 9 {
            arena.reset();
            assert_eq!(arena.used(), 0);
        }
    }
}

// Edge cases
#[test]
fn test_empty_string_allocation() {
    let arena = SafeArena::new(1024);
    let result = arena.alloc(String::new());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "");
}

#[test]
fn test_empty_vec_allocation() {
    let arena = SafeArena::new(1024);
    let result = arena.alloc(Vec::<i32>::new());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), Vec::<i32>::new());
}

#[test]
fn test_unit_type_allocation() {
    let arena = SafeArena::new(1024);
    let result = arena.alloc(());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), ());
}

// Big O Complexity Analysis:
// - new(): O(1) - Simple struct initialization
// - alloc(): O(1) - Constant time allocation with size check
// - reset(): O(n) where n is number of allocations (clearing vector)
// - used(): O(1) - Simple field access
//
// Memory Complexity:
// - O(n) where n is total size of allocated objects
// - Additional O(m) for m allocation metadata entries