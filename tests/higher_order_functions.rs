//! Regression tests for higher-order functions (BUG-002)
//!
//! These tests ensure that functions can be passed as parameters and called correctly.
//! This was broken in v1.17.0 where function parameters were incorrectly typed as String.

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_higher_order_function() {
    let code = r"
fun apply(f, x) {
    f(x)
}

fun double(n) {
    n * 2
}

apply(double, 5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    // Ensure function parameter is typed correctly
    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("impl Fn"),
        "Function parameter should use impl Fn trait"
    );
    assert!(
        !rust_str.contains("f : String"),
        "Function parameter should not be typed as String"
    );
}

#[test]
fn test_higher_order_with_multiple_params() {
    let code = r"
fun compose(f, g, x) {
    f(g(x))
}

fun add_one(n) {
    n + 1
}

fun double(n) {
    n * 2
}

compose(double, add_one, 5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    // Both f and g should be function types
    assert!(
        rust_str.contains("f : impl Fn"),
        "First function parameter should use impl Fn"
    );
    assert!(
        rust_str.contains("g : impl Fn"),
        "Second function parameter should use impl Fn"
    );
}

#[test]
fn test_lambda_as_argument() {
    let code = r"
fun apply(f, x) {
    f(x)
}

apply(|n| { n * 3 }, 7)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("impl Fn"),
        "Function parameter should accept lambdas"
    );
}

#[test]
fn test_function_returning_function() {
    let code = r"
fun make_adder(n) {
    |x| { x + n }
}

let add_five = make_adder(5)
add_five(10)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    // Should transpile without errors
    assert!(!rust_code.to_string().is_empty());
}

#[test]
fn test_map_with_function() {
    let code = r"
fun map(f, list) {
    f(list)
}

fun square(n) {
    n * n
}

map(square, 4)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("impl Fn"),
        "Map should accept function parameter"
    );
    assert!(
        rust_str.contains("-> i32"),
        "Functions should have proper return types"
    );
}

#[test]
fn test_filter_with_predicate() {
    let code = r"
fun filter(pred, value) {
    if pred(value) {
        value
    } else {
        0
    }
}

fun is_even(n) {
    n % 2 == 0
}

filter(is_even, 4)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("pred : impl Fn"),
        "Predicate should be a function type"
    );
}

#[test]
fn test_reduce_with_function() {
    let code = r"
fun reduce(f, initial, value) {
    f(initial, value)
}

fun add(a, b) {
    a + b
}

reduce(add, 0, 10)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("f : impl Fn"),
        "Reducer function should have proper type"
    );
}

#[test]
fn test_curry_function() {
    let code = r"
fun curry(f) {
    |x| { |y| { f(x, y) } }
}

fun multiply(x, y) {
    x * y
}

let curried = curry(multiply)
let times_two = curried(2)
times_two(5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    // Should transpile without panic
    let _ = transpiler.transpile(&ast);
}

#[test]
fn test_function_composition() {
    let code = r"
fun pipe(f, g) {
    |x| { g(f(x)) }
}

fun inc(n) {
    n + 1
}

fun double(n) {
    n * 2
}

let inc_then_double = pipe(inc, double)
inc_then_double(5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("f : impl Fn"),
        "First function in pipe should have proper type"
    );
    assert!(
        rust_str.contains("g : impl Fn"),
        "Second function in pipe should have proper type"
    );
}

#[test]
fn test_recursive_higher_order() {
    let code = r"
fun until(pred, f, x) {
    if pred(x) {
        x
    } else {
        until(pred, f, f(x))
    }
}

fun is_ten(n) {
    n == 10
}

fun inc(n) {
    n + 1
}

until(is_ten, inc, 0)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");

    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("pred : impl Fn"),
        "Predicate in until should be function type"
    );
    assert!(
        rust_str.contains("f : impl Fn"),
        "Transform function in until should be function type"
    );
}

/// Property test: Any function used as a parameter should be typed as impl Fn, not String
#[test]
fn property_test_no_string_function_params() {
    let test_cases = vec![
        "fun f(g, x) { g(x) }",
        "fun map(mapper, val) { mapper(val) }",
        "fun apply(func, arg) { func(arg) }",
        "fun twice(f, x) { f(f(x)) }",
        "fun conditional(pred, val) { if pred(val) { 1 } else { 0 } }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser
            .parse()
            .unwrap_or_else(|_| panic!("Failed to parse: {code}"));
        let mut transpiler = Transpiler::new();
        let rust_code = transpiler
            .transpile(&ast)
            .unwrap_or_else(|_| panic!("Failed to transpile: {code}"));

        let rust_str = rust_code.to_string();
        // Check that function parameters are not typed as String
        assert!(
            !rust_str.contains("g : String"),
            "Function parameter 'g' should not be String in: {code}"
        );
        assert!(
            !rust_str.contains("mapper : String"),
            "Function parameter 'mapper' should not be String in: {code}"
        );
        assert!(
            !rust_str.contains("func : String"),
            "Function parameter 'func' should not be String in: {code}"
        );
        assert!(
            !rust_str.contains("f : String"),
            "Function parameter 'f' should not be String in: {code}"
        );
        assert!(
            !rust_str.contains("pred : String"),
            "Function parameter 'pred' should not be String in: {code}"
        );
    }
}
