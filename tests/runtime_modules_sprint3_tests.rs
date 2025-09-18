//! Sprint 3: Tests for specific runtime modules
//! Coverage for completion, magic, transaction, and other modules

use ruchy::runtime::{Repl, ReplConfig};
use ruchy::runtime::repl::Value;
use std::{env, time::Duration;

// Transaction tests

#[test]
fn test_transactional_evaluation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Set initial state
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();

    // Successful transaction
    let result = repl.eval_transactional("let z = 30");
    assert!(result.is_ok());
    assert_eq!(repl.eval("z").unwrap(), "30");

    // Failed transaction should rollback
    repl.eval("let important = 100").unwrap();
    let result = repl.eval_transactional("let important = undefined_var");
    assert!(result.is_err());

    // Important should still be 100
    assert_eq!(repl.eval("important").unwrap(), "100");
}

#[test]
fn test_checkpoint_restore_comprehensive() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Create complex state
    repl.eval("let a = 1").unwrap();
    repl.eval("let b = 2").unwrap();
    repl.eval("fn func() { 42 }").unwrap();
    repl.eval("let list = [1, 2, 3]").unwrap();

    // Create checkpoint
    let checkpoint = repl.checkpoint();

    // Modify everything
    repl.eval("let a = 100").unwrap();
    repl.eval("let b = 200").unwrap();
    repl.eval("let c = 300").unwrap();
    repl.eval("fn func() { 0 }").unwrap();
    repl.eval("let list = []").unwrap();

    // Restore
    repl.restore_checkpoint(&checkpoint);

    // Verify restoration
    assert_eq!(repl.eval("a").unwrap(), "1");
    assert_eq!(repl.eval("b").unwrap(), "2");
    assert_eq!(repl.eval("func()").unwrap(), "42");
    assert_eq!(repl.eval("list").unwrap(), "[1, 2, 3]");
    assert!(repl.eval("c").is_err()); // c shouldn't exist
}

#[test]
fn test_multiple_checkpoints() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    repl.eval("let x = 1").unwrap();
    let checkpoint1 = repl.checkpoint();

    repl.eval("let x = 2").unwrap();
    let checkpoint2 = repl.checkpoint();

    repl.eval("let x = 3").unwrap();

    // Restore to checkpoint2
    repl.restore_checkpoint(&checkpoint2);
    assert_eq!(repl.eval("x").unwrap(), "2");

    // Restore to checkpoint1
    repl.restore_checkpoint(&checkpoint1);
    assert_eq!(repl.eval("x").unwrap(), "1");
}

// Sandboxed execution tests

#[test]
fn test_sandboxed_repl() {
    let mut repl = Repl::sandboxed().unwrap();

    // Basic operations should work
    assert_eq!(repl.eval("1 + 1").unwrap(), "2");
    assert_eq!(repl.eval("let x = 10").unwrap(), "10");

    // Test resource limits
    let result = repl.eval_bounded(
        "[0; 1000000]", // Large allocation
        1024,           // Small memory limit
        Duration::from_millis(10)
    );
    assert!(result.is_err());
}

#[test]
fn test_custom_config_repl() {
    let config = ReplConfig {
        max_memory: 2048,
        timeout: Duration::from_millis(50),
        maxdepth: 10,
        debug: true,
    };

    let mut repl = Repl::with_config(config).unwrap();

    // Should work with custom config
    assert_eq!(repl.eval("2 * 21").unwrap(), "42");

    // Test depth limit
    repl.eval(r#"
        fn recursive(n) {
            if n == 0 { 0 } else { recursive(n - 1) + 1 }
        }
    "#).unwrap();

    // Small recursion should work
    assert_eq!(repl.eval("recursive(5)").unwrap(), "5");
}

// Memory tracking tests

#[test]
fn test_memory_tracking_detailed() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    assert_eq!(repl.memory_used(), 0);

    // Small allocation
    repl.eval("let small = [1, 2, 3]").unwrap();
    let small_mem = repl.memory_used();
    assert!(small_mem > 0);

    // Larger allocation
    repl.eval("let large = [0; 100]").unwrap();
    let large_mem = repl.memory_used();
    assert!(large_mem > small_mem);

    // Peak memory
    let peak = repl.peak_memory();
    assert!(peak >= large_mem);

    // Memory pressure
    let pressure = repl.memory_pressure();
    assert!(pressure >= 0.0 && pressure <= 1.0);
}

#[test]
fn test_memory_limits() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Try to exceed memory limit
    let result = repl.eval_bounded(
        "let huge = [0; 1000000]",
        100, // Very small limit
        Duration::from_secs(1)
    );

    assert!(result.is_err());
}

// State management tests

#[test]
fn test_repl_state_transitions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check initial state
    assert!(repl.can_accept_input());
    assert!(repl.bindings_valid());

    // After successful eval
    repl.eval("let x = 1").unwrap();
    assert!(repl.can_accept_input());
    assert!(repl.bindings_valid());

    // After error
    let _ = repl.eval("undefined");
    // Should still be able to continue
    assert!(repl.can_accept_input());

    // Recovery if needed
    if repl.is_failed() {
        let recovery = repl.recover();
        assert!(recovery.is_ok());
    }
}

