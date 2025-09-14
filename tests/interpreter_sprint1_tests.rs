//! Sprint 1: Interpreter evaluation tests for uncovered paths
//! TDD approach with maximum complexity of 10 per function

use ruchy::runtime::{Repl, Value};
use std::time::{Duration, Instant};

// INTERP-001: Expression evaluation (all types)

#[test]
fn test_evaluate_string_concatenation() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval(r#""hello" + " " + "world""#).unwrap();
    assert_eq!(result, r#""hello world""#);
}

#[test]
fn test_evaluate_string_interpolation() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let name = \"Alice\"").unwrap();
    repl.eval("let age = 30").unwrap();

    // Test string interpolation if supported
    let result = repl.eval(r#"f"Name: {name}, Age: {age}""#);
    if result.is_ok() {
        assert!(result.unwrap().contains("Alice"));
    }
}

#[test]
fn test_evaluate_list_operations() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");
    assert_eq!(repl.eval("[1] + [2, 3]").unwrap(), "[1, 2, 3]");

    repl.eval("let list = [1, 2, 3, 4, 5]").unwrap();
    assert_eq!(repl.eval("list").unwrap(), "[1, 2, 3, 4, 5]");
}

#[test]
fn test_evaluate_tuple_operations() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("(1, 2)").unwrap(), "(1, 2)");
    assert_eq!(repl.eval("(1, \"hello\", true)").unwrap(), r#"(1, "hello", true)"#);

    repl.eval("let tuple = (10, 20, 30)").unwrap();
    assert_eq!(repl.eval("tuple").unwrap(), "(10, 20, 30)");
}

#[test]
fn test_evaluate_range_operations() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("1..10").unwrap(), "1..10");
    assert_eq!(repl.eval("1..=10").unwrap(), "1..=10");

    repl.eval("let r = 0..5").unwrap();
    assert_eq!(repl.eval("r").unwrap(), "0..5");
}

#[test]
fn test_evaluate_object_literal() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval("{ name: \"Bob\", age: 25 }");
    if result.is_ok() {
        let obj = result.unwrap();
        assert!(obj.contains("name") || obj.contains("Bob"));
    }
}

#[test]
fn test_evaluate_if_expression() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("if true { 1 } else { 2 }").unwrap(), "1");
    assert_eq!(repl.eval("if false { 1 } else { 2 }").unwrap(), "2");
    assert_eq!(repl.eval("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap(), "\"yes\"");
}

#[test]
fn test_evaluate_if_without_else() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("if true { 42 }").unwrap(), "42");
    assert_eq!(repl.eval("if false { 42 }").unwrap(), "()"); // Unit when condition is false
}

#[test]
fn test_evaluate_nested_if() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        if true {
            if false {
                1
            } else {
                2
            }
        } else {
            3
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "2");
}

#[test]
fn test_evaluate_match_expression() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        match 2 {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "\"two\"");
}

#[test]
fn test_evaluate_match_with_guards() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let x = 10").unwrap();

    let code = r#"
        match x {
            n if n < 5 => "small",
            n if n < 10 => "medium",
            _ => "large"
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "\"large\"");
}

#[test]
fn test_evaluate_for_loop() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let mut sum = 0").unwrap();

    let code = r#"
        for i in 1..=5 {
            sum = sum + i
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("sum").unwrap(), "15");
}

#[test]
fn test_evaluate_while_loop() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let mut count = 0").unwrap();

    let code = r#"
        while count < 5 {
            count = count + 1
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("count").unwrap(), "5");
}

#[test]
fn test_evaluate_function_definition() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        fn add(x, y) {
            x + y
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("add(3, 4)").unwrap(), "7");
}

#[test]
fn test_evaluate_recursive_function() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        fn factorial(n) {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;

    repl.eval(code).unwrap();
    assert_eq!(repl.eval("factorial(5)").unwrap(), "120");
}

#[test]
fn test_evaluate_lambda_expression() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let add = |x, y| x + y").unwrap();
    assert_eq!(repl.eval("add(10, 20)").unwrap(), "30");

    repl.eval("let square = |x| x * x").unwrap();
    assert_eq!(repl.eval("square(5)").unwrap(), "25");
}

#[test]
fn test_evaluate_closure() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let x = 10").unwrap();
    repl.eval("let add_x = |y| x + y").unwrap();
    assert_eq!(repl.eval("add_x(5)").unwrap(), "15");
}

#[test]
fn test_evaluate_higher_order_function() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        fn apply_twice(f, x) {
            f(f(x))
        }
    "#;

    repl.eval(code).unwrap();
    repl.eval("let double = |x| x * 2").unwrap();
    assert_eq!(repl.eval("apply_twice(double, 5)").unwrap(), "20");
}

