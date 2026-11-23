//! JIT-008: Return Statement (Early Function Exits)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Add return statement support to JIT compiler
//! Target: Enable early exits, guard clauses, and natural control flow
//!
//! Test Strategy:
//! 1. Simple return - explicit return value
//! 2. Early return - guard clause patterns
//! 3. Return in conditionals - different branches
//! 4. Return in loops - search patterns
//! 5. Multiple returns - complex control flow
//! 6. Nested functions - return from inner functions

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Simple Return Statement
// ============================================================================

#[test]
fn test_jit_008_return_simple() {
    // Test: Explicit return from function
    let code = r"
        fun get_value() -> i32 {
            return 42;
        }
        get_value()
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile simple return: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 42, "Function should return 42");
}

#[test]
fn test_jit_008_return_expression() {
    // Test: Return with expression
    let code = r"
        fun calculate(x: i32) -> i32 {
            return x * 2 + 10;
        }
        calculate(5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return with expression: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 20, "5*2+10 should be 20");
}

// ============================================================================
// RED-002: Early Return (Guard Clauses)
// ============================================================================

#[test]
fn test_jit_008_return_early_guard() {
    // Test: Guard clause - early return on invalid input
    let code = r"
        fun safe_divide(a: i32, b: i32) -> i32 {
            if b == 0 {
                return -1;
            }
            a / b
        }
        safe_divide(10, 0)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile guard clause: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), -1, "Should return -1 for division by zero");
}

#[test]
fn test_jit_008_return_early_success() {
    // Test: Guard clause - normal path when guard doesn't trigger
    let code = r"
        fun safe_divide(a: i32, b: i32) -> i32 {
            if b == 0 {
                return -1;
            }
            a / b
        }
        safe_divide(10, 2)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile guard success path: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "10/2 should be 5");
}

#[test]
fn test_jit_008_return_multiple_guards() {
    // Test: Multiple guard clauses
    let code = r"
        fun validate_range(x: i32) -> i32 {
            if x < 0 {
                return -1;
            }
            if x > 100 {
                return -2;
            }
            x
        }
        validate_range(150)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile multiple guards: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), -2, "Should return -2 for x>100");
}

// ============================================================================
// RED-003: Return in Conditionals
// ============================================================================

#[test]
fn test_jit_008_return_if_else_both_branches() {
    // Test: Return in both branches of if/else
    let code = r"
        fun abs_value(x: i32) -> i32 {
            if x < 0 {
                return -x;
            } else {
                return x;
            }
        }
        abs_value(-5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return in both branches: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "abs(-5) should be 5");
}

#[test]
fn test_jit_008_return_nested_if() {
    // Test: Return in nested conditionals
    let code = r"
        fun classify(x: i32) -> i32 {
            if x > 0 {
                if x > 10 {
                    return 2;
                }
                return 1;
            }
            return 0;
        }
        classify(15)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile nested if returns: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 2, "classify(15) should be 2 (>10)");
}

// ============================================================================
// RED-004: Return in Loops (Search Patterns)
// ============================================================================

#[test]
fn test_jit_008_return_in_while_loop() {
    // Test: Return from inside while loop (search pattern)
    let code = r"
        fun find_first_even(start: i32, end: i32) -> i32 {
            let mut i = start;
            while i < end {
                if i % 2 == 0 {
                    return i;
                }
                i = i + 1;
            }
            return -1;
        }
        find_first_even(5, 10)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return in while: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 6, "First even in [5,10) should be 6");
}

#[test]
fn test_jit_008_return_in_for_loop() {
    // Test: Return from inside for loop
    let code = r"
        fun find_target(target: i32) -> i32 {
            for i in 0..20 {
                if i * i == target {
                    return i;
                }
            }
            return -1;
        }
        find_target(64)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return in for: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 8, "sqrt(64) should be 8");
}

#[test]
fn test_jit_008_return_nested_loops() {
    // Test: Return from nested loops
    let code = r"
        fun find_pair_sum(target: i32) -> i32 {
            for i in 0..10 {
                for j in 0..10 {
                    if i + j == target {
                        return i * 100 + j;
                    }
                }
            }
            return -1;
        }
        find_pair_sum(7)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return in nested loops: {:?}",
        result.err()
    );
    // First pair that sums to 7: i=0,j=7 → 007, or i=1,j=6 → 106, etc.
    // Should find i=0, j=7 first
    assert_eq!(
        result.unwrap(),
        7,
        "First pair summing to 7 should be (0,7)"
    );
}

// ============================================================================
// RED-005: Multiple Return Points
// ============================================================================

#[test]
fn test_jit_008_multiple_returns_complex() {
    // Test: Function with many return points
    let code = r"
        fun grade(score: i32) -> i32 {
            if score >= 90 {
                return 4;
            }
            if score >= 80 {
                return 3;
            }
            if score >= 70 {
                return 2;
            }
            if score >= 60 {
                return 1;
            }
            return 0;
        }
        grade(85)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile multiple returns: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 3, "Grade for 85 should be 3 (B)");
}

// ============================================================================
// RED-006: Return vs Break/Continue
// ============================================================================

#[test]
fn test_jit_008_return_vs_break() {
    // Test: Return exits function, break exits loop
    let code = r"
        fun test_return() -> i32 {
            let mut count_return = 0;
            for i in 0..10 {
                if i == 3 {
                    return 100;
                }
                count_return = count_return + 1;
            }
            return count_return;
        }

        fun test_break() -> i32 {
            let mut count_break = 0;
            for i in 0..10 {
                if i == 3 {
                    break;
                }
                count_break = count_break + 1;
            }
            return count_break;
        }

        test_return() * 10 + test_break()
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile return vs break: {:?}",
        result.err()
    );
    // test_return() = 100 (returns immediately at i=3, count_return still 3)
    // test_break() = 3 (breaks at i=3, count_break=3, returns 3)
    // result = 100*10 + 3 = 1003
    assert_eq!(result.unwrap(), 1003, "return=100, break=3 → 1003");
}

// ============================================================================
// RED-007: Algorithms Using Return
// ============================================================================

#[test]
fn test_jit_008_is_prime_algorithm() {
    // Test: Prime checking using early return
    let code = r"
        fun is_prime(n: i32) -> i32 {
            if n <= 1 {
                return 0;
            }
            if n <= 3 {
                return 1;
            }
            let mut i = 2;
            while i * i <= n {
                if n % i == 0 {
                    return 0;
                }
                i = i + 1;
            }
            return 1;
        }
        is_prime(17)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile prime checker: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "17 is prime, should return 1");
}

#[test]
fn test_jit_008_binary_search_pattern() {
    // Test: Binary search using early return (simplified)
    let code = r"
        fun binary_search_iterative(target: i32) -> i32 {
            let mut low = 0;
            let mut high = 100;
            while low <= high {
                let mid = (low + high) / 2;
                if mid == target {
                    return mid;
                }
                if mid < target {
                    low = mid + 1;
                } else {
                    high = mid - 1;
                }
            }
            return -1;
        }
        binary_search_iterative(42)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile binary search: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 42, "Should find target 42");
}
