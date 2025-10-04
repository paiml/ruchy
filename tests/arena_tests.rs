//! Tests for arena memory allocation module

use ruchy::frontend::arena::Arena;
use std::rc::Rc;

#[test]
fn test_arena_creation() {
    let arena = Arena::new();
    assert_eq!(arena.num_items(), 0);
    assert_eq!(arena.total_allocated(), 0);
}

#[test]
fn test_arena_alloc_single() {
    let arena = Arena::new();
    let value: Rc<i32> = arena.alloc(42);

    assert_eq!(arena.num_items(), 1);
    assert_eq!(arena.total_allocated(), 1);
    assert_eq!(*value, 42);
}

#[test]
fn test_arena_alloc_multiple() {
    let arena = Arena::new();

    let v1: Rc<i32> = arena.alloc(1);
    let v2: Rc<i32> = arena.alloc(2);
    let v3: Rc<i32> = arena.alloc(3);

    assert_eq!(arena.num_items(), 3);
    assert_eq!(arena.total_allocated(), 3);
    assert_eq!(*v1, 1);
    assert_eq!(*v2, 2);
    assert_eq!(*v3, 3);
}

#[test]
fn test_arena_strings() {
    let arena = Arena::new();

    let s1: Rc<String> = arena.alloc("hello".to_string());
    let s2: Rc<String> = arena.alloc("world".to_string());

    assert_eq!(&**s1, "hello");
    assert_eq!(&**s2, "world");
    assert_eq!(arena.num_items(), 2);
}

#[test]
fn test_arena_clear() {
    let arena = Arena::new();

    arena.alloc(1);
    arena.alloc(2);
    arena.alloc(3);

    assert_eq!(arena.num_items(), 3);
    assert_eq!(arena.total_allocated(), 3);

    arena.clear();

    assert_eq!(arena.num_items(), 0);
    assert_eq!(arena.total_allocated(), 0);
}

#[test]
fn test_arena_mixed_types() {
    #[derive(Debug, PartialEq)]
    struct TestStruct {
        x: i32,
        y: String,
    }

    let arena = Arena::new();

    let s1 = TestStruct {
        x: 42,
        y: "test".to_string(),
    };
    let s2 = TestStruct {
        x: 100,
        y: "hello".to_string(),
    };

    let rc1: Rc<TestStruct> = arena.alloc(s1);
    let rc2: Rc<TestStruct> = arena.alloc(s2);

    assert_eq!(rc1.x, 42);
    assert_eq!(rc1.y, "test");
    assert_eq!(rc2.x, 100);
    assert_eq!(rc2.y, "hello");
}

#[test]
fn test_arena_rc_sharing() {
    let arena = Arena::new();

    let value: Rc<i32> = arena.alloc(100);
    let clone1 = Rc::clone(&value);
    let clone2 = Rc::clone(&value);

    // All clones point to the same value
    assert_eq!(*value, 100);
    assert_eq!(*clone1, 100);
    assert_eq!(*clone2, 100);

    // Strong count should be 4 (original in arena + value + 2 clones)
    assert_eq!(Rc::strong_count(&value), 4);
}

#[test]
fn test_arena_large_allocation() {
    let arena = Arena::new();
    let mut values = Vec::new();

    // Allocate many items
    for i in 0..1000 {
        values.push(arena.alloc(i));
    }

    assert_eq!(arena.num_items(), 1000);
    assert_eq!(arena.total_allocated(), 1000);

    // Check all values
    for (i, value) in values.iter().enumerate() {
        assert_eq!(**value, i);
    }
}

#[test]
fn test_arena_with_vec() {
    let arena = Arena::new();

    let v1: Rc<Vec<i32>> = arena.alloc(vec![1, 2, 3]);
    let v2: Rc<Vec<i32>> = arena.alloc(vec![4, 5, 6, 7]);

    assert_eq!(v1.len(), 3);
    assert_eq!(v2.len(), 4);
    assert_eq!(v1[0], 1);
    assert_eq!(v2[3], 7);
}

#[test]
fn test_arena_zero_sized_types() {
    let arena = Arena::new();

    let _u1: Rc<()> = arena.alloc(());
    let _u2: Rc<()> = arena.alloc(());

    assert_eq!(arena.num_items(), 2);
    assert_eq!(arena.total_allocated(), 2);
}

#[test]
fn test_arena_default() {
    let arena = Arena::default();
    assert_eq!(arena.num_items(), 0);
    assert_eq!(arena.total_allocated(), 0);
}

#[test]
fn test_arena_reuse_after_clear() {
    let arena = Arena::new();

    // First allocation phase
    arena.alloc(1);
    arena.alloc(2);
    assert_eq!(arena.total_allocated(), 2);

    // Clear
    arena.clear();
    assert_eq!(arena.total_allocated(), 0);

    // Second allocation phase
    arena.alloc(3);
    arena.alloc(4);
    arena.alloc(5);
    assert_eq!(arena.total_allocated(), 3);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_arena_alloc_never_panics(values: Vec<i32>) {
            let arena = Arena::new();
            let mut rcs = Vec::new();

            for value in values {
                rcs.push(arena.alloc(value));
            }

            prop_assert_eq!(arena.num_items(), rcs.len());
        }

        #[test]
        fn prop_arena_clear_resets(ops in prop::collection::vec(0i32..1000, 0..100)) {
            let arena = Arena::new();

            for op in ops {
                arena.alloc(op);
            }

            arena.clear();
            prop_assert_eq!(arena.num_items(), 0);
            prop_assert_eq!(arena.total_allocated(), 0);
        }

        #[test]
        fn prop_arena_rc_values_stable(values: Vec<u32>) {
            let arena = Arena::new();
            let mut rcs = Vec::new();

            for (i, value) in values.iter().enumerate() {
                let rc = arena.alloc(*value);
                rcs.push(rc);

                // Previously allocated values should remain unchanged
                for (j, prev_rc) in rcs[..i].iter().enumerate() {
                    prop_assert_eq!(**prev_rc, values[j]);
                }
            }
        }
    }
}
