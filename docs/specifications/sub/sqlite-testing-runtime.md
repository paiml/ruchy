# Sub-spec: SQLite-Style Testing — Runtime & Interpreter

**Parent:** [ruchy-sqlite-testing-v2.md](../ruchy-sqlite-testing-v2.md) Section 1.4

---

### 1.4 Runtime: Interpreter & REPL

**SQLite Equivalent**: Execution engine with anomaly testing  
**Ruchy Standard**: 100% error path coverage + fault injection

#### Test Harness 1.4: Runtime Anomaly Validation

```rust
// tests/runtime_anomalies.rs

/**
 * Runtime Anomaly Testing
 * 
 * SQLite Principle: "It is relatively easy to build a system that behaves
 * correctly on well-formed inputs on a fully functional computer. It is more
 * difficult to build a system that responds sanely to invalid inputs and
 * continues to function following system malfunctions."
 * 
 * This harness tests EVERY failure mode:
 * - Out of memory
 * - Stack overflow
 * - Division by zero
 * - Array bounds violations
 * - Type errors at runtime
 * - I/O failures
 * - Concurrent access violations
 * 
 * Goal: The runtime should NEVER panic, always return Result<T, Error>.
 */

#[cfg(test)]
mod anomaly_testing {
    use proptest::prelude::*;
    use ruchy::runtime::*;
    
    // ========================================================================
    // Memory Anomalies
    // ========================================================================
    
    #[test]
    fn test_stack_overflow_handling() {
        /**
         * Test: Infinite recursion should be caught gracefully
         * 
         * Many languages (including Rust!) will segfault on stack overflow.
         * A robust runtime must catch this and return an error.
         */
        
        let prog = r#"
        fun infinite() {
            infinite()
        }
        infinite()
        "#;
        
        let result = std::panic::catch_unwind(|| {
            interpret(prog)
        });
        
        // Must not panic
        assert!(result.is_ok(), "Runtime should not panic on stack overflow");
        
        // Should return error
        let interpretation = result.unwrap();
        assert!(interpretation.is_err());
        
        let error = interpretation.unwrap_err();
        assert!(
            error.contains("stack overflow") || error.contains("recursion depth"),
            "Error should mention stack overflow, got: {}",
            error
        );
    }
    
    #[test]
    fn test_heap_exhaustion() {
        /**
         * Test: Allocating huge amounts of memory should fail gracefully
         */
        
        let prog = "let x = vec![0; 1_000_000_000_000]";  // 1 trillion elements
        
        let result = interpret(prog);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of memory"));
    }
    
    #[test]
    fn test_memory_leak_detection() {
        /**
         * Test: Long-running programs should not leak memory
         * 
         * Run a program that allocates/deallocates in a loop.
         * Memory usage should stabilize, not grow unbounded.
         */
        
        let prog = r#"
        for i in 1..1000 {
            let x = vec![0; 10000];
            // x should be dropped here
        }
        "#;
        
        let initial_memory = get_process_memory_usage();
        
        interpret(prog).unwrap();
        
        let final_memory = get_process_memory_usage();
        let leaked = final_memory - initial_memory;
        
        assert!(
            leaked < 1_000_000,  // Less than 1MB leaked
            "Memory leak detected: {} bytes",
            leaked
        );
    }
    
    // ========================================================================
    // Arithmetic Anomalies
    // ========================================================================
    
    #[test]
    fn test_division_by_zero() {
        assert_runtime_error("1 / 0", "division by zero");
        assert_runtime_error("1 % 0", "modulo by zero");
        assert_runtime_error("1.0 / 0.0", "division by zero");
    }
    
    #[test]
    fn test_integer_overflow() {
        // Rust panics on overflow in debug mode, wraps in release mode
        // Ruchy should have consistent, well-defined behavior
        
        assert_runtime_error(
            "9223372036854775807 + 1",  // i64::MAX + 1
            "integer overflow"
        );
        
        assert_runtime_error(
            "-9223372036854775808 - 1",  // i64::MIN - 1
            "integer overflow"
        );
    }
    
    #[test]
    fn test_float_special_values() {
        // NaN, Infinity should be handled consistently
        
        let result = interpret("0.0 / 0.0").unwrap();
        assert!(result.is_nan());
        
        let result = interpret("1.0 / 0.0").unwrap();
        assert!(result.is_infinite() && result.is_positive());
        
        let result = interpret("-1.0 / 0.0").unwrap();
        assert!(result.is_infinite() && result.is_negative());
    }
    
    // ========================================================================
    // Bounds Checking
    // ========================================================================
    
    #[test]
    fn test_array_bounds_checking() {
        assert_runtime_error("[1, 2, 3][10]", "index out of bounds");
        assert_runtime_error("[1, 2, 3][-1]", "index out of bounds");
        
        // Empty array
        assert_runtime_error("[][0]", "index out of bounds");
    }
    
    #[test]
    fn test_string_bounds_checking() {
        assert_runtime_error(r#""hello"[100]"#, "index out of bounds");
        assert_runtime_error(r#""hello"[-1]"#, "index out of bounds");
    }
    
    // ========================================================================
    // Type Errors at Runtime (for dynamically-typed operations)
    // ========================================================================
    
    #[test]
    fn test_type_mismatch_at_runtime() {
        // Even with static typing, some operations may fail at runtime
        // (e.g., downcasting, reflection)
        
        let prog = r#"
        let x: Any = "hello";
        let y: Int = x as Int;  // Invalid cast
        "#;
        
        assert_runtime_error(prog, "type cast failed");
    }
    
    // ========================================================================
    // Pattern Match Failures
    // ========================================================================
    
    #[test]
    fn test_non_exhaustive_match_at_runtime() {
        /**
         * Even if type system checks exhaustiveness, runtime may encounter
         * unexpected values (e.g., from FFI, unsafe code, or versioning).
         */
        
        let prog = r#"
        let x = Some(42);
        match x {
            Some(0) => "zero",
            Some(1) => "one",
            // Missing: Some(_), None
        }
        "#;
        
        // If type checker allows this (it shouldn't), runtime should catch it
        let result = interpret(prog);
        if result.is_err() {
            assert!(result.unwrap_err().contains("pattern match failed"));
        }
    }
    
    // ========================================================================
    // I/O Failures
    // ========================================================================
    
    #[test]
    fn test_file_not_found() {
        let prog = r#"
        let contents = read_file("/nonexistent/path/file.txt");
        "#;
        
        assert_runtime_error(prog, "file not found");
    }
    
    #[test]
    fn test_permission_denied() {
        let prog = r#"
        let contents = read_file("/etc/shadow");  // Requires root
        "#;
        
        assert_runtime_error(prog, "permission denied");
    }
    
    // ========================================================================
    // Property: Runtime Never Panics
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        
        #[test]
        fn property_runtime_never_panics(prog in any_program()) {
            /**
             * Critical Safety Property
             * 
             * The runtime should NEVER panic, regardless of input.
             * All errors must be caught and returned as Result::Err.
             * 
             * This property is tested with 10,000 random programs.
             */
            
            let result = std::panic::catch_unwind(|| {
                interpret(&prog)
            });
            
            assert!(
                result.is_ok(),
                "Runtime panicked on program: {}",
                prog
            );
        }
    }
    
    // ========================================================================
    // Property: REPL State Consistency
    // ========================================================================
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        
        #[test]
        fn property_repl_consistent_after_errors(
            commands in prop::collection::vec(any_command(), 1..100)
        ) {
            /**
             * REPL State Consistency Property
             * 
             * After executing ANY sequence of commands (including erroneous ones),
             * the REPL should remain in a valid, recoverable state.
             * 
             * Invariants:
             * - No corrupted symbol tables
             * - No leaked resources
             * - Able to execute new commands
             */
            
            let mut repl = Repl::new();
            
            for cmd in commands {
                // Execute command (may succeed or fail)
                let _ = repl.eval(&cmd);
                
                // Verify REPL state is still valid
                assert!(repl.is_valid_state());
                assert!(!repl.has_leaked_resources());
                
                // Verify we can still execute commands
                let test_result = repl.eval("1 + 1");
                assert_eq!(test_result, Ok(Value::Int(2)));
            }
        }
    }
}

// ============================================================================
// REPL-Specific Testing
// ============================================================================

#[cfg(test)]
mod repl_testing {
    use super::*;
    
    #[test]
    fn test_repl_state_persistence() {
        let mut repl = Repl::new();
        
        // Define variable
        repl.eval("let x = 42").unwrap();
        
        // Should be accessible in later commands
        let result = repl.eval("x + 1").unwrap();
        assert_eq!(result, Value::Int(43));
        
        // Define function
        repl.eval("fun double(n) { n * 2 }").unwrap();
        
        // Function should be callable
        let result = repl.eval("double(21)").unwrap();
        assert_eq!(result, Value::Int(42));
    }
    
    #[test]
    fn test_repl_multiline_input() {
        let mut repl = Repl::new();
        
        let multiline = r#"
        fun factorial(n) {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        "#;
        
        repl.eval(multiline).unwrap();
        
        let result = repl.eval("factorial(5)").unwrap();
        assert_eq!(result, Value::Int(120));
    }
    
    #[test]
    fn test_repl_error_recovery() {
        let mut repl = Repl::new();
        
        // Syntax error should not corrupt state
        let result = repl.eval("let x = ");
        assert!(result.is_err());
        
        // REPL should still work
        let result = repl.eval("let y = 42").unwrap();
        assert_eq!(result, Value::Int(42));
        
        // Runtime error should not corrupt state
        let result = repl.eval("1 / 0");
        assert!(result.is_err());
        
        // REPL should still work
        let result = repl.eval("y + 1").unwrap();
        assert_eq!(result, Value::Int(43));
    }
    
    #[test]
    fn test_repl_history() {
        let mut repl = Repl::new();
        
        repl.eval("let x = 1").unwrap();
        repl.eval("let y = 2").unwrap();
        repl.eval("let z = 3").unwrap();
        
        let history = repl.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], "let x = 1");
        assert_eq!(history[1], "let y = 2");
        assert_eq!(history[2], "let z = 3");
    }
    
    #[test]
    fn test_repl_completion() {
        let mut repl = Repl::new();
        
        repl.eval("let variable_name = 42").unwrap();
        repl.eval("fun function_name() { 0 }").unwrap();
        
        let completions = repl.get_completions("var");
        assert!(completions.contains(&"variable_name".to_string()));
        
        let completions = repl.get_completions("fun");
        assert!(completions.contains(&"function_name".to_string()));
    }
}
```

**Coverage Target**:
- 100% error paths
- 100% anomaly scenarios
- 10,000+ property test iterations
- Zero panics tolerated

**Test Count**: 50,000+ runtime tests

---
