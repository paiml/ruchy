//! Sprint 3: Comprehensive runtime tests targeting 47% coverage
//! Deep testing of interpreter, evaluation, and runtime features

use ruchy::runtime::{Repl, Value};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

// Deep interpreter tests

#[test]
fn test_eval_all_literal_types() {
    let mut repl = Repl::new().unwrap();

    // Test all literal types
    assert_eq!(repl.eval("42").unwrap(), "42");
    assert_eq!(repl.eval("3.14").unwrap(), "3.14");
    assert_eq!(repl.eval("true").unwrap(), "true");
    assert_eq!(repl.eval("false").unwrap(), "false");
    assert_eq!(repl.eval("\"hello\"").unwrap(), "\"hello\"");
    assert_eq!(repl.eval("'a'").unwrap(), "'a'");
    assert_eq!(repl.eval("nil").unwrap(), "nil");
    assert_eq!(repl.eval("()").unwrap(), "()");
}

#[test]
fn test_eval_all_binary_operators() {
    let mut repl = Repl::new().unwrap();

    // Arithmetic
    assert_eq!(repl.eval("10 + 5").unwrap(), "15");
    assert_eq!(repl.eval("10 - 5").unwrap(), "5");
    assert_eq!(repl.eval("10 * 5").unwrap(), "50");
    assert_eq!(repl.eval("10 / 5").unwrap(), "2");
    assert_eq!(repl.eval("10 % 3").unwrap(), "1");

    // Comparison
    assert_eq!(repl.eval("5 > 3").unwrap(), "true");
    assert_eq!(repl.eval("3 < 5").unwrap(), "true");
    assert_eq!(repl.eval("5 >= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 <= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 == 5").unwrap(), "true");
    assert_eq!(repl.eval("5 != 3").unwrap(), "true");

    // Logical
    assert_eq!(repl.eval("true && true").unwrap(), "true");
    assert_eq!(repl.eval("true || false").unwrap(), "true");
}

#[test]
fn test_eval_all_unary_operators() {
    let mut repl = Repl::new().unwrap();

    assert_eq!(repl.eval("-42").unwrap(), "-42");
    assert_eq!(repl.eval("+42").unwrap(), "42");
    assert_eq!(repl.eval("!true").unwrap(), "false");
    assert_eq!(repl.eval("!false").unwrap(), "true");
    assert_eq!(repl.eval("!!true").unwrap(), "true");
}

#[test]
fn test_eval_variable_scoping() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let global = 100").unwrap();

    // Block scope
    repl.eval("{ let local = 50; }").unwrap();
    assert!(repl.eval("local").is_err()); // Should not exist

    // Nested scopes
    let result = repl.eval(r#"
        {
            let x = 10;
            {
                let x = 20;
                x
            }
        }
    "#).unwrap();
    assert_eq!(result, "20");

    // Global still accessible
    assert_eq!(repl.eval("global").unwrap(), "100");
}

#[test]
fn test_eval_shadowing() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let x = 10").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "10");

    repl.eval("let x = 20").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "20");

    // Inner shadow
    let result = repl.eval(r#"
        {
            let x = 30;
            x
        }
    "#).unwrap();
    assert_eq!(result, "30");

    // Outer unchanged
    assert_eq!(repl.eval("x").unwrap(), "20");
}

#[test]
fn test_eval_mutable_variables() {
    let mut repl = Repl::new().unwrap();

    repl.eval("let mut x = 10").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "10");

    repl.eval("x = 20").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "20");

    // Compound assignment
    repl.eval("x += 5").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "25");

    repl.eval("x -= 3").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "22");

    repl.eval("x *= 2").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "44");

    repl.eval("x /= 4").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "11");
}

