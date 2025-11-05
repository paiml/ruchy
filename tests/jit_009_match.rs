//! JIT-009: Match Expressions (Pattern Matching)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Add match expression support to JIT compiler
//! Target: Enable pattern matching, exhaustive checks, and Rust-style control flow
//!
//! Test Strategy:
//! 1. Literal patterns - match specific values
//! 2. Wildcard pattern - default case with _
//! 3. Multiple arms - several patterns
//! 4. Match values - match on variables and expressions
//! 5. Match in functions - using match for logic
//! 6. Match with return - early exits
//! 7. Complex patterns - realistic use cases

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Literal Pattern Matching
// ============================================================================

#[test]
fn test_jit_009_match_literal_single() {
    // Test: Match single literal value
    let code = r"
        let x = 5;
        match x {
            5 => 100,
            _ => 0,
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match literal: {:?}", result.err());
    assert_eq!(result.unwrap(), 100, "x=5 should match first arm");
}

#[test]
fn test_jit_009_match_literal_wildcard() {
    // Test: Wildcard pattern matches anything
    let code = r"
        let x = 99;
        match x {
            5 => 100,
            _ => 200,
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match wildcard: {:?}", result.err());
    assert_eq!(result.unwrap(), 200, "x=99 should match wildcard");
}

#[test]
fn test_jit_009_match_multiple_literals() {
    // Test: Multiple literal patterns
    let code = r"
        let x = 3;
        match x {
            1 => 10,
            2 => 20,
            3 => 30,
            4 => 40,
            _ => 0,
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile multiple patterns: {:?}", result.err());
    assert_eq!(result.unwrap(), 30, "x=3 should match third arm");
}

// ============================================================================
// RED-002: Match with Expressions
// ============================================================================

#[test]
fn test_jit_009_match_expression_arms() {
    // Test: Arms can be expressions, not just literals
    let code = r"
        let x = 2;
        match x {
            1 => 5 + 5,
            2 => 10 * 3,
            _ => 0,
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match with expressions: {:?}", result.err());
    assert_eq!(result.unwrap(), 30, "x=2 should evaluate to 10*3=30");
}

#[test]
fn test_jit_009_match_variable() {
    // Test: Match on variable value
    let code = r"
        let value = 7;
        let result = match value {
            5 => 1,
            7 => 2,
            10 => 3,
            _ => 0,
        };
        result
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match on variable: {:?}", result.err());
    assert_eq!(result.unwrap(), 2, "value=7 should match second arm");
}

// ============================================================================
// RED-003: Match in Functions
// ============================================================================

#[test]
fn test_jit_009_match_in_function() {
    // Test: Match as function body
    let code = r"
        fun classify(n: i32) -> i32 {
            match n {
                0 => 0,
                1 => 10,
                2 => 20,
                _ => 99,
            }
        }
        classify(1)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match in function: {:?}", result.err());
    assert_eq!(result.unwrap(), 10, "classify(1) should return 10");
}

#[test]
fn test_jit_009_match_fizzbuzz_style() {
    // Test: FizzBuzz-style matching
    let code = r"
        fun fizzbuzz_check(n: i32) -> i32 {
            match n % 15 {
                0 => 3,
                _ => {
                    match n % 3 {
                        0 => 1,
                        _ => {
                            match n % 5 {
                                0 => 2,
                                _ => 0,
                            }
                        }
                    }
                }
            }
        }
        fizzbuzz_check(15)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile nested match: {:?}", result.err());
    assert_eq!(result.unwrap(), 3, "15 % 15 = 0 should return 3 (FizzBuzz)");
}

// ============================================================================
// RED-004: Match with Return
// ============================================================================

#[test]
fn test_jit_009_match_with_return() {
    // Test: Return from match arm
    let code = r"
        fun find_category(code: i32) -> i32 {
            match code {
                100 => return 1,
                200 => return 2,
                300 => return 3,
                _ => return 0,
            }
        }
        find_category(200)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match with return: {:?}", result.err());
    assert_eq!(result.unwrap(), 2, "code=200 should return 2");
}

// ============================================================================
// RED-005: Match Algorithms
// ============================================================================

#[test]
fn test_jit_009_match_sign_function() {
    // Test: Sign function using match
    let code = r"
        fun sign(x: i32) -> i32 {
            if x < 0 {
                return -1;
            }
            if x > 0 {
                return 1;
            }
            return 0;
        }

        fun test_sign(n: i32) -> i32 {
            let s = sign(n);
            match s {
                -1 => 100,
                0 => 200,
                1 => 300,
                _ => 0,
            }
        }
        test_sign(5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile sign function: {:?}", result.err());
    assert_eq!(result.unwrap(), 300, "sign(5) = 1 should match arm 300");
}

#[test]
fn test_jit_009_match_grade_calculator() {
    // Test: Grade calculator using match
    let code = r"
        fun letter_grade(score: i32) -> i32 {
            let grade = score / 10;
            match grade {
                10 => 4,
                9 => 4,
                8 => 3,
                7 => 2,
                6 => 1,
                _ => 0,
            }
        }
        letter_grade(85)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile grade calculator: {:?}", result.err());
    assert_eq!(result.unwrap(), 3, "85/10=8 should match grade 3 (B)");
}

#[test]
fn test_jit_009_match_day_of_week() {
    // Test: Day of week matcher
    let code = r"
        fun is_weekend(day: i32) -> i32 {
            match day {
                0 => 1,
                6 => 1,
                _ => 0,
            }
        }
        is_weekend(6)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile day matcher: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "day=6 (Saturday) should be weekend");
}

// ============================================================================
// RED-006: Match with Complex Control Flow
// ============================================================================

#[test]
fn test_jit_009_match_in_loop() {
    // Test: Match inside loop
    let code = r"
        fun sum_categories(limit: i32) -> i32 {
            let mut sum = 0;
            for i in 0..limit {
                let category = match i % 3 {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                };
                sum = sum + category;
            }
            sum
        }
        sum_categories(9)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile match in loop: {:?}", result.err());
    // 0%3=0→1, 1%3=1→2, 2%3=2→3, 3%3=0→1, 4%3=1→2, 5%3=2→3, 6%3=0→1, 7%3=1→2, 8%3=2→3
    // sum = 1+2+3+1+2+3+1+2+3 = 18
    assert_eq!(result.unwrap(), 18, "Sum should be 18");
}

#[test]
fn test_jit_009_match_state_machine() {
    // Test: Simple state machine using match
    let code = r"
        fun state_machine(state: i32, input: i32) -> i32 {
            match state {
                0 => {
                    if input == 1 {
                        return 1;
                    }
                    return 0;
                }
                1 => {
                    if input == 2 {
                        return 2;
                    }
                    return 0;
                }
                2 => {
                    return 3;
                }
                _ => 0,
            }
        }
        state_machine(1, 2)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile state machine: {:?}", result.err());
    assert_eq!(result.unwrap(), 2, "State 1 with input 2 should transition to state 2");
}
