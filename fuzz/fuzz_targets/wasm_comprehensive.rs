#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::notebook::testing::sandbox::{ResourceLimits, WasmSandbox};
use std::time::Duration;

// Comprehensive WASM fuzzing target
// Tests all aspects of WASM compilation and execution

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string (potential Ruchy code)
    let Ok(code) = std::str::from_utf8(data) else {
        return; // Skip invalid UTF-8
    };

    // Skip extremely large inputs
    if code.len() > 10000 {
        return;
    }

    // Test 1: Basic compilation without execution
    test_compilation(code);

    // Test 2: Compilation and execution with default limits
    test_execution_default(code);

    // Test 3: Execution with strict resource limits
    test_execution_limited(code);

    // Test 4: Deterministic compilation
    test_determinism(code);

    // Test 5: Sandbox isolation
    test_isolation(code);

    // Test 6: Edge cases with special inputs
    test_edge_cases(code);
});

fn test_compilation(code: &str) {
    let sandbox = WasmSandbox::new();

    // Try to compile - should not panic
    let _ = sandbox.compile_sandboxed(code);
}

fn test_execution_default(code: &str) {
    let mut sandbox = WasmSandbox::new();

    // Try to compile and execute with default settings
    let _ = sandbox.compile_and_execute(code, Duration::from_millis(100));
}

fn test_execution_limited(code: &str) {
    let mut sandbox = WasmSandbox::new();

    // Set strict resource limits
    let limits = ResourceLimits {
        memory_mb: 1,
        cpu_time_ms: 100,
        stack_size_kb: 64,
        heap_size_mb: 1,
        file_access: false,
        network_access: false,
    };

    let _ = sandbox.configure(limits);

    // Try to execute with limits - should handle gracefully
    let _ = sandbox.compile_and_execute(code, Duration::from_millis(50));
}

fn test_determinism(code: &str) {
    let sandbox1 = WasmSandbox::new();
    let sandbox2 = WasmSandbox::new();

    // Compile twice and verify determinism
    let result1 = sandbox1.compile_sandboxed(code);
    let result2 = sandbox2.compile_sandboxed(code);

    match (result1, result2) {
        (Ok(bytes1), Ok(bytes2)) => {
            // Should produce identical bytecode
            assert_eq!(bytes1, bytes2, "WASM compilation should be deterministic");
        }
        (Err(_), Err(_)) => {
            // Both failed - that's consistent
        }
        _ => {
            // One succeeded, one failed - shouldn't happen for same input
            panic!("Inconsistent compilation results for same input");
        }
    }
}

fn test_isolation(code: &str) {
    let mut sandbox1 = WasmSandbox::new();
    let mut sandbox2 = WasmSandbox::new();

    // Execute in parallel sandboxes
    let result1 = sandbox1.compile_and_execute(code, Duration::from_millis(100));
    let result2 = sandbox2.compile_and_execute(code, Duration::from_millis(100));

    // Results should be independent and consistent
    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            assert_eq!(
                r1.output, r2.output,
                "Isolated sandboxes should produce same output"
            );
        }
        (Err(_), Err(_)) => {
            // Both failed consistently
        }
        _ => {
            panic!("Inconsistent execution in isolated sandboxes");
        }
    }
}

fn test_edge_cases(code: &str) {
    // Test with various transformations of the input

    // Test with wrapped main function
    let wrapped = format!("fun main() {{ {} }}", code);
    let mut sandbox = WasmSandbox::new();
    let _ = sandbox.compile_and_execute(&wrapped, Duration::from_millis(50));

    // Test with trimmed input
    let trimmed = code.trim();
    if !trimmed.is_empty() {
        let mut sandbox = WasmSandbox::new();
        let _ = sandbox.compile_and_execute(trimmed, Duration::from_millis(50));
    }

    // Test with simple return statement
    if code.chars().all(|c| c.is_ascii_digit() || c == '-') && !code.is_empty() {
        let return_stmt = format!("fun main() {{ return {} }}", code);
        let mut sandbox = WasmSandbox::new();
        let _ = sandbox.compile_and_execute(&return_stmt, Duration::from_millis(50));
    }
}

// Additional fuzz target for testing WASM with structured input
#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    function_name: String,
    return_value: i32,
    has_params: bool,
    param_count: u8,
}

fuzz_target!(|input: StructuredInput| {
    // Sanitize function name
    let name = input
        .function_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .take(20)
        .collect::<String>();

    let name = if name.is_empty() { "main" } else { &name };

    // Build function based on structured input
    let params = if input.has_params {
        (0..input.param_count.min(5))
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    };

    let code = format!(
        "fun {}({}) {{ return {} }}",
        name, params, input.return_value
    );

    // Test the generated code
    let mut sandbox = WasmSandbox::new();

    // Should handle any valid function structure
    match sandbox.compile_and_execute(&code, Duration::from_millis(100)) {
        Ok(result) => {
            // If it's main function, output should match return value
            if name == "main" && params.is_empty() {
                assert_eq!(
                    result.output.trim(),
                    input.return_value.to_string(),
                    "Main function output should match return value"
                );
            }
        }
        Err(_) => {
            // Non-main functions or functions with params might fail - that's ok
        }
    }
});
