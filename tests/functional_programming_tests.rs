//! Tests for functional programming features
#![allow(clippy::unwrap_used)]

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_list_sum() {
    let code = "[1, 2, 3, 4, 5].sum()";
    assert!(is_valid_syntax(code));
    // Evaluator would return 15
}

#[test]
fn test_list_reverse() {
    let code = "[1, 2, 3].reverse()";
    assert!(is_valid_syntax(code));
    // Evaluator would return [3, 2, 1]
}

#[test]
fn test_list_head() {
    let code = "[1, 2, 3].head()";
    assert!(is_valid_syntax(code));
    // Evaluator would return 1
}

#[test]
fn test_list_tail() {
    let code = "[1, 2, 3].tail()";
    assert!(is_valid_syntax(code));
    // Evaluator would return [2, 3]
}

#[test]
fn test_list_last() {
    let code = "[1, 2, 3].last()";
    assert!(is_valid_syntax(code));
    // Evaluator would return 3
}

#[test]
fn test_list_length() {
    let code = "[1, 2, 3, 4, 5].len()";
    assert!(is_valid_syntax(code));
    // Evaluator would return 5
}

#[test]
fn test_string_upper() {
    let code = r#""hello".upper()"#;
    assert!(is_valid_syntax(code));
    // Evaluator would return "HELLO"
}

#[test]
fn test_string_lower() {
    let code = r#""HELLO".lower()"#;
    assert!(is_valid_syntax(code));
    // Evaluator would return "hello"
}

#[test]
fn test_string_trim() {
    let code = r#""  hello  ".trim()"#;
    assert!(is_valid_syntax(code));
    // Evaluator would return "hello"
}

#[test]
fn test_string_length() {
    let code = r#""hello".len()"#;
    assert!(is_valid_syntax(code));
    // Evaluator would return 5
}

#[test]
fn test_lambda_syntax() {
    // Backslash syntax
    assert!(is_valid_syntax(r"\x -> x + 1"));
    assert!(is_valid_syntax(r"\x, y -> x + y"));
    
    // Pipe syntax
    assert!(is_valid_syntax("|x| x + 1"));
    assert!(is_valid_syntax("|x, y| x + y"));
    
    // Empty lambda
    assert!(is_valid_syntax("|| 42"));
}

#[test]
fn test_list_map() {
    let code = "[1, 2, 3].map(|x| x * 2)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("map"));
}

#[test]
fn test_list_filter() {
    let code = "[1, 2, 3, 4, 5].filter(|x| x > 2)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("filter"));
}

#[test]
fn test_chained_operations() {
    let code = "[1, 2, 3, 4, 5].filter(|x| x > 2).map(|x| x * 2)";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_function_composition() {
    // Pipeline operator
    let code = "5 |> double |> square";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_list_comprehension_map() {
    let code = "[x * 2 for x in [1, 2, 3]]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("map"));
    assert!(result.contains("collect"));
}

#[test]
fn test_list_comprehension_filter() {
    let code = "[x for x in [1, 2, 3, 4, 5] if x > 2]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("filter"));
    assert!(result.contains("collect"));
}

#[test]
fn test_lambda_in_variable() {
    let code = "let add = |x, y| x + y";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_higher_order_function() {
    let code = r"
        fn apply_twice(f, x) {
            f(f(x))
        }
    ";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_closure() {
    let code = r"
        fn make_adder(n) {
            |x| x + n
        }
    ";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_reduce_syntax() {
    // While reduce isn't implemented yet, the syntax should parse
    let code = "[1, 2, 3].reduce(|acc, x| acc + x, 0)";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_function_as_value() {
    let code = r"
        let f = square
        f(5)
    ";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_partial_application_syntax() {
    // Partial application via closures
    let code = "|x| add(5, x)";
    assert!(is_valid_syntax(code));
}