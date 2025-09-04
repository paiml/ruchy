//! Comprehensive TDD test suite for transpiler expressions
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every expression transpilation path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Parser, Transpiler};

// ==================== LITERAL EXPRESSION TESTS ====================

#[test]
fn test_transpile_integer_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("42"));
}

#[test]
fn test_transpile_float_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("3.14");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("3.14"));
}

#[test]
fn test_transpile_boolean_literals() {
    let transpiler = Transpiler::new();
    
    let mut parser_true = Parser::new("true");
    let ast_true = parser_true.parse().unwrap();
    assert!(transpiler.transpile(&ast_true).is_ok());
    
    let mut parser_false = Parser::new("false");
    let ast_false = parser_false.parse().unwrap();
    assert!(transpiler.transpile(&ast_false).is_ok());
}

#[test]
fn test_transpile_string_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#""hello world""#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("hello world"));
}

#[test]
fn test_transpile_char_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("'a'");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("'a'"));
}

#[test]
fn test_transpile_nil_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("nil");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("()"));
}

// ==================== BINARY EXPRESSION TESTS ====================

#[test]
fn test_transpile_arithmetic_operations() {
    let transpiler = Transpiler::new();
    let operations = vec![
        ("1 + 2", "+"),
        ("5 - 3", "-"),
        ("4 * 6", "*"),
        ("10 / 2", "/"),
        ("7 % 3", "%"),
    ];
    
    for (expr, op) in operations {
        let mut parser = Parser::new(expr);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap().to_string();
        assert!(result.contains(op));
    }
}

#[test]
fn test_transpile_comparison_operations() {
    let transpiler = Transpiler::new();
    let operations = vec![
        ("1 < 2", "<"),
        ("3 > 2", ">"),
        ("4 <= 4", "<="),
        ("5 >= 5", ">="),
        ("6 == 6", "=="),
        ("7 != 8", "!="),
    ];
    
    for (expr, op) in operations {
        let mut parser = Parser::new(expr);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap().to_string();
        assert!(result.contains(op));
    }
}

#[test]
fn test_transpile_logical_operations() {
    let transpiler = Transpiler::new();
    
    let mut parser_and = Parser::new("true && false");
    let ast_and = parser_and.parse().unwrap();
    assert!(transpiler.transpile(&ast_and).unwrap().to_string().contains("&&"));
    
    let mut parser_or = Parser::new("true || false");
    let ast_or = parser_or.parse().unwrap();
    assert!(transpiler.transpile(&ast_or).unwrap().to_string().contains("||"));
}

#[test]
fn test_transpile_bitwise_operations() {
    let transpiler = Transpiler::new();
    let operations = vec![
        ("5 & 3", "&"),
        ("5 | 3", "|"),
        ("5 ^ 3", "^"),
        ("5 << 2", "<<"),
        ("20 >> 2", ">>"),
    ];
    
    for (expr, op) in operations {
        let mut parser = Parser::new(expr);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
}

// ==================== UNARY EXPRESSION TESTS ====================

#[test]
fn test_transpile_unary_negation() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("-42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("-"));
}

#[test]
fn test_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("!true");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("!"));
}

// ==================== PARENTHESIZED EXPRESSION TESTS ====================

#[test]
fn test_transpile_parenthesized() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("(1 + 2) * 3");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let output = result.unwrap().to_string();
    assert!(output.contains("(") && output.contains(")"));
}

#[test]
fn test_transpile_nested_parentheses() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("((1 + 2) * (3 + 4))");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== ARRAY EXPRESSION TESTS ====================

#[test]
fn test_transpile_empty_array() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("[]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("vec!"));
}

#[test]
fn test_transpile_array_with_elements() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("[1, 2, 3, 4, 5]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let output = result.unwrap().to_string();
    assert!(output.contains("vec!") || output.contains("["));
}

