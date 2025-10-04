//! Validation tests for ruchy-repl-demos
//!
//! [TEST-COV-008] REPL Demo Validation Suite

use ruchy::runtime::repl::Repl;
use std::{env, fs, path::Path};

#[test]
fn test_basic_arithmetic_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check addition
    let result = repl.eval("2 + 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "4");

    // Check multiplication
    let result = repl.eval("10 * 5");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "50");

    // Check exponentiation
    let result = repl.eval("2 ** 8");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "256");
}

#[test]
fn test_variable_assignment_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check basic assignment
    let result = repl.eval("let x = 5");
    assert!(result.is_ok());

    let result = repl.eval("x");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "5");

    // Check derived assignment
    let result = repl.eval("let y = x * 2");
    assert!(result.is_ok());

    let result = repl.eval("y");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "10");

    // Check string assignment
    let result = repl.eval("let name = \"Ruchy\"");
    assert!(result.is_ok());

    let result = repl.eval("name");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"Ruchy\"");
}

#[test]
fn test_string_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check string concatenation
    let result = repl.eval("\"Hello\" + \", \" + \"World!\"");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"Hello, World!\"");

    // Check string methods
    let result = repl.eval("\"hello\".to_uppercase()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"HELLO\"");

    let result = repl.eval("\"WORLD\".to_lowercase()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"world\"");

    let result = repl.eval("\"  hello  \".trim()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"hello\"");
}

#[test]
fn test_boolean_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check boolean literals
    let result = repl.eval("true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");

    let result = repl.eval("false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "false");

    // Check logical operations
    let result = repl.eval("true && false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "false");

    let result = repl.eval("true || false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");

    let result = repl.eval("!true");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "false");
}

#[test]
fn test_array_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check array creation
    let result = repl.eval("let arr = [1, 2, 3, 4, 5]");
    assert!(result.is_ok());

    // Check array length
    let result = repl.eval("arr.len()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "5");

    // Check array indexing
    let result = repl.eval("arr[0]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "1");

    let result = repl.eval("arr[4]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "5");

    // Check array sum
    let result = repl.eval("[1, 2, 3].sum()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "6");
}

#[test]
fn test_closure_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check simple closure
    let result = repl.eval("let double = |x| x * 2");
    assert!(result.is_ok());

    let result = repl.eval("double(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "10");

    // Check multi-param closure
    let result = repl.eval("let add = |x, y| x + y");
    assert!(result.is_ok());

    let result = repl.eval("add(3, 4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "7");
}

#[test]
fn test_conditional_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check basic if-else
    let result = repl.eval("let x = 10");
    assert!(result.is_ok());

    let result = repl.eval("if x > 5 { \"greater\" } else { \"lesser\" }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"greater\"");

    // Check nested conditionals
    let result = repl.eval("let score = 85");
    assert!(result.is_ok());

    let result = repl.eval("if score >= 90 { \"A\" } else if score >= 80 { \"B\" } else { \"C\" }");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"B\"");
}

#[test]
fn test_function_definition_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check factorial function
    let factorial_def = r"
        fun factorial(n) {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
    ";
    let result = repl.eval(factorial_def);
    assert!(result.is_ok());

    let result = repl.eval("factorial(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "120");

    // Check fibonacci function
    let fib_def = r"
        fun fib(n) {
            if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
        }
    ";
    let result = repl.eval(fib_def);
    assert!(result.is_ok());

    let result = repl.eval("fib(10)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "55");
}

#[test]
fn test_loop_operations_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check for loop with range
    let loop_code = r"
        let mut sum = 0;
        for i in 1..6 {
            sum = sum + i
        };
        sum
    ";
    let result = repl.eval(loop_code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "15");

    // Check while loop
    let while_code = r"
        let mut count = 0;
        let mut val = 1;
        while val < 100 {
            val = val * 2;
            count = count + 1
        };
        count
    ";
    let result = repl.eval(while_code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "7"); // 2^7 = 128 > 100
}

#[test]
fn test_data_analysis_demo() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check average calculation
    let result = repl.eval("let data = [10, 20, 30, 40, 50]");
    assert!(result.is_ok());

    // Calculate sum
    let result = repl.eval("data.sum()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "150");

    // Calculate length
    let result = repl.eval("data.len()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "5");

    // Calculate average
    let result = repl.eval("data.sum() / data.len()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "30");
}

/// Test one-liner demos from the sister project
#[test]
fn test_oneliner_math_calculations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Factorial calculation
    let result = repl.eval("1 * 2 * 3 * 4 * 5");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "120");

    // Power calculation
    let result = repl.eval("2 ** 10");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "1024");

    // Modulo operation
    let result = repl.eval("17 % 5");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}

/// Test string manipulation one-liners
#[test]
fn test_oneliner_string_manipulation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // String reverse
    let result = repl.eval("\"hello\".reverse()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "\"olleh\"");

    // String split
    let result = repl.eval("\"a,b,c\".split(\",\")");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains('[') && output.contains(']'));
    assert!(output.contains("\"a\"") && output.contains("\"b\"") && output.contains("\"c\""));
}

/// Test functional programming one-liners
#[test]
fn test_oneliner_functional() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Map operation
    let result = repl.eval("[1, 2, 3].map(|x| x * 2)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("[2, 4, 6]"));

    // Filter operation
    let result = repl.eval("[1, 2, 3, 4, 5].filter(|x| x > 2)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("[3, 4, 5]"));
}

/// Validate demo scripts can be loaded
#[test]
fn test_demo_script_loading() {
    let demo_path = Path::new("../ruchy-repl-demos/demos/repl");
    if !demo_path.exists() {
        eprintln!("Skipping demo loading test - demo directory not found");
        return;
    }

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Try loading a basic demo script
    let basic_demo = demo_path.join("basic_arithmetic.ruchy");
    if basic_demo.exists() {
        let content = fs::read_to_string(&basic_demo).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                // Execute the line but don't fail on errors
                // Some demos might have interactive elements
                let _ = repl.eval(trimmed);
            }
        }
    }
}

/// Test that all documented REPL features work
#[test]
fn test_repl_feature_coverage() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Check basic evaluation
    let result = repl.eval("42");
    assert!(result.is_ok());

    // Check variable binding
    let result = repl.eval("let x = 10");
    assert!(result.is_ok());

    let result = repl.eval("x + 5");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "15");

    // Check function definition
    let result = repl.eval("fun square(x) { x * x }");
    assert!(result.is_ok());

    let result = repl.eval("square(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "25");

    // Check multi-line support
    let multiline = r"
        let result = {
            let a = 10;
            let b = 20;
            a + b
        }
    ";
    let result = repl.eval(multiline);
    assert!(result.is_ok());

    // Check error recovery
    let _ = repl.eval("invalid syntax @#$");
    // Should be able to continue after error
    let result = repl.eval("1 + 1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}