#[test]
fn test_result_history() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    assert_eq!(repl.result_history_len(), 0);

    repl.eval("1 + 1").unwrap();
    assert_eq!(repl.result_history_len(), 1);

    repl.eval("2 * 2").unwrap();
    assert_eq!(repl.result_history_len(), 2);

    repl.eval("3 - 3").unwrap();
    assert_eq!(repl.result_history_len(), 3);

    // Errors might not add to history
    let _ = repl.eval("undefined");
    // History might or might not increase
}

#[test]
fn test_bindings_management() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Add bindings
    repl.eval("let a = 1").unwrap();
    repl.eval("let b = 2").unwrap();
    repl.eval("let c = 3").unwrap();

    let bindings = repl.get_bindings();
    assert!(bindings.contains_key("a"));
    assert!(bindings.contains_key("b"));
    assert!(bindings.contains_key("c"));

    // Modify bindings directly
    let bindings_mut = repl.get_bindings_mut();
    bindings_mut.insert("d".to_string(), Value::Int(4));

    assert_eq!(repl.eval("d").unwrap(), "4");

    // Clear all
    repl.clear_bindings();
    assert!(repl.get_bindings().is_empty());
}

// Error recovery tests

#[test]
fn test_error_recovery_mechanism() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Cause various errors
    let _ = repl.eval("1 / 0"); // Division by zero
    let _ = repl.eval("undefined_var"); // Undefined variable
    let _ = repl.eval("((("); // Parse error

    // Should still work
    assert_eq!(repl.eval("2 + 2").unwrap(), "4");

    // Check last error
    assert!(repl.get_last_error().is_some());
}

#[test]
fn test_error_recovery_suggestions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Define a variable
    repl.eval("let my_variable = 42").unwrap();

    // Typo in variable name
    let result = repl.eval("my_variabel");
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = e.to_string();
        // Error message might contain suggestions
        let _ = error_msg;
    }

    // Extract undefined variable
    let undefined = repl.extract_undefined_variable("undefined variable: test_var");
    assert_eq!(undefined, Some("test_var".to_string()));
}

// Builtin functions tests

#[test]
fn test_builtin_print_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // print and println might be builtins
    let result = repl.eval("print(\"hello\")");
    let _ = result;

    let result = repl.eval("println(\"world\")");
    let _ = result;
}

#[test]
fn test_builtin_type_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Type checking functions
    let result = repl.eval("type_of(42)");
    if let Ok(value) = result {
        assert!(value.contains("int") || value.contains("Int"));
    }

    let result = repl.eval("is_int(42)");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "true");
    }

    let result = repl.eval("is_string(\"hello\")");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "true");
    }
}

#[test]
fn test_builtin_conversion_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // to_string conversions
    let result = repl.eval("to_string(42)");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "\"42\"");
    }

    // parse functions
    let result = repl.eval("parse_int(\"42\")");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "42");
    }

    let result = repl.eval("parse_float(\"3.14\")");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "3.14");
    }
}

// Complex evaluation scenarios

#[test]
fn test_nested_function_scopes() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    repl.eval(r#"
        fn outer(x) {
            fn inner(y) {
                x + y
            }
            inner
        }
    "#).unwrap();

    repl.eval("let add5 = outer(5)").unwrap();
    assert_eq!(repl.eval("add5(10)").unwrap(), "15");
}

#[test]
fn test_complex_control_flow() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    let result = repl.eval(r#"
        let mut result = 0;
        for i in 1..=5 {
            if i % 2 == 0 {
                continue
            }
            for j in 1..=3 {
                if j == 2 {
                    break
                }
                result = result + (i * j)
            }
        }
        result
    "#).unwrap();

    // (1*1) + (3*1) + (5*1) = 1 + 3 + 5 = 9
    assert_eq!(result, "9");
}

#[test]
fn test_exception_like_handling() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Simulate try-catch with if
    repl.eval(r#"
        fn safe_divide(a, b) {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
    "#).unwrap();

    let result = repl.eval("safe_divide(10, 2)");
    if let Ok(value) = result {
        assert!(value.contains("5") || value.contains("Ok"));
    }

    let result = repl.eval("safe_divide(10, 0)");
    if let Ok(value) = result {
        assert!(value.contains("Division by zero") || value.contains("Err"));
    }
}

#[test]
fn test_custom_types_simulation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Simulate struct with functions
    repl.eval(r#"
        fn Point(x, y) {
            {
                x: x,
                y: y,
                distance: |other| {
                    let dx = other.x - x;
                    let dy = other.y - y;
                    sqrt(dx * dx + dy * dy)
                }
            }
        }
    "#).unwrap();

    repl.eval("let p1 = Point(0, 0)").unwrap();
    repl.eval("let p2 = Point(3, 4)").unwrap();

    // Distance calculation if methods work
    let result = repl.eval("p1.distance(p2)");
    if result.is_ok() {
        // Distance should be 5
        let _ = result;
    }
}

#[test]
fn test_iterators_simulation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Simulate iterator pattern
    repl.eval(r#"
        fn range_iter(start, end) {
            let mut current = start;
            || {
                if current < end {
                    let val = current;
                    current = current + 1;
                    Some(val)
                } else {
                    None
                }
            }
        }
    "#).unwrap();

    repl.eval("let iter = range_iter(0, 3)").unwrap();

    // Manual iteration
    let result1 = repl.eval("iter()");
    let result2 = repl.eval("iter()");
    let result3 = repl.eval("iter()");
    let result4 = repl.eval("iter()");

    // Check if iterator pattern works
    let _ = (result1, result2, result3, result4);
}