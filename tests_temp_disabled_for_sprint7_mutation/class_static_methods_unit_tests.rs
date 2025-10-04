//! EXTREME TDD: Unit tests for static methods in classes
//! These tests verify static method implementation

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_static_method() {
    let code = r"
        class Math {
            static fn add(x: i32, y: i32) -> i32 {
                x + y
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with static method in impl
    assert!(result_str.contains("struct Math"));
    assert!(result_str.contains("impl Math"));
    // Static methods don't take self parameter
    assert!(
        result_str.contains("fn add (x : i32 , y : i32) -> i32")
            || result_str.contains("fn add(x: i32, y: i32) -> i32")
    );
}

#[test]
fn test_static_constructor_pattern() {
    let code = r"
        class Counter {
            mut count: i32,

            static fn new_zero() -> Self {
                Counter { count: 0 }
            }

            static fn new_with_value(value: i32) -> Self {
                Counter { count: value }
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should have static constructors
    assert!(
        result_str.contains("fn new_zero () -> Self")
            || result_str.contains("fn new_zero() -> Self")
    );
    assert!(result_str.contains("fn new_with_value"));
}

#[test]
fn test_static_with_regular_methods() {
    let code = r"
        class Calculator {
            mut result: i32,

            fn add(&mut self, x: i32) {
                self.result = self.result + x
            }

            static fn multiply(x: i32, y: i32) -> i32 {
                x * y
            }

            fn get(&self) -> i32 {
                self.result
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should have both instance and static methods
    assert!(result_str.contains("fn add (& mut self"));
    assert!(
        result_str.contains("fn multiply (x : i32 , y : i32)")
            || result_str.contains("fn multiply(x: i32, y: i32)")
    );
    assert!(result_str.contains("fn get (& self"));
}

#[test]
fn test_pub_static_method() {
    let code = r"
        class Utils {
            pub static fn is_even(n: i32) -> bool {
                n % 2 == 0
            }

            static fn helper() -> i32 {
                42
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Public static method should have pub modifier
    assert!(result_str.contains("pub fn is_even"));
    // Private static method should not have pub
    assert!(!result_str.contains("pub fn helper"));
}

#[test]
fn test_static_method_ast_structure() {
    let code = r"
        class Test {
            static fn compute() -> i32 {
                42
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure for static methods
    if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &ast.kind {
        assert_eq!(methods.len(), 1, "Should have 1 method");
        let method = &methods[0];
        assert_eq!(method.name, "compute");
        assert!(method.is_static, "Method should be marked as static");
        assert!(
            matches!(method.self_type, ruchy::frontend::ast::SelfType::None),
            "Static method should have SelfType::None"
        );
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_static_method_with_generics() {
    let code = r"
        class Container<T> {
            static fn create_empty() -> Vec<T> {
                Vec::new()
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("struct Container"));
    assert!(result_str.contains("fn create_empty"));
}