#[test]
fn test_transpile_nested_arrays() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("[[1, 2], [3, 4], [5, 6]]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== TUPLE EXPRESSION TESTS ====================

#[test]
fn test_transpile_empty_tuple() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("()"));
}

#[test]
fn test_transpile_tuple_with_elements() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("(1, \"hello\", true)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let output = result.unwrap().to_string();
    assert!(output.contains("(") && output.contains(",") && output.contains(")"));
}

// ==================== FUNCTION CALL TESTS ====================

#[test]
fn test_transpile_function_call_no_args() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("print()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("print"));
}

#[test]
fn test_transpile_function_call_with_args() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("add(1, 2, 3)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let output = result.unwrap().to_string();
    assert!(output.contains("add") && output.contains("1") && output.contains("2"));
}

#[test]
fn test_transpile_nested_function_calls() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("max(min(1, 2), 3)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== METHOD CALL TESTS ====================

#[test]
fn test_transpile_method_call() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("\"hello\".len()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("len"));
}

#[test]
fn test_transpile_chained_method_calls() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("\"hello\".to_uppercase().trim()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== FIELD ACCESS TESTS ====================

#[test]
fn test_transpile_field_access() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("point.x");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("."));
}

#[test]
fn test_transpile_nested_field_access() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("user.address.city");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== INDEX ACCESS TESTS ====================

#[test]
fn test_transpile_array_index() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("arr[0]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("["));
}

#[test]
fn test_transpile_nested_index() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("matrix[i][j]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== LAMBDA EXPRESSION TESTS ====================

#[test]
fn test_transpile_simple_lambda() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("|x| x + 1");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("|"));
}

#[test]
fn test_transpile_multi_param_lambda() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("|x, y| x + y");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_lambda_with_types() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("|x: i32, y: i32| -> i32 { x + y }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== IF EXPRESSION TESTS ====================

#[test]
fn test_transpile_if_expression() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("if x > 0 { positive() } else { negative() }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let output = result.unwrap().to_string();
    assert!(output.contains("if") && output.contains("else"));
}

#[test]
fn test_transpile_if_without_else() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("if condition { do_something() }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_nested_if() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("if a { if b { c } else { d } } else { e }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== MATCH EXPRESSION TESTS ====================

#[test]
fn test_transpile_simple_match() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"
    match x {
        1 => "one",
        2 => "two",
        _ => "other"
    }
    "#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("match"));
}

#[test]
fn test_transpile_match_with_guards() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"
    match x {
        n if n > 0 => "positive",
        n if n < 0 => "negative",
        _ => "zero"
    }
    "#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== RANGE EXPRESSION TESTS ====================

#[test]
fn test_transpile_exclusive_range() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("0..10");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains(".."));
}

#[test]
fn test_transpile_inclusive_range() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("0..=10");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("..="));
}

// ==================== CAST EXPRESSION TESTS ====================

#[test]
fn test_transpile_type_cast() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("x as i32");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("as"));
}

// ==================== STRING INTERPOLATION TESTS ====================

#[test]
fn test_transpile_string_interpolation() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"f"Hello, {name}!""#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("format!"));
}

#[test]
fn test_transpile_complex_string_interpolation() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"f"Result: {x + y} = {calculate(x, y)}""#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== ASYNC/AWAIT TESTS ====================

#[test]
fn test_transpile_await_expression() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("await fetch_data()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("await"));
}

// ==================== TRY EXPRESSION TESTS ====================

#[test]
fn test_transpile_try_expression() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("try risky_operation()");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== QUESTION MARK OPERATOR TESTS ====================

#[test]
fn test_transpile_question_mark() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("result?");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    assert!(result.unwrap().to_string().contains("?"));
}

// ==================== COMPLEX EXPRESSION TESTS ====================

#[test]
fn test_transpile_complex_arithmetic() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("(a + b) * (c - d) / (e % f)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_deeply_nested() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("f(g(h(i(j(k(x))))))");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// Run all tests with: cargo test transpiler_expressions_tdd --test transpiler_expressions_tdd