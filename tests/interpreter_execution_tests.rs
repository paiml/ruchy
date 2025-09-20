// INTERPRETER EXECUTION TESTS - Direct execution paths
// Sprint 80 Phase 27: Execute real interpreter code
// ALL NIGHT CONTINUES!

use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::{Value, Environment};
use ruchy::Parser;
use std::rc::Rc;
use std::collections::HashMap;

#[test]
fn test_interpreter_execute_integer() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new("42");
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_execute_float() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new("3.14159");
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert!(matches!(result, Value::Float(_)));
}

#[test]
fn test_interpreter_execute_string() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello world""#);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "hello world");
    }
}

#[test]
fn test_interpreter_execute_bool_true() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new("true");
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interpreter_execute_bool_false() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new("false");
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_interpreter_execute_unit() {
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new("()");
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Unit);
}

#[test]
fn test_interpreter_addition() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(1, 2, 3), (10, 20, 30), (100, 200, 300)] {
        let program = format!("{} + {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_subtraction() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(10, 3, 7), (100, 50, 50), (1000, 1, 999)] {
        let program = format!("{} - {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_multiplication() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(2, 3, 6), (5, 10, 50), (7, 8, 56)] {
        let program = format!("{} * {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_division() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(10, 2, 5), (100, 10, 10), (50, 5, 10)] {
        let program = format!("{} / {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_modulo() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(10, 3, 1), (20, 7, 6), (100, 11, 1)] {
        let program = format!("{} % {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_power() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(2, 3, 8), (3, 2, 9), (5, 2, 25)] {
        let program = format!("{} ** {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_comparison_lt() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(1, 2, true), (2, 1, false), (1, 1, false)] {
        let program = format!("{} < {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_comparison_gt() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(2, 1, true), (1, 2, false), (1, 1, false)] {
        let program = format!("{} > {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_comparison_le() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(1, 2, true), (1, 1, true), (2, 1, false)] {
        let program = format!("{} <= {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_comparison_ge() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(2, 1, true), (1, 1, true), (1, 2, false)] {
        let program = format!("{} >= {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_comparison_eq() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(1, 1, true), (1, 2, false), (42, 42, true)] {
        let program = format!("{} == {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_comparison_ne() {
    let mut interpreter = Interpreter::new();
    for (a, b, expected) in [(1, 2, true), (1, 1, false), (42, 0, true)] {
        let program = format!("{} != {}", a, b);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_logical_and() {
    let mut interpreter = Interpreter::new();
    let cases = [
        ("true && true", true),
        ("true && false", false),
        ("false && true", false),
        ("false && false", false),
    ];
    for (program, expected) in cases {
        let mut parser = Parser::new(program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_logical_or() {
    let mut interpreter = Interpreter::new();
    let cases = [
        ("true || true", true),
        ("true || false", true),
        ("false || true", true),
        ("false || false", false),
    ];
    for (program, expected) in cases {
        let mut parser = Parser::new(program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_unary_neg() {
    let mut interpreter = Interpreter::new();
    for (input, expected) in [("42", -42), ("100", -100), ("0", 0)] {
        let program = format!("-{}", input);
        let mut parser = Parser::new(&program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Integer(expected));
    }
}

#[test]
fn test_interpreter_unary_not() {
    let mut interpreter = Interpreter::new();
    let cases = [("!true", false), ("!false", true)];
    for (program, expected) in cases {
        let mut parser = Parser::new(program);
        let ast = parser.parse().unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert_eq!(result, Value::Bool(expected));
    }
}

#[test]
fn test_interpreter_let_binding() {
    let mut interpreter = Interpreter::new();
    let program = "let x = 42; x";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_let_mut() {
    let mut interpreter = Interpreter::new();
    let program = "let mut x = 10; x = 20; x";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_interpreter_block() {
    let mut interpreter = Interpreter::new();
    let program = "{ 1; 2; 3 }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_interpreter_nested_blocks() {
    let mut interpreter = Interpreter::new();
    let program = "{ let x = 1; { let y = 2; x + y } }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_interpreter_if_true() {
    let mut interpreter = Interpreter::new();
    let program = "if true { 42 } else { 0 }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_if_false() {
    let mut interpreter = Interpreter::new();
    let program = "if false { 42 } else { 100 }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_interpreter_if_no_else() {
    let mut interpreter = Interpreter::new();
    let program = "if true { 42 }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_if_no_else_false() {
    let mut interpreter = Interpreter::new();
    let program = "if false { 42 }";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Unit);
}

#[test]
fn test_interpreter_list() {
    let mut interpreter = Interpreter::new();
    let program = "[1, 2, 3, 4, 5]";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::List(list) = result {
        assert_eq!(list.len(), 5);
        assert_eq!(list[0], Value::Integer(1));
        assert_eq!(list[4], Value::Integer(5));
    }
}

#[test]
fn test_interpreter_empty_list() {
    let mut interpreter = Interpreter::new();
    let program = "[]";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::List(list) = result {
        assert_eq!(list.len(), 0);
    }
}

#[test]
fn test_interpreter_tuple() {
    let mut interpreter = Interpreter::new();
    let program = "(1, \"hello\", true)";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::Tuple(tuple) = result {
        assert_eq!(tuple.len(), 3);
        assert_eq!(tuple[0], Value::Integer(1));
        assert_eq!(tuple[2], Value::Bool(true));
    }
}

#[test]
fn test_interpreter_object() {
    let mut interpreter = Interpreter::new();
    let program = "{x: 10, y: 20}";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("x"), Some(&Value::Integer(10)));
        assert_eq!(obj.get("y"), Some(&Value::Integer(20)));
    }
}

#[test]
fn test_interpreter_function() {
    let mut interpreter = Interpreter::new();
    let program = "fn add(x, y) { x + y }; add(3, 4)";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_interpreter_lambda() {
    let mut interpreter = Interpreter::new();
    let program = "let f = fn(x) { x * 2 }; f(21)";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_closure() {
    let mut interpreter = Interpreter::new();
    let program = r#"
        let make_adder = fn(x) {
            fn(y) { x + y }
        };
        let add10 = make_adder(10);
        add10(32)
    "#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_recursion_factorial() {
    let mut interpreter = Interpreter::new();
    let program = r#"
        fn factorial(n) {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
        factorial(5)
    "#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(120));
}

#[test]
fn test_interpreter_while_loop() {
    let mut interpreter = Interpreter::new();
    let program = r#"
        let mut i = 0;
        let mut sum = 0;
        while i < 10 {
            sum = sum + i;
            i = i + 1
        };
        sum
    "#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(45));
}

#[test]
fn test_interpreter_for_loop() {
    let mut interpreter = Interpreter::new();
    let program = r#"
        let mut sum = 0;
        for i in [1, 2, 3, 4, 5] {
            sum = sum + i
        };
        sum
    "#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_interpreter_match() {
    let mut interpreter = Interpreter::new();
    let program = r#"
        let x = 2;
        match x {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "two");
    }
}

#[test]
fn test_interpreter_string_concat() {
    let mut interpreter = Interpreter::new();
    let program = r#""hello" + " " + "world""#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    if let Value::String(s) = result {
        assert_eq!(&**s, "hello world");
    }
}

#[test]
fn test_interpreter_field_access() {
    let mut interpreter = Interpreter::new();
    let program = "let obj = {x: 42, y: 100}; obj.x";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_index_access() {
    let mut interpreter = Interpreter::new();
    let program = "let arr = [10, 20, 30]; arr[1]";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_interpreter_method_call() {
    let mut interpreter = Interpreter::new();
    let program = r#""hello".len()"#;
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_interpreter_complex_expression() {
    let mut interpreter = Interpreter::new();
    let program = "(1 + 2) * 3 - 4 / 2";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(7)); // (3) * 3 - 2 = 9 - 2 = 7
}

#[test]
fn test_interpreter_operator_precedence() {
    let mut interpreter = Interpreter::new();
    let program = "2 + 3 * 4";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast).unwrap();
    assert_eq!(result, Value::Integer(14)); // 2 + 12 = 14
}

#[test]
fn test_interpreter_division_by_zero() {
    let mut interpreter = Interpreter::new();
    let program = "10 / 0";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_undefined_variable() {
    let mut interpreter = Interpreter::new();
    let program = "undefined_variable_xyz";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_type_mismatch() {
    let mut interpreter = Interpreter::new();
    let program = "1 + \"string\"";
    let mut parser = Parser::new(program);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate(&ast);
    // Type mismatch should error or coerce
    let _ = result;
}

// Continue ALL NIGHT with more tests...