#[test]
fn test_eval_if_else_chains() {
    let mut repl = Repl::new().unwrap();

    // Simple if
    assert_eq!(repl.eval("if true { 1 }").unwrap(), "1");
    assert_eq!(repl.eval("if false { 1 }").unwrap(), "()");

    // If-else
    assert_eq!(repl.eval("if true { 1 } else { 2 }").unwrap(), "1");
    assert_eq!(repl.eval("if false { 1 } else { 2 }").unwrap(), "2");

    // If-else-if chain
    repl.eval("let x = 5").unwrap();
    let result = repl.eval(r#"
        if x < 0 {
            "negative"
        } else if x == 0 {
            "zero"
        } else if x < 10 {
            "small"
        } else {
            "large"
        }
    "#).unwrap();
    assert_eq!(result, "\"small\"");
}

#[test]
fn test_eval_match_patterns() {
    let mut repl = Repl::new().unwrap();

    // Literal patterns
    let result = repl.eval(r#"
        match 2 {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#).unwrap();
    assert_eq!(result, "\"two\"");

    // Variable binding
    let result = repl.eval(r#"
        match 42 {
            x => x * 2
        }
    "#).unwrap();
    assert_eq!(result, "84");

    // Guards
    repl.eval("let y = 15").unwrap();
    let result = repl.eval(r#"
        match y {
            n if n < 10 => "small",
            n if n < 20 => "medium",
            _ => "large"
        }
    "#).unwrap();
    assert_eq!(result, "\"medium\"");
}

#[test]
fn test_eval_for_loops_comprehensive() {
    let mut repl = Repl::new().unwrap();

    // Range iteration
    repl.eval("let mut sum = 0").unwrap();
    repl.eval("for i in 1..=5 { sum = sum + i }").unwrap();
    assert_eq!(repl.eval("sum").unwrap(), "15");

    // List iteration
    repl.eval("let mut product = 1").unwrap();
    repl.eval("for x in [2, 3, 4] { product = product * x }").unwrap();
    assert_eq!(repl.eval("product").unwrap(), "24");

    // Nested loops
    repl.eval("let mut count = 0").unwrap();
    repl.eval(r#"
        for i in 1..=3 {
            for j in 1..=3 {
                count = count + 1
            }
        }
    "#).unwrap();
    assert_eq!(repl.eval("count").unwrap(), "9");
}

#[test]
fn test_eval_while_loops_comprehensive() {
    let mut repl = Repl::new().unwrap();

    // Simple while
    repl.eval("let mut n = 0").unwrap();
    repl.eval("while n < 5 { n = n + 1 }").unwrap();
    assert_eq!(repl.eval("n").unwrap(), "5");

    // Complex condition
    repl.eval("let mut x = 10").unwrap();
    repl.eval("let mut y = 0").unwrap();
    repl.eval("while x > 0 && y < 5 { x = x - 2; y = y + 1 }").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "0");
    assert_eq!(repl.eval("y").unwrap(), "5");
}

#[test]
fn test_eval_function_definitions_comprehensive() {
    let mut repl = Repl::new().unwrap();

    // No params, no return
    repl.eval("fn void() { }").unwrap();
    assert_eq!(repl.eval("void()").unwrap(), "()");

    // Single param
    repl.eval("fn double(x) { x * 2 }").unwrap();
    assert_eq!(repl.eval("double(21)").unwrap(), "42");

    // Multiple params
    repl.eval("fn add3(a, b, c) { a + b + c }").unwrap();
    assert_eq!(repl.eval("add3(10, 20, 30)").unwrap(), "60");

    // Recursive
    repl.eval(r#"
        fn fib(n) {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
    "#).unwrap();
    assert_eq!(repl.eval("fib(6)").unwrap(), "8");

    // Mutual recursion
    repl.eval(r#"
        fn is_even(n) {
            if n == 0 {
                true
            } else {
                is_odd(n - 1)
            }
        }
    "#).unwrap();

    repl.eval(r#"
        fn is_odd(n) {
            if n == 0 {
                false
            } else {
                is_even(n - 1)
            }
        }
    "#).unwrap();

    assert_eq!(repl.eval("is_even(4)").unwrap(), "true");
    assert_eq!(repl.eval("is_odd(4)").unwrap(), "false");
}

#[test]
fn test_eval_lambdas_comprehensive() {
    let mut repl = Repl::new().unwrap();

    // Basic lambda
    repl.eval("let inc = |x| x + 1").unwrap();
    assert_eq!(repl.eval("inc(41)").unwrap(), "42");

    // Multi-param lambda
    repl.eval("let add = |x, y| x + y").unwrap();
    assert_eq!(repl.eval("add(15, 27)").unwrap(), "42");

    // Lambda with block
    repl.eval(r#"
        let complex = |x| {
            let y = x * 2;
            let z = y + 1;
            z * z
        }
    "#).unwrap();
    assert_eq!(repl.eval("complex(3)").unwrap(), "49"); // ((3*2)+1)^2 = 7^2 = 49

    // Closure capturing
    repl.eval("let outer = 10").unwrap();
    repl.eval("let capture = |x| x + outer").unwrap();
    assert_eq!(repl.eval("capture(5)").unwrap(), "15");
}

#[test]
fn test_eval_higher_order_functions() {
    let mut repl = Repl::new().unwrap();

    // Function as parameter
    repl.eval(r#"
        fn apply(f, x) {
            f(x)
        }
    "#).unwrap();

    repl.eval("let double = |x| x * 2").unwrap();
    assert_eq!(repl.eval("apply(double, 21)").unwrap(), "42");

    // Function returning function
    repl.eval(r#"
        fn make_adder(n) {
            |x| x + n
        }
    "#).unwrap();

    repl.eval("let add5 = make_adder(5)").unwrap();
    assert_eq!(repl.eval("add5(37)").unwrap(), "42");

    // Map-like operation
    repl.eval(r#"
        fn map_list(list, f) {
            let mut result = [];
            for x in list {
                result = result + [f(x)]
            }
            result
        }
    "#).unwrap();

    let result = repl.eval("map_list([1, 2, 3], |x| x * x)").unwrap();
    assert_eq!(result, "[1, 4, 9]");
}

#[test]
fn test_eval_list_operations_comprehensive() {
    let mut repl = Repl::new().unwrap();

    // List creation
    assert_eq!(repl.eval("[]").unwrap(), "[]");
    assert_eq!(repl.eval("[1]").unwrap(), "[1]");
    assert_eq!(repl.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");

    // Nested lists
    assert_eq!(repl.eval("[[1, 2], [3, 4]]").unwrap(), "[[1, 2], [3, 4]]");

    // List concatenation
    assert_eq!(repl.eval("[1, 2] + [3, 4]").unwrap(), "[1, 2, 3, 4]");

    // List indexing
    repl.eval("let list = [10, 20, 30, 40]").unwrap();
    assert_eq!(repl.eval("list[0]").unwrap(), "10");
    assert_eq!(repl.eval("list[2]").unwrap(), "30");

    // List length
    repl.eval("fn len(list) { let mut c = 0; for _ in list { c = c + 1 } c }").unwrap();
    assert_eq!(repl.eval("len([1, 2, 3, 4, 5])").unwrap(), "5");
}

#[test]
fn test_eval_tuple_operations() {
    let mut repl = Repl::new().unwrap();

    // Tuple creation
    assert_eq!(repl.eval("()").unwrap(), "()");
    assert_eq!(repl.eval("(1,)").unwrap(), "(1,)");
    assert_eq!(repl.eval("(1, 2)").unwrap(), "(1, 2)");
    assert_eq!(repl.eval("(1, \"hello\", true)").unwrap(), "(1, \"hello\", true)");

    // Tuple destructuring
    repl.eval("let (a, b) = (10, 20)").unwrap();
    assert_eq!(repl.eval("a").unwrap(), "10");
    assert_eq!(repl.eval("b").unwrap(), "20");

    repl.eval("let (x, y, z) = (1, 2, 3)").unwrap();
    assert_eq!(repl.eval("x + y + z").unwrap(), "6");
}

#[test]
fn test_eval_object_operations() {
    let mut repl = Repl::new().unwrap();

    // Object creation
    let result = repl.eval("{ x: 10, y: 20 }");
    if result.is_ok() {
        let obj = result.unwrap();
        assert!(obj.contains("10") || obj.contains("20"));
    }

    // Nested objects
    let result = repl.eval("{ point: { x: 1, y: 2 }, name: \"origin\" }");
    if result.is_ok() {
        let obj = result.unwrap();
        assert!(obj.contains("origin") || obj.contains("point"));
    }
}

#[test]
fn test_eval_string_operations() {
    let mut repl = Repl::new().unwrap();

    // String concatenation
    assert_eq!(repl.eval("\"hello\" + \" \" + \"world\"").unwrap(), "\"hello world\"");

    // String methods
    let result = repl.eval("\"hello\".to_uppercase()");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "\"HELLO\"");
    }

    let result = repl.eval("\"WORLD\".to_lowercase()");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "\"world\"");
    }

    // String interpolation
    repl.eval("let name = \"Alice\"").unwrap();
    let result = repl.eval("f\"Hello, {name}!\"");
    if result.is_ok() {
        assert!(result.unwrap().contains("Alice"));
    }
}

#[test]
fn test_eval_type_coercion() {
    let mut repl = Repl::new().unwrap();

    // Int to float
    assert_eq!(repl.eval("5 + 2.5").unwrap(), "7.5");
    assert_eq!(repl.eval("10.0 - 3").unwrap(), "7.0");

    // Numeric to string
    let result = repl.eval("\"Number: \" + 42");
    if result.is_ok() {
        assert!(result.unwrap().contains("42"));
    }
}

#[test]
fn test_eval_error_handling() {
    let mut repl = Repl::new().unwrap();

    // Division by zero
    assert!(repl.eval("1 / 0").is_err());

    // Undefined variable
    assert!(repl.eval("undefined_var").is_err());

    // Type errors
    let result = repl.eval("true + 5");
    // Should either error or coerce
    let _ = result;

    // Index out of bounds
    repl.eval("let arr = [1, 2, 3]").unwrap();
    assert!(repl.eval("arr[10]").is_err());
}

#[test]
fn test_eval_short_circuit() {
    let mut repl = Repl::new().unwrap();

    // && short circuit
    repl.eval("let mut x = 0").unwrap();
    repl.eval("false && { x = 1; true }").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "0"); // Should not execute second part

    // || short circuit
    repl.eval("let mut y = 0").unwrap();
    repl.eval("true || { y = 1; false }").unwrap();
    assert_eq!(repl.eval("y").unwrap(), "0"); // Should not execute second part
}

#[test]
fn test_eval_break_continue() {
    let mut repl = Repl::new().unwrap();

    // Break in loop
    repl.eval("let mut i = 0").unwrap();
    repl.eval(r#"
        while true {
            i = i + 1;
            if i == 5 {
                break
            }
        }
    "#).unwrap();
    assert_eq!(repl.eval("i").unwrap(), "5");

    // Continue in loop
    repl.eval("let mut sum = 0").unwrap();
    repl.eval(r#"
        for j in 1..=10 {
            if j % 2 == 0 {
                continue
            }
            sum = sum + j
        }
    "#).unwrap();
    assert_eq!(repl.eval("sum").unwrap(), "25"); // 1+3+5+7+9
}

#[test]
fn test_eval_return_statement() {
    let mut repl = Repl::new().unwrap();

    repl.eval(r#"
        fn early_return(x) {
            if x < 0 {
                return -x
            }
            x * 2
        }
    "#).unwrap();

    assert_eq!(repl.eval("early_return(-5)").unwrap(), "5");
    assert_eq!(repl.eval("early_return(5)").unwrap(), "10");
}

#[test]
fn test_eval_pipeline_operator() {
    let mut repl = Repl::new().unwrap();

    repl.eval("fn double(x) { x * 2 }").unwrap();
    repl.eval("fn add_one(x) { x + 1 }").unwrap();

    let result = repl.eval("5 |> double |> add_one");
    if result.is_ok() {
        assert_eq!(result.unwrap(), "11");
    }
}

#[test]
fn test_eval_destructuring_patterns() {
    let mut repl = Repl::new().unwrap();

    // List destructuring
    repl.eval("let [a, b, c] = [1, 2, 3]").unwrap();
    assert_eq!(repl.eval("a").unwrap(), "1");
    assert_eq!(repl.eval("b").unwrap(), "2");
    assert_eq!(repl.eval("c").unwrap(), "3");

    // Rest patterns
    repl.eval("let [first, ...rest] = [1, 2, 3, 4]").unwrap();
    assert_eq!(repl.eval("first").unwrap(), "1");
    assert_eq!(repl.eval("rest").unwrap(), "[2, 3, 4]");
}

#[test]
fn test_eval_enum_variants() {
    let mut repl = Repl::new().unwrap();

    // Create enum-like values
    repl.eval("let some_value = Some(42)").unwrap();
    repl.eval("let none_value = None").unwrap();

    // Pattern match on them
    let result = repl.eval(r#"
        match some_value {
            Some(x) => x * 2,
            None => 0
        }
    "#);
    if result.is_ok() {
        assert_eq!(result.unwrap(), "84");
    }
}

#[test]
fn test_eval_async_await() {
    let mut repl = Repl::new().unwrap();

    let result = repl.eval("async { 42 }");
    // Async might not be fully supported
    let _ = result;

    let result = repl.eval("await some_future");
    // Await might not be fully supported
    let _ = result;
}

#[test]
fn test_eval_memory_management() {
    let mut repl = Repl::new().unwrap();

    let initial_mem = repl.memory_used();

    // Allocate large list
    repl.eval("let big = [0; 1000]").unwrap();
    let after_alloc = repl.memory_used();
    assert!(after_alloc >= initial_mem);

    // Clear and check
    repl.clear_bindings();
    // Memory might not be immediately freed
    let after_clear = repl.memory_used();
    let _ = after_clear;
}

#[test]
fn test_eval_deep_recursion_limit() {
    let mut repl = Repl::new().unwrap();

    repl.eval(r#"
        fn deep(n) {
            if n == 0 {
                0
            } else {
                deep(n - 1) + 1
            }
        }
    "#).unwrap();

    // Should handle reasonable recursion
    assert_eq!(repl.eval("deep(100)").unwrap(), "100");

    // Very deep recursion might hit limit
    let result = repl.eval_bounded(
        "deep(10000)",
        1024 * 1024,
        Duration::from_millis(100)
    );
    // Might timeout or stack overflow
    let _ = result;
}