#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::notebook::testing::sandbox::{WasmSandbox, ResourceLimits};
use std::time::Duration;

// Security-focused WASM fuzzing
// Tests sandbox boundaries, resource limits, and potential escape attempts

fuzz_target!(|data: &[u8]| {
    let Ok(code) = std::str::from_utf8(data) else {
        return;
    };
    
    // Skip extremely large inputs
    if code.len() > 5000 {
        return;
    }
    
    // Test various security scenarios
    test_memory_exhaustion(code);
    test_infinite_loops(code);
    test_stack_overflow(code);
    test_integer_overflow(code);
    test_sandbox_escape_attempts(code);
    test_resource_limit_enforcement(code);
});

fn test_memory_exhaustion(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Set very strict memory limit
    let limits = ResourceLimits {
        memory_mb: 1,  // Only 1MB
        cpu_time_ms: 100,
        stack_size_kb: 32,
        heap_size_mb: 1,
        file_access: false,
        network_access: false,
    };
    
    let _ = sandbox.configure(limits);
    
    // Try to allocate large arrays
    let malicious = format!(
        r#"fun main() {{
            let huge = [0; 1000000];  // Try to allocate 1M integers
            {}
            return 1
        }}"#,
        code
    );
    
    // Should fail gracefully or respect memory limits
    let result = sandbox.compile_and_execute(&malicious, Duration::from_millis(50));
    
    if let Ok(exec_result) = result {
        // Memory usage should be within limits
        assert!(
            exec_result.memory_used <= 1024 * 1024,
            "Memory limit violated: {} bytes used",
            exec_result.memory_used
        );
    }
}

fn test_infinite_loops(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Set strict CPU time limit
    let limits = ResourceLimits {
        memory_mb: 1,
        cpu_time_ms: 10,  // Only 10ms
        stack_size_kb: 32,
        heap_size_mb: 1,
        file_access: false,
        network_access: false,
    };
    
    let _ = sandbox.configure(limits);
    
    // Try various infinite loop patterns
    let loops = vec![
        format!("fun main() {{ while true {{ {} }} return 0 }}", code),
        format!("fun main() {{ for i in 0..1000000000 {{ {} }} return 0 }}", code),
        format!("fun main() {{ loop {{ {} }} }}", code),
    ];
    
    for loop_code in loops {
        // Should timeout gracefully
        let result = sandbox.compile_and_execute(&loop_code, Duration::from_millis(20));
        
        match result {
            Ok(exec_result) => {
                // Should have terminated within time limit
                assert!(
                    exec_result.execution_time.as_millis() <= 20,
                    "Execution time exceeded limit"
                );
            }
            Err(_) => {
                // Expected to timeout - good
            }
        }
    }
}

fn test_stack_overflow(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Set strict stack limit
    let limits = ResourceLimits {
        memory_mb: 1,
        cpu_time_ms: 100,
        stack_size_kb: 8,  // Very small stack
        heap_size_mb: 1,
        file_access: false,
        network_access: false,
    };
    
    let _ = sandbox.configure(limits);
    
    // Try deep recursion
    let recursive = format!(
        r#"fun recurse(n) {{
            if n > 0 {{
                return recurse(n - 1)
            }}
            {}
            return n
        }}
        
        fun main() {{
            return recurse(10000)  // Deep recursion
        }}"#,
        code
    );
    
    // Should fail gracefully with stack overflow
    let _ = sandbox.compile_and_execute(&recursive, Duration::from_millis(50));
}

fn test_integer_overflow(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Test various integer overflow scenarios
    let overflow_tests = vec![
        format!("fun main() {{ return 2147483647 + {} }}", code.len()),
        format!("fun main() {{ return -2147483648 - {} }}", code.len()),
        format!("fun main() {{ return 1000000 * 1000000 * {} }}", code.len()),
    ];
    
    for test_code in overflow_tests {
        // Should handle overflow gracefully
        let result = sandbox.compile_and_execute(&test_code, Duration::from_millis(50));
        
        // Should not crash
        match result {
            Ok(_) | Err(_) => {
                // Either succeeds with wrapped value or fails - both ok
            }
        }
    }
}

