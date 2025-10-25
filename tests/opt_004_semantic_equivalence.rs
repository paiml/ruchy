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
// Test Suite 8: Loop Expressions (7 tests)
// ============================================================================

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

#[test]
fn test_opt_004_08_while_loop_with_counter() {
    // While loop with mutation - simple counter
    assert_semantic_equivalence(
        "{ let mut i = 0; while i < 3 { i = i + 1 }; i }",
        Value::Integer(3),
    );
}

#[test]
fn test_opt_004_08_while_loop_with_accumulator() {
    // While loop with accumulator pattern
    assert_semantic_equivalence(
        "{ let mut sum = 0; let mut i = 1; while i <= 5 { sum = sum + i; i = i + 1 }; sum }",
        Value::Integer(15),
    );
}

#[test]
fn test_opt_004_08_while_loop_countdown() {
    // While loop counting down
    assert_semantic_equivalence(
        "{ let mut i = 5; while i > 0 { i = i - 1 }; i }",
        Value::Integer(0),
    );
}

#[test]
fn test_opt_004_08_while_loop_fibonacci() {
    // While loop computing Fibonacci-like sequence
    assert_semantic_equivalence(
        "{ let mut a = 0; let mut b = 1; let mut i = 0; while i < 7 { let temp = a + b; a = b; b = temp; i = i + 1 }; a }",
        Value::Integer(13),
    );
}

#[test]
fn test_opt_004_08_while_loop_result_after() {
    // While loop result with value after loop
    assert_semantic_equivalence(
        "{ let mut x = 0; while x < 10 { x = x + 2 }; x + 2 }",
        Value::Integer(12),
    );
}

// ============================================================================
// Test Suite 9: Assignment Expressions (5 tests)
// ============================================================================

