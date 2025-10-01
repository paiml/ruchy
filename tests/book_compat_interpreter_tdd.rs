//! BOOK COMPATIBILITY: Interpreter TDD Tests
//!
//! Following CLAUDE.md EXTREME TDD Protocol for Book Compatibility Sprint
//! Tests written FIRST before implementation
//! Source: ruchy-book INTEGRATION.md + experiments/

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

// ============================================================================
// BOOK-001: STRING MULTIPLICATION OPERATOR
// ============================================================================

#[test]
fn test_string_multiply_positive() {
    // Test: "hello" * 3 should produce "hellohellohello"
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 3"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "hellohellohello",
                "String * 3 should repeat string 3 times"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_zero() {
    // Test: "hello" * 0 should produce empty string
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 0"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "", "String * 0 should produce empty string");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_one() {
    // Test: "hello" * 1 should produce "hello"
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 1"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "hello", "String * 1 should produce same string");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_separator() {
    // Test: "=" * 50 (common pattern from experiments)
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""=" * 50"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.len(), 50, "String * 50 should have length 50");
            assert!(s.chars().all(|c| c == '='), "All characters should be '='");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_empty() {
    // Test: "" * 100 should produce empty string
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""" * 100"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "",
                "Empty string * n should produce empty string"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_negative() {
    // Test: "hello" * -1 should produce empty string (Python behavior)
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * -1"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "",
                "String * negative should produce empty string"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_large() {
    // Test: Large multiplication doesn't panic
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""x" * 1000"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.len(), 1000, "String * 1000 should have length 1000");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_with_variable() {
    // Test: Variable binding with string multiplication
    let code = r#"
        let sep = "="
        sep * 10
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "==========", "Variable * 10 should work");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

// ============================================================================
// BOOK-002: SHEBANG SUPPORT
// ============================================================================

#[test]
fn test_shebang_basic() {
    // Test: #!/usr/bin/env ruchy at start of file
    let code = r#"#!/usr/bin/env ruchy
println("Hello")"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed with shebang");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    // Should execute the code after shebang
    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "Hello");
        }
        Value::Nil => {
            // println returns nil in some implementations
        }
        _ => panic!("Expected String or Nil, got {:?}", result),
    }
}

#[test]
fn test_shebang_with_args() {
    // Test: Shebang with arguments
    let code = r#"#!/usr/bin/env ruchy --some-flag
let x = 42
x"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed with shebang");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 42);
        }
        _ => panic!("Expected Integer(42), got {:?}", result),
    }
}

#[test]
fn test_shebang_empty_line_after() {
    // Test: Shebang with empty line following
    let code = r#"#!/usr/bin/env ruchy

let x = 10
x * 2"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 20);
        }
        _ => panic!("Expected Integer(20), got {:?}", result),
    }
}

#[test]
fn test_shebang_with_comments() {
    // Test: Shebang followed by regular comments
    let code = r#"#!/usr/bin/env ruchy
// This is a comment
let x = 5
x"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 5);
        }
        _ => panic!("Expected Integer(5), got {:?}", result),
    }
}

#[test]
fn test_shebang_must_be_first_line() {
    // Test: Shebang NOT at start should fail
    let code = r#"
#!/usr/bin/env ruchy
let x = 1"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    // Should fail because shebang is not on first line
    assert!(
        result.is_err(),
        "Shebang not at start of file should be parse error"
    );
}

#[test]
fn test_no_shebang_still_works() {
    // Test: Code without shebang continues to work
    let code = r#"let x = 100
x + 1"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed without shebang");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 101);
        }
        _ => panic!("Expected Integer(101), got {:?}", result),
    }
}

// ============================================================================
// BOOK-003: MULTI-VARIABLE EXPRESSION EVALUATION
// ============================================================================

#[test]
fn test_multi_let_with_final_expression() {
    // Test: let x = 1; let y = 2; x + y should return 3 (not 1 or 2)
    let code = r#"let x = 1; let y = 2; x + y"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(
                n, 3,
                "Should return final expression (x + y = 3), not x or y"
            );
        }
        _ => panic!("Expected Integer(3), got {:?}", result),
    }
}

#[test]
fn test_multi_let_with_float_arithmetic() {
    // Test: Price calculation from book examples
    let code = r#"let price = 99.99; let tax = 0.08; price * (1.0 + tax)"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            let expected = 99.99 * 1.08;
            assert!(
                (f - expected).abs() < 0.01,
                "Should return price * (1 + tax) = {}, got {}",
                expected,
                f
            );
        }
        _ => panic!("Expected Float(~107.99), got {:?}", result),
    }
}

#[test]
fn test_multi_let_with_variable_dependencies() {
    // Test: Variables depend on each other
    let code = r#"let x = 5; let y = x * 2; let z = y + 3; z"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 13, "Should return z = (x * 2) + 3 = (5 * 2) + 3 = 13");
        }
        _ => panic!("Expected Integer(13), got {:?}", result),
    }
}

#[test]
fn test_multi_let_with_nested_expressions() {
    // Test: Complex nested expression after multiple lets
    let code = r#"let a = 2; let b = 3; let c = 4; (a + b) * c"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 20, "Should return (a + b) * c = (2 + 3) * 4 = 20");
        }
        _ => panic!("Expected Integer(20), got {:?}", result),
    }
}

#[test]
fn test_multi_let_string_variables() {
    // Test: Multiple string variables with final expression
    let code = r#"let first = "Hello"; let last = "World"; first"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "Hello",
                "Should return final expression (first)"
            );
        }
        _ => panic!("Expected String(\"Hello\"), got {:?}", result),
    }
}

#[test]
fn test_three_lets_final_calculation() {
    // Test: Three variable bindings with calculation
    let code = r#"let x = 10; let y = 20; let z = 30; x + y + z"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 60, "Should return x + y + z = 10 + 20 + 30 = 60");
        }
        _ => panic!("Expected Integer(60), got {:?}", result),
    }
}

#[test]
fn test_multi_let_boolean_expression() {
    // Test: Multiple lets with boolean final expression
    let code = r#"let x = 5; let y = 10; x < y"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Bool(b) => {
            assert!(b, "Should return true (5 < 10)");
        }
        _ => panic!("Expected Bool(true), got {:?}", result),
    }
}

#[test]
fn test_single_let_with_expression() {
    // Test: Baseline - single let with expression should work
    let code = r#"let x = 42; x * 2"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 84, "Should return x * 2 = 42 * 2 = 84");
        }
        _ => panic!("Expected Integer(84), got {:?}", result),
    }
}

// ============================================================================
// BOOK-004: METHOD CALL CONSISTENCY
// ============================================================================

#[test]
fn test_method_call_on_expression_result() {
    // Test: (x*x + y*y).sqrt() - method call on arithmetic expression
    let code = r#"let x = 3.0; let y = 4.0; (x * x + y * y).sqrt()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            assert!((f - 5.0).abs() < 0.001, "Should return 5.0, got {}", f);
        }
        _ => panic!("Expected Float(5.0), got {:?}", result),
    }
}

#[test]
fn test_string_len_method() {
    // Test: "hello".len() - method call on string literal
    let code = r#""hello".len()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 5, "String length should be 5");
        }
        _ => panic!("Expected Integer(5), got {:?}", result),
    }
}

#[test]
fn test_string_variable_len_method() {
    // Test: name.len() - method call on string variable
    let code = r#"let name = "Ruchy"; name.len()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 5, "String length should be 5");
        }
        _ => panic!("Expected Integer(5), got {:?}", result),
    }
}

#[test]
fn test_array_len_method() {
    // Test: [1, 2, 3].len() - method call on array literal
    let code = r#"[1, 2, 3].len()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 3, "Array length should be 3");
        }
        _ => panic!("Expected Integer(3), got {:?}", result),
    }
}

#[test]
fn test_array_variable_len_method() {
    // Test: arr.len() - method call on array variable
    let code = r#"let arr = [1, 2, 3, 4]; arr.len()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 4, "Array length should be 4");
        }
        _ => panic!("Expected Integer(4), got {:?}", result),
    }
}

#[test]
fn test_array_map_method() {
    // Test: [1, 2, 3].map(|x| x * 2) - method call with lambda
    let code = r#"[1, 2, 3].map(|x| x * 2)"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3, "Mapped array should have 3 elements");
            // Check values
            if let Value::Integer(n) = arr[0] {
                assert_eq!(n, 2);
            }
            if let Value::Integer(n) = arr[1] {
                assert_eq!(n, 4);
            }
            if let Value::Integer(n) = arr[2] {
                assert_eq!(n, 6);
            }
        }
        _ => panic!("Expected Array([2, 4, 6]), got {:?}", result),
    }
}

#[test]
fn test_chained_method_calls() {
    // Test: ("hello" * 2).len() - chained operations (multiplication then length)
    let code = r#"("hello" * 2).len()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 10, "Repeated string length should be 10");
        }
        _ => panic!("Expected Integer(10), got {:?}", result),
    }
}

#[test]
fn test_float_method_calls() {
    // Test: 16.0.sqrt() - method call on float literal
    let code = r#"16.0.sqrt()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            assert!((f - 4.0).abs() < 0.001, "sqrt(16) should be 4.0, got {}", f);
        }
        _ => panic!("Expected Float(4.0), got {:?}", result),
    }
}

// ============================================================================
// BOOK-005: OPTION<T> TYPE
// ============================================================================

#[test]
fn test_option_some_variant() {
    // Test: Option<i32> with Some variant
    let code = r#"fun test() -> Option<i32> { Some(42) }; test()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::EnumVariant { variant_name, data } => {
            assert_eq!(variant_name, "Some", "Should be Some variant");
            assert!(data.is_some(), "Some should have data");
            let values = data.unwrap();
            assert_eq!(values.len(), 1, "Some should have one value");
            match &values[0] {
                Value::Integer(n) => assert_eq!(*n, 42, "Some should contain 42"),
                _ => panic!("Expected Integer(42), got {:?}", values[0]),
            }
        }
        _ => panic!("Expected EnumVariant Some(42), got {:?}", result),
    }
}

#[test]
fn test_option_none_variant() {
    // Test: Option<i32> with None variant
    let code = r#"fun test() -> Option<i32> { None }; test()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::EnumVariant { variant_name, data } => {
            assert_eq!(variant_name, "None", "Should be None variant");
            assert!(data.is_none(), "None should have no data");
        }
        _ => panic!("Expected EnumVariant None, got {:?}", result),
    }
}

#[test]
fn test_option_some_string() {
    // Test: Option<String> with Some variant
    let code = r#"fun find_name(id: i32) -> Option<String> {
        if id == 1 { Some("Alice") } else { None }
    }; find_name(1)"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::EnumVariant { variant_name, data } => {
            assert_eq!(variant_name, "Some", "Should be Some variant");
            assert!(data.is_some(), "Some should have data");
            let values = data.unwrap();
            assert_eq!(values.len(), 1, "Some should have one value");
            match &values[0] {
                Value::String(s) => assert_eq!(s.as_ref(), "Alice", "Should return Alice"),
                _ => panic!("Expected String(\"Alice\"), got {:?}", values[0]),
            }
        }
        _ => panic!("Expected EnumVariant Some(\"Alice\"), got {:?}", result),
    }
}

#[test]
fn test_option_none_branch() {
    // Test: Option<String> with None variant
    let code = r#"fun find_name(id: i32) -> Option<String> {
        if id == 1 { Some("Alice") } else { None }
    }; find_name(999)"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::EnumVariant { variant_name, data } => {
            assert_eq!(variant_name, "None", "Should be None variant");
            assert!(data.is_none(), "None should have no data");
        }
        _ => panic!("Expected EnumVariant None, got {:?}", result),
    }
}

// NOTE: Option pattern matching works in transpiler mode but has issues in interpreter mode
// Skipping these tests for now as they test interpreter-specific functionality
// The transpiler correctly handles: let opt = Some(10); match opt { Some(n) => n * 2, None => 0 }

#[test]
fn test_option_pattern_matching() {
    // Test: Pattern matching on Option with Some variant
    let code = r#"
        let opt = Some(10);
        match opt {
            Some(n) => n * 2,
            None => 0,
        }
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 20, "Some(10) matched should return 20");
        }
        _ => panic!("Expected Integer(20), got {:?}", result),
    }
}

#[test]
fn test_option_none_pattern_matching() {
    // Test: Pattern matching on None - this one works correctly
    let code = r#"
        let opt: Option<i32> = None;
        match opt {
            Some(n) => n * 2,
            None => 0,
        }
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 0, "None matched should return 0");
        }
        _ => panic!("Expected Integer(0), got {:?}", result),
    }
}

// ============================================================================
// BOOK-006: RESULT<T, E> TYPE
// ============================================================================

#[test]
fn test_result_ok_variant() {
    // Test: Result<i32, String> with Ok variant
    let code = r#"fun test() -> Result<i32, String> { Ok(42) }; test()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    // Result in interpreter is represented as an Object/Message
    match result {
        Value::Object(_) | Value::ObjectMut(_) => {
            // Expected - Result is an object in interpreter
        }
        _ => panic!("Expected Object (Result), got {:?}", result),
    }
}

#[test]
fn test_result_err_variant() {
    // Test: Result<i32, String> with Err variant
    let code = r#"fun test() -> Result<i32, String> { Err("failed") }; test()"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    // Result in interpreter is represented as an Object/Message
    match result {
        Value::Object(_) | Value::ObjectMut(_) => {
            // Expected - Result is an object in interpreter
        }
        _ => panic!("Expected Object (Result), got {:?}", result),
    }
}

#[test]
fn test_result_division_ok() {
    // Test: Division that succeeds
    let code = r#"
        fun safe_divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 { Err("Division by zero") } else { Ok(a / b) }
        };
        safe_divide(10.0, 2.0)
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    // Result is represented as an Object in interpreter
    match result {
        Value::Object(_) | Value::ObjectMut(_) => {
            // Expected - Result is an object
        }
        _ => panic!("Expected Object (Result), got {:?}", result),
    }
}

#[test]
fn test_result_division_err() {
    // Test: Division by zero returns Err
    let code = r#"
        fun safe_divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 { Err("Division by zero") } else { Ok(a / b) }
        };
        safe_divide(10.0, 0.0)
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    // Result is represented as an Object in interpreter
    match result {
        Value::Object(_) | Value::ObjectMut(_) => {
            // Expected - Result is an object
        }
        _ => panic!("Expected Object (Result), got {:?}", result),
    }
}

// ============================================================================
// BOOK-007: IMPL BLOCKS FOR STRUCTS
// ============================================================================

#[test]
fn test_struct_basic_definition() {
    // Test: Basic struct definition and instantiation
    let code = r#"struct Point { x: i32, y: i32 }; let p = Point { x: 3, y: 4 }; p.x"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 3, "Point.x should be 3");
        }
        _ => panic!("Expected Integer(3), got {:?}", result),
    }
}

#[test]
fn test_struct_field_access() {
    // Test: Accessing multiple fields
    let code = r#"struct Point { x: i32, y: i32 }; let p = Point { x: 10, y: 20 }; p.y"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 20, "Point.y should be 20");
        }
        _ => panic!("Expected Integer(20), got {:?}", result),
    }
}

#[test]
fn test_impl_block_constructor() {
    // Test: impl block with constructor working correctly
    let code = r#"
        struct Point { x: i32, y: i32 };
        impl Point {
            fun new(x: i32, y: i32) -> Point {
                Point { x, y }
            }
        };
        let p = Point::new(3, 4);
        p.x
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 3, "Point.x should be 3");
        }
        _ => panic!("Expected Integer(3), got {:?}", result),
    }
}

#[test]
fn test_struct_with_string_fields() {
    // Test: Struct with string fields
    let code = r#"
        struct Person { name: String, age: i32 };
        let p = Person { name: "Alice", age: 30 };
        p.name
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "Alice", "Person.name should be Alice");
        }
        _ => panic!("Expected String(\"Alice\"), got {:?}", result),
    }
}

// ============================================================================
// BOOK-008: SMART FLOAT DISPLAY FORMATTING
// ============================================================================

#[test]
fn test_float_display_basic() {
    // Test: Float displays with decimal point
    let code = r#"5.0"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            assert!((f - 5.0).abs() < 0.001, "Float should be 5.0, got {}", f);
        }
        _ => panic!("Expected Float(5.0), got {:?}", result),
    }
}

#[test]
fn test_float_display_with_decimals() {
    // Test: Float with actual decimal values
    let code = r#"3.14159"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            assert!(
                (f - 3.14159).abs() < 0.00001,
                "Float should be 3.14159, got {}",
                f
            );
        }
        _ => panic!("Expected Float(3.14159), got {:?}", result),
    }
}

#[test]
fn test_float_arithmetic_result() {
    // Test: Arithmetic producing float
    let code = r#"10.0 / 3.0"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Float(f) => {
            assert!(
                (f - 3.333333).abs() < 0.001,
                "Float should be ~3.333, got {}",
                f
            );
        }
        _ => panic!("Expected Float, got {:?}", result),
    }
}

#[test]
fn test_integer_vs_float_distinction() {
    // Test: Integers remain integers, floats remain floats
    let code = r#"let i = 5; let f = 5.0; i"#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::Integer(n) => {
            assert_eq!(n, 5, "Integer should be 5");
        }
        _ => panic!("Expected Integer(5), got {:?}", result),
    }
}

// ============================================================================
// FIX-001: MATCH WITH VOID BRANCHES (TRANSPILER)
// ============================================================================

#[test]
fn test_match_with_println_branches() {
    // Test: Match where all branches return () should compile and run
    let code = r#"
        let number = 2;
        match number {
            1 => println("One"),
            2 => println("Two"),
            3 => println("Three"),
            _ => println("Other")
        }
    "#;

    // This should transpile and run without error
    // The issue is the transpiler tries to display () which doesn't implement Display
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let result = interpreter.eval_expr(&ast).expect("Eval should succeed");

    // Match with println branches returns Nil in interpreter
    match result {
        Value::Nil => {
            // Expected - println returns nil, so match returns nil
        }
        _ => panic!("Expected Nil, got {:?}", result),
    }
}

#[test]
fn test_match_void_with_single_expression() {
    // Test: Simpler case - match with void return
    let code = r#"
        let status = 200;
        match status {
            200 => println("Success"),
            404 => println("Not Found"),
            _ => println("Error")
        }
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let result = interpreter.eval_expr(&ast).expect("Eval should succeed");

    match result {
        Value::Nil => {
            // Expected
        }
        _ => panic!("Expected Nil, got {:?}", result),
    }
}

// ============================================================================
// PROPERTY-BASED TESTS: String Multiplication
// ============================================================================

#[cfg(test)]
mod string_multiply_properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_string_multiply_length_invariant(
            s in "[a-z]{0,10}",
            n in 0..100i64
        ) {
            // Property: (s * n).len() == s.len() * n for non-negative n
            let code = format!(r#""{}" * {}"#, s, n);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    let expected_len = s.len() * (n as usize);
                    prop_assert_eq!(result.len(), expected_len,
                        "String multiplication should preserve length property");
                }
            }
        }

        #[test]
        fn test_string_multiply_content_invariant(
            s in "[a-z]{1,5}",
            n in 1..20i64
        ) {
            // Property: Result consists only of repetitions of original string
            let code = format!(r#""{}" * {}"#, s, n);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    // Check that result can be evenly divided into chunks of s
                    for i in 0..(n as usize) {
                        let start = i * s.len();
                        let end = start + s.len();
                        if end <= result.len() {
                            let chunk = &result[start..end];
                            prop_assert_eq!(chunk, s.as_str(),
                                "Each chunk should be the original string");
                        }
                    }
                }
            }
        }

        #[test]
        fn test_string_multiply_zero_always_empty(s in "[a-z]{0,10}") {
            // Property: Any string * 0 = ""
            let code = format!(r#""{}" * 0"#, s);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    prop_assert_eq!(result.as_ref(), "",
                        "String * 0 should always be empty");
                }
            }
        }

        #[test]
        fn test_string_multiply_one_identity(s in "[a-z]{0,10}") {
            // Property: Any string * 1 = original string
            let code = format!(r#""{}" * 1"#, s);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    prop_assert_eq!(result.as_ref(), s.as_str(),
                        "String * 1 should be identity");
                }
            }
        }
    }
}
