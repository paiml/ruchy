// BOOK-003: Void Function Return Type Inference Tests
// Following Toyota Way TDD - RED-GREEN-REFACTOR phases

#![allow(clippy::needless_raw_string_hashes)] // Test file with embedded Ruchy code
#![allow(clippy::expect_used)] // Test assertions need expect for clear error messages

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Helper to transpile code and return the generated Rust code
fn transpile(code: &str) -> String {
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Should transpile");
    result.to_string()
}

// ============================================================================
// PHASE 1: Basic Void Function Detection Tests
// ============================================================================

#[test]
fn test_println_function_no_return_type() {
    let code = r#"
        fun log_message(msg) {
            println(msg)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("-> i32"), "println function should not have return type");
    assert!(!transpiled.contains("->"), "println function should be void");
}

#[test]
fn test_print_function_no_return_type() {
    let code = r#"
        fun display(text) {
            print(text)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "print function should be void");
}

#[test]
fn test_multiple_println_no_return_type() {
    let code = r#"
        fun log_startup() {
            println("System starting...")
            println("Loading configuration...")
            println("Ready!")
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Multiple println calls should be void");
}

// ============================================================================
// PHASE 2: Control Flow with Void Tests
// ============================================================================

#[test]
fn test_if_with_println_no_return_type() {
    let code = r#"
        fun validate_and_log(x) {
            if x > 0 {
                println("Valid")
            } else {
                println("Invalid")
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "if with only println should be void");
}

#[test]
fn test_config_management_example() {
    let code = r#"
        fun apply_config(value) {
            println("Applying configuration...")
            if value > 0 {
                println("Configuration applied successfully")
            } else {
                println("Invalid configuration")
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("-> i32"), "Config management function should be void");
}

#[test]
fn test_while_loop_void() {
    let code = r#"
        fun process_loop(n) {
            while n > 0 {
                println(n)
                n = n - 1
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "While loop should be void");
}

#[test]
fn test_for_loop_void() {
    let code = r#"
        fun print_numbers() {
            for i in [1, 2, 3] {
                println(i)
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "For loop should be void");
}

// ============================================================================
// PHASE 3: Assignment and Side Effects Tests
// ============================================================================

#[test]
fn test_assignment_is_void() {
    let code = r#"
        fun set_value(x) {
            let mut state = 0
            state = x
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Assignment should be void");
}

#[test]
fn test_compound_assignment_void() {
    let code = r#"
        fun increment(x) {
            let mut counter = x
            counter += 1
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Compound assignment should be void");
}

// ============================================================================
// PHASE 4: Mixed Value/Void Detection Tests
// ============================================================================

#[test]
fn test_value_returning_function() {
    let code = r#"
        fun calculate(x) {
            println("Calculating...")
            x * 2
        }
    "#;
    let transpiled = transpile(code);
    assert!(transpiled.contains("-> i32"), "Function returning value should have return type");
}

#[test]
fn test_explicit_return_has_type() {
    let code = r#"
        fun compute(x) {
            if x > 10 {
                return x * 2
            }
            x + 1
        }
    "#;
    let transpiled = transpile(code);
    assert!(transpiled.contains("-> i32"), "Function with explicit return should have return type");
}

#[test]
fn test_block_with_value_expression() {
    let code = r#"
        fun process(x) {
            {
                println("Processing")
                x * 2
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(transpiled.contains("-> i32"), "Block returning value should have return type");
}

// ============================================================================
// PHASE 5: Other Void Functions Tests
// ============================================================================

#[test]
fn test_panic_is_void() {
    let code = r#"
        fun fail_fast(msg) {
            panic(msg)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("-> i32"), "panic function should be void");
}

#[test]
fn test_assert_is_void() {
    let code = r#"
        fun check_invariant(x) {
            assert(x > 0)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "assert function should be void");
}

#[test]
fn test_debug_functions_void() {
    let code = r#"
        fun debug_value(x) {
            dbg(x)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "dbg function should be void");
}

#[test]
fn test_eprintln_void() {
    let code = r#"
        fun log_error(msg) {
            eprintln(msg)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "eprintln function should be void");
}

// ============================================================================
// PHASE 6: Special Cases Tests
// ============================================================================

#[test]
fn test_main_function_no_return_type() {
    let code = r#"
        fun main() {
            println("Hello, world!")
            42
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("fn main() -> i32"), "main should never have return type");
    assert!(transpiled.contains("fn main()"), "main should have no return type");
}

#[test]
fn test_empty_function_body() {
    let code = r#"
        fun do_nothing() {
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Empty function should be void");
}

#[test]
fn test_unit_literal() {
    let code = r#"
        fun return_unit() {
            ()
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Unit literal should be void");
}

// ============================================================================
// PHASE 7: Complex Nested Structure Tests
// ============================================================================

#[test]
fn test_nested_if_all_void() {
    let code = r#"
        fun complex_logic(x, y) {
            if x > 0 {
                if y > 0 {
                    println("Both positive")
                } else {
                    println("X positive, Y not")
                }
            } else {
                println("X not positive")
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Nested if with all void branches should be void");
}

#[test]
fn test_match_all_void_branches() {
    let code = r#"
        fun handle_option(opt) {
            match opt {
                Some(x) => println(x),
                None => println("Nothing")
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Match with all void branches should be void");
}

#[test]
fn test_match_with_value_branches() {
    let code = r#"
        fun process_option(opt) {
            match opt {
                Some(x) => x * 2,
                None => 0
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(transpiled.contains("-> i32"), "Match returning values should have return type");
}

// ============================================================================
// PHASE 8: Book Example Regression Tests
// ============================================================================

#[test]
fn test_book_config_management() {
    // From test_03_config_management.ruchy
    let code = r#"
        fun configure_system(level) {
            println("Configuring system...")
            
            if level == 0 {
                println("Using default configuration")
            } else {
                println("Using custom configuration level: ")
                println(level)
            }
            
            println("Configuration complete")
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("-> i32"), "Book config example should be void");
}

#[test]
fn test_book_logging_example() {
    let code = r#"
        fun log_application_state(state, verbose) {
            if verbose {
                println("=== Application State ===")
                println("State value: ")
                println(state)
                println("========================")
            } else {
                print("State: ")
                println(state)
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Book logging example should be void");
}

// ============================================================================
// PHASE 9: Performance and Edge Cases
// ============================================================================

#[test]
fn test_deeply_nested_void() {
    let code = r#"
        fun deeply_nested(x) {
            if x > 0 {
                if x > 10 {
                    if x > 100 {
                        println("Very large")
                    } else {
                        println("Large")
                    }
                } else {
                    println("Small")
                }
            } else {
                println("Non-positive")
            }
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Deeply nested void should be detected");
}

#[test]
fn test_mixed_expressions_ending_void() {
    let code = r#"
        fun process_and_log(x) {
            let result = x * 2
            let formatted = result + 1
            println(formatted)
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Function ending with void should be void");
}

#[test]
fn test_early_return_void() {
    let code = r#"
        fun check_and_log(x) {
            if x < 0 {
                println("Negative")
                return
            }
            println("Non-negative")
        }
    "#;
    let transpiled = transpile(code);
    assert!(!transpiled.contains("->"), "Early void return should make function void");
}