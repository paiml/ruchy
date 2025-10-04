//! EXTREME TDD: Integration tests for complete class features
//! Tests all class features working together in realistic scenarios

use ruchy::{Parser, Transpiler};

#[test]
fn test_complete_class_hierarchy() {
    let code = r#"
        #[derive(Debug, Clone)]
        pub class Animal {
            name: String,
            age: i32,

            pub new(name: String, age: i32) {
                self.name = name
                self.age = age
            }

            pub fn speak(&self) -> String {
                "Some sound".to_string()
            }

            pub static fn species_count() -> i32 {
                100
            }
        }

        #[derive(Debug)]
        pub class Dog : Animal {
            breed: String,

            pub new(name: String, age: i32, breed: String) {
                self.name = name
                self.age = age
                self.breed = breed
            }

            pub new puppy(breed: String) {
                self.name = "Puppy"
                self.age = 0
                self.breed = breed
            }

            pub override fn speak(&self) -> String {
                "Woof!".to_string()
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Verify all features are present
    assert!(
        result_str.contains("# [derive (Debug , Clone)]")
            || result_str.contains("#[derive(Debug, Clone)]")
    );
    assert!(result_str.contains("pub struct Animal"));
    assert!(result_str.contains("pub struct Dog"));
    assert!(result_str.contains("pub fn new"));
    assert!(result_str.contains("pub fn puppy") || result_str.contains("pub fn new_puppy"));
    assert!(result_str.contains("pub fn speak"));
    assert!(result_str.contains("fn species_count"));
}

#[test]
fn test_complex_trait_mixing() {
    let code = r#"
        trait Display {
            fun display(&self) -> String
        }

        trait Serialize {
            fun serialize(&self) -> String
        }

        class Base {
            value: i32,
        }

        class Extended : Base + Display + Serialize {
            extra: String,

            fn display(&self) -> String {
                format!("{}", self.value)
            }

            fn serialize(&self) -> String {
                format!("{{\"value\":{},\"extra\":\"{}\"}}", self.value, self.extra)
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify the AST has correct trait mixing
    if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
        // Find the Extended class
        for expr in exprs {
            if let ruchy::frontend::ast::ExprKind::Class {
                name,
                superclass,
                traits,
                ..
            } = &expr.kind
            {
                if name == "Extended" {
                    assert_eq!(superclass.as_ref().unwrap(), "Base");
                    assert_eq!(traits.len(), 2);
                    assert!(traits.contains(&"Display".to_string()));
                    assert!(traits.contains(&"Serialize".to_string()));
                }
            }
        }
    }
}

#[test]
fn test_all_constructor_variants() {
    let code = r#"
        class Config {
            host: String,
            port: i32,
            timeout: i32 = 5000,  // Field default
            debug: bool = false,

            // Primary constructor
            new(host: String, port: i32) {
                self.host = host
                self.port = port
                // timeout and debug use defaults
            }

            // Named constructor with all params
            pub new full(host: String, port: i32, timeout: i32, debug: bool) {
                self.host = host
                self.port = port
                self.timeout = timeout
                self.debug = debug
            }

            // Named constructor with defaults
            pub new localhost(port: i32) {
                self.host = "127.0.0.1"
                self.port = port
            }

            // Named constructor returning Result
            pub new validated(host: String, port: i32) -> Result<Self> {
                if port <= 0 || port > 65535 {
                    Err("Invalid port")
                } else {
                    Ok(Config {
                        host: host,
                        port: port,
                        timeout: 5000,
                        debug: false
                    })
                }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify all constructor variants
    if let ruchy::frontend::ast::ExprKind::Class {
        constructors,
        fields,
        ..
    } = &ast.kind
    {
        assert_eq!(constructors.len(), 4, "Should have 4 constructors");
        assert_eq!(fields.len(), 4, "Should have 4 fields");

        // Check field defaults
        assert!(
            fields[2].default_value.is_some(),
            "timeout should have default"
        );
        assert!(
            fields[3].default_value.is_some(),
            "debug should have default"
        );

        // Check constructor names
        assert!(constructors[0].name.is_none(), "First should be primary");
        assert_eq!(constructors[1].name.as_ref().unwrap(), "full");
        assert_eq!(constructors[2].name.as_ref().unwrap(), "localhost");
        assert_eq!(constructors[3].name.as_ref().unwrap(), "validated");

        // Check return type
        assert!(
            constructors[3].return_type.is_some(),
            "validated should have Result return type"
        );
    }
}

#[test]
fn test_static_and_instance_methods() {
    let code = r"
        class MathUtils {
            mut result: f64,

            new() {
                self.result = 0.0
            }

            // Static utility methods
            pub static fn add(a: f64, b: f64) -> f64 {
                a + b
            }

            pub static fn multiply(a: f64, b: f64) -> f64 {
                a * b
            }

            // Instance methods
            pub fn add_to_result(&mut self, value: f64) {
                self.result = self.result + value
            }

            pub fn get_result(&self) -> f64 {
                self.result
            }

            pub fn reset(mut self) {
                self.result = 0.0
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify methods
    if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &ast.kind {
        assert_eq!(methods.len(), 5, "Should have 5 methods");

        // Check static methods
        assert!(methods[0].is_static, "add should be static");
        assert!(methods[1].is_static, "multiply should be static");

        // Check instance methods
        assert!(!methods[2].is_static, "add_to_result should be instance");
        assert!(!methods[3].is_static, "get_result should be instance");
        assert!(!methods[4].is_static, "reset should be instance");

        // Check self types
        assert!(matches!(
            methods[0].self_type,
            ruchy::frontend::ast::SelfType::None
        ));
        assert!(matches!(
            methods[2].self_type,
            ruchy::frontend::ast::SelfType::MutBorrowed
        ));
        assert!(matches!(
            methods[3].self_type,
            ruchy::frontend::ast::SelfType::Borrowed
        ));
        assert!(matches!(
            methods[4].self_type,
            ruchy::frontend::ast::SelfType::Owned
        ));
    }
}

#[test]
fn test_inheritance_with_overrides() {
    let code = r#"
        class Shape {
            pub fn area(&self) -> f64 { 0.0 }
            pub fn perimeter(&self) -> f64 { 0.0 }
            pub fn name(&self) -> String { "Shape".to_string() }
        }

        class Rectangle : Shape {
            width: f64,
            height: f64,

            pub new(width: f64, height: f64) {
                self.width = width
                self.height = height
            }

            pub override fn area(&self) -> f64 {
                self.width * self.height
            }

            pub override fn perimeter(&self) -> f64 {
                2.0 * (self.width + self.height)
            }

            pub override fn name(&self) -> String {
                "Rectangle".to_string()
            }
        }

        class Square : Rectangle {
            pub new square(size: f64) {
                self.width = size
                self.height = size
            }

            pub override fn name(&self) -> String {
                "Square".to_string()
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify inheritance chain and overrides
    if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
        assert_eq!(exprs.len(), 3, "Should have 3 classes");

        // Check Rectangle
        if let ruchy::frontend::ast::ExprKind::Class {
            name,
            superclass,
            methods,
            ..
        } = &exprs[1].kind
        {
            assert_eq!(name, "Rectangle");
            assert_eq!(superclass.as_ref().unwrap(), "Shape");
            assert!(
                methods.iter().all(|m| m.is_override),
                "All Rectangle methods should be override"
            );
        }

        // Check Square
        if let ruchy::frontend::ast::ExprKind::Class {
            name,
            superclass,
            constructors,
            methods,
            ..
        } = &exprs[2].kind
        {
            assert_eq!(name, "Square");
            assert_eq!(superclass.as_ref().unwrap(), "Rectangle");
            assert_eq!(constructors[0].name.as_ref().unwrap(), "square");
            assert!(methods[0].is_override, "name method should be override");
        }
    }
}
