// EXTREME TDD: Interpreter tests for comprehensions
// Tests to ensure comprehensions work correctly in the interpreter

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

fn eval_code(interpreter: &mut Interpreter, code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interpreter.eval_expr(&expr).map_err(|e| e.to_string())
}

#[test]
fn test_simple_list_comprehension() {
    let mut interpreter = Interpreter::new();
    let code = "[x * 2 for x in 0..5]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Integer(0));
        assert_eq!(arr[1], Value::Integer(2));
        assert_eq!(arr[2], Value::Integer(4));
        assert_eq!(arr[3], Value::Integer(6));
        assert_eq!(arr[4], Value::Integer(8));
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
fn test_list_comprehension_with_filter() {
    let mut interpreter = Interpreter::new();
    let code = "[x for x in 0..10 if x % 2 == 0]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Integer(0));
        assert_eq!(arr[1], Value::Integer(2));
        assert_eq!(arr[2], Value::Integer(4));
        assert_eq!(arr[3], Value::Integer(6));
        assert_eq!(arr[4], Value::Integer(8));
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
fn test_nested_list_comprehension() {
    let mut interpreter = Interpreter::new();
    let code = "[x + y for x in 0..3 for y in 0..3]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 9);
        // Expected: [0, 1, 2, 1, 2, 3, 2, 3, 4]
        assert_eq!(arr[0], Value::Integer(0)); // 0+0
        assert_eq!(arr[1], Value::Integer(1)); // 0+1
        assert_eq!(arr[2], Value::Integer(2)); // 0+2
        assert_eq!(arr[3], Value::Integer(1)); // 1+0
        assert_eq!(arr[4], Value::Integer(2)); // 1+1
        assert_eq!(arr[5], Value::Integer(3)); // 1+2
        assert_eq!(arr[6], Value::Integer(2)); // 2+0
        assert_eq!(arr[7], Value::Integer(3)); // 2+1
        assert_eq!(arr[8], Value::Integer(4)); // 2+2
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
#[ignore = "Set literals not yet implemented - waiting for EXTR-001"]
fn test_set_comprehension() {
    let mut interpreter = Interpreter::new();
    let code = "{x % 3 for x in 0..10}";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    // TODO: Implement Set when EXTR-001 is completed
    // if let Value::Set(set) = result {
    //     assert_eq!(set.len(), 3); // Should only have 0, 1, 2
    //     assert!(set.contains(&Value::Integer(0)));
    //     assert!(set.contains(&Value::Integer(1)));
    //     assert!(set.contains(&Value::Integer(2)));
    // } else {
    //     panic!("Expected set, got {:?}", result);
    // }

    // Temporary: Just verify it doesn't crash for now
    let _ = result;
}

#[test]
#[ignore = "Set literals not yet implemented - waiting for EXTR-001"]
fn test_set_comprehension_with_filter() {
    let mut interpreter = Interpreter::new();
    let code = "{x for x in 0..10 if x % 2 == 1}";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    // TODO: Implement Set when EXTR-001 is completed
    // if let Value::Set(set) = result {
    //     assert_eq!(set.len(), 5); // Should have 1, 3, 5, 7, 9
    //     assert!(set.contains(&Value::Integer(1)));
    //     assert!(set.contains(&Value::Integer(3)));
    //     assert!(set.contains(&Value::Integer(5)));
    //     assert!(set.contains(&Value::Integer(7)));
    //     assert!(set.contains(&Value::Integer(9)));
    // } else {
    //     panic!("Expected set, got {:?}", result);
    // }

    // Temporary: Just verify it doesn't crash for now
    let _ = result;
}

#[test]
fn test_comprehension_over_array() {
    let mut interpreter = Interpreter::new();
    // First create an array
    let code1 = "let arr = [1, 2, 3, 4, 5]";
    eval_code(&mut interpreter, code1).expect("Failed to evaluate");

    // Then use comprehension over it
    let code2 = "[x * x for x in arr]";
    let result = eval_code(&mut interpreter, code2).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Integer(1));
        assert_eq!(arr[1], Value::Integer(4));
        assert_eq!(arr[2], Value::Integer(9));
        assert_eq!(arr[3], Value::Integer(16));
        assert_eq!(arr[4], Value::Integer(25));
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
fn test_comprehension_with_complex_expression() {
    let mut interpreter = Interpreter::new();
    let code = "[x * 2 + 1 for x in 0..5 if x > 1]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3); // Only 2, 3, 4 pass the filter
        assert_eq!(arr[0], Value::Integer(5)); // 2*2+1
        assert_eq!(arr[1], Value::Integer(7)); // 3*2+1
        assert_eq!(arr[2], Value::Integer(9)); // 4*2+1
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
fn test_empty_comprehension() {
    let mut interpreter = Interpreter::new();
    let code = "[x for x in 0..10 if x > 10]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected empty array, got {result:?}");
    }
}

#[test]
fn test_comprehension_with_tuple_result() {
    let mut interpreter = Interpreter::new();
    let code = "[(x, x * x) for x in 0..4]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 4);
        // Check first tuple
        if let Value::Tuple(t) = &arr[0] {
            assert_eq!(t[0], Value::Integer(0));
            assert_eq!(t[1], Value::Integer(0));
        } else {
            panic!("Expected tuple at index 0");
        }
        // Check last tuple
        if let Value::Tuple(t) = &arr[3] {
            assert_eq!(t[0], Value::Integer(3));
            assert_eq!(t[1], Value::Integer(9));
        } else {
            panic!("Expected tuple at index 3");
        }
    } else {
        panic!("Expected array, got {result:?}");
    }
}

#[test]
fn test_nested_comprehension_with_filter() {
    let mut interpreter = Interpreter::new();
    let code = "[x * y for x in 1..4 for y in 1..4 if x != y]";
    let result = eval_code(&mut interpreter, code).expect("Failed to evaluate");

    if let Value::Array(arr) = result {
        // Should have 6 elements: (1,2), (1,3), (2,1), (2,3), (3,1), (3,2)
        assert_eq!(arr.len(), 6);
        assert_eq!(arr[0], Value::Integer(2)); // 1*2
        assert_eq!(arr[1], Value::Integer(3)); // 1*3
        assert_eq!(arr[2], Value::Integer(2)); // 2*1
        assert_eq!(arr[3], Value::Integer(6)); // 2*3
        assert_eq!(arr[4], Value::Integer(3)); // 3*1
        assert_eq!(arr[5], Value::Integer(6)); // 3*2
    } else {
        panic!("Expected array, got {result:?}");
    }
}