fn test_sandbox_escape_attempts(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Ensure sandbox is fully restricted
    let limits = ResourceLimits {
        memory_mb: 1,
        cpu_time_ms: 100,
        stack_size_kb: 32,
        heap_size_mb: 1,
        file_access: false,  // No file access
        network_access: false,  // No network access
    };
    
    let _ = sandbox.configure(limits);
    
    // Try various escape attempts (these should all fail or be ignored)
    let escape_attempts = vec![
        // Try to import forbidden modules
        format!("import fs from 'fs'; {}", code),
        format!("import net from 'net'; {}", code),
        format!("import process from 'process'; {}", code),
        
        // Try to access global objects (if they existed)
        format!("fun main() {{ let x = global; {} return 0 }}", code),
        format!("fun main() {{ let x = window; {} return 0 }}", code),
        
        // Try to execute system commands (should be impossible)
        format!("fun main() {{ let x = exec('ls'); {} return 0 }}", code),
        
        // Try to access environment variables
        format!("fun main() {{ let x = env.HOME; {} return 0 }}", code),
    ];
    
    for attempt in escape_attempts {
        let result = sandbox.compile_and_execute(&attempt, Duration::from_millis(50));
        
        // Should either fail to compile or execute safely without access
        match result {
            Ok(exec_result) => {
                // If it somehow succeeds, verify no actual access occurred
                assert_eq!(exec_result.files_accessed, 0, "File access detected!");
                assert_eq!(exec_result.network_calls, 0, "Network access detected!");
            }
            Err(_) => {
                // Expected to fail - good
            }
        }
    }
}

fn test_resource_limit_enforcement(code: &str) {
    let mut sandbox = WasmSandbox::new();
    
    // Test with progressively stricter limits
    let limit_configs = vec![
        (10, 1024, 10),   // 10MB memory, 1024KB stack, 10MB heap
        (5, 512, 5),      // 5MB memory, 512KB stack, 5MB heap
        (2, 128, 2),      // 2MB memory, 128KB stack, 2MB heap
        (1, 32, 1),       // 1MB memory, 32KB stack, 1MB heap
    ];
    
    for (mem_mb, stack_kb, heap_mb) in limit_configs {
        let limits = ResourceLimits {
            memory_mb: mem_mb,
            cpu_time_ms: 100,
            stack_size_kb: stack_kb,
            heap_size_mb: heap_mb,
            file_access: false,
            network_access: false,
        };
        
        let _ = sandbox.configure(limits);
        
        // Test with the fuzzer input
        let test_code = format!("fun main() {{ {} return 1 }}", code);
        let result = sandbox.compile_and_execute(&test_code, Duration::from_millis(50));
        
        if let Ok(exec_result) = result {
            // Verify limits are respected
            assert!(
                exec_result.memory_used <= mem_mb * 1024 * 1024,
                "Memory limit {} MB violated: {} bytes used",
                mem_mb,
                exec_result.memory_used
            );
        }
    }
}

// Additional structured fuzzing for security patterns
#[derive(Debug, arbitrary::Arbitrary)]
struct SecurityTestCase {
    memory_ops: Vec<MemoryOp>,
    control_flow: ControlFlow,
    attempts_escape: bool,
}

#[derive(Debug, arbitrary::Arbitrary)]
enum MemoryOp {
    Allocate(u32),
    Access(u32),
    Free,
}

#[derive(Debug, arbitrary::Arbitrary)]
enum ControlFlow {
    Linear,
    Loop(u8),
    Recursion(u8),
    Conditional,
}

fuzz_target!(|test_case: SecurityTestCase| {
    let mut code = String::new();
    
    // Build memory operations
    for op in &test_case.memory_ops {
        match op {
            MemoryOp::Allocate(size) => {
                code.push_str(&format!("let arr = [0; {}]; ", size.min(&1000)));
            }
            MemoryOp::Access(index) => {
                code.push_str(&format!("let x = arr[{}]; ", index % 100));
            }
            MemoryOp::Free => {
                code.push_str("arr = null; ");
            }
        }
    }
    
    // Add control flow
    let body = match test_case.control_flow {
        ControlFlow::Linear => format!("fun main() {{ {} return 0 }}", code),
        ControlFlow::Loop(count) => {
            format!("fun main() {{ for i in 0..{} {{ {} }} return 0 }}", count.min(100), code)
        }
        ControlFlow::Recursion(depth) => {
            format!(
                "fun recurse(n) {{ if n <= 0 {{ return 0 }} {} return recurse(n-1) }} fun main() {{ return recurse({}) }}",
                code,
                depth.min(10)
            )
        }
        ControlFlow::Conditional => {
            format!("fun main() {{ if true {{ {} }} return 0 }}", code)
        }
    };
    
    // Add escape attempt if specified
    let final_code = if test_case.attempts_escape {
        format!("import sys; {}", body)
    } else {
        body
    };
    
    // Test with strict security limits
    let mut sandbox = WasmSandbox::new();
    let limits = ResourceLimits {
        memory_mb: 1,
        cpu_time_ms: 50,
        stack_size_kb: 32,
        heap_size_mb: 1,
        file_access: false,
        network_access: false,
    };
    
    let _ = sandbox.configure(limits);
    let _ = sandbox.compile_and_execute(&final_code, Duration::from_millis(50));
    
    // If we reach here without panic, security boundaries held
});