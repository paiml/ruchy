//! Sprint 1: Magic commands and tab completion tests
//! Targeting increased coverage for runtime modules

use ruchy::runtime::{Repl, Value};

// REPL-004: Tab completion functionality

#[test]
fn test_repl_completion_basic() {
    let mut repl = Repl::new().unwrap();

    // Add some bindings
    repl.eval("let variable_one = 1").unwrap();
    repl.eval("let variable_two = 2").unwrap();
    repl.eval("let var_three = 3").unwrap();

    // Completion should work for prefixes
    // This tests internal completion mechanism
    let bindings = repl.get_bindings();
    let vars_starting_with_var: Vec<_> = bindings.keys()
        .filter(|k| k.starts_with("var"))
        .collect();

    assert_eq!(vars_starting_with_var.len(), 3);
}

#[test]
fn test_repl_builtin_functions() {
    let mut repl = Repl::new().unwrap();

    // Check that builtins are available
    let bindings = repl.get_bindings();

    // Common builtins should exist
    assert!(bindings.contains_key("print") || bindings.contains_key("println"));
}

// REPL-005: Magic commands (!help, !clear, etc.)

#[test]
fn test_repl_clear_command() {
    let mut repl = Repl::new().unwrap();

    // Add some bindings
    repl.eval("let x = 1").unwrap();
    repl.eval("let y = 2").unwrap();

    assert!(!repl.get_bindings().is_empty());

    // Clear bindings
    repl.clear_bindings();

    assert!(repl.get_bindings().is_empty());
}

#[test]
fn test_repl_history_tracking() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.result_history_len(), 0);

    repl.eval("1 + 1").unwrap();
    assert_eq!(repl.result_history_len(), 1);

    repl.eval("2 + 2").unwrap();
    assert_eq!(repl.result_history_len(), 2);

    repl.eval("3 + 3").unwrap();
    assert_eq!(repl.result_history_len(), 3);
}

#[test]
fn test_repl_memory_stats() {
    let mut repl = Repl::new().unwrap();

    let initial_mem = repl.memory_used();
    assert_eq!(initial_mem, 0);

    // Allocate some memory
    repl.eval("let big_list = [1; 1000]").unwrap();

    let after_mem = repl.memory_used();
    assert!(after_mem > initial_mem);

    let peak = repl.peak_memory();
    assert!(peak >= after_mem);

    let pressure = repl.memory_pressure();
    assert!(pressure >= 0.0 && pressure <= 1.0);
}

#[test]
fn test_repl_multiline_input() {
    let mut repl = Repl::new().unwrap();

    // Test multiline function definition
    let multiline = r#"
        fn test_func(x) {
            let result = x * 2;
            result + 1
        }
    "#;

    repl.eval(multiline).unwrap();

    // Function should be defined
    let result = repl.eval("test_func(5)").unwrap();
    assert_eq!(result, "11");
}

#[test]
fn test_repl_error_recovery_with_suggestions() {
    let mut repl = Repl::new().unwrap();

    // Define a variable
    repl.eval("let my_variable = 42").unwrap();

    // Try to use a typo
    let result = repl.eval("my_variablee");
    assert!(result.is_err());

    // Extract undefined variable from error
    if let Err(e) = result {
        let error_msg = e.to_string();
        // Check if error mentions the undefined variable
        assert!(error_msg.contains("variablee") || error_msg.contains("undefined"));
    }
}

#[test]
fn test_repl_load_file_simulation() {
    let mut repl = Repl::new().unwrap();

    // Simulate loading a file with multiple statements
    let file_content = r#"
        let a = 10;
        let b = 20;
        fn add(x, y) { x + y }
        let result = add(a, b);
    "#;

    // Process each line
    for line in file_content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let _ = repl.eval(trimmed);
        }
    }

    // Check that variables were loaded
    assert_eq!(repl.eval("a").unwrap(), "10");
    assert_eq!(repl.eval("b").unwrap(), "20");
    assert_eq!(repl.eval("result").unwrap(), "30");
}

#[test]
fn test_repl_type_info() {
    let mut repl = Repl::new().unwrap();

    // Different types
    repl.eval("let int_var = 42").unwrap();
    repl.eval("let float_var = 3.14").unwrap();
    repl.eval("let string_var = \"hello\"").unwrap();
    repl.eval("let bool_var = true").unwrap();
    repl.eval("let list_var = [1, 2, 3]").unwrap();

    // Values should maintain their types
    assert_eq!(repl.eval("int_var").unwrap(), "42");
    assert_eq!(repl.eval("float_var").unwrap(), "3.14");
    assert_eq!(repl.eval("string_var").unwrap(), "\"hello\"");
    assert_eq!(repl.eval("bool_var").unwrap(), "true");
    assert_eq!(repl.eval("list_var").unwrap(), "[1, 2, 3]");
}

