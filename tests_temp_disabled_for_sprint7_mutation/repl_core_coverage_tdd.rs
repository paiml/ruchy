//! TDD tests for REPL core functionality
//! Target: Improve coverage from 10.9% to 60%+

// ReplSession no longer exists, using Repl directly
use ruchy::runtime::repl::Repl;
use ruchy::runtime::Value;

#[test]
fn test_repl_evaluate_arithmetic() {
    let mut repl = ReplSession::new();

    // Test basic arithmetic
    let result = repl.eval_line("2 + 3").unwrap();
    assert_eq!(result, Value::Int(5));

    let result = repl.eval_line("10 - 4").unwrap();
    assert_eq!(result, Value::Int(6));

    let result = repl.eval_line("3 * 7").unwrap();
    assert_eq!(result, Value::Int(21));

    let result = repl.eval_line("15 / 3").unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_repl_evaluate_variables() {
    let mut repl = ReplSession::new();

    // Define and use variables
    repl.eval_line("let x = 10").unwrap();
    let result = repl.eval_line("x + 5").unwrap();
    assert_eq!(result, Value::Int(15));

    repl.eval_line("let y = 20").unwrap();
    let result = repl.eval_line("x + y").unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_repl_evaluate_strings() {
    let mut repl = ReplSession::new();

    let result = repl.eval_line("\"hello\"").unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    let result = repl.eval_line("\"hello\" + \" world\"").unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn test_repl_evaluate_booleans() {
    let mut repl = ReplSession::new();

    let result = repl.eval_line("true").unwrap();
    assert_eq!(result, Value::Bool(true));

    let result = repl.eval_line("false").unwrap();
    assert_eq!(result, Value::Bool(false));

    let result = repl.eval_line("5 > 3").unwrap();
    assert_eq!(result, Value::Bool(true));

    let result = repl.eval_line("2 == 2").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_repl_evaluate_lists() {
    let mut repl = ReplSession::new();

    let result = repl.eval_line("[1, 2, 3]").unwrap();
    assert!(matches!(result, Value::List(_)));

    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Int(1));
        assert_eq!(items[1], Value::Int(2));
        assert_eq!(items[2], Value::Int(3));
    }
}

#[test]
fn test_repl_evaluate_functions() {
    let mut repl = ReplSession::new();

    // Define a function
    repl.eval_line("fun add(x, y) { x + y }").unwrap();

    // Call the function
    let result = repl.eval_line("add(3, 4)").unwrap();
    assert_eq!(result, Value::Int(7));

    // Define and call a recursive function
    repl.eval_line("fun factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }")
        .unwrap();
    let result = repl.eval_line("factorial(5)").unwrap();
    assert_eq!(result, Value::Int(120));
}

#[test]
fn test_repl_evaluate_if_else() {
    let mut repl = ReplSession::new();

    let result = repl.eval_line("if true { 1 } else { 2 }").unwrap();
    assert_eq!(result, Value::Int(1));

    let result = repl.eval_line("if false { 1 } else { 2 }").unwrap();
    assert_eq!(result, Value::Int(2));

    let result = repl
        .eval_line("if 5 > 3 { \"yes\" } else { \"no\" }")
        .unwrap();
    assert_eq!(result, Value::String("yes".to_string()));
}

#[test]
fn test_repl_evaluate_loops() {
    let mut repl = ReplSession::new();

    // For loop with range
    repl.eval_line("let mut sum = 0").unwrap();
    repl.eval_line("for i in 1..4 { sum = sum + i }").unwrap();
    let result = repl.eval_line("sum").unwrap();
    assert_eq!(result, Value::Int(6)); // 1 + 2 + 3

    // While loop
    repl.eval_line("let mut count = 0").unwrap();
    repl.eval_line("while count < 3 { count = count + 1 }")
        .unwrap();
    let result = repl.eval_line("count").unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_repl_evaluate_match() {
    let mut repl = ReplSession::new();

    let result = repl
        .eval_line("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }")
        .unwrap();
    assert_eq!(result, Value::String("two".to_string()));

    repl.eval_line("let x = Some(5)").unwrap();
    let result = repl
        .eval_line("match x { Some(n) => n * 2, None => 0 }")
        .unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_repl_evaluate_objects() {
    let mut repl = ReplSession::new();

    repl.eval_line("let obj = {name: \"Alice\", age: 30}")
        .unwrap();
    let result = repl.eval_line("obj.name").unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));

    let result = repl.eval_line("obj.age").unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_repl_error_handling() {
    let mut repl = ReplSession::new();

    // Undefined variable
    let result = repl.eval_line("undefined_var");
    assert!(result.is_err());

    // Division by zero
    let result = repl.eval_line("10 / 0");
    assert!(result.is_err());

    // Type mismatch
    let result = repl.eval_line("\"hello\" + 5");
    assert!(result.is_err());
}

#[test]
fn test_repl_multiline_input() {
    let mut repl = ReplSession::new();

    // Test multiline function definition
    let input = "fun fibonacci(n) {\n  if n <= 1 {\n    n\n  } else {\n    fibonacci(n-1) + fibonacci(n-2)\n  }\n}";
    repl.eval_line(input).unwrap();

    let result = repl.eval_line("fibonacci(6)").unwrap();
    assert_eq!(result, Value::Int(8)); // 6th fibonacci number
}
