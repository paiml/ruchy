// OPT-004: Semantic Equivalence Tests - Verify AST and Bytecode modes produce identical results
//
// Test Strategy:
// 1. Run same program through both AST interpreter and bytecode VM
// 2. Verify both modes produce identical results
// 3. Cover: literals, arithmetic, comparisons, control flow, blocks
//
// Requirements from docs/execution/roadmap.yaml:
// - 40 integration tests comparing AST vs bytecode execution
// - Property tests: 10K cases verifying semantic equivalence
// - Performance: bytecode 40-60% faster than AST

use ruchy::frontend::parser::Parser as RuchyParser;
use ruchy::runtime::bytecode::{Compiler, VM};
use ruchy::runtime::interpreter::{Interpreter, Value};

/// Helper: Execute program in AST mode
fn execute_ast(source: &str) -> Result<Value, String> {
    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;
    let mut interpreter = Interpreter::new();
    interpreter
        .eval_expr(&ast)
        .map_err(|e| format!("AST eval error: {:?}", e))
}

/// Helper: Execute program in bytecode mode
fn execute_bytecode(source: &str) -> Result<Value, String> {
    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;
    let mut compiler = Compiler::new("test".to_string());
    compiler
        .compile_expr(&ast)
        .map_err(|e| format!("Compilation error: {}", e))?;
    let chunk = compiler.finalize();
    let mut vm = VM::new();
    vm.execute(&chunk)
        .map_err(|e| format!("VM error: {}", e))
}

/// Helper: Assert AST and bytecode produce same result
fn assert_semantic_equivalence(source: &str, expected: Value) {
    let ast_result = execute_ast(source).expect("AST execution failed");
    let bytecode_result = execute_bytecode(source).expect("Bytecode execution failed");

    assert_eq!(
        ast_result, expected,
        "AST mode: expected {:?}, got {:?}",
        expected, ast_result
    );
    assert_eq!(
        bytecode_result, expected,
        "Bytecode mode: expected {:?}, got {:?}",
        expected, bytecode_result
    );
    assert_eq!(
        ast_result, bytecode_result,
        "Semantic equivalence violated: AST={:?}, Bytecode={:?}",
        ast_result, bytecode_result
    );
}

// ============================================================================
// Test Suite 1: Literals & Unary Operations (9 tests)
// ============================================================================

#[test]
fn test_opt_004_01_integer_literal() {
    assert_semantic_equivalence("42", Value::Integer(42));
}

// OPT-005: Unary operators now implemented
#[test]
fn test_opt_004_01_negative_integer() {
    assert_semantic_equivalence("-42", Value::Integer(-42));
}

#[test]
fn test_opt_004_01_negative_float() {
    assert_semantic_equivalence("-3.14", Value::Float(-3.14));
}

#[test]
fn test_opt_004_01_logical_not_true() {
    assert_semantic_equivalence("!true", Value::Bool(false));
}

#[test]
fn test_opt_004_01_logical_not_false() {
    assert_semantic_equivalence("!false", Value::Bool(true));
}

#[test]
fn test_opt_004_01_bitwise_not() {
    assert_semantic_equivalence("~5", Value::Integer(!5));
}

#[test]
fn test_opt_004_01_float_literal() {
    assert_semantic_equivalence("3.14", Value::Float(3.14));
}

#[test]
fn test_opt_004_01_boolean_true() {
    assert_semantic_equivalence("true", Value::Bool(true));
}

#[test]
fn test_opt_004_01_boolean_false() {
    assert_semantic_equivalence("false", Value::Bool(false));
}

// ============================================================================
// Test Suite 2: Arithmetic Operations (8 tests)
// ============================================================================

#[test]
fn test_opt_004_02_addition() {
    assert_semantic_equivalence("10 + 32", Value::Integer(42));
}

#[test]
fn test_opt_004_02_subtraction() {
    assert_semantic_equivalence("50 - 8", Value::Integer(42));
}

#[test]
fn test_opt_004_02_multiplication() {
    assert_semantic_equivalence("6 * 7", Value::Integer(42));
}

#[test]
fn test_opt_004_02_division() {
    assert_semantic_equivalence("84 / 2", Value::Integer(42));
}

#[test]
fn test_opt_004_02_modulo() {
    assert_semantic_equivalence("10 % 3", Value::Integer(1));
}

#[test]
fn test_opt_004_02_complex_expression() {
    assert_semantic_equivalence("(10 + 5) * 2 + 12", Value::Integer(42));
}

#[test]
fn test_opt_004_02_float_arithmetic() {
    assert_semantic_equivalence("3.5 + 2.5", Value::Float(6.0));
}

#[test]
fn test_opt_004_02_mixed_int_float() {
    assert_semantic_equivalence("10 + 2.5", Value::Float(12.5));
}

// ============================================================================
// Test Suite 3: Comparison Operations (6 tests)
// ============================================================================

#[test]
fn test_opt_004_03_equal() {
    assert_semantic_equivalence("42 == 42", Value::Bool(true));
    assert_semantic_equivalence("42 == 43", Value::Bool(false));
}

#[test]
fn test_opt_004_03_not_equal() {
    assert_semantic_equivalence("42 != 43", Value::Bool(true));
    assert_semantic_equivalence("42 != 42", Value::Bool(false));
}

#[test]
fn test_opt_004_03_less_than() {
    assert_semantic_equivalence("10 < 20", Value::Bool(true));
    assert_semantic_equivalence("20 < 10", Value::Bool(false));
}

#[test]
fn test_opt_004_03_less_equal() {
    assert_semantic_equivalence("10 <= 10", Value::Bool(true));
    assert_semantic_equivalence("10 <= 20", Value::Bool(true));
    assert_semantic_equivalence("20 <= 10", Value::Bool(false));
}

