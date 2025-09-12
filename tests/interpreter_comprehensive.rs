//! Comprehensive test suite for the Ruchy interpreter
//! 
//! This test suite aims to improve code coverage for the interpreter module,
//! testing all major execution paths and edge cases.

use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[test]
fn test_interpreter_basic_arithmetic() {
    let mut interpreter = Interpreter::new();
    
    // Test integer arithmetic
    let result = interpreter.eval_string("2 + 3").unwrap();
    assert_eq!(result, Value::Integer(5));
    
    let result = interpreter.eval_string("10 - 4").unwrap();
    assert_eq!(result, Value::Integer(6));
    
    let result = interpreter.eval_string("3 * 7").unwrap();
    assert_eq!(result, Value::Integer(21));
    
    let result = interpreter.eval_string("15 / 3").unwrap();
    assert_eq!(result, Value::Integer(5));
    
    let result = interpreter.eval_string("17 % 5").unwrap();
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_interpreter_float_arithmetic() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("2.5 + 1.5").unwrap();
    assert_eq!(result, Value::Float(4.0));
    
    let result = interpreter.eval_string("10.0 - 2.5").unwrap();
    assert_eq!(result, Value::Float(7.5));
    
    let result = interpreter.eval_string("3.0 * 2.5").unwrap();
    assert_eq!(result, Value::Float(7.5));
    
    let result = interpreter.eval_string("10.0 / 4.0").unwrap();
    assert_eq!(result, Value::Float(2.5));
}

