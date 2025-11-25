//! TRANSPILER-PARAM-INFERENCE: Parameter Type Inference Bug Tests
//!
//! RED phase tests for function parameter type inference.
//!
//! Bug: Parameters default to `&str` regardless of usage context
//! - Arrays used with indexing → Inferred as `&str` instead of array/Vec type
//! - Integers used as indices → Inferred as `&str` instead of `i32`/`usize`
//!
//! Blocks: BENCH-002 (matrix multiplication benchmark)

use ruchy::backend::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
fn test_transpiler_param_array_with_indexing() {
    let source = r"
fun get_first(arr) {
    arr[0]
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Parameter used with indexing should infer as Vec/array type, not &str
    assert!(
        !code_str.contains("arr : & str") && !code_str.contains("arr: &str"),
        "Array parameter should not be inferred as &str, got: {code_str}"
    );
}

#[test]
fn test_transpiler_param_integer_as_index() {
    let source = r"
fun get_at_index(arr, i) {
    arr[i]
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Parameter used as index should infer as integer type, not &str
    assert!(
        !code_str.contains("i : & str") && !code_str.contains("i: &str"),
        "Index parameter should not be inferred as &str, got: {code_str}"
    );
}

#[test]
fn test_transpiler_param_2d_array_double_indexing() {
    let source = r"
fun get_cell(matrix, i, j) {
    matrix[i][j]
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // 2D array parameter should not be &str
    assert!(
        !code_str.contains("matrix : & str") && !code_str.contains("matrix: &str"),
        "2D array parameter should not be inferred as &str, got: {code_str}"
    );

    // Index parameters should not be &str
    assert!(
        !code_str.contains("i : & str") && !code_str.contains("i: &str"),
        "Index parameter i should not be inferred as &str, got: {code_str}"
    );
    assert!(
        !code_str.contains("j : & str") && !code_str.contains("j: &str"),
        "Index parameter j should not be inferred as &str, got: {code_str}"
    );
}

#[test]
fn test_transpiler_param_bench_002_pattern() {
    // Minimal reproduction of BENCH-002 parameter inference bug
    let source = r"
fun multiply_cell(a, b, i, j, k_max) {
    let mut sum = 0
    let mut k = 0
    while k < k_max {
        sum = sum + (a[i][k] * b[k][j])
        k = k + 1
    }
    sum
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // 2D arrays a and b should not be &str
    assert!(
        !code_str.contains("a : & str") && !code_str.contains("a: &str"),
        "Array parameter a should not be inferred as &str, got: {code_str}"
    );
    assert!(
        !code_str.contains("b : & str") && !code_str.contains("b: &str"),
        "Array parameter b should not be inferred as &str, got: {code_str}"
    );

    // Indices i and j should not be &str
    assert!(
        !code_str.contains("i : & str") && !code_str.contains("i: &str"),
        "Index parameter i should not be inferred as &str, got: {code_str}"
    );
    assert!(
        !code_str.contains("j : & str") && !code_str.contains("j: &str"),
        "Index parameter j should not be inferred as &str, got: {code_str}"
    );

    // k_max is correctly inferred as i32 from comparison (already works)
    assert!(
        code_str.contains("k_max : i32") || code_str.contains("k_max: i32"),
        "k_max should be inferred as i32, got: {code_str}"
    );
}

#[test]
fn test_transpiler_param_array_with_len() {
    let source = r"
fun array_size(arr) {
    len(arr)
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Parameter used with len() should infer as Vec/array type, not &str
    // Note: &str has len() too, but this is testing array context
    assert!(
        !code_str.contains("arr : & str") && !code_str.contains("arr: &str"),
        "Array parameter (used with len) should not be inferred as &str, got: {code_str}"
    );
}

#[test]
fn test_transpiler_param_mixed_usage() {
    // Test parameter used both as index and in numeric operations
    let source = r"
fun complex(arr, i) {
    arr[i] + arr[i + 1]
}
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Array parameter should not be &str
    assert!(
        !code_str.contains("arr : & str") && !code_str.contains("arr: &str"),
        "Array parameter should not be inferred as &str, got: {code_str}"
    );

    // Index used in numeric operation should infer as integer, not &str
    assert!(
        !code_str.contains("i : & str") && !code_str.contains("i: &str"),
        "Index parameter (used numerically) should not be inferred as &str, got: {code_str}"
    );
}