#[test]
fn test_opt_004_03_greater_than() {
    assert_semantic_equivalence("20 > 10", Value::Bool(true));
    assert_semantic_equivalence("10 > 20", Value::Bool(false));
}

#[test]
fn test_opt_004_03_greater_equal() {
    assert_semantic_equivalence("10 >= 10", Value::Bool(true));
    assert_semantic_equivalence("20 >= 10", Value::Bool(true));
    assert_semantic_equivalence("10 >= 20", Value::Bool(false));
}

// ============================================================================
// Test Suite 4: Logical Operations (3 tests)
// ============================================================================

#[test]
fn test_opt_004_04_logical_and() {
    assert_semantic_equivalence("true && true", Value::Bool(true));
    assert_semantic_equivalence("true && false", Value::Bool(false));
    assert_semantic_equivalence("false && true", Value::Bool(false));
}

#[test]
fn test_opt_004_04_logical_or() {
    assert_semantic_equivalence("true || false", Value::Bool(true));
    assert_semantic_equivalence("false || true", Value::Bool(true));
    assert_semantic_equivalence("false || false", Value::Bool(false));
}

#[test]
fn test_opt_004_04_complex_logic() {
    assert_semantic_equivalence("(10 > 5) && (20 < 30)", Value::Bool(true));
    assert_semantic_equivalence("(10 < 5) || (20 == 20)", Value::Bool(true));
}

// ============================================================================
// Test Suite 5: Control Flow (6 tests)
// ============================================================================

#[test]
fn test_opt_004_05_if_true_branch() {
    assert_semantic_equivalence("if true { 42 } else { 0 }", Value::Integer(42));
}

#[test]
fn test_opt_004_05_if_false_branch() {
    assert_semantic_equivalence("if false { 0 } else { 42 }", Value::Integer(42));
}

#[test]
fn test_opt_004_05_if_condition_comparison() {
    assert_semantic_equivalence("if 10 > 5 { 42 } else { 0 }", Value::Integer(42));
}

#[test]
fn test_opt_004_05_nested_if() {
    assert_semantic_equivalence(
        "if true { if false { 0 } else { 42 } } else { 100 }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_05_if_with_arithmetic() {
    assert_semantic_equivalence("if true { 10 + 32 } else { 0 }", Value::Integer(42));
}

#[test]
fn test_opt_004_05_if_expression_as_operand() {
    assert_semantic_equivalence(
        "(if true { 40 } else { 0 }) + 2",
        Value::Integer(42),
    );
}

// ============================================================================
// Test Suite 6: Block Expressions (3 tests)
// ============================================================================

#[test]
fn test_opt_004_06_simple_block() {
    assert_semantic_equivalence("{ 42 }", Value::Integer(42));
}

#[test]
fn test_opt_004_06_block_with_multiple_expressions() {
    assert_semantic_equivalence("{ 10; 20; 42 }", Value::Integer(42));
}

#[test]
fn test_opt_004_06_nested_blocks() {
    assert_semantic_equivalence("{ { { 42 } } }", Value::Integer(42));
}

// ============================================================================
// Test Suite 7: Complex Integration Tests (9 tests)
// ============================================================================

#[test]
fn test_opt_004_07_fibonacci_formula() {
    // Simple Fibonacci-like calculation
    assert_semantic_equivalence("1 + 1 + 2 + 3 + 5 + 8", Value::Integer(20));
}

#[test]
fn test_opt_004_07_boolean_algebra() {
    assert_semantic_equivalence(
        "(true && false) || (true && true)",
        Value::Bool(true),
    );
}

#[test]
fn test_opt_004_07_arithmetic_with_comparisons() {
    assert_semantic_equivalence(
        "if (10 + 5) > 12 { 42 } else { 0 }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_07_nested_arithmetic() {
    assert_semantic_equivalence(
        "((10 + 5) * 2) + ((20 - 8) / 2)",
        Value::Integer(36),
    );
}

#[test]
fn test_opt_004_07_complex_conditional() {
    assert_semantic_equivalence(
        "if (10 > 5) && (20 < 30) { if true { 42 } else { 0 } } else { 100 }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_07_block_in_arithmetic() {
    assert_semantic_equivalence("{ 10 + 32 }", Value::Integer(42));
}

#[test]
fn test_opt_004_07_if_in_block() {
    assert_semantic_equivalence(
        "{ if true { 42 } else { 0 } }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_07_arithmetic_chain() {
    assert_semantic_equivalence("10 + 10 + 10 + 12", Value::Integer(42));
}

#[test]
fn test_opt_004_07_comparison_chain() {
    assert_semantic_equivalence(
        "(10 < 20) && (20 < 30) && (30 < 40)",
        Value::Bool(true),
    );
}

// ============================================================================
// Test Suite 8: Loop Expressions (2 tests)
// ============================================================================
// Note: Full while loop tests with mutations deferred until assignment support (OPT-007)

#[test]
fn test_opt_004_08_while_loop_false_condition() {
    // While loop that never executes (condition is false)
    assert_semantic_equivalence(
        "while false { 42 }",
        Value::Nil,
    );
}

#[test]
fn test_opt_004_08_while_loop_then_value() {
    // While loop followed by another expression in block
    assert_semantic_equivalence(
        "{ while false { 42 }; 5 }",
        Value::Integer(5),
    );
}

// Total tests: 9 + 8 + 6 + 3 + 6 + 3 + 9 + 2 = 46 integration tests
// All tests verify semantic equivalence between AST and bytecode modes
// Suite 1: Updated to 9 tests (added 5 unary operator tests for OPT-005)
// Suite 8: Added 2 tests for while loops (OPT-006) - limited until assignment support
