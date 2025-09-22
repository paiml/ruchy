#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::notebook::testing::sandbox::{ResourceLimits, WasmSandbox};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Stress testing and performance fuzzing for WASM
// Tests concurrent execution, resource contention, and performance boundaries

fuzz_target!(|data: &[u8]| {
    let Ok(code) = std::str::from_utf8(data) else {
        return;
    };

    if code.len() > 2000 {
        return;
    }

    // Run various stress tests
    test_concurrent_execution(code);
    test_rapid_compilation(code);
    test_memory_pressure(code);
    test_varying_timeouts(code);
    test_binary_size_limits(code);
});

fn test_concurrent_execution(code: &str) {
    // Test multiple sandboxes executing simultaneously
    let code = Arc::new(code.to_string());
    let mut handles = vec![];

    // Spawn 4 concurrent executions
    for i in 0..4 {
        let code_clone = Arc::clone(&code);
        let handle = thread::spawn(move || {
            let mut sandbox = WasmSandbox::new();

            // Each thread has slightly different limits
            let limits = ResourceLimits {
                memory_mb: 1 + i,
                cpu_time_ms: 50 + (i * 10) as u64,
                stack_size_kb: 32 + (i * 8),
                heap_size_mb: 1,
                file_access: false,
                network_access: false,
            };

            let _ = sandbox.configure(limits);

            // Execute with thread-specific timeout
            let timeout = Duration::from_millis(50 + (i * 10) as u64);
            let _ = sandbox.compile_and_execute(&code_clone, timeout);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let _ = handle.join();
    }
}

fn test_rapid_compilation(code: &str) {
    // Test rapid repeated compilation of same code
    let sandbox = WasmSandbox::new();

    // Compile same code multiple times rapidly
    for _ in 0..10 {
        let result = sandbox.compile_sandboxed(code);

        // All compilations should produce same result
        match result {
            Ok(bytes) => {
                // Verify bytecode is reasonable size
                assert!(
                    bytes.len() < 100000,
                    "Bytecode unexpectedly large: {} bytes",
                    bytes.len()
                );
            }
            Err(_) => {
                // Consistent compilation error is ok
                break;
            }
        }
    }
}

fn test_memory_pressure(code: &str) {
    // Test execution under varying memory pressure
    let memory_limits = vec![1, 2, 4, 8, 16];

    for mem_mb in memory_limits {
        let mut sandbox = WasmSandbox::new();

        let limits = ResourceLimits {
            memory_mb: mem_mb,
            cpu_time_ms: 100,
            stack_size_kb: 64,
            heap_size_mb: mem_mb,
            file_access: false,
            network_access: false,
        };

        let _ = sandbox.configure(limits);

        // Create code that uses memory proportional to limit
        let memory_test = format!(
            r#"fun main() {{
                let size = {};
                let data = [0; size];
                {}
                return data[0]
            }}"#,
            mem_mb * 100, // Scale with memory limit
            code
        );

        let result = sandbox.compile_and_execute(&memory_test, Duration::from_millis(100));

        if let Ok(exec_result) = result {
            // Memory usage should respect limits
            assert!(
                exec_result.memory_used <= mem_mb * 1024 * 1024,
                "Memory limit violated"
            );
        }
    }
}

fn test_varying_timeouts(code: &str) {
    // Test with different timeout values
    let timeouts = vec![1, 5, 10, 50, 100, 500];

    for timeout_ms in timeouts {
        let mut sandbox = WasmSandbox::new();

        let limits = ResourceLimits {
            memory_mb: 2,
            cpu_time_ms: timeout_ms,
            stack_size_kb: 64,
            heap_size_mb: 2,
            file_access: false,
            network_access: false,
        };

        let _ = sandbox.configure(limits);

        // Test with varying complexity based on timeout
        let complexity = timeout_ms / 10;
        let complex_code = format!(
            r#"fun main() {{
                let sum = 0;
                for i in 0..{} {{
                    sum = sum + i;
                    {}
                }}
                return sum
            }}"#,
            complexity, code
        );

        let start = std::time::Instant::now();
        let result = sandbox.compile_and_execute(&complex_code, Duration::from_millis(timeout_ms));
        let elapsed = start.elapsed();

        // Should respect timeout
        assert!(
            elapsed.as_millis() <= (timeout_ms * 2) as u128,
            "Timeout not respected: {:?} > {}ms",
            elapsed,
            timeout_ms * 2
        );

        if let Ok(exec_result) = result {
            assert!(
                exec_result.execution_time.as_millis() <= timeout_ms as u128,
                "Execution exceeded timeout"
            );
        }
    }
}

fn test_binary_size_limits(code: &str) {
    // Test with code that generates different binary sizes
    let sandbox = WasmSandbox::new();

    // Generate increasingly complex code
    let mut accumulated_code = String::new();

    for i in 0..10 {
        accumulated_code.push_str(&format!("let x{} = {}; ", i, i * 2));
        accumulated_code.push_str(code);

        let test_code = format!("fun main() {{ {} return 0 }}", accumulated_code);

        if let Ok(bytecode) = sandbox.compile_sandboxed(&test_code) {
            // Binary size should grow reasonably
            let expected_max = 100 + (accumulated_code.len() * 2);
            assert!(
                bytecode.len() < expected_max,
                "Binary size {} exceeds expected max {} for code length {}",
                bytecode.len(),
                expected_max,
                test_code.len()
            );
        }
    }
}

// Structured stress test input
#[derive(Debug, arbitrary::Arbitrary)]
struct StressTestCase {
    num_functions: u8,
    num_variables: u8,
    num_loops: u8,
    recursion_depth: u8,
    concurrent_sandboxes: u8,
}

fuzz_target!(|test_case: StressTestCase| {
    // Build complex program based on stress parameters
    let mut program = String::new();

    // Generate helper functions
    for i in 0..test_case.num_functions.min(10) {
        program.push_str(&format!("fun helper{}(x) {{ return x * {} }} ", i, i + 1));
    }

    // Generate main function with complexity
    program.push_str("fun main() { ");

    // Add variables
    for i in 0..test_case.num_variables.min(20) {
        program.push_str(&format!("let v{} = {}; ", i, i * 3));
    }

    // Add loops
    for i in 0..test_case.num_loops.min(5) {
        program.push_str(&format!(
            "for j{} in 0..{} {{ v0 = v0 + 1; }} ",
            i,
            (i + 1) * 10
        ));
    }

    // Add recursive function if specified
    if test_case.recursion_depth > 0 {
        program.push_str(&format!(
            "fun recurse(n) {{ if n <= 0 {{ return 1 }} return n * recurse(n - 1) }} "
        ));
        program.push_str(&format!(
            "let factorial = recurse({}); ",
            test_case.recursion_depth.min(10)
        ));
    }

    program.push_str("return v0 }");

    // Test with concurrent sandboxes
    let num_sandboxes = test_case.concurrent_sandboxes.min(4) as usize;
    let program = Arc::new(program);
    let mut handles = vec![];

    for i in 0..num_sandboxes {
        let program_clone = Arc::clone(&program);
        let handle = thread::spawn(move || {
            let mut sandbox = WasmSandbox::new();

            let limits = ResourceLimits {
                memory_mb: 2,
                cpu_time_ms: 100,
                stack_size_kb: 64,
                heap_size_mb: 2,
                file_access: false,
                network_access: false,
            };

            let _ = sandbox.configure(limits);
            let _ = sandbox.compile_and_execute(&program_clone, Duration::from_millis(50));
        });
        handles.push(handle);
    }

    // Wait for all concurrent executions
    for handle in handles {
        let _ = handle.join();
    }
});