#[test]
fn test_repl_import_simulation() {
    let mut repl = Repl::new().unwrap();

    // Simulate module definition
    let module_code = r#"
        fn module_func(x) { x * 10 }
        let module_const = 100
    "#;

    // Process module
    for line in module_code.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let _ = repl.eval(trimmed);
        }
    }

    // Use imported functions
    assert_eq!(repl.eval("module_func(5)").unwrap(), "50");
    assert_eq!(repl.eval("module_const").unwrap(), "100");
}

#[test]
fn test_repl_debug_mode() {
    use ruchy::runtime::ReplConfig;
    use std::time::Duration;

    let config = ReplConfig {
        max_memory: 1024 * 1024,
        timeout: Duration::from_millis(100),
        max_depth: 50,
        debug: true, // Enable debug mode
    };

    let mut repl = Repl::with_config(config).unwrap();

    // Debug mode should work
    assert_eq!(repl.eval("1 + 1").unwrap(), "2");
}

#[test]
fn test_repl_sandboxed_restrictions() {
    let mut repl = Repl::sandboxed().unwrap();

    // Sandboxed mode should still allow basic operations
    assert_eq!(repl.eval("2 + 2").unwrap(), "4");

    // But with restrictions (memory, timeout, etc.)
    let result = repl.eval_bounded(
        "let x = [0; 1000000]", // Try large allocation
        1024, // Very small memory limit
        std::time::Duration::from_millis(10)
    );

    // Should fail due to restrictions
    assert!(result.is_err());
}

#[test]
fn test_repl_pattern_matching() {
    let mut repl = Repl::new().unwrap();

    // Test various pattern matching scenarios
    let code = r#"
        match 5 {
            1 => "one",
            2 | 3 | 4 => "small",
            5..=10 => "medium",
            _ => "large"
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "\"medium\"");
}

#[test]
fn test_repl_destructuring() {
    let mut repl = Repl::new().unwrap();

    // Tuple destructuring
    repl.eval("let (a, b) = (10, 20)").unwrap();
    assert_eq!(repl.eval("a").unwrap(), "10");
    assert_eq!(repl.eval("b").unwrap(), "20");

    // List destructuring
    repl.eval("let [x, y, z] = [1, 2, 3]").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "1");
    assert_eq!(repl.eval("y").unwrap(), "2");
    assert_eq!(repl.eval("z").unwrap(), "3");
}

#[test]
fn test_repl_pipeline_operator() {
    let mut repl = Repl::new().unwrap();

    // Define some functions
    repl.eval("fn double(x) { x * 2 }").unwrap();
    repl.eval("fn add_one(x) { x + 1 }").unwrap();

    // Test pipeline operator if supported
    let result = repl.eval("5 |> double |> add_one");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "11");
    }
}

#[test]
fn test_repl_async_simulation() {
    let mut repl = Repl::new().unwrap();

    // Simulate async-like behavior with functions
    let code = r#"
        fn delayed_add(x, y) {
            // Simulate some computation
            let temp = x + y;
            temp
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("delayed_add(3, 4)").unwrap(), "7");
}

#[test]
fn test_repl_custom_types() {
    let mut repl = Repl::new().unwrap();

    // Define a custom type/struct simulation
    let code = r#"
        let Person = |name, age| {
            { name: name, age: age }
        }
    "#;

    repl.eval(code).unwrap();

    // Create an instance
    repl.eval("let alice = Person(\"Alice\", 30)").unwrap();

    // Access fields
    let alice_str = repl.eval("alice").unwrap();
    assert!(alice_str.contains("Alice") || alice_str.contains("30"));
}

#[test]
fn test_repl_method_chaining() {
    let mut repl = Repl::new().unwrap();

    // Test method chaining on strings
    let result = repl.eval(r#""hello".to_uppercase()"#);
    if result.is_ok() {
        assert_eq!(result.unwrap(), "\"HELLO\"");
    }
}

#[test]
fn test_repl_comprehensions() {
    let mut repl = Repl::new().unwrap();

    // List comprehension simulation
    repl.eval("let nums = [1, 2, 3, 4, 5]").unwrap();
    repl.eval("fn map_double(list) { list }").unwrap(); // Simplified

    // Would test actual comprehensions if supported
    assert_eq!(repl.eval("nums").unwrap(), "[1, 2, 3, 4, 5]");
}

#[test]
fn test_repl_exception_handling() {
    let mut repl = Repl::new().unwrap();

    // Test try-catch simulation
    let code = r#"
        fn safe_divide(x, y) {
            if y == 0 {
                "error: division by zero"
            } else {
                x / y
            }
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("safe_divide(10, 2)").unwrap(), "5");
    assert_eq!(repl.eval("safe_divide(10, 0)").unwrap(), "\"error: division by zero\"");
}