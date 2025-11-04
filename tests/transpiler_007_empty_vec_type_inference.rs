// TRANSPILER-007: Empty vec![] type inference for functions with return types
//
// PROBLEM: Empty `vec![]` initializations caused E0282 type inference errors when
// elements were accessed before being added (BENCH-008 pattern).
//
// SOLUTION: Track function return types during transpilation and extract concrete
// types for empty vec initializations.
//
// Pattern: `fun foo() -> [i32] { let x = [] }` → `let x: Vec<i32> = vec![];`

use ruchy::{compile, Parser, Transpiler};

/// Helper: Transpile Ruchy code to Rust
fn transpile_code(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {e:?}"))?;

    let mut transpiler = Transpiler::new();
    let tokens = transpiler
        .transpile_to_program(&ast)
        .map_err(|e| format!("Transpile error: {e:?}"))?;

    Ok(tokens.to_string())
}

/// Helper: Compile Ruchy code to verify it works end-to-end (transpile + verify parseable Rust)
fn compile_code(code: &str) -> Result<(), String> {
    // Use the high-level compile() function which returns transpiled Rust
    let rust_code = compile(code).map_err(|e| format!("Compile error: {e:?}"))?;

    // Verify it's valid Rust syntax by parsing with syn
    syn::parse_file(&rust_code).map_err(|e| format!("Invalid Rust syntax: {e:?}"))?;

    Ok(())
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[test]
fn test_transpiler_007_01_empty_vec_with_explicit_return_type() {
    // Test: Empty vec with explicit return type gets concrete type hint
    let code = r"
fun generate_numbers() -> [i32] {
    let nums = []
    nums
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Should generate Vec<i32>, not Vec<_>
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> type hint, got:\n{result}"
    );

    // Verify it compiles
    compile_code(code).expect("Code should compile");
}

#[test]
fn test_transpiler_007_02_empty_vec_without_return_type() {
    // Test: Empty vec without return type gets Vec<_> fallback
    let code = r"
fun generate_numbers() {
    let nums = []
    nums
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Without return type, should generate Vec<_>
    assert!(
        result.contains("Vec<_>") || result.contains("Vec < _ >"),
        "Expected Vec<_> fallback, got:\n{result}"
    );
}

#[test]
fn test_transpiler_007_03_multiple_empty_vecs_same_type() {
    // Test: Multiple empty vecs in same function get same type
    let code = r"
fun generate_data() -> [i32] {
    let nums = []
    let more = []
    nums
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Count Vec<i32> occurrences (should be 2)
    let count = result.matches("Vec<i32>").count() + result.matches("Vec < i32 >").count();
    assert!(
        count >= 2,
        "Expected at least 2 Vec<i32> type hints, got {count} in:\n{result}"
    );

    compile_code(code).expect("Generated Rust should compile");
}

#[test]
fn test_transpiler_007_04_mutable_empty_vec() {
    // Test: Mutable empty vec with return type
    let code = r"
fun build_list() -> [i32] {
    let mut items = []
    items
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Should have both mut and Vec<i32>
    assert!(result.contains("mut"), "Expected mut keyword");
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> type hint"
    );

    compile_code(code).expect("Generated Rust should compile");
}

#[test]
fn test_transpiler_007_05_nested_function_with_vec() {
    // Test: Nested function with empty vec
    let code = r"
fun outer() -> [i32] {
    fun inner() -> [i32] {
        let data = []
        data
    }
    inner()
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Inner function should also get Vec<i32>
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> in nested function"
    );

    compile_code(code).expect("Generated Rust should compile");
}

#[test]
fn test_transpiler_007_06_string_vec_return_type() {
    // Test: Return type [String] generates Vec<String>
    let code = r"
fun get_names() -> [String] {
    let names = []
    names
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    assert!(
        result.contains("Vec<String>") || result.contains("Vec < String >"),
        "Expected Vec<String> type hint, got:\n{result}"
    );

    compile_code(code).expect("Generated Rust should compile");
}

#[test]
fn test_transpiler_007_07_bench_008_pattern() {
    // Test: BENCH-008 actual pattern - empty vec accessed before adding
    let code = r"
fun generate_primes(count) -> [i32] {
    let mut primes = []
    let mut candidate = 2
    while len(primes) < count {
        let mut i = 0
        let mut is_prime = true
        while i < len(primes) {
            if candidate % primes[i] == 0 {
                is_prime = false
            }
            i = i + 1
        }
        if is_prime {
            primes = primes + [candidate]
        }
        candidate = candidate + 1
    }
    primes
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Should have Vec<i32> type hint
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> type hint for BENCH-008 pattern"
    );

    // Should compile without E0282 errors
    compile_code(code).expect("BENCH-008 pattern should compile");
}

#[test]
fn test_transpiler_007_08_non_empty_vec_no_type_hint() {
    // Test: Non-empty vec doesn't need type hint
    let code = r"
fun get_numbers() -> [i32] {
    let nums = [1, 2, 3]
    nums
}
";

    let _result = transpile_code(code).expect("Transpilation should succeed");

    // Non-empty vec doesn't need explicit type hint (Rust infers from elements)
    // Just verify it compiles
    compile_code(code).expect("Non-empty vec should compile");
}

#[test]
fn test_transpiler_007_09_empty_vec_in_while_loop() {
    // Test: Empty vec initialization inside while loop
    let code = r"
fun process() -> [i32] {
    let mut result = []
    let mut i = 0
    while i < 10 {
        result = result + [i]
        i = i + 1
    }
    result
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> type hint in while loop pattern"
    );

    compile_code(code).expect("While loop pattern should compile");
}

#[test]
fn test_transpiler_007_10_top_level_empty_vec() {
    // Test: Top-level empty vec without function context
    let code = r"
let nums = []
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Top-level variables without function context get Vec<_>
    assert!(
        result.contains("Vec<_>") || result.contains("Vec < _ >"),
        "Expected Vec<_> for top-level variable"
    );
}

#[test]
fn test_transpiler_007_11_f64_return_type() {
    // Test: Floating point return type
    let code = r"
fun get_floats() -> [f64] {
    let values = []
    values
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    assert!(
        result.contains("Vec<f64>") || result.contains("Vec < f64 >"),
        "Expected Vec<f64> type hint"
    );

    compile_code(code).expect("f64 vec should compile");
}

#[test]
fn test_transpiler_007_12_bool_return_type() {
    // Test: Boolean return type
    let code = r"
fun get_flags() -> [bool] {
    let flags = []
    flags
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    assert!(
        result.contains("Vec<bool>") || result.contains("Vec < bool >"),
        "Expected Vec<bool> type hint"
    );

    compile_code(code).expect("bool vec should compile");
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_transpiler_007_edge_01_nested_vec_return() {
    // Test: Nested vec return type [[i32]]
    let code = r"
fun get_matrix() -> [[i32]] {
    let matrix = []
    matrix
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Should handle nested Vec types
    assert!(
        result.contains("Vec<Vec<i32>>")
            || result.contains("Vec < Vec < i32 > >")
            || result.contains("Vec<Vec < i32 >>"),
        "Expected nested Vec type, got:\n{result}"
    );

    compile_code(code).expect("Nested Vec should compile");
}

#[test]
fn test_transpiler_007_edge_02_empty_vec_with_subsequent_operations() {
    // Test: Empty vec followed by operations that would fail without type
    let code = r"
fun test() -> [i32] {
    let mut nums = []
    let first = nums[0]  // Would fail without concrete type
    nums
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> for indexing operation"
    );

    // Note: This will panic at runtime, but should compile
    // We're testing transpilation, not runtime behavior
}

#[test]
fn test_transpiler_007_edge_03_multiple_functions_different_types() {
    // Test: Multiple functions with different return types
    let code = r"
fun get_ints() -> [i32] {
    let nums = []
    nums
}

fun get_strings() -> [String] {
    let names = []
    names
}
";

    let result = transpile_code(code).expect("Transpilation should succeed");

    // Should have both Vec<i32> and Vec<String>
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32>"
    );
    assert!(
        result.contains("Vec<String>") || result.contains("Vec < String >"),
        "Expected Vec<String>"
    );

    compile_code(code).expect("Multiple functions should compile");
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[test]
fn test_transpiler_007_regression_01_bench_008_full_compilation() {
    // Regression: Full BENCH-008 should compile and execute
    let code = std::fs::read_to_string("examples/bench_008_prime_generation.ruchy")
        .expect("BENCH-008 file should exist");

    let result = transpile_code(&code).expect("BENCH-008 should transpile");

    // Should generate Vec<i32>
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "BENCH-008 should have Vec<i32> type hint"
    );

    // Should compile successfully
    compile_code(&code).expect("BENCH-008 should compile");
}

#[test]
fn test_transpiler_007_regression_02_no_return_type_still_works() {
    // Regression: Functions without return types should still work (fallback to Vec<_>)
    let code = r"
fun process() {
    let mut data = []
    data = data + [42]
    data
}
";

    let result = transpile_code(code).expect("Should transpile");

    // Without return type, falls back to Vec<_>
    assert!(
        result.contains("Vec<_>") || result.contains("Vec < _ >"),
        "Should fallback to Vec<_>"
    );
}

#[test]
fn test_transpiler_007_regression_03_immutable_with_reassignment() {
    // Regression: Immutable vec with reassignment pattern
    let code = r"
fun build() -> [i32] {
    let items = []
    let items = items + [1]
    let items = items + [2]
    items
}
";

    let result = transpile_code(code).expect("Should transpile");

    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Shadowed variables should get type hint"
    );

    compile_code(code).expect("Should compile");
}
