//! JIT-007: Tuple Support (Stack-allocated Fixed-size Collections)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Add tuple support to JIT compiler
//! Target: Enable multiple return values and basic data grouping
//!
//! Test Strategy:
//! 1. Tuple literals - creating 2-tuple, 3-tuple
//! 2. Tuple access - reading elements by index (.0, .1, .2)
//! 3. Tuple return values - functions returning tuples
//! 4. Tuple destructuring - let (a, b) = tuple
//! 5. Tuples in algorithms - divmod, coordinate math, etc.

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Tuple Literals (Basic Creation)
// ============================================================================

#[test]
fn test_jit_007_tuple_literal_pair() {
    // Test: Create 2-tuple and access first element
    let code = r"
        let pair = (10, 20);
        pair.0
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile tuple literal: {:?}", result.err());
    assert_eq!(result.unwrap(), 10, "pair.0 should be 10");
}

#[test]
fn test_jit_007_tuple_literal_second_element() {
    // Test: Access second element of tuple
    let code = r"
        let pair = (100, 200);
        pair.1
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should access tuple.1: {:?}", result.err());
    assert_eq!(result.unwrap(), 200, "pair.1 should be 200");
}

#[test]
fn test_jit_007_tuple_literal_triple() {
    // Test: 3-tuple access
    let code = r"
        let triple = (1, 2, 3);
        triple.2
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile 3-tuple: {:?}", result.err());
    assert_eq!(result.unwrap(), 3, "triple.2 should be 3");
}

// ============================================================================
// RED-002: Tuple Access with Expressions
// ============================================================================

#[test]
fn test_jit_007_tuple_access_in_expression() {
    // Test: Use tuple elements in expressions
    let code = r"
        let point = (3, 4);
        point.0 + point.1
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should use tuple in expression: {:?}", result.err());
    assert_eq!(result.unwrap(), 7, "3 + 4 should be 7");
}

#[test]
fn test_jit_007_tuple_access_computed() {
    // Test: Compute with tuple elements
    let code = r"
        let dims = (10, 20);
        dims.0 * dims.1
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compute with tuple: {:?}", result.err());
    assert_eq!(result.unwrap(), 200, "10 * 20 should be 200");
}

// ============================================================================
// RED-003: Functions Returning Tuples
// ============================================================================

#[test]
#[ignore = "Function return tuples require type tracking - deferred to JIT-007B"]
fn test_jit_007_function_return_tuple() {
    // Test: Function returns tuple
    let code = r"
        fun make_pair(a: i32, b: i32) -> (i32, i32) {
            (a, b)
        }
        let result = make_pair(5, 10);
        result.0 + result.1
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should return tuple from function: {:?}", result.err());
    assert_eq!(result.unwrap(), 15, "5 + 10 should be 15");
}

#[test]
#[ignore = "Function return tuples require type tracking - deferred to JIT-007B"]
fn test_jit_007_divmod_algorithm() {
    // Test: Classic divmod algorithm returning (quotient, remainder)
    let code = r"
        fun divmod(a: i32, b: i32) -> (i32, i32) {
            let quotient = a / b;
            let remainder = a % b;
            (quotient, remainder)
        }
        let result = divmod(17, 5);
        result.0 * 10 + result.1
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compute divmod: {:?}", result.err());
    // quotient=3, remainder=2, result = 3*10+2 = 32
    assert_eq!(result.unwrap(), 32, "divmod(17,5) should give (3,2) → 32");
}

// ============================================================================
// RED-004: Tuple Destructuring
// ============================================================================

#[test]
#[ignore = "Tuple destructuring requires LetPattern support - deferred to JIT-007B"]
fn test_jit_007_tuple_destructuring_simple() {
    // Test: Destructure tuple into variables
    let code = r"
        let pair = (42, 84);
        let (a, b) = pair;
        a + b
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should destructure tuple: {:?}", result.err());
    assert_eq!(result.unwrap(), 126, "42 + 84 should be 126");
}

#[test]
#[ignore = "Tuple destructuring + function returns requires LetPattern + type tracking - deferred to JIT-007B"]
fn test_jit_007_tuple_destructuring_from_function() {
    // Test: Destructure tuple returned from function
    let code = r"
        fun swap(a: i32, b: i32) -> (i32, i32) {
            (b, a)
        }
        let (x, y) = swap(10, 20);
        x - y
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should destructure from function: {:?}", result.err());
    assert_eq!(result.unwrap(), 10, "20 - 10 should be 10");
}

// ============================================================================
// RED-005: Tuples in Algorithms
// ============================================================================

#[test]
#[ignore = "Tuple reassignment in loops requires mutable tuple support - deferred to JIT-007B"]
fn test_jit_007_fibonacci_pair() {
    // Test: Fibonacci using tuple state
    let code = r"
        fun fib_n(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                let mut pair = (0, 1);
                let mut i = 2;
                while i <= n {
                    let next = pair.0 + pair.1;
                    pair = (pair.1, next);
                    i = i + 1;
                }
                pair.1
            }
        }
        fib_n(10)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compute fibonacci with tuples: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "fib(10) should be 55");
}

#[test]
#[ignore = "Tuple parameters in functions require type tracking - deferred to JIT-007B"]
fn test_jit_007_coordinate_distance() {
    // Test: 2D point distance using tuples
    let code = r"
        fun distance_squared(p1: (i32, i32), p2: (i32, i32)) -> i32 {
            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            dx * dx + dy * dy
        }
        let point1 = (0, 0);
        let point2 = (3, 4);
        distance_squared(point1, point2)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compute distance: {:?}", result.err());
    assert_eq!(result.unwrap(), 25, "3²+4² should be 25");
}

#[test]
#[ignore = "Function return tuples require type tracking - deferred to JIT-007B"]
fn test_jit_007_min_max_pair() {
    // Test: Return both min and max in one pass
    let code = r"
        fun min_max(a: i32, b: i32, c: i32) -> (i32, i32) {
            let mut min_val = a;
            let mut max_val = a;

            if b < min_val {
                min_val = b;
            }
            if b > max_val {
                max_val = b;
            }

            if c < min_val {
                min_val = c;
            }
            if c > max_val {
                max_val = c;
            }

            (min_val, max_val)
        }
        let result = min_max(5, 2, 8);
        result.1 - result.0
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should find min/max: {:?}", result.err());
    assert_eq!(result.unwrap(), 6, "max(8) - min(2) should be 6");
}