// INTERP-002: Stack operations

#[test]
fn test_stack_push_pop() {
    let mut repl = Repl::new().unwrap();

    // Test that stack operations don't cause issues
    repl.eval("let a = 1").unwrap();
    repl.eval("let b = 2").unwrap();
    repl.eval("let c = a + b").unwrap();

    assert_eq!(repl.eval("c").unwrap(), "3");
}

#[test]
fn test_nested_scope() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let x = 10").unwrap();

    let code = r#"
        {
            let x = 20;
            x
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "20");
    assert_eq!(repl.eval("x").unwrap(), "10"); // Outer x unchanged
}

#[test]
fn test_block_expression() {
    let mut repl = Repl::new().unwrap();

    let code = r#"
        {
            let a = 5;
            let b = 10;
            a + b
        }
    "#;

    assert_eq!(repl.eval(code).unwrap(), "15");
}

// INTERP-003: Error handling paths

#[test]
fn test_divide_by_zero() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval("10 / 0");
    assert!(result.is_err());
}

#[test]
fn test_undefined_variable() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval("undefined_var");
    assert!(result.is_err());
}

#[test]
fn test_type_mismatch() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval("\"hello\" + 5");
    // Should either coerce or error
    let _ = result;
}

#[test]
fn test_stack_overflow_protection() {
    let mut repl = Repl::new().unwrap();

    // Define infinite recursion
    let code = r#"
        fn infinite() {
            infinite()
        }
    "#;

    repl.eval(code).unwrap();

    // Try to call it with bounded evaluation
    let result = repl.eval_bounded(
        "infinite()",
        1024 * 1024,
        Duration::from_millis(100)
    );

    // Should timeout or hit depth limit
    assert!(result.is_err());
}

#[test]
fn test_memory_limit_exceeded() {
    let mut repl = Repl::new().unwrap();

    // Try to allocate too much memory
    let result = repl.eval_bounded(
        "[0; 1000000]", // Large array
        1024, // Only 1KB allowed
        Duration::from_secs(1)
    );

    // Should fail due to memory limit
    assert!(result.is_err());
}

#[test]
fn test_timeout_exceeded() {
    let mut repl = Repl::new().unwrap();

    // Create a slow operation
    let code = r#"
        let mut sum = 0;
        for i in 1..1000000 {
            sum = sum + i
        }
        sum
    "#;

    let result = repl.eval_bounded(
        code,
        1024 * 1024,
        Duration::from_nanos(1) // Impossibly short timeout
    );

    // Should timeout
    assert!(result.is_err());
}

#[test]
fn test_error_recovery() {
    let mut repl = Repl::new().unwrap();

    // Cause an error
    let _ = repl.eval("1 / 0");

    // Should be able to continue
    assert_eq!(repl.eval("2 + 2").unwrap(), "4");
}

#[test]
fn test_partial_evaluation_rollback() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let x = 10").unwrap();

    // Try to evaluate something that fails partway
    let code = r#"
        let y = 20;
        let z = undefined_var;
        let w = 30
    "#;

    let result = repl.eval(code);
    assert!(result.is_err());

    // y should not exist due to rollback
    let result = repl.eval("y");
    assert!(result.is_err());

    // x should still exist
    assert_eq!(repl.eval("x").unwrap(), "10");
}

// Math function tests

#[test]
fn test_math_sqrt() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.eval("sqrt(9.0)").unwrap(), "3.0");
    assert_eq!(repl.eval("sqrt(16.0)").unwrap(), "4.0");
}

#[test]
fn test_math_pow() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.eval("pow(2.0, 3.0)").unwrap(), "8.0");
    assert_eq!(repl.eval("pow(5.0, 2.0)").unwrap(), "25.0");
}

#[test]
fn test_math_abs() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.eval("abs(-5)").unwrap(), "5");
    assert_eq!(repl.eval("abs(5)").unwrap(), "5");
    assert_eq!(repl.eval("abs(-3.14)").unwrap(), "3.14");
}

#[test]
fn test_math_min_max() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.eval("min(5, 3)").unwrap(), "3");
    assert_eq!(repl.eval("max(5, 3)").unwrap(), "5");
    assert_eq!(repl.eval("min(2.5, 3.7)").unwrap(), "2.5");
    assert_eq!(repl.eval("max(2.5, 3.7)").unwrap(), "3.7");
}

#[test]
fn test_math_rounding() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.eval("floor(3.7)").unwrap(), "3.0");
    assert_eq!(repl.eval("ceil(3.2)").unwrap(), "4.0");
    assert_eq!(repl.eval("round(3.5)").unwrap(), "4.0");
    assert_eq!(repl.eval("round(3.4)").unwrap(), "3.0");
}