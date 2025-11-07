//! TRANSPILER-TYPE: Empty Array Type Inference Bug Tests
//!
//! RED phase tests for type inference of empty arrays with usage context.
//!
//! Bug: `let mut result = []` transpiles to `Mutex<i32>` instead of `Mutex<Vec<i32>>`
//! Blocks: BENCH-002 (matrix multiplication benchmark)

use ruchy::backend::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
fn test_transpiler_type_empty_array_with_append() {
    let source = r"
let mut result = []
result = result + [42]
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Should infer Vec<i32>, not i32
    assert!(
        code_str.contains("Vec <") || code_str.contains("Vec<"),
        "Empty array with append should infer Vec type, got: {code_str}"
    );
    assert!(
        !code_str.contains("Mutex < i32 >") && !code_str.contains("Mutex<i32>"),
        "Should not infer plain i32 for array operations"
    );
}

#[test]
fn test_transpiler_type_empty_array_with_index_access() {
    let source = r"
let mut nums = []
nums = nums + [1, 2, 3]
let x = nums[0]
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Should infer Vec type due to indexing
    assert!(
        code_str.contains("Vec <") || code_str.contains("Vec<"),
        "Array with indexing should infer Vec type"
    );
}

#[test]
fn test_transpiler_type_empty_array_with_len() {
    let source = r"
let mut data = []
data = data + [10]
let size = len(data)
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Should infer Vec type due to len() call
    assert!(
        code_str.contains("Vec <") || code_str.contains("Vec<"),
        "Array with len() should infer Vec type"
    );
}

#[test]
fn test_transpiler_type_bench_002_pattern() {
    // Minimal reproduction of BENCH-002 bug
    let source = r"
let mut result = []

fun add_value(n) {
    result = result + [n]
}

add_value(42)
println(result[0])
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Should generate Vec type for global mutable array
    assert!(
        code_str.contains("Vec <") || code_str.contains("Vec<"),
        "Global array with function mutation should infer Vec type, got: {code_str}"
    );

    // Should NOT be plain i32
    assert!(
        !code_str.contains("Mutex < i32 >") && !code_str.contains("Mutex<i32>"),
        "Should not infer i32 for array with indexing operations"
    );
}

#[test]
fn test_transpiler_type_empty_array_2d() {
    // Test 2D array inference
    let source = r"
let mut matrix = []
matrix = matrix + [[1, 2]]
let cell = matrix[0][1]
";

    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse should succeed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("Transpile should succeed");

    let code_str = rust_code.to_string();

    // Should infer nested Vec type
    assert!(
        code_str.contains("Vec <") || code_str.contains("Vec<"),
        "2D array should infer Vec type"
    );
}

#[test]
#[ignore] // TODO: Blocked by parameter type inference bug - transpiler infers `a`, `b`, `i`, `j` as `&str`
fn test_transpiler_type_compile_bench_002() {
    // Integration test: BENCH-002 should compile successfully
    use std::fs;
    use std::process::Command;

    let source = fs::read_to_string("examples/bench_002_matrix_multiply.ruchy")
        .expect("BENCH-002 file should exist");

    let mut parser = Parser::new(&source);
    let ast = parser.parse().expect("BENCH-002 should parse");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_program(&ast)
        .expect("BENCH-002 should transpile");

    let code_str = rust_code.to_string();

    // Write to temp file and try to compile with rustc
    let temp_file = "/tmp/bench_002_test.rs";
    fs::write(temp_file, code_str.clone()).expect("Write temp file");

    let output = Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "-o",
            "/tmp/bench_002_test",
            temp_file,
        ])
        .output()
        .expect("rustc should run");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "BENCH-002 should compile successfully. Errors:\n{stderr}\n\nGenerated code:\n{code_str}"
        );
    }
}
