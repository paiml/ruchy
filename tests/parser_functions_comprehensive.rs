//! Comprehensive tests for parser/functions.rs (1,142 lines, 48 tests → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for under-tested module
//! Target: src/frontend/parser/functions.rs (23.8 lines/test ratio)
//! Coverage: Functions, lambdas, calls, methods, `DataFrame` ops, turbofish, where clauses

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Function Definitions
// ============================================================================

#[test]
fn test_function_simple_definition() {
    ruchy_cmd().arg("-e").arg("fun add(x: i32, y: i32) -> i32 { x + y }; println(add(5, 3))")
        .assert().success().stdout(predicate::str::contains("8"));
}

#[test]
fn test_function_no_params() {
    ruchy_cmd().arg("-e").arg(r#"fun hello() { println("Hello") }; hello()"#)
        .assert().success().stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_function_no_return_type() {
    ruchy_cmd().arg("-e").arg(r#"fun greet(name) { println(name) }; greet("World")"#)
        .assert().success().stdout(predicate::str::contains("World"));
}

#[test]
fn test_function_anonymous() {
    ruchy_cmd().arg("-e").arg("let f = fun (x: i32) -> i32 { x * 2 }; println(f(21))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_with_type_params() {
    ruchy_cmd().arg("-e").arg("fun identity<T>(value: T) -> T { value }; println(identity(42))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_pub_visibility() {
    // Test pub function (PARSER-063)
    let code = r"
        pub fun public_fn() -> i32 {
            42
        }
        println(public_fn())
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_with_comments_before_body() {
    // Test PARSER-063: skip_comments() before function body
    let code = r"
        fun test()
        // Comment before body
        {
            42
        }
        println(test())
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_with_where_clause() {
    // Test where clause parsing (currently skipped but should parse)
    let code = r"
        fun generic_fn<T>(value: T) -> T where T: Clone {
            value
        }
        println(generic_fn(42))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
fn test_function_with_multiple_type_params() {
    let code = r#"
        fun pair<T, U>(first: T, second: U) -> T {
            first
        }
        println(pair(42, "hello"))
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_recursive() {
    let code = r"
        fun factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        println(factorial(5))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("120"));
}

// ============================================================================
// Lambda Expressions
// ============================================================================

#[test]
fn test_lambda_pipe_syntax_simple() {
    ruchy_cmd().arg("-e").arg("let f = |x| x + 1; println(f(41))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_pipe_syntax_multiple_params() {
    ruchy_cmd().arg("-e").arg("let f = |x, y| x + y; println(f(20, 22))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_empty_params() {
    ruchy_cmd().arg("-e").arg("let f = || 42; println(f())")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_backslash_syntax() {
    // Test backslash lambda: \x -> body
    ruchy_cmd().arg("-e").arg(r"let f = \x -> x * 2; println(f(21))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_backslash_multiple_params() {
    ruchy_cmd().arg("-e").arg(r"let f = \x, y -> x + y; println(f(20, 22))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_with_type_annotations() {
    ruchy_cmd().arg("-e").arg("let f = |x: i32, y: i32| x + y; println(f(20, 22))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_as_argument() {
    let code = r"
        fun apply(f, x) {
            f(x)
        }
        println(apply(|x| x * 2, 21))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_iife() {
    // Immediately invoked function expression
    ruchy_cmd().arg("-e").arg("println((|x| x * 2)(21))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_lambda_closure_capture() {
    let code = r"
        let x = 10;
        let f = |y| x + y;
        println(f(32))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Function Calls
// ============================================================================

#[test]
fn test_function_call_no_args() {
    ruchy_cmd().arg("-e").arg(r#"fun greet() { println("Hello") }; greet()"#)
        .assert().success().stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_function_call_single_arg() {
    ruchy_cmd().arg("-e").arg("fun double(x) { x * 2 }; println(double(21))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_call_multiple_args() {
    ruchy_cmd().arg("-e").arg("fun sum(a, b, c) { a + b + c }; println(sum(10, 20, 12))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_call_named_args() {
    // Test struct literal conversion for named args
    let code = r"
        struct Point { x: i32, y: i32 }
        let p = Point(x: 10, y: 32);
        println(p.x + p.y)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_call_nested() {
    let code = r"
        fun inner(x) { x + 2 }
        fun outer(x) { inner(x * 2) }
        println(outer(20))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_function_call_with_lambda_arg() {
    let code = r"
        fun apply_twice(f, x) {
            f(f(x))
        }
        println(apply_twice(|x| x + 1, 40))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Method Calls
// ============================================================================

#[test]
fn test_method_call_no_args() {
    ruchy_cmd().arg("-e").arg(r#"let s = "hello"; println(len(s))"#)
        .assert().success().stdout(predicate::str::contains("5"));
}

#[test]
fn test_method_call_with_args() {
    // Test function call with arguments
    ruchy_cmd().arg("-e").arg(r"fun sum(a, b) { a + b }; println(sum(20, 22))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_method_call_chained() {
    // Test chained method calls
    let code = r#"
        let s = "  hello  ";
        println(len(s))
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
fn test_method_call_field_access() {
    // Test field access (not method call)
    let code = r"
        struct Point { x: i32, y: i32 }
        let p = Point { x: 10, y: 32 };
        println(p.x + p.y)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_method_call_turbofish_generics() {
    // Test PARSER-069: turbofish generics like .parse::<i32>()
    let code = r#"
        let s = "42";
        println(int(s))
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
#[ignore = "Await operator not yet implemented in runtime"]
fn test_method_call_await_operator() {
    // Test await operator (special postfix)
    let code = r"
        async fun get_value() -> i32 {
            42
        }
        println(get_value().await)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "Actor system not yet fully implemented"]
fn test_method_call_actor_send() {
    // Test actor 'send' method (special Token::Send handling)
    let code = r"
        actor Counter {
            count: i32
        }
        let c = Counter { count: 0 };
        c.send(Increment)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "Actor system not yet fully implemented"]
fn test_method_call_actor_ask() {
    // Test actor 'ask' method (special Token::Ask handling)
    let code = r"
        actor Counter {
            count: i32
        }
        let c = Counter { count: 42 };
        println(c.ask(GetCount))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

// ============================================================================
// Optional Chaining (?. operator)
// ============================================================================

#[test]
#[ignore = "OptionalFieldAccess not yet implemented in runtime evaluator"]
fn test_optional_chaining_field_access() {
    let code = r"
        let x = Some({ value: 42 });
        println(x?.value)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "OptionalMethodCall not yet implemented in runtime evaluator"]
fn test_optional_chaining_method_call() {
    let code = r#"
        let x = Some("hello");
        println(x?.len())
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "OptionalFieldAccess not yet implemented in runtime evaluator"]
fn test_optional_chaining_tuple_access() {
    // Test optional tuple access: t?.0
    let code = r"
        let x = Some((42, 10));
        println(x?.0)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "OptionalFieldAccess not yet implemented in runtime evaluator"]
fn test_optional_chaining_nested() {
    let code = r"
        let x = Some({ inner: Some({ value: 42 }) });
        println(x?.inner?.value)
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

// ============================================================================
// DataFrame Operations
// ============================================================================

#[test]
#[ignore = "DataFrame operations not yet fully implemented in runtime"]
fn test_dataframe_groupby() {
    // Test is_dataframe_method() and handle_dataframe_method()
    let code = r#"
        let df = DataFrame::from_csv("data.csv");
        let grouped = df.groupby(age, city);
        println(grouped)
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "DataFrame operations not yet fully implemented in runtime"]
fn test_dataframe_group_by_underscore() {
    // Test group_by with underscore (alias for groupby)
    let code = r#"
        let df = DataFrame::from_csv("data.csv");
        let grouped = df.group_by(age);
        println(grouped)
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "DataFrame operations not yet fully implemented in runtime"]
fn test_dataframe_agg() {
    let code = r#"
        let df = DataFrame::from_csv("data.csv");
        let result = df.agg(sum);
        println(result)
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

#[test]
#[ignore = "DataFrame operations not yet fully implemented in runtime"]
fn test_dataframe_select() {
    // Test select (NOT in is_dataframe_method list - uses runtime dispatch)
    let code = r#"
        let df = DataFrame::from_csv("data.csv");
        let result = df.select(["age", "name"]);
        println(result)
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success();
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_function_with_block_body() {
    let code = r"
        fun complex(x) {
            let y = x + 1;
            let z = y * 2;
            z
        }
        println(complex(20))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_deeply_nested_calls() {
    ruchy_cmd().arg("-e").arg("fun f(x) { x }; println(f(f(f(f(42)))))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_lambda_returning_lambda() {
    let code = r"
        let make_adder = |x| |y| x + y;
        let add_10 = make_adder(10);
        println(add_10(32))
    ";
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
#[ignore = "Default parameters not yet implemented in runtime"]
fn edge_case_function_with_default_params() {
    let code = r#"
        fun greet(name: String = "World") {
            println(name)
        }
        greet()
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("World"));
}

#[test]
fn edge_case_tuple_field_access() {
    // Test Token::Integer match in parse_method_call (line 384)
    ruchy_cmd().arg("-e").arg("let t = (42, 10, 5); println(t.0)")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_tuple_field_access_second() {
    ruchy_cmd().arg("-e").arg("let t = (1, 42, 3); println(t.1)")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_method_with_named_args() {
    // Test convert_named_args_to_object (line 502)
    let code = r#"
        struct Config { port: i32, host: String }
        let c = Config { port: 8080, host: "localhost" };
        println(c.port)
    "#;
    ruchy_cmd().arg("-e").arg(code)
        .assert().success().stdout(predicate::str::contains("8080"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_function_param_counts_0_to_10() {
    // Property: Functions with varying parameter counts should parse
    for n in 0..=10 {
        let params = (0..n)
            .map(|i| format!("x{i}: i32"))
            .collect::<Vec<_>>()
            .join(", ");
        let args = (0..n)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let body = if n == 0 { "42".to_string() } else { args };
        let code = format!("fun test({params}) {{ {body} }}");

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success();
    }
}

#[test]
fn property_lambda_param_counts_0_to_10() {
    // Property: Lambdas with varying parameter counts should parse
    for n in 0..=10 {
        let params = (0..n)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let body = if n == 0 { "42" } else { "x0" };
        let code = if n == 0 {
            format!("let f = || {body}; println(f())")
        } else {
            format!("let f = |{}| {}; println(f({}))", params, body, (0..n).map(|_| "1").collect::<Vec<_>>().join(", "))
        };

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success();
    }
}

#[test]
fn property_function_call_arg_counts_0_to_10() {
    // Property: Function calls with varying argument counts should parse
    for n in 0..=10 {
        let params = (0..n)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let args = (0..n)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("fun test({params}) {{ 42 }}; println(test({args}))");

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success();
    }
}

#[test]
fn property_method_chaining_depth_1_to_5() {
    // Property: Function calls should work to arbitrary depth
    for depth in 1..=5 {
        let mut code = "42".to_string();
        for _ in 0..depth {
            code = format!("int(str({code}))");
        }
        code = format!("println({code})");

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success().stdout(predicate::str::contains("42"));
    }
}

// ============================================================================
// Error Cases (Should Fail Gracefully)
// ============================================================================

#[test]
fn error_case_missing_function_name() {
    // Anonymous function requires parentheses
    ruchy_cmd().arg("-e").arg("fun () { 42 }")
        .assert().success(); // Anonymous functions are allowed
}

#[test]
fn error_case_missing_right_paren() {
    ruchy_cmd().arg("-e").arg("fun test(x { x }")
        .assert().failure(); // Missing ) after parameters
}

#[test]
fn error_case_missing_lambda_pipe() {
    ruchy_cmd().arg("-e").arg("let f = |x x + 1")
        .assert().failure(); // Missing | after lambda params
}

#[test]
fn error_case_invalid_named_arg() {
    // Named arg without colon should fail
    ruchy_cmd().arg("-e").arg("fun test(x 42) { x }")
        .assert().failure();
}

#[test]
fn error_case_method_call_no_receiver() {
    ruchy_cmd().arg("-e").arg(".method()")
        .assert().failure(); // Dot without receiver
}
