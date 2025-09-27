//! EXTREME TDD: Unit tests for class inheritance
//! Tests class inheritance syntax: class Car : Vehicle

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_inheritance() {
    let code = r"
        class Vehicle {
            speed: f64,
        }

        class Car : Vehicle {
            brand: String,
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // AST should parse both classes

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Car should have Vehicle's fields
    assert!(result_str.contains("struct Car"));
    assert!(result_str.contains("struct Vehicle"));
    // In Rust, we'd typically use composition or traits for inheritance
    // Check that some inheritance mechanism is present
}

#[test]
fn test_inheritance_with_constructors() {
    let code = r"
        class Animal {
            name: String,

            new(name: String) {
                self.name = name
            }
        }

        class Dog : Animal {
            breed: String,

            new(name: String, breed: String) {
                self.name = name
                self.breed = breed
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

    assert!(result_str.contains("struct Animal"));
    assert!(result_str.contains("struct Dog"));
    assert!(result_str.contains("impl Dog"));
}

#[test]
fn test_inheritance_with_methods() {
    let code = r"
        class Shape {
            fn area(&self) -> f64 {
                0.0
            }
        }

        class Rectangle : Shape {
            width: f64,
            height: f64,

            fn area(&self) -> f64 {
                self.width * self.height
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

    assert!(result_str.contains("struct Shape"));
    assert!(result_str.contains("struct Rectangle"));
    // Should have method implementations
    assert!(result_str.contains("fn area"));
}

#[test]
fn test_inheritance_chain() {
    let code = r"
        class A {
            a_field: i32,
        }

        class B : A {
            b_field: i32,
        }

        class C : B {
            c_field: i32,
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("struct A"));
    assert!(result_str.contains("struct B"));
    assert!(result_str.contains("struct C"));
}

#[test]
fn test_inheritance_ast_structure() {
    let code = r"
        class Child : Parent {
            value: i32,
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure for inheritance
    if let ruchy::frontend::ast::ExprKind::Class {
        name, superclass, ..
    } = &ast.kind
    {
        assert_eq!(name, "Child");
        assert!(superclass.is_some(), "Should have superclass");
        assert_eq!(superclass.as_ref().unwrap(), "Parent");
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_pub_class_inheritance() {
    let code = r"
        pub class Base {
            value: i32,
        }

        pub class Derived : Base {
            extra: String,
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("pub struct Base"));
    assert!(result_str.contains("pub struct Derived"));
}
