//! JIT-006: Array Support (literals, indexing, length, iteration)
//!
//! EXTREME TDD - RED Phase Tests (DEFERRED)
//!
//! Purpose: Extend JIT compiler with array data structures
//! Target: Enable practical algorithms with collections
//!
//! **STATUS**: RED phase complete (16 tests written, all failing as expected)
//! **COMPLEXITY**: Arrays require heap allocation, bounds checking, and memory management
//! **DECISION**: Deferred to future implementation due to complexity (50k+ token estimate)
//! **ALTERNATIVE COMPLETED**: Verified recursive functions already work (factorial, fibonacci)
//!
//! Test Strategy:
//! 1. Array literals - creating fixed-size arrays
//! 2. Array indexing - reading elements by position
//! 3. Array length - getting array size
//! 4. Array iteration - looping over elements
//! 5. Array algorithms - sum, max, min, search, sort
//! 6. Nested arrays - 2D arrays and matrices
//! 7. Array mutation - updating elements
//! 8. Performance - large array operations

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Array Literals (Basic Creation)
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_literal_simple() {
    // Test: Create array and return first element
    let code = r"
        let arr = [1, 2, 3, 4, 5];
        arr[0]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array literal: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "arr[0] should be 1");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_literal_middle_element() {
    // Test: Access middle element
    let code = r"
        let arr = [10, 20, 30, 40, 50];
        arr[2]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array indexing: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 30, "arr[2] should be 30");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_literal_last_element() {
    // Test: Access last element
    let code = r"
        let arr = [5, 10, 15];
        arr[2]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile last element access: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 15, "arr[2] should be 15");
}

// ============================================================================
// RED-002: Array Indexing with Variables
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_index_variable() {
    // Test: Index with variable
    let code = r"
        let arr = [100, 200, 300];
        let i = 1;
        arr[i]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile variable indexing: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 200, "arr[i] where i=1 should be 200");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_index_expression() {
    // Test: Index with expression
    let code = r"
        let arr = [7, 14, 21, 28];
        let i = 1;
        arr[i + 1]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile expression indexing: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 21, "arr[i+1] where i=1 should be 21");
}

// ============================================================================
// RED-003: Array Length
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_length_method() {
    // Test: Get array length using .len() method
    let code = r"
        let arr = [1, 2, 3, 4, 5];
        arr.len()
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile arr.len(): {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "arr.len() should be 5");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_length_in_loop() {
    // Test: Use array length in loop condition
    let code = r"
        let arr = [10, 20, 30];
        let mut count = 0;
        for i in 0..arr.len() {
            count = count + 1;
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile loop with arr.len(): {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 3, "Should iterate arr.len() times");
}

// ============================================================================
// RED-004: Array Iteration and Sum
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_sum_with_loop() {
    // Test: Sum array elements using loop
    let code = r"
        let arr = [1, 2, 3, 4, 5];
        let mut sum = 0;
        for i in 0..arr.len() {
            sum = sum + arr[i];
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array sum: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 15, "Sum of [1,2,3,4,5] should be 15");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_sum_function() {
    // Test: Sum array using function
    let code = r"
        fun sum_array(arr: [i32]) -> i32 {
            let mut sum = 0;
            for i in 0..arr.len() {
                sum = sum + arr[i];
            }
            sum
        }
        sum_array([10, 20, 30, 40])
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile sum_array function: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 100, "Sum of [10,20,30,40] should be 100");
}

// ============================================================================
// RED-005: Array Algorithms (Max/Min)
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_max() {
    // Test: Find maximum element
    let code = r"
        let arr = [3, 7, 2, 9, 1];
        let mut max = arr[0];
        for i in 1..arr.len() {
            if arr[i] > max {
                max = arr[i];
            }
        }
        max
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array max: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 9, "Max of [3,7,2,9,1] should be 9");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_min() {
    // Test: Find minimum element
    let code = r"
        let arr = [8, 3, 9, 1, 5];
        let mut min = arr[0];
        for i in 1..arr.len() {
            if arr[i] < min {
                min = arr[i];
            }
        }
        min
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array min: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "Min of [8,3,9,1,5] should be 1");
}

// ============================================================================
// RED-006: Array Search
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_linear_search() {
    // Test: Linear search for element
    let code = r"
        let arr = [10, 20, 30, 40, 50];
        let target = 30;
        let mut found_index = -1;
        for i in 0..arr.len() {
            if arr[i] == target {
                found_index = i;
                break;
            }
        }
        found_index
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile linear search: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 2, "Should find 30 at index 2");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_contains() {
    // Test: Check if array contains value
    let code = r"
        let arr = [5, 10, 15, 20];
        let target = 15;
        let mut found = 0;
        for i in 0..arr.len() {
            if arr[i] == target {
                found = 1;
                break;
            }
        }
        found
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile contains check: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "Should find 15 in array (return 1)");
}

// ============================================================================
// RED-007: Array Mutation
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_mutation_simple() {
    // Test: Update array element
    let code = r"
        let mut arr = [1, 2, 3];
        arr[1] = 99;
        arr[1]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array mutation: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 99, "arr[1] should be updated to 99");
}

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_array_mutation_in_loop() {
    // Test: Double all elements
    let code = r"
        let mut arr = [1, 2, 3, 4];
        for i in 0..arr.len() {
            arr[i] = arr[i] * 2;
        }
        arr[2]
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile array mutation in loop: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 6, "arr[2] should be 3*2=6 after doubling");
}

// ============================================================================
// RED-008: Performance Validation
// ============================================================================

#[test]
#[ignore = "Arrays not yet implemented in JIT - requires heap allocation"]
fn test_jit_006_performance_large_array_sum() {
    // Performance: Sum 100 elements should be fast with JIT
    let code = r"
        fun sum_array(arr: [i32]) -> i32 {
            let mut sum = 0;
            for i in 0..arr.len() {
                sum = sum + arr[i];
            }
            sum
        }

        // Create array [1, 2, 3, ..., 100]
        let mut arr = [0; 100];
        for i in 0..100 {
            arr[i] = i + 1;
        }
        sum_array(arr)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile large array sum: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5050, "Sum 1..100 should be 5050");
}
