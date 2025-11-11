/// RUNTIME-098: Class constructors return nil instead of instance
///
/// EXTREME TDD Test Suite for runtime class constructor evaluation.
///
/// ROOT CAUSE: Interpreter evaluates class constructor but returns nil
/// instead of the constructed instance.
///
/// RED Phase: All tests MUST fail with nil values or runtime errors.
/// Coverage:
/// - Simple constructor returning instance
/// - Constructor with field access after creation
/// - Constructor with method calls
/// - Constructor with parameters
/// - Multiple constructor calls

use ruchy::runtime::interpreter::{Interpreter, Value};
use ruchy::frontend::parser::Parser;

#[test]
fn test_runtime_098_01_constructor_returns_instance() {
    let code = r#"
class Counter {
    count: i32

    pub new() -> Counter {
        Counter { count: 0 }
    }

    pub fun get_count(&self) -> i32 {
        self.count
    }
}

let c = Counter::new()
c.get_count()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 0, NOT nil
    assert_ne!(result, Value::Nil,
        "Constructor should return instance, not nil");
    assert_eq!(result.to_string(), "0",
        "get_count() should return 0, got: {}", result);
}

#[test]
fn test_runtime_098_02_constructor_field_access() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }
}

let p = Point::new(3, 4)
p.x
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    assert_eq!(result.to_string(), "3",
        "Field access should return correct value, got: {}", result);
}

#[test]
fn test_runtime_098_03_multiple_constructors() {
    let code = r#"
class Counter {
    count: i32

    pub new(initial: i32) -> Counter {
        Counter { count: initial }
    }

    pub fun get_count(&self) -> i32 {
        self.count
    }
}

let c1 = Counter::new(10)
let c2 = Counter::new(20)
let sum = c1.get_count() + c2.get_count()
sum
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    assert_eq!(result.to_string(), "30",
        "Sum should be 30, got: {}", result);
}

#[test]
fn test_runtime_098_04_constructor_with_method_chain() {
    let code = r#"
class Calculator {
    value: i32

    pub new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, n: i32) -> i32 {
        self.value = self.value + n
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    assert_eq!(result.to_string(), "5",
        "add() should return 5, got: {}", result);
}

#[test]
fn test_runtime_098_05_impl_block_constructor() {
    // Test that impl blocks work (they should already work)
    let code = r#"
struct Counter {
    count: i32
}

impl Counter {
    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    pub fun get_count(&self) -> i32 {
        self.count
    }
}

let c = Counter::new()
c.get_count()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    assert_eq!(result.to_string(), "0",
        "Impl block constructor should work, got: {}", result);
}