#[test]
fn test_opt_004_09_simple_assignment() {
    // Simple variable assignment
    assert_semantic_equivalence(
        "{ let mut x = 10; x = 42; x }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_09_assignment_returns_value() {
    // Assignment is an expression that returns the assigned value
    assert_semantic_equivalence(
        "{ let mut x = 10; let y = (x = 42); y }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_09_assignment_with_arithmetic() {
    // Assignment with arithmetic expression (self-referencing)
    assert_semantic_equivalence(
        "{ let mut x = 10; x = x + 32; x }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_09_multiple_assignments() {
    // Multiple assignments in sequence
    assert_semantic_equivalence(
        "{ let mut x = 0; x = 10; x = 20; x = 42; x }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_09_assignment_in_expression() {
    // Assignment used in larger expression
    assert_semantic_equivalence(
        "{ let mut x = 10; (x = 40) + 2 }",
        Value::Integer(42),
    );
}

// ============================================================================
// Test Suite 10: Function Calls (OPT-011)
// ============================================================================

#[test]
fn test_opt_004_10_simple_function_call() {
    // Simple function call with no arguments
    assert_semantic_equivalence(
        "{ fn answer() { 42 }; answer() }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_10_function_with_one_arg() {
    // Function call with single argument
    assert_semantic_equivalence(
        "{ fn double(x: i32) { x * 2 }; double(21) }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_10_function_with_multiple_args() {
    // Function call with multiple arguments
    assert_semantic_equivalence(
        "{ fn add(a: i32, b: i32) { a + b }; add(10, 32) }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_10_nested_function_calls() {
    // Nested function calls
    assert_semantic_equivalence(
        "{ fn inc(x: i32) { x + 1 }; fn double(x: i32) { x * 2 }; double(inc(20)) }",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_10_function_with_expression_args() {
    // Function called with expression arguments
    assert_semantic_equivalence(
        "{ fn add(a: i32, b: i32) { a + b }; add(10 + 20, 6 * 2) }",
        Value::Integer(42),
    );
}

// ============================================================================
// Test Suite 11: For Loops (OPT-012)
// ============================================================================

#[test]
fn test_opt_004_11_simple_for_loop() {
    // Simple for-loop over array
    assert_semantic_equivalence(
        "{ let mut sum = 0; for i in [1, 2, 3, 4, 5] { sum = sum + i }; sum }",
        Value::Integer(15),
    );
}

#[test]
fn test_opt_004_11_for_loop_with_range() {
    // For-loop returns last iteration value
    assert_semantic_equivalence(
        "{ let mut result = 0; for i in [10, 20, 30] { result = i }; result }",
        Value::Integer(30),
    );
}

#[test]
fn test_opt_004_11_empty_for_loop() {
    // For-loop over empty array
    assert_semantic_equivalence(
        "{ let mut sum = 0; for i in [] { sum = sum + 1 }; sum }",
        Value::Integer(0),
    );
}

#[test]
fn test_opt_004_11_nested_for_loops() {
    // Nested for-loops
    assert_semantic_equivalence(
        "{ let mut sum = 0; for i in [1, 2] { for j in [10, 20] { sum = sum + i + j } }; sum }",
        Value::Integer(66), // (1+10) + (1+20) + (2+10) + (2+20) = 11 + 21 + 12 + 22 = 66
    );
}

#[test]
fn test_opt_004_11_for_loop_in_function() {
    // For-loop inside function
    assert_semantic_equivalence(
        "{ fn sum_array(arr: Vec<i32>) { let mut s = 0; for x in arr { s = s + x }; s }; sum_array([5, 10, 15]) }",
        Value::Integer(30),
    );
}

// ============================================================================
// Test Suite 12: Array Indexing (OPT-013)
// ============================================================================

#[test]
fn test_opt_004_12_simple_array_index() {
    // Basic array indexing
    assert_semantic_equivalence(
        "[1, 2, 3][0]",
        Value::Integer(1),
    );
}

#[test]
fn test_opt_004_12_array_index_middle() {
    // Index middle element
    assert_semantic_equivalence(
        "[10, 20, 30][1]",
        Value::Integer(20),
    );
}

#[test]
fn test_opt_004_12_array_index_last() {
    // Index last element (positive index)
    assert_semantic_equivalence(
        "[5, 10, 15][2]",
        Value::Integer(15),
    );
}

#[test]
fn test_opt_004_12_array_index_negative() {
    // Negative indexing: -1 is last element
    assert_semantic_equivalence(
        "[10, 20, 30][-1]",
        Value::Integer(30),
    );
}

#[test]
fn test_opt_004_12_array_index_with_let() {
    // Array indexing with variable
    assert_semantic_equivalence(
        "{ let arr = [1, 2, 3]; arr[1] }",
        Value::Integer(2),
    );
}

#[test]
fn test_opt_004_12_nested_array_index() {
    // Nested array indexing with variable index
    assert_semantic_equivalence(
        "{ let arr = [1, 2, 3]; let idx = 0; arr[idx] }",
        Value::Integer(1),
    );
}

// ============================================================================
// Test Suite 13: Method Calls (OPT-014)
// ============================================================================

#[test]
fn test_opt_004_13_array_len() {
    // Array.len() method
    assert_semantic_equivalence(
        "[1, 2, 3].len()",
        Value::Integer(3),
    );
}

#[test]
fn test_opt_004_13_string_len() {
    // String.len() method
    assert_semantic_equivalence(
        "\"hello\".len()",
        Value::Integer(5),
    );
}

#[test]
fn test_opt_004_13_to_string() {
    // Integer.to_string() method
    assert_semantic_equivalence(
        "42.to_string()",
        Value::from_string("42".to_string()),
    );
}

#[test]
fn test_opt_004_13_method_in_let() {
    // Method call on variable
    assert_semantic_equivalence(
        "{ let arr = [10, 20, 30]; arr.len() }",
        Value::Integer(3),
    );
}

#[test]
fn test_opt_004_13_method_chain() {
    // Simple method chain (to_string.len)
    assert_semantic_equivalence(
        "42.to_string().len()",
        Value::Integer(2),
    );
}

// ============================================================================
// Test Suite 14: Tuple Literals (OPT-017)
// ============================================================================
// Literal-only tuples compiled to constant pool (same as arrays)
// Enables field access testing for tuples

#[test]
fn test_opt_004_14_tuple_basic() {
    // Basic 2-element tuple
    use std::sync::Arc;
    assert_semantic_equivalence(
        "(42, \"hello\")",
        Value::Tuple(Arc::from([Value::Integer(42), Value::from_string("hello".to_string())].as_slice())),
    );
}

#[test]
fn test_opt_004_14_tuple_single() {
    // Single-element tuple
    use std::sync::Arc;
    assert_semantic_equivalence(
        "(100,)",
        Value::Tuple(Arc::from([Value::Integer(100)].as_slice())),
    );
}

#[test]
fn test_opt_004_14_tuple_unit() {
    // Empty parentheses () are treated as unit/nil in Ruchy (not empty tuple)
    assert_semantic_equivalence(
        "()",
        Value::Nil,
    );
}

#[test]
fn test_opt_004_14_tuple_mixed_types() {
    // Tuple with mixed types
    use std::sync::Arc;
    assert_semantic_equivalence(
        "(10, 3.14, true, \"test\")",
        Value::Tuple(Arc::from([
            Value::Integer(10),
            Value::Float(3.14),
            Value::Bool(true),
            Value::from_string("test".to_string()),
        ].as_slice())),
    );
}

#[test]
fn test_opt_004_14_tuple_nested() {
    // Nested tuple (currently unsupported - would require expression support)
    // This test will fail with "Tuple elements must be literals for now"
    // TODO: Enable when full expression support is added
    // assert_semantic_equivalence(
    //     "((1, 2), (3, 4))",
    //     Value::Tuple(Arc::from([...].as_slice())),
    // );
}

// ============================================================================
// Test Suite 15: Field Access (OPT-015)
// ============================================================================
// Field access compilation and VM handler implemented
// Can now test tuple field access with OPT-017 complete

#[test]
fn test_opt_004_15_tuple_field() {
    // Tuple field access via numeric index
    assert_semantic_equivalence(
        "(42, \"hello\").0",
        Value::Integer(42),
    );
}

#[test]
fn test_opt_004_15_tuple_field_string() {
    // Tuple field access - second element
    assert_semantic_equivalence(
        "(42, \"hello\").1",
        Value::from_string("hello".to_string()),
    );
}

#[test]
fn test_opt_004_15_tuple_field_in_expression() {
    // Tuple field access in arithmetic expression
    assert_semantic_equivalence(
        "(10, 20, 30).1 + (10, 20, 30).2",
        Value::Integer(50),
    );
}

// ============================================================================
// Test Suite 16: Object Field Access (OPT-016 - BLOCKED)
// ============================================================================
// NOTE: Object field access testing still blocked by OPT-016 (ObjectLiteral)
//
// Tests will be added once OPT-016 is implemented:
// - test_opt_004_16_object_field: requires ObjectLiteral
// - test_opt_004_16_nested_field: requires ObjectLiteral
// - test_opt_004_16_field_in_expression: requires ObjectLiteral

// Total tests: 9 + 8 + 6 + 3 + 6 + 3 + 9 + 7 + 5 + 5 + 5 + 6 + 5 = 77 integration tests
// All tests verify semantic equivalence between AST and bytecode modes
// Suite 1: Updated to 9 tests (added 5 unary operator tests for OPT-005)
// Suite 8: Updated to 7 tests (2 basic OPT-006, 5 with mutations OPT-009)
// Suite 9: Added 5 tests for assignments (OPT-007), self-referencing bug fixed in OPT-008
// Suite 10: 5 function call tests (OPT-011)
// Suite 11: 5 for-loop tests (OPT-012)
// Suite 12: 6 array indexing tests (OPT-013)
// Suite 13: 5 method call tests (OPT-014)
// Suite 14: 4 field access tests (OPT-015)