#[test]
fn test_interpreter_boolean_operations() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("true && true").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("true && false").unwrap();
    assert_eq!(result, Value::Bool(false));
    
    let result = interpreter.eval_string("true || false").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("!true").unwrap();
    assert_eq!(result, Value::Bool(false));
    
    let result = interpreter.eval_string("!false").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interpreter_comparison_operations() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("5 > 3").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("3 < 5").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("5 >= 5").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("3 <= 5").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("5 == 5").unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string("5 != 3").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interpreter_string_operations() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string(r#""hello" + " world""#).unwrap();
    assert_eq!(result, Value::String(Rc::new("hello world".to_string())));
    
    let result = interpreter.eval_string(r#""test".length()"#).unwrap();
    assert_eq!(result, Value::Integer(4));
    
    let result = interpreter.eval_string(r#""HELLO".lower()"#).unwrap();
    assert_eq!(result, Value::String(Rc::new("hello".to_string())));
    
    let result = interpreter.eval_string(r#""hello".upper()"#).unwrap();
    assert_eq!(result, Value::String(Rc::new("HELLO".to_string())));
}

#[test]
fn test_interpreter_variable_assignment() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("let x = 10").unwrap();
    let result = interpreter.eval_string("x").unwrap();
    assert_eq!(result, Value::Integer(10));
    
    interpreter.eval_string("let y = x * 2").unwrap();
    let result = interpreter.eval_string("y").unwrap();
    assert_eq!(result, Value::Integer(20));
    
    interpreter.eval_string("let name = \"Ruchy\"").unwrap();
    let result = interpreter.eval_string("name").unwrap();
    assert_eq!(result, Value::String(Rc::new("Ruchy".to_string())));
}

#[test]
fn test_interpreter_if_else() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("if true { 1 } else { 2 }").unwrap();
    assert_eq!(result, Value::Integer(1));
    
    let result = interpreter.eval_string("if false { 1 } else { 2 }").unwrap();
    assert_eq!(result, Value::Integer(2));
    
    let result = interpreter.eval_string("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap();
    assert_eq!(result, Value::String(Rc::new("yes".to_string())));
}

#[test]
fn test_interpreter_function_definition_and_call() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("fun add(x, y) { x + y }").unwrap();
    let result = interpreter.eval_string("add(3, 4)").unwrap();
    assert_eq!(result, Value::Integer(7));
    
    interpreter.eval_string("fun factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }").unwrap();
    let result = interpreter.eval_string("factorial(5)").unwrap();
    assert_eq!(result, Value::Integer(120));
}

#[test]
fn test_interpreter_list_operations() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("[1, 2, 3]").unwrap();
    if let Value::Array(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Integer(1));
        assert_eq!(items[1], Value::Integer(2));
        assert_eq!(items[2], Value::Integer(3));
    } else {
        panic!("Expected list");
    }
    
    interpreter.eval_string("let nums = [1, 2, 3, 4, 5]").unwrap();
    let result = interpreter.eval_string("nums[2]").unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_interpreter_map_operations() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string(r#"let person = {"name": "Alice", "age": 30}"#).unwrap();
    let result = interpreter.eval_string(r#"person["name"]"#).unwrap();
    assert_eq!(result, Value::String(Rc::new("Alice".to_string())));
    
    let result = interpreter.eval_string(r#"person["age"]"#).unwrap();
    assert_eq!(result, Value::Integer(30));
}

#[test]
fn test_interpreter_for_loop() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("let sum = 0").unwrap();
    interpreter.eval_string("for i in [1, 2, 3, 4, 5] { sum = sum + i }").unwrap();
    let result = interpreter.eval_string("sum").unwrap();
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_interpreter_while_loop() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("let count = 0").unwrap();
    interpreter.eval_string("let sum = 0").unwrap();
    interpreter.eval_string("while count < 5 { sum = sum + count; count = count + 1 }").unwrap();
    let result = interpreter.eval_string("sum").unwrap();
    assert_eq!(result, Value::Integer(10)); // 0 + 1 + 2 + 3 + 4
}

#[test]
fn test_interpreter_match_expression() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string(r#"
        match 2 {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#).unwrap();
    assert_eq!(result, Value::String(Rc::new("two".to_string())));
}

#[test]
fn test_interpreter_closure() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("let make_adder = fun(x) { fun(y) { x + y } }").unwrap();
    interpreter.eval_string("let add5 = make_adder(5)").unwrap();
    let result = interpreter.eval_string("add5(3)").unwrap();
    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_interpreter_error_handling() {
    let mut interpreter = Interpreter::new();
    
    // Division by zero
    let result = interpreter.eval_string("10 / 0");
    assert!(result.is_err());
    
    // Undefined variable
    let result = interpreter.eval_string("undefined_var");
    assert!(result.is_err());
    
    // Type mismatch
    let result = interpreter.eval_string(r#"5 + "string""#);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_math_functions() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("abs(-5)").unwrap();
    assert_eq!(result, Value::Integer(5));
    
    let result = interpreter.eval_string("min(3, 7)").unwrap();
    assert_eq!(result, Value::Integer(3));
    
    let result = interpreter.eval_string("max(3, 7)").unwrap();
    assert_eq!(result, Value::Integer(7));
    
    let result = interpreter.eval_string("sqrt(16.0)").unwrap();
    assert_eq!(result, Value::Float(4.0));
    
    let result = interpreter.eval_string("pow(2.0, 3.0)").unwrap();
    assert_eq!(result, Value::Float(8.0));
}

#[test]
fn test_interpreter_type_conversion() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("int(3.7)").unwrap();
    assert_eq!(result, Value::Integer(3));
    
    let result = interpreter.eval_string("float(5)").unwrap();
    assert_eq!(result, Value::Float(5.0));
    
    let result = interpreter.eval_string(r#"str(42)"#).unwrap();
    assert_eq!(result, Value::String(Rc::new("42".to_string())));
    
    let result = interpreter.eval_string(r#"bool(1)"#).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let result = interpreter.eval_string(r#"bool(0)"#).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_interpreter_lambda_expressions() {
    let mut interpreter = Interpreter::new();
    
    interpreter.eval_string("let double = |x| x * 2").unwrap();
    let result = interpreter.eval_string("double(5)").unwrap();
    assert_eq!(result, Value::Integer(10));
    
    interpreter.eval_string("let nums = [1, 2, 3, 4, 5]").unwrap();
    interpreter.eval_string("let doubled = nums.map(|x| x * 2)").unwrap();
    let result = interpreter.eval_string("doubled").unwrap();
    if let Value::Array(items) = result {
        assert_eq!(items[0], Value::Integer(2));
        assert_eq!(items[4], Value::Integer(10));
    }
}

#[test]
fn test_interpreter_pipeline_operator() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.eval_string("5 |> (|x| x * 2) |> (|x| x + 1)").unwrap();
    assert_eq!(result, Value::Integer(11));
}

#[test]
fn test_interpreter_async_operations() {
    let mut interpreter = Interpreter::new();
    
    // Basic async function
    interpreter.eval_string("async fun fetch_data() { 42 }").unwrap();
    let result = interpreter.eval_string("await fetch_data()").unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_interpreter_complex_expression() {
    let mut interpreter = Interpreter::new();
    
    // Complex nested expression
    let result = interpreter.eval_string("(10 + 5) * 2 - (8 / 4) + 3").unwrap();
    assert_eq!(result, Value::Integer(31)); // (15 * 2) - 2 + 3 = 30 - 2 + 3 = 31
    
    // With variables
    interpreter.eval_string("let a = 10").unwrap();
    interpreter.eval_string("let b = 20").unwrap();
    interpreter.eval_string("let c = 30").unwrap();
    let result = interpreter.eval_string("(a + b) * 2 + c").unwrap();
    assert_eq!(result, Value::Integer(90)); // (10 + 20) * 2 + 30 = 60 + 30 = 90
}